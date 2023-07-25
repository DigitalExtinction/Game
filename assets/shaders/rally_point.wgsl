#import bevy_pbr::mesh_view_bindings  globals
#import bevy_pbr::mesh_bindings       mesh
#import bevy_pbr::mesh_vertex_output  MeshVertexOutput

struct CustomMaterial {
    color: vec4<f32>,
    pointiness: f32,
    speed: f32,
    length: f32,
    spacing: f32,
    fade: f32,
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;

const COLOR: vec4<f32> = vec4<f32>(0.0, 0.5, 0.0, 0.8);
const POINTINESS: f32 = 2.;
const SPEED: f32 = 3.;
const LENGTH: f32 = 1.;
const SPACING: f32 = 0.5;
const FADE: f32 = 3.;

@fragment
fn fragment(
    in: MeshVertexOutput,
) -> @location(0) vec4<f32> {
    let world_space_length: f32 = length(mesh.model[0].xyz);
    let scaled_x: f32 = in.uv.x * world_space_length;
    let offset_y: f32 = abs(in.uv.y - 0.5) * POINTINESS;
    let scaled_time: f32 = globals.time * SPEED;
    let total_length = LENGTH + SPACING;

    let value = scaled_x + offset_y - scaled_time;
    // Ensure that the result of the modulo operation is always positive
    let positive_modulo = (value % total_length + total_length) % total_length;
    let alpha = step(SPACING, positive_modulo);

    let start_fade: f32 = (floor(value / total_length) * total_length + scaled_time) / FADE;
    let end_fade: f32 = (world_space_length - ((ceil(value / total_length) * total_length + scaled_time))) / FADE;
    let fade = min(1., min(start_fade, end_fade));

    return COLOR * vec4(1., 1., 1., alpha * fade);
}
