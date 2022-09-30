use bevy::{
    prelude::Material,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "9e124e04-fdf1-4836-b82d-fa2f01fddb62"]
pub struct TerrainMaterial {
    // TODO
}

impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain_shader.wgsl".into()
    }
}
