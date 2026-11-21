// Psito 3D: the trace pass.
//
// One sample per pixel per frame, averaged into the accumulation texture, so
// the image starts noisy and sharpens while the person looks at it.
//
// The world is ray marched: the terrain is a heightfield, the objects are
// signed distance functions combined in order, the sea is a plane, and the
// sky is analytic. A path follows mirror, glass and water reflections for a
// few bounces; matte surfaces end the path with direct sun and sky light.
//
// Units: world units, y up. Colours are linear and unbounded (HDR); the
// resolve pass tone maps them.

struct Globals {
    cam_pos: vec4f,    // xyz eye, w = tan(fov / 2)
    cam_fwd: vec4f,    // xyz, w = aspect (width / height)
    cam_right: vec4f,
    cam_up: vec4f,
    sun_dir: vec4f,    // xyz towards the sun, w = haze
    sky: vec4f,        // cloud cover, cloud height, unused, unused
    fog: vec4f,        // density, mist, mist height, unused
    sea: vec4f,        // enabled, level, ripple, unused
    sea_color: vec4f,  // rgb deep water
    terrain: vec4f,    // extent, height, resolution, unused
    res: vec4f,        // width, height, jitter x, jitter y (pixels)
    info: vec4u,       // frame index, object count, unused, unused
    quality: vec4u,    // march steps, shadow steps, bounces, unused
}

struct Object {
    rot0: vec4f,       // rows of the world-to-local rotation; w of rot0..2 = position
    rot1: vec4f,
    rot2: vec4f,
    size: vec4f,       // xyz full size, w unused
    color: vec4f,      // rgb, w = metallic
    mat: vec4f,        // roughness, transmission, ior, blend radius
    kind: vec4u,       // shape, combine, visible, unused
}

@group(0) @binding(0) var<uniform> g: Globals;
@group(0) @binding(1) var<storage, read> objects: array<Object>;
@group(0) @binding(2) var heightmap: texture_2d<f32>;
@group(0) @binding(3) var previous: texture_2d<f32>;

const PI: f32 = 3.14159265;
const FAR: f32 = 6000.0;
const KIND_SKY: u32 = 0u;
const KIND_TERRAIN: u32 = 1u;
const KIND_OBJECT: u32 = 2u;
const KIND_SEA: u32 = 3u;

// ---------------------------------------------------------------- random --

var<private> rng_state: u32;

fn pcg(v: u32) -> u32 {
    let state = v * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn rand() -> f32 {
    rng_state = pcg(rng_state);
    return f32(rng_state) / 4294967296.0;
}

// A direction around `n`, cosine weighted, for rough reflections.
fn around(n: vec3f, spread: f32) -> vec3f {
    let a = rand() * 2.0 * PI;
    let r = sqrt(rand()) * spread;
    let t = normalize(select(vec3f(1.0, 0.0, 0.0), vec3f(0.0, 1.0, 0.0), abs(n.x) > 0.9));
    let u = normalize(cross(t, n));
    let v = cross(n, u);
    return normalize(n + (u * cos(a) + v * sin(a)) * r);
}

// ----------------------------------------------------------------- noise --

fn hash2(p: vec2f) -> f32 {
    let q = fract(p * vec2f(123.34, 456.21));
    let r = q + dot(q, q + 45.32);
    return fract(r.x * r.y);
}

fn value_noise(p: vec2f) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash2(i);
    let b = hash2(i + vec2f(1.0, 0.0));
    let c = hash2(i + vec2f(0.0, 1.0));
    let d = hash2(i + vec2f(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p0: vec2f, octaves: i32) -> f32 {
    var p = p0;
    var sum = 0.0;
    var amp = 0.5;
    for (var i = 0; i < octaves; i++) {
        sum += amp * value_noise(p);
        p = p * 2.03 + vec2f(17.1, 9.2);
        amp *= 0.5;
    }
    return sum;
}

// ------------------------------------------------------------------- sky --

fn sun_strength() -> f32 {
    return smoothstep(-0.12, 0.08, g.sun_dir.y);
}

// The colour of the sunlight: white at noon, orange near the horizon.
fn sun_color() -> vec3f {
    let low = 1.0 - smoothstep(0.0, 0.4, g.sun_dir.y);
    return mix(vec3f(1.0, 0.96, 0.9), vec3f(1.0, 0.45, 0.15), low) * 3.2 * sun_strength();
}

// The sky without the sun disk and clouds, also used for fog and ambient.
fn sky_base(rd: vec3f) -> vec3f {
    let s = g.sun_dir.xyz;
    let haze = g.sun_dir.w;
    let day = sun_strength();
    let sunset = (1.0 - smoothstep(0.0, 0.5, s.y)) * smoothstep(-0.2, 0.0, s.y);
    let up = max(rd.y, 0.0);

    // A deep blue overhead, as in the 90s renders, paling towards the horizon.
    let zenith = mix(vec3f(0.004, 0.006, 0.02), mix(vec3f(0.05, 0.15, 0.5), vec3f(0.3, 0.4, 0.58), haze), day);
    var horizon = mix(vec3f(0.02, 0.025, 0.05), mix(vec3f(0.5, 0.66, 0.9), vec3f(0.8, 0.8, 0.82), haze), day);
    // Low sun: the horizon glows orange, strongest on the sun's side.
    let toward = 0.5 + 0.5 * dot(normalize(vec2f(rd.x, rd.z) + vec2f(1e-5)), normalize(s.xz + vec2f(1e-5)));
    horizon = mix(horizon, vec3f(1.0, 0.42, 0.16) * (1.0 + haze), sunset * mix(0.35, 0.95, toward));

    var col = mix(horizon, zenith, pow(up, 0.35 + 0.4 * haze));
    // The glow around the sun, wider when the air is hazy.
    let mu = max(dot(rd, s), 0.0);
    col += sun_color() * (pow(mu, 6.0) * (0.12 + 0.3 * sunset) + pow(mu, 48.0) * 0.25) * (0.5 + haze);
    return col;
}

fn stars(rd: vec3f) -> vec3f {
    let night = 1.0 - smoothstep(-0.2, 0.0, g.sun_dir.y);
    if (night <= 0.0 || rd.y <= 0.0) {
        return vec3f(0.0);
    }
    let cell = floor(rd * 300.0);
    let h = hash2(cell.xy + cell.z * 7.1);
    return vec3f(step(0.998, h) * night * 0.8 * rd.y);
}

fn clouds(ro: vec3f, rd: vec3f, base: vec3f) -> vec3f {
    let cover = g.sky.x;
    if (cover <= 0.0 || rd.y <= 0.01) {
        return base;
    }
    let t = (g.sky.y - ro.y) / rd.y;
    if (t <= 0.0) {
        return base;
    }
    let p = (ro + rd * t).xz * 0.01;
    let n = fbm(p, 6);
    // A narrow band turns the noise into puffs with clear sky between them.
    let density = smoothstep(1.0 - cover * 0.75, 1.0 - cover * 0.75 + 0.12, n);
    // Lit from the sun side: sample the noise a little towards the sun.
    let towards = fbm(p + g.sun_dir.xz * 0.05, 4);
    let lit = clamp(0.6 + (n - towards) * 3.0, 0.2, 1.2);
    let col = mix(sky_base(vec3f(rd.x, 0.0, rd.z)) * 0.6, sun_color() * 0.45 + vec3f(0.05), lit);
    let fade = smoothstep(0.01, 0.15, rd.y);
    return mix(base, col, density * fade * 0.9);
}

fn sky(ro: vec3f, rd: vec3f, with_sun: bool) -> vec3f {
    var col = sky_base(rd) + stars(rd);
    if (with_sun) {
        let mu = dot(rd, g.sun_dir.xyz);
        col += sun_color() * smoothstep(0.99955, 0.9998, mu) * 12.0;
    }
    return clouds(ro, rd, col);
}

// --------------------------------------------------------------- terrain --

fn terrain_height(xz: vec2f) -> f32 {
    let extent = g.terrain.x;
    let res = g.terrain.z;
    let uv = (xz / extent + 0.5) * (res - 1.0);
    if (any(uv < vec2f(0.0)) || any(uv > vec2f(res - 1.0))) {
        return 0.0;
    }
    let i = vec2i(floor(uv));
    let f = fract(uv);
    let m = i32(res) - 1;
    let a = textureLoad(heightmap, clamp(i, vec2i(0), vec2i(m)), 0).r;
    let b = textureLoad(heightmap, clamp(i + vec2i(1, 0), vec2i(0), vec2i(m)), 0).r;
    let c = textureLoad(heightmap, clamp(i + vec2i(0, 1), vec2i(0), vec2i(m)), 0).r;
    let d = textureLoad(heightmap, clamp(i + vec2i(1, 1), vec2i(0), vec2i(m)), 0).r;
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y) * g.terrain.y;
}

fn terrain_normal(p: vec3f) -> vec3f {
    let e = g.terrain.x / g.terrain.z;
    let dx = terrain_height(p.xz + vec2f(e, 0.0)) - terrain_height(p.xz - vec2f(e, 0.0));
    let dz = terrain_height(p.xz + vec2f(0.0, e)) - terrain_height(p.xz - vec2f(0.0, e));
    return normalize(vec3f(-dx, 2.0 * e, -dz));
}

// A lower bound of the distance to the terrain. Above the highest point it
// is exact; below, a heightfield is not a true distance, so the step shrinks.
fn terrain_distance(p: vec3f) -> f32 {
    if (p.y > g.terrain.y) {
        return p.y - g.terrain.y + 0.5;
    }
    return (p.y - terrain_height(p.xz)) * 0.45;
}

// --------------------------------------------------------------- objects --

fn sd_shape(shape: u32, p: vec3f, size: vec3f) -> f32 {
    let h = size * 0.5;
    switch shape {
        case 0u: { // sphere, stretched into an ellipsoid
            let k0 = length(p / h);
            let k1 = length(p / (h * h));
            return k0 * (k0 - 1.0) / max(k1, 1e-5);
        }
        case 1u: { // rounded cube
            let r = 0.06 * min(h.x, min(h.y, h.z));
            let q = abs(p) - h + r;
            return length(max(q, vec3f(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0) - r;
        }
        case 2u: { // cylinder
            let r = min(h.x, h.z);
            let d = abs(vec2f(length(p.xz), p.y)) - vec2f(r, h.y);
            return min(max(d.x, d.y), 0.0) + length(max(d, vec2f(0.0)));
        }
        case 3u: { // cone, tip up
            let r = min(h.x, h.z);
            let q = vec2f(length(p.xz), p.y);
            let k1 = vec2f(0.0, h.y);
            let k2 = vec2f(-r, 2.0 * h.y);
            let ca = vec2f(q.x - min(q.x, select(r, 0.0, q.y > 0.0)), abs(q.y) - h.y);
            let cb = q - k1 + k2 * clamp(dot(k1 - q, k2) / dot(k2, k2), 0.0, 1.0);
            let s = select(1.0, -1.0, cb.x < 0.0 && ca.y < 0.0);
            return s * sqrt(min(dot(ca, ca), dot(cb, cb)));
        }
        default: { // torus lying flat
            let r = min(h.x, h.z);
            let q = vec2f(length(p.xz) - r * 0.7, p.y);
            return length(q) - r * 0.3;
        }
    }
}

fn object_distance(i: u32, p: vec3f) -> f32 {
    let o = objects[i];
    let rel = p - vec3f(o.rot0.w, o.rot1.w, o.rot2.w);
    let local = vec3f(dot(o.rot0.xyz, rel), dot(o.rot1.xyz, rel), dot(o.rot2.xyz, rel));
    return sd_shape(o.kind.x, local, o.size.xyz);
}

fn smin(a: f32, b: f32, k: f32) -> vec2f {
    // Returns the blended distance and how much of `b` it takes.
    if (k <= 0.0) {
        return select(vec2f(a, 0.0), vec2f(b, 1.0), b < a);
    }
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return vec2f(mix(b, a, h) - k * h * (1.0 - h), 1.0 - h);
}

struct ObjHit {
    d: f32,
    id: u32,
}

// All objects combined in order: each one joins, carves or intersects what
// came before. The id is the object whose surface is closest.
fn objects_distance(p: vec3f) -> ObjHit {
    var d = FAR;
    var id = 0xffffffffu;
    for (var i = 0u; i < g.info.y; i++) {
        let o = objects[i];
        if (o.kind.z == 0u) {
            continue;
        }
        let di = object_distance(i, p);
        let k = o.mat.w;
        switch o.kind.y {
            case 1u: { // subtract
                let m = smin(-d, di, k);
                d = -m.x;
            }
            case 2u: { // intersect
                let m = smin(-d, -di, k);
                d = -m.x;
                if (di > d - 1e-3) { id = i; }
            }
            default: { // union
                let m = smin(d, di, k);
                d = m.x;
                if (m.y > 0.5) { id = i; }
            }
        }
    }
    return ObjHit(d, id);
}

// ------------------------------------------------------------------ march --

struct Hit {
    t: f32,
    kind: u32,
    id: u32,
}

fn scene_distance(p: vec3f) -> f32 {
    return min(terrain_distance(p), objects_distance(p).d);
}

fn sea_t(ro: vec3f, rd: vec3f) -> f32 {
    if (g.sea.x < 0.5 || rd.y >= 0.0 || ro.y < g.sea.y) {
        return FAR;
    }
    return (g.sea.y - ro.y) / rd.y;
}

fn march(ro: vec3f, rd: vec3f) -> Hit {
    let t_sea = sea_t(ro, rd);
    let t_max = min(FAR, t_sea);
    var t = 0.02;
    for (var i = 0u; i < g.quality.x; i++) {
        let p = ro + rd * t;
        let dt = terrain_distance(p);
        let o = objects_distance(p);
        let d = min(dt, o.d);
        let eps = 0.0015 * t + 0.002;
        if (d < eps) {
            if (o.d < dt) {
                return Hit(t, KIND_OBJECT, o.id);
            }
            return Hit(t, KIND_TERRAIN, 0u);
        }
        t += d;
        if (t > t_max) {
            break;
        }
    }
    if (t_sea < FAR) {
        return Hit(t_sea, KIND_SEA, 0u);
    }
    return Hit(FAR, KIND_SKY, 0u);
}

fn object_normal(p: vec3f) -> vec3f {
    let e = vec2f(0.002, -0.002);
    return normalize(
        e.xyy * objects_distance(p + e.xyy).d + e.yyx * objects_distance(p + e.yyx).d +
        e.yxy * objects_distance(p + e.yxy).d + e.xxx * objects_distance(p + e.xxx).d);
}

// Soft shadow towards the sun (after Inigo Quilez).
fn shadow(ro: vec3f) -> f32 {
    let rd = g.sun_dir.xyz;
    if (rd.y < -0.1) {
        return 0.0;
    }
    var res = 1.0;
    var t = 0.08;
    for (var i = 0u; i < g.quality.y; i++) {
        let h = scene_distance(ro + rd * t);
        res = min(res, 10.0 * h / t);
        if (res < 0.01 || t > 400.0) {
            break;
        }
        t += clamp(h, 0.05, 8.0);
    }
    return clamp(res, 0.0, 1.0);
}

// -------------------------------------------------------------- shading --

fn fog_transmittance(ro: vec3f, rd: vec3f, t: f32) -> f32 {
    let dist = g.fog.x * g.fog.x * 0.012 * t;
    // Exponential height fog: thick at the bottom, thinning upwards.
    let b = 3.0 / max(g.fog.z, 0.1);
    let a = g.fog.y * 0.15;
    var mist = a * exp(-ro.y * b) * t;
    if (abs(rd.y) > 1e-4) {
        mist = a * exp(-ro.y * b) * (1.0 - exp(-t * rd.y * b)) / (rd.y * b);
    }
    return exp(-(dist + max(mist, 0.0)));
}

fn fog_color(rd: vec3f) -> vec3f {
    let mu = max(dot(rd, g.sun_dir.xyz), 0.0);
    return sky_base(vec3f(rd.x, 0.02, rd.z)) * 0.9 + sun_color() * pow(mu, 8.0) * 0.25;
}

fn terrain_albedo(p: vec3f, n: vec3f) -> vec3f {
    let hf = p.y / max(g.terrain.y, 1e-3);
    let detail = value_noise(p.xz * 0.35) * 0.5 + value_noise(p.xz * 1.7) * 0.25;
    let rock = mix(vec3f(0.22, 0.18, 0.15), vec3f(0.38, 0.33, 0.28), detail);
    let grass = mix(vec3f(0.06, 0.13, 0.04), vec3f(0.16, 0.2, 0.07), detail);
    let sand = vec3f(0.55, 0.48, 0.34);
    let snow = vec3f(0.9, 0.92, 0.96);

    let flat = smoothstep(0.72, 0.86, n.y);
    var col = mix(rock, grass, flat * (1.0 - smoothstep(0.45, 0.7, hf + detail * 0.15)));
    let sea_line = select(-10.0, g.sea.y / max(g.terrain.y, 1e-3), g.sea.x > 0.5);
    col = mix(col, sand, (1.0 - smoothstep(0.0, 0.035, hf - sea_line)) * smoothstep(0.5, 0.8, n.y));
    col = mix(col, snow, smoothstep(0.62, 0.72, hf + detail * 0.12) * smoothstep(0.55, 0.75, n.y));
    return col;
}

// Direct light on a matte surface: the sun with its shadow, plus the sky.
fn diffuse_light(p: vec3f, n: vec3f, albedo: vec3f) -> vec3f {
    let ndl = max(dot(n, g.sun_dir.xyz), 0.0);
    var light = vec3f(0.0);
    if (ndl > 0.0) {
        light += sun_color() * ndl * shadow(p + n * 0.05);
    }
    let sky_up = sky_base(normalize(vec3f(n.x, max(n.y, 0.05), n.z)));
    light += sky_up * (0.5 + 0.5 * n.y) * 0.9;
    // A little light bounced from the ground.
    light += vec3f(0.08, 0.07, 0.05) * sun_strength() * (0.5 - 0.5 * n.y);
    return albedo * light / PI * 2.0;
}

fn fresnel(cos_theta: f32, f0: f32) -> f32 {
    return f0 + (1.0 - f0) * pow(1.0 - clamp(cos_theta, 0.0, 1.0), 5.0);
}

fn sea_normal(p: vec3f) -> vec3f {
    let r = g.sea.z;
    let q = p.xz * 0.15;
    let dx = (value_noise(q + vec2f(0.05, 0.0)) - value_noise(q - vec2f(0.05, 0.0))) * 2.0
           + (value_noise(q * 3.1 + vec2f(0.05, 0.0)) - value_noise(q * 3.1 - vec2f(0.05, 0.0)));
    let dz = (value_noise(q + vec2f(0.0, 0.05)) - value_noise(q - vec2f(0.0, 0.05))) * 2.0
           + (value_noise(q * 3.1 + vec2f(0.0, 0.05)) - value_noise(q * 3.1 - vec2f(0.0, 0.05)));
    return normalize(vec3f(-dx * r * 2.0, 1.0, -dz * r * 2.0));
}

// Leaves the inside of a glass object: marches the negated distance until
// it comes out, and returns the point and normal of the exit.
fn exit_glass(ro: vec3f, rd: vec3f) -> vec4f {
    var t = 0.02;
    for (var i = 0; i < 64; i++) {
        let d = -objects_distance(ro + rd * t).d;
        if (d < 0.002) {
            break;
        }
        t += max(d, 0.01);
    }
    return vec4f(ro + rd * t, t);
}

fn radiance(ro0: vec3f, rd0: vec3f) -> vec3f {
    var ro = ro0;
    var rd = rd0;
    var color = vec3f(0.0);
    var throughput = vec3f(1.0);

    for (var bounce = 0u; bounce <= g.quality.z; bounce++) {
        let hit = march(ro, rd);
        let fog_t = fog_transmittance(ro, rd, min(hit.t, 4000.0));
        if (hit.kind == KIND_SKY) {
            let s = sky(ro, rd, true);
            color += throughput * mix(fog_color(rd), s, max(fog_t, smoothstep(0.0, 0.25, rd.y)));
            break;
        }
        color += throughput * (1.0 - fog_t) * fog_color(rd);
        throughput *= fog_t;

        let p = ro + rd * hit.t;
        let last = bounce == g.quality.z;

        if (hit.kind == KIND_TERRAIN) {
            let n = terrain_normal(p);
            color += throughput * diffuse_light(p, n, terrain_albedo(p, n));
            break;
        }

        if (hit.kind == KIND_SEA) {
            var n = sea_normal(p);
            n = around(n, 0.015 * g.sea.z);
            let f = fresnel(dot(-rd, n), 0.02);
            // The water body: deep colour lit by the sky and a hint of sun.
            let body = g.sea_color.rgb * (sky_base(vec3f(0.0, 1.0, 0.0)) * 1.2 + sun_color() * 0.15 * max(g.sun_dir.y, 0.0));
            color += throughput * (1.0 - f) * body;
            throughput *= f;
            rd = reflect(rd, n);
            if (rd.y < 0.002) { rd = normalize(vec3f(rd.x, 0.002, rd.z)); }
            ro = vec3f(p.x, g.sea.y + 0.01, p.z);
            if (last) { color += throughput * sky(ro, rd, true); }
            continue;
        }

        // An object.
        let o = objects[hit.id];
        let n = object_normal(p);
        let albedo = o.color.rgb;
        let metallic = o.color.w;
        let rough = o.mat.x;
        let transmission = o.mat.y;
        let cos_i = dot(-rd, n);

        if (transmission > 0.5) {
            // Glass: reflect or go through, chosen by the Fresnel term, so the
            // average over many samples gives both.
            let f = fresnel(cos_i, 0.04);
            if (rand() < f) {
                rd = around(reflect(rd, n), rough);
                ro = p + n * 0.02;
            } else {
                let eta = 1.0 / o.mat.z;
                let inside = refract(rd, n, eta);
                let exit = exit_glass(p - n * 0.02, inside);
                let n_out = -object_normal(exit.xyz);
                var out_dir = refract(inside, n_out, o.mat.z);
                if (dot(out_dir, out_dir) < 1e-6) {
                    out_dir = reflect(inside, n_out); // total internal reflection
                }
                rd = around(normalize(out_dir), rough);
                ro = exit.xyz - n_out * 0.03;
                throughput *= albedo;
            }
            if (last) { color += throughput * sky(ro, rd, true); }
            continue;
        }

        // Paint and metal: the matte part is lit directly, the shiny part
        // follows the reflection.
        let f0 = mix(vec3f(0.04), albedo, metallic);
        let f = f0 + (vec3f(1.0) - f0) * pow(1.0 - clamp(cos_i, 0.0, 1.0), 5.0);
        let spec = f * (1.0 - rough * 0.7);
        let matte = (vec3f(1.0) - spec) * (1.0 - metallic);
        color += throughput * matte * diffuse_light(p, n, albedo);
        throughput *= spec;
        if (max(throughput.x, max(throughput.y, throughput.z)) < 0.02) {
            break;
        }
        rd = around(reflect(rd, n), rough);
        ro = p + n * 0.02;
        if (last) { color += throughput * sky(ro, rd, true); }
    }
    return color;
}

// --------------------------------------------------------------- entries --

struct VertexOut {
    @builtin(position) pos: vec4f,
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOut {
    // One triangle that covers the whole screen.
    let uv = vec2f(f32((i << 1u) & 2u), f32(i & 2u));
    return VertexOut(vec4f(uv * 2.0 - 1.0, 0.0, 1.0));
}

@fragment
fn fs_main(v: VertexOut) -> @location(0) vec4f {
    let px = vec2u(v.pos.xy);
    rng_state = pcg(px.x * 1973u + px.y * 9277u + g.info.x * 26699u) | 1u;

    let pixel = v.pos.xy + g.res.zw - 0.5;
    let ndc = vec2f(pixel.x / g.res.x * 2.0 - 1.0, 1.0 - pixel.y / g.res.y * 2.0);
    let rd = normalize(g.cam_fwd.xyz
        + g.cam_right.xyz * ndc.x * g.cam_pos.w * g.cam_fwd.w
        + g.cam_up.xyz * ndc.y * g.cam_pos.w);

    var c = radiance(g.cam_pos.xyz, rd);
    // Keep fireflies from a single lucky sample out of the average.
    c = min(c, vec3f(40.0));
    if (any(c != c)) {
        c = vec3f(0.0);
    }

    let n = f32(g.info.x);
    if (g.info.x == 0u) {
        return vec4f(c, 1.0);
    }
    let prev = textureLoad(previous, vec2i(px), 0).rgb;
    return vec4f(prev + (c - prev) / (n + 1.0), 1.0);
}
