#import bevy_pbr::mesh_vertex_output       MeshVertexOutput
#import bevy_pbr::mesh_bindings            mesh
#import bevy_pbr::mesh_view_bindings       view
#import bevy_core_pipeline::tonemapping    tone_mapping
#import bevy_pbr::pbr_functions            PbrInput, pbr_input_new, prepare_world_normal, apply_normal_mapping, calculate_view, pbr
#import bevy_pbr::pbr_types as pbr_types


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
    in: MeshVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    var pbr_input: PbrInput = pbr_input_new();
    pbr_input.material.perceptual_roughness = 0.8;
    pbr_input.material.metallic = 0.23;
    pbr_input.material.reflectance = 0.06;

    pbr_input.material.base_color = textureSample(
        terrain_texture,
        terrain_sampler,
        in.uv / TEXTURE_SIZE
    );

#ifdef VERTEX_COLORS
    pbr_input.material.base_color = pbr_input.material.base_color * in.color;
#endif

    pbr_input.frag_coord = in.position;
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = prepare_world_normal(
        in.world_normal,
        (pbr_input.material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
        is_front,
    );

    pbr_input.is_orthographic = view.projection[3].w == 1.0;

    pbr_input.N = apply_normal_mapping(
        pbr_input.material.flags,
        pbr_input.world_normal,
#ifdef VERTEX_TANGENTS
#ifdef STANDARDMATERIAL_NORMAL_MAP
        in.world_tangent,
#endif
#endif
#ifdef VERTEX_UVS
        in.uv,
#endif
        view.mip_bias,
    );
    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);
    pbr_input.flags = mesh.flags;

    var output_color = pbr(pbr_input);

#ifdef TONEMAP_IN_SHADER
    output_color = tone_mapping(output_color, view.color_grading);
#endif

    output_color = draw_circles(output_color, in.uv);
    output_color = draw_rectangles(output_color, in.uv);
    return output_color;
}
