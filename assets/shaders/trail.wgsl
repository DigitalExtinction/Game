let COLOR = vec4<f32>(1., 0.85, 0.1, 0.7);

@group(1) @binding(0)
var<uniform> time: f32;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    var color = COLOR;
    color.a *= pow(0.001, time) * sqrt(1. - abs(uv.y));
    return color;
}
