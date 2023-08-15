use bevy::prelude::*;

use crate::MenuState;

pub(super) struct CurrentGamePlugin;

impl Plugin for CurrentGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(MenuState::Multiplayer), cleanup);
    }
}

#[derive(Resource)]
pub(super) struct GameNameRes(String);

impl GameNameRes {
    pub(super) fn new<S: ToString>(name: S) -> Self {
        Self(name.to_string())
    }

    pub(super) fn name_owned(&self) -> String {
        self.0.to_owned()
    }
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<GameNameRes>();
}
