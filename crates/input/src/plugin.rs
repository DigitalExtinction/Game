//! Contains main plugin exported by this crate.

use core::marker::PhantomData;

use bevy::app::{App, Plugin};
use bevy::ecs::prelude::*;
use bevy::input::InputSystem;
use bevy::prelude::PostUpdate;
use de_core::schedule::PreInputSchedule;
use leafwing_input_manager::plugin::InputManagerSystem;
use leafwing_input_manager::prelude::{ActionState, ClashStrategy, ToggleActions};
use leafwing_input_manager::Actionlike;

/// A [`Plugin`] that collects [`Input`](bevy::input::Input)
/// from disparate sources, producing an [`ActionState`] that
/// can be conveniently checked
///
/// This plugin needs to be passed in an [`Actionlike`] enum type
/// that you've created for your game. Each variant represents a
/// "virtual button" whose state is stored in an [`ActionState`] struct.
///
/// Each [`InputManagerBundle`](crate::InputManagerBundle) contains:
///  -  an [`InputMap`](crate::input_map::InputMap) component, which
///     stores an entity-specific mapping between the assorted input
///     streams and an internal representation of "actions"
///  -  an [`ActionState`] component, which stores the current
///     input state for that entity in an source-agnostic fashion
///
/// If you have more than one distinct type of action
/// (e.g. menu actions, camera actions and player actions),
/// consider creating multiple `Actionlike` enums
/// and adding a copy of this plugin for each `Actionlike` type.
///
/// ## Systems
///
/// All systems added by this plugin can be dynamically enabled
/// and disabled by setting the value of the [`ToggleActions<A>`] resource is set.
/// This can be useful when working with states to pause the game,
/// navigate menus or so on.
///
/// Complete list:
///
/// -   [`tick_action_state`](leafwing_input_manager::systems::tick_action_state),
///     which resets the `pressed` and `just_pressed` fields of
///     the [`ActionState`] each frame
/// -   [`update_action_state`](leafwing_input_manager::systems::update_action_state),
///     which collects [`Input`](bevy::input::Input) resources to update
///     the [`ActionState`]
/// -   [`update_action_state_from_interaction`](leafwing_input_manager::systems::update_action_state_from_interaction),
///     for triggering actions from buttons
///     -   powers the [`ActionStateDriver`](leafwing_input_manager::action_state::ActionStateDriver)
///         component based on an [`Interaction`](bevy::ui::Interaction)
///         component
/// -   [`release_on_disable`](leafwing_input_manager::systems::release_on_disable),
///     which resets action states when [`ToggleActions`] is flipped, to avoid persistent presses.
pub struct InputManagerPlugin<A: Actionlike> {
    _phantom: PhantomData<A>,
}

// Deriving default induces an undesired bound on the generic
impl<A: Actionlike> Default for InputManagerPlugin<A> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<A: Actionlike> Plugin for InputManagerPlugin<A> {
    fn build(&self, app: &mut App) {
        use leafwing_input_manager::systems::*;

        app.add_systems(
            PreInputSchedule,
            tick_action_state::<A>
                .run_if(run_if_enabled::<A>)
                .in_set(InputManagerSystem::Tick)
                .before(InputManagerSystem::Update),
        )
        .add_systems(
            PreInputSchedule,
            release_on_disable::<A>
                .in_set(InputManagerSystem::ReleaseOnDisable)
                .after(InputManagerSystem::Update),
        )
        .add_systems(PostUpdate, release_on_input_map_removed::<A>);

        app.add_systems(
            PreInputSchedule,
            update_action_state::<A>.in_set(InputManagerSystem::Update),
        );

        app.configure_set(
            PreInputSchedule,
            InputManagerSystem::Update
                .run_if(run_if_enabled::<A>)
                .after(InputSystem),
        );

        app.register_type::<ActionState<A>>()
            // Resources
            .init_resource::<ToggleActions<A>>()
            .insert_resource(ClashStrategy::UseActionOrder);
    }
}
