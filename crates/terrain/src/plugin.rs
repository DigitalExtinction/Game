use bevy::prelude::*;
use de_core::stages::GameStage;

use crate::terrain::Terrain;

pub(crate) struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(GameStage::Update, init);
    }
}

fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    uninitialized: Query<(Entity, &Terrain, &Transform), Without<Handle<Mesh>>>,
) {
    for (entity, terrain, transform) in uninitialized.iter() {
        commands.entity(entity).insert_bundle(PbrBundle {
            mesh: meshes.add(terrain.generate_mesh()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: *transform,
            ..Default::default()
        });
    }
}
