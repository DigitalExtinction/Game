use bevy::prelude::{App, CoreStage, Plugin, StageLabel, SystemStage};

pub struct StagesPlugin;

impl Plugin for StagesPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_before(CoreStage::Update, GameStage::Input, SystemStage::parallel())
            .add_stage_before(
                CoreStage::Update,
                GameStage::PreMovement,
                SystemStage::parallel(),
            )
            .add_stage_before(
                CoreStage::Update,
                GameStage::Movement,
                SystemStage::parallel(),
            )
            .add_stage_before(
                CoreStage::Update,
                GameStage::PostMovement,
                SystemStage::parallel(),
            )
            .add_stage_before(
                CoreStage::Update,
                GameStage::PreUpdate,
                SystemStage::parallel(),
            )
            .add_stage_before(
                CoreStage::Update,
                GameStage::Update,
                SystemStage::parallel(),
            )
            .add_stage_before(
                CoreStage::Update,
                GameStage::PostUpdate,
                SystemStage::parallel(),
            );
    }
}

/// This enum defines execution stage labels. All are executed in-order just
/// before [`bevy::prelude::CoreStage::Update`].
#[derive(StageLabel)]
pub enum GameStage {
    /// All user input is handled during this stage.
    Input,
    /// The game state is prepared for movement stage during this stage. The
    /// preparation includes, among other things, global path finding &
    /// planning related updates.
    PreMovement,
    /// All of "game active" entity movement (changes to [`bevy::prelude::Transform`])
    /// happens during this stage (an in no other stage).
    ///
    /// "Game active" entities are those which impact the game dynamics. For
    /// example buildings, units or the terrain. Auxiliary entities, for
    /// example building drafts, might be moved during other stages.
    Movement,
    /// This stage includes for example update to spatial index of movable
    /// objects.
    PostMovement,
    /// This stage includes all necessary preparation for the game update, e.g.
    /// insertion of components which need to be present during game update.
    PreUpdate,
    /// Most of the (movement unrelated) game logic happens during this stage.
    /// For example unit AI, attacking, object health updates and so on.
    ///
    /// This is the only stage during which "game active" entities are
    /// (de)spawned.
    Update,
    /// For example this stage includes update to spatial index necessary due
    /// to (de)spawning of objects to the game.
    PostUpdate,
}
