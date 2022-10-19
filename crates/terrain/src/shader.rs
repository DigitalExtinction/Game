use bevy::{
    prelude::{Handle, Image, Material},
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "9e124e04-fdf1-4836-b82d-fa2f01fddb62"]
pub struct TerrainMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
}

impl TerrainMaterial {
    pub(crate) fn new(texture: Handle<Image>) -> Self {
        Self { texture }
    }
}

impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }
}
