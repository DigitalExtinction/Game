#import bevy_pbr::mesh_bindings  mesh
#import bevy_pbr::mesh_functions mesh_position_local_to_clip

const BACKGROUND_COLOR = vec4<f32>(0., 0., 0., 0.75);
const FOREGROUND_COLOR = vec4<f32>(0.6, 1., 0.6, 0.75);

@group(1) @binding(0)
var<uniform> value: f32;

struct Vertex {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) x: f32,
};

struct FragmentInput {
     @location(0) x: f32,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = mesh_position_local_to_clip(mesh.model, vec4<f32>(0., 0., 0., 1.0));

    let scale = max(1., out.clip_position.w / 40.);
    out.clip_position += vec4<f32>(scale * vertex.position, 0., 0.);

    out.x = vertex.uv.x;
    return out;
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var color = FOREGROUND_COLOR;
    if in.x > value {
        color = BACKGROUND_COLOR;
    }
    return color;
}
