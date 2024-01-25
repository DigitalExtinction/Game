#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

// How large (in meters) is a texture.
const TEXTURE_SIZE = 16.;
const SHAPE_COLOR = vec4<f32>(1., 1., 1., 0.75);
const SHAPE_THICKNESS = 0.15;
// Keep these array lengths in sync with /crates/terrain/src/shader.rs.
const MAX_KD_TREE_SIZE = 127u;
const MAX_RECTANGLE_ARRAY_SIZE = 31u;

struct KdTreeNode {
    @align(16) location: vec2<f32>,
    radius: f32,
};

struct KdTree {
    @align(16) nodes: array<KdTreeNode, MAX_KD_TREE_SIZE>,
    count: u32,
};

struct Rectangle {
    inverse_transform: mat3x3<f32>,
    half_size: vec2<f32>,
};

struct Rectangles {
    items: array<Rectangle, MAX_RECTANGLE_ARRAY_SIZE>,
    count: u32,
};

@group(1) @binding(0)
var<uniform> circles: KdTree;
@group(1) @binding(1)
var<uniform> rectangles: Rectangles;
@group(1) @binding(2)
var terrain_texture: texture_2d<f32>;
@group(1) @binding(3)
var terrain_sampler: sampler;

fn mix_colors(base: vec4<f32>, cover: vec4<f32>) -> vec4<f32> {
    let alpha = base.a * cover.a;
    let rgb = base.rgb * cover.a + cover.rgb * (1. - cover.a);
    return vec4<f32>(rgb, alpha);
}

fn draw_circle(
    base: vec4<f32>,
    uv: vec2<f32>,
    center: vec2<f32>,
    radius: f32,
) -> vec4<f32> {
    let distance: f32 = distance(uv, center);
    if distance <= (radius + SHAPE_THICKNESS) && radius <= distance {
        return mix_colors(base, SHAPE_COLOR);
    }
    return base;
}

struct KdRecord {
    index: u32,
    distance: f32,
}

struct Next {
    index: u32,
    depth: u32,
    potential: f32,
}

fn nearest(uv: vec2<f32>) -> u32 {
    if circles.count == 0u {
        return MAX_KD_TREE_SIZE;
    }

    var best: KdRecord;
    best.index = 0u;
    best.distance = distance(circles.nodes[0].location, uv);

    var stack_size: u32 = 1u;
    // Make sure that the stack size is large enought to cover balanced three
    // of size MAX_KD_TREE_SIZE.
    var stack: array<Next, 8>;
    stack[0].index = 0u;
    stack[0].potential = 0.;
    stack[0].depth = 0u;

    while stack_size > 0u {
        stack_size -= 1u;
        let next = stack[stack_size];

        if next.potential >= best.distance {
            continue;
        }

        let node = circles.nodes[next.index];

        let distance = distance(node.location, uv);
        if distance < best.distance {
            best.index = next.index;
            best.distance = distance;
        }

        let axis = next.depth % 2u;
        let diff = uv[axis] - node.location[axis];

        var close = 2u * next.index + 2u;
        var away = 2u * next.index + 1u;

        if diff <= 0. {
            close -= 1u;
            away += 1u;
        }

        if away < circles.count {
            stack[stack_size].index = away;
            stack[stack_size].potential = abs(diff);
            stack[stack_size].depth = next.depth + 1u;
            stack_size += 1u;
        }

        if close < circles.count {
            stack[stack_size].index = close;
            stack[stack_size].potential = 0.;
            stack[stack_size].depth = next.depth + 1u;
            stack_size += 1u;
        }
    }

    return best.index;
}

fn draw_circles(base: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    var output_color = base;

    let index = nearest(uv);
    if index < MAX_KD_TREE_SIZE {
        let node = circles.nodes[index];
        let center = node.location;
        let radius = node.radius;
        output_color = draw_circle(output_color, uv, center, radius);
    }

    return output_color;
}

fn draw_rectangles(base: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    for (var i = 0u; i < rectangles.count; i++) {
        let rectangle = rectangles.items[i];
        let local_uv = (rectangle.inverse_transform * vec3(uv, 1.0)).xy;
        if all(abs(local_uv) <= rectangle.half_size + SHAPE_THICKNESS) && any(rectangle.half_size <= abs(local_uv)) {
            return mix_colors(base, SHAPE_COLOR);
        }
    }

    return base;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    pbr_input.material.perceptual_roughness = 0.8;
    pbr_input.material.metallic = 0.23;
    pbr_input.material.reflectance = 0.06;

    pbr_input.material.base_color = textureSample(
        terrain_texture,
        terrain_sampler,
        in.uv / TEXTURE_SIZE
    );
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    // TODO remove this if deffered mode is not used
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);

    out.color = draw_circles(out.color, in.uv);
    out.color = draw_rectangles(out.color, in.uv);

    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}
