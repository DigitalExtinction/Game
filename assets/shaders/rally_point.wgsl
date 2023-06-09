#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

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

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let world_space_length: f32 = length(mesh.model[0].xyz);
    let scaled_x: f32 = uv.x * world_space_length;
    let offset_y: f32 = abs(uv.y - 0.5) * material.pointiness;
    let scaled_time: f32 = globals.time * material.speed;
    let total_length = material.length + material.spacing;

    let value = scaled_x + offset_y - scaled_time;
    // Ensure that the result of the modulo operation is always positive
    let positive_modulo = (value % total_length + total_length) % total_length;
    let alpha = step(material.spacing, positive_modulo);

    let start_fade: f32 = (floor(value / total_length) * total_length + scaled_time) / material.fade;
    let end_fade: f32 = (world_space_length - ((ceil(value / total_length) * total_length + scaled_time))) / material.fade;
    let fade = min(1., min(start_fade, end_fade));

    return material.color * vec4(1., 1., 1., alpha * fade);
}
