//! "Surprise me": a whole world from one seed.

use psito3d_scene::{
    Combine, Material, ParamKind, ParamValue, ParamValues, Scene, Shape, SplitMix64, Sun, TerrainSource, Transform,
};
use psito3d_ui::organisms::GeneratorChoice;

/// The sun of a surprise: mostly the golden hours, sometimes high noon, now
/// and then a night.
fn sun(r: &mut SplitMix64) -> Sun {
    let roll = r.next_f32();
    let elevation = if roll < 0.6 {
        r.range(2.0, 18.0)
    } else if roll < 0.92 {
        r.range(18.0, 65.0)
    } else {
        r.range(-14.0, -5.0)
    };
    Sun { azimuth: r.range(0.0, 360.0), elevation }
}

fn material(r: &mut SplitMix64) -> Material {
    match r.index(5) {
        0 | 1 => Material::chrome(),
        2 => Material::gold(),
        3 => Material::glass(),
        _ => Material::clay([r.range(0.2, 1.0), r.range(0.1, 0.9), r.range(0.1, 0.9)]),
    }
}

/// A random world from `seed`, using one of `generators`. The same seed and
/// generators always give the same world. The camera and the finishing
/// filter are taken from `keep`, so the person's view does not jump.
pub fn surprise(seed: u64, generators: &[GeneratorChoice], keep: &Scene) -> Scene {
    let mut r = SplitMix64::new(seed);
    let mut scene = Scene { camera: keep.camera, filter: keep.filter, ..Scene::default() };

    if let Some(g) = (!generators.is_empty()).then(|| &generators[r.index(generators.len())]) {
        let mut params = ParamValues::new();
        for p in &g.params {
            let v = match &p.kind {
                // A low "height" sinks the land under the sea: keep it in the upper half.
                ParamKind::Slider { min, max, .. } if p.id == "height" => {
                    ParamValue::Number(r.range(min.max(*min + (*max - *min) * 0.5), *max))
                }
                ParamKind::Slider { min, max, .. } => ParamValue::Number(r.range(*min, *max)),
                ParamKind::Toggle { .. } => ParamValue::Bool(r.chance(0.5)),
                ParamKind::Choice { options, .. } => ParamValue::Choice(options[r.index(options.len())].id.clone()),
            };
            params.insert(p.id.clone(), v);
        }
        scene.terrain.source = TerrainSource::Generator { id: g.id.clone(), params };
    }
    scene.terrain.seed = r.next_u64() & 0xFFFF_FFFF;
    scene.terrain.height = r.range(14.0, 36.0);

    scene.sky.sun = sun(&mut r);
    scene.sky.haze = r.range(0.0, 0.5);
    scene.sky.cloud_cover = r.range(0.0, 0.8);
    scene.sky.cloud_height = r.range(2.5, 7.0);
    scene.sea.enabled = r.chance(0.85);
    scene.sea.level = r.range(0.04, 0.2);
    scene.sea.ripple = r.range(0.05, 0.6);
    scene.fog.density = r.range(0.0, 0.45);
    scene.fog.mist = r.range(0.0, 0.5);

    let count = r.index(4);
    for i in 0..count {
        let shape = Shape::ALL[r.index(Shape::ALL.len())];
        let Some(id) = scene.add_object(shape, material(&mut r)) else { break };
        let height = scene.terrain.height;
        let extent = scene.terrain.extent;
        let o = scene.object_mut(id).expect("just added");
        let size = r.range(3.0, 12.0);
        o.transform = Transform {
            position: [r.range(-0.3, 0.3) * extent, height * r.range(0.6, 1.6), r.range(-0.3, 0.3) * extent].into(),
            rotation: [0.0, r.range(-180.0, 180.0), 0.0].into(),
            scale: [size; 3].into(),
        };
        if i > 0 && r.chance(0.35) {
            o.blend = r.range(1.0, 5.0);
        }
        o.combine = Combine::Union;
    }
    scene
}

#[cfg(test)]
mod tests {
    use super::*;
    use psito3d_scene::{Filter, LocalizedText, ParamSpec};

    fn gens() -> Vec<GeneratorChoice> {
        vec![
            GeneratorChoice {
                id: "a".into(),
                name: "A".into(),
                description: String::new(),
                params: vec![ParamSpec::slider("h", LocalizedText::new("H"), 0.2, 0.4, 0.3)],
            },
            GeneratorChoice { id: "b".into(), name: "B".into(), description: String::new(), params: vec![] },
        ]
    }

    #[test]
    fn same_seed_same_world() {
        let keep = Scene::default();
        assert_eq!(surprise(9, &gens(), &keep), surprise(9, &gens(), &keep));
        assert_ne!(surprise(9, &gens(), &keep), surprise(10, &gens(), &keep));
    }

    #[test]
    fn parameters_stay_in_their_range_and_the_view_is_kept() {
        let mut keep = Scene::default();
        keep.camera.yaw = 123.0;
        keep.filter = Filter::Crt;
        for seed in 0..200 {
            let s = surprise(seed, &gens(), &keep);
            assert_eq!(s.camera.yaw, 123.0);
            assert_eq!(s.filter, Filter::Crt);
            if let TerrainSource::Generator { id, params } = &s.terrain.source
                && id == "a"
            {
                assert!((0.2..0.4).contains(&params["h"].as_f32(0.0)));
            }
            assert!(s.objects.len() <= 3);
            assert!((-90.0..=90.0).contains(&s.sky.sun.elevation));
        }
    }

    #[test]
    fn many_seeds_give_variety() {
        let keep = Scene::default();
        let worlds: Vec<Scene> = (0..50).map(|s| surprise(s, &gens(), &keep)).collect();
        assert!(worlds.iter().any(|w| !w.sea.enabled));
        assert!(worlds.iter().any(|w| w.sky.sun.is_night()));
        assert!(worlds.iter().any(|w| w.objects.is_empty()) && worlds.iter().any(|w| w.objects.len() == 3));
    }
}
