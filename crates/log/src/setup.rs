use bevy::prelude::*;

pub(crate) struct LogPlugin {}

#[derive(Resource)]
pub(crate) struct CurrentLogFile; // TODO: something meaningful with this

impl Plugin for LogPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentLogFile);
    }
}
