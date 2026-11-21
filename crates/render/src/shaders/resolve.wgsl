// Psito 3D: the resolve pass.
//
// Scales the accumulated HDR image to the output, tone maps it and lays the
// finishing filter over it. The output is sRGB, so this writes linear values
// and the hardware encodes them.

struct Resolve {
    params: vec4f,   // exposure multiplier, unused, unused, unused
    sizes: vec4f,    // accumulation width, height, output width, height
    look: vec4u,     // filter, unused, unused, unused
}

@group(0) @binding(0) var<uniform> r: Resolve;
@group(0) @binding(1) var accum: texture_2d<f32>;

struct VertexOut {
    @builtin(position) pos: vec4f,
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOut {
    let uv = vec2f(f32((i << 1u) & 2u), f32(i & 2u));
    return VertexOut(vec4f(uv * 2.0 - 1.0, 0.0, 1.0));
}

// The accumulation is 32-bit float, which cannot be filtered by the
// sampler, so the bilinear filter is done by hand.
fn sample_accum(uv: vec2f) -> vec3f {
    let size = r.sizes.xy;
    let p = uv * size - 0.5;
    let i = vec2i(floor(p));
    let f = fract(p);
    let m = vec2i(size) - 1;
    let a = textureLoad(accum, clamp(i, vec2i(0), m), 0).rgb;
    let b = textureLoad(accum, clamp(i + vec2i(1, 0), vec2i(0), m), 0).rgb;
    let c = textureLoad(accum, clamp(i + vec2i(0, 1), vec2i(0), m), 0).rgb;
    let d = textureLoad(accum, clamp(i + vec2i(1, 1), vec2i(0), m), 0).rgb;
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

// The ACES filmic curve, fitted by Krzysztof Narkowicz.
fn aces(x: vec3f) -> vec3f {
    return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), vec3f(0.0), vec3f(1.0));
}

fn to_srgb(c: vec3f) -> vec3f {
    return select(1.055 * pow(c, vec3f(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3f(0.0031308));
}

fn to_linear(c: vec3f) -> vec3f {
    return select(pow((c + 0.055) / 1.055, vec3f(2.4)), c / 12.92, c <= vec3f(0.04045));
}

// A 4x4 Bayer matrix, for ordered dithering.
fn bayer(p: vec2u) -> f32 {
    let m = array<f32, 16>(0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
    return m[(p.y % 4u) * 4u + (p.x % 4u)] / 16.0 - 0.5;
}

@fragment
fn fs_main(v: VertexOut) -> @location(0) vec4f {
    let out_size = r.sizes.zw;
    var uv = v.pos.xy / out_size;

    if (r.look.x == 2u) {
        // CRT: the glass bulges a little.
        let c = uv * 2.0 - 1.0;
        uv = (c * (1.0 + dot(c, c) * 0.04)) * 0.5 + 0.5;
        if (any(uv < vec2f(0.0)) || any(uv > vec2f(1.0))) {
            return vec4f(0.0, 0.0, 0.0, 1.0);
        }
    }

    var col = aces(sample_accum(uv) * r.params.x);

    switch r.look.x {
        case 1u: {
            // 256 colours: a 6x6x6 cube (216) dithered, in sRGB steps.
            let s = to_srgb(col) + bayer(vec2u(v.pos.xy)) / 5.0;
            col = to_linear(clamp(round(s * 5.0) / 5.0, vec3f(0.0), vec3f(1.0)));
        }
        case 2u: {
            let line = 0.78 + 0.22 * sin(v.pos.y * 3.14159);
            let mask = vec3f(
                select(0.85, 1.0, u32(v.pos.x) % 3u == 0u),
                select(0.85, 1.0, u32(v.pos.x) % 3u == 1u),
                select(0.85, 1.0, u32(v.pos.x) % 3u == 2u));
            let c = uv * 2.0 - 1.0;
            let vignette = 1.0 - dot(c, c) * 0.25;
            col = col * line * mask * vignette * 1.15;
        }
        default: {}
    }
    return vec4f(col, 1.0);
}
