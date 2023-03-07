use bevy::prelude::*;

pub struct StagesPlugin;

impl Plugin for StagesPlugin {
    fn build(&self, app: &mut App) {
        app.edit_schedule(CoreSchedule::Main, |schedule| {
            schedule.configure_sets(
                (
                    GameSet::Input,
                    GameSet::InputFlush,
                    GameSet::PreMovement,
                    GameSet::PreMovementFlush,
                    GameSet::Movement,
                    GameSet::MovementFlush,
                    GameSet::PostMovement,
                    GameSet::PostMovementFlush,
                    GameSet::PreUpdate,
                    GameSet::PreUpdateFlush,
                    GameSet::Update,
                    GameSet::UpdateFlush,
                    GameSet::PostUpdate,
                    GameSet::PostUpdateFlush,
                )
                    .chain(),
            );

            schedule.configure_sets((CoreSet::FixedUpdate, GameSet::Input).chain());
            schedule.configure_sets((GameSet::PostUpdateFlush, CoreSet::Update).chain());

            schedule.add_system(apply_system_buffers.in_base_set(GameSet::InputFlush));
            schedule.add_system(apply_system_buffers.in_base_set(GameSet::PreMovementFlush));
            schedule.add_system(apply_system_buffers.in_base_set(GameSet::MovementFlush));
            schedule.add_system(apply_system_buffers.in_base_set(GameSet::PostMovementFlush));
            schedule.add_system(apply_system_buffers.in_base_set(GameSet::PreUpdateFlush));
            schedule.add_system(apply_system_buffers.in_base_set(GameSet::UpdateFlush));
            schedule.add_system(apply_system_buffers.in_base_set(GameSet::PostUpdateFlush));
        });
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
#[system_set(base)]
pub enum GameSet {
    /// All user input is handled during this stage.
    Input,
    InputFlush,
    /// The game state is prepared for movement stage during this stage. The
    /// preparation includes, among other things, global path finding &
    /// planning related updates.
    PreMovement,
    PreMovementFlush,
    /// All of "game active" entity movement (changes to [`bevy::prelude::Transform`])
    /// happens during this stage (an in no other stage).
    ///
    /// "Game active" entities are those which impact the game dynamics. For
    /// example buildings, units or the terrain. Auxiliary entities, for
    /// example building drafts, might be moved during other stages.
    Movement,
    MovementFlush,
    /// This stage includes for example update to spatial index of movable
    /// objects.
    PostMovement,
    PostMovementFlush,
    /// This stage includes all necessary preparation for the game update, e.g.
    /// insertion of components which need to be present during game update.
    PreUpdate,
    PreUpdateFlush,
    /// Most of the (movement unrelated) game logic happens during this stage.
    /// For example unit AI, attacking, object health updates and so on.
    ///
    /// This is the only stage during which "game active" entities are
    /// (de)spawned.
    Update,
    UpdateFlush,
    /// For example this stage includes update to spatial index necessary due
    /// to (de)spawning of objects to the game.
    PostUpdate,
    PostUpdateFlush,
}
