#import bevy_pbr::mesh_view_bindings  globals
#import bevy_pbr::mesh_vertex_output  MeshVertexOutput

const COLOR = vec4<f32>(1., 0.85, 0.1, 0.7);

@group(1) @binding(0)
var<uniform> start_time: f32;

@fragment
fn fragment(
    in: MeshVertexOutput,
) -> @location(0) vec4<f32> {
    var color = COLOR;
    // Use max(0., ...) because the times are wrapping.
    color.a *= pow(0.001, max(0., globals.time - start_time)) * sqrt(1. - abs(in.uv.y));
    return color;
}
