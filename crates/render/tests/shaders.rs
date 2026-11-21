//! The shaders parse and validate, without a GPU.

use naga::valid::{Capabilities, ValidationFlags, Validator};

fn validate(name: &str, source: &str) {
    let module =
        naga::front::wgsl::parse_str(source).unwrap_or_else(|e| panic!("{name}: {}", e.emit_to_string(source)));
    Validator::new(ValidationFlags::all(), Capabilities::empty())
        .validate(&module)
        .unwrap_or_else(|e| panic!("{name}: {e:?}"));
}

#[test]
fn trace_shader_is_valid() {
    validate("trace.wgsl", psito3d_render::TRACE_WGSL);
}

#[test]
fn resolve_shader_is_valid() {
    validate("resolve.wgsl", psito3d_render::RESOLVE_WGSL);
}
