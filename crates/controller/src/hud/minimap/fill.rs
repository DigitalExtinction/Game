use bevy::prelude::*;
use de_core::{
    gconfig::GameConfig, objects::ObjectType, player::Player, projection::ToFlat,
    stages::GameStage, state::GameState,
};
use de_map::size::MapBounds;
use de_objects::{IchnographyCache, ObjectCache};
use iyes_loopless::prelude::*;

use super::draw::DrawingParam;

const TERRAIN_COLOR: Color = Color::rgb(0.61, 0.46, 0.32);
const PLAYER_COLOR: Color = Color::rgb(0.1, 0.1, 0.9);
const ENEMY_COLOR: Color = Color::rgb(0.9, 0.1, 0.1);
const MIN_ENTITY_SIZE: Vec2 = Vec2::splat(0.02);

pub(super) struct FillPlugin;

impl Plugin for FillPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            GameStage::PostMovement,
            SystemSet::new()
                .with_system(
                    clear_system
                        .run_in_state(GameState::Playing)
                        .label(FillLabel::Clear),
                )
                .with_system(
                    draw_entities_system
                        .run_in_state(GameState::Playing)
                        .after(FillLabel::Clear),
                ),
        );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
enum FillLabel {
    Clear,
}

fn clear_system(mut drawing: DrawingParam) {
    let mut drawing = drawing.drawing();
    drawing.fill(TERRAIN_COLOR);
}

fn draw_entities_system(
    mut drawing: DrawingParam,
    bounds: Res<MapBounds>,
    cache: Res<ObjectCache>,
    game: Res<GameConfig>,
    entities: Query<(&Transform, &Player, &ObjectType)>,
) {
    let mut drawing = drawing.drawing();

    for (transform, &player, &object_type) in entities.iter() {
        let flat_position = transform.translation.to_flat();
        let minimap_position = Vec2::new(
            flat_position.x - bounds.min().x,
            bounds.max().y - flat_position.y,
        ) / bounds.size();
        let color = if game.is_local_player(player) {
            PLAYER_COLOR
        } else {
            ENEMY_COLOR
        };

        let radius = cache.get_ichnography(object_type).radius();
        let rect_size = MIN_ENTITY_SIZE.max(Vec2::splat(radius) / bounds.size());
        drawing.rect(minimap_position, rect_size, color);
    }
}
