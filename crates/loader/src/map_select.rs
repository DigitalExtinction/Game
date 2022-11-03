use bevy::prelude::*;
use de_core::{gconfig::GameConfig, player::Player, state::GameState};
use iyes_loopless::{prelude::IntoConditionalSystem, state::NextState};

pub struct MapSelectPlugin;

impl Plugin for MapSelectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapList>();
        app.add_system(cycle_map.run_in_state(GameState::Playing));
    }
}

pub struct MapList {
    pub maps: Vec<String>,
    pub current_id: usize,
}

impl Default for MapList {
    fn default() -> Self {
        Self {
            maps: vec!["map.tar".into(), "de_map_test.tar".into()],
            current_id: 0,
        }
    }
}

fn cycle_map(
    mut commands: Commands,
    mut config: ResMut<GameConfig>,
    mut map_list: ResMut<MapList>,
    kbd: Res<Input<KeyCode>>,
) {
    if kbd.just_pressed(KeyCode::M) {
        map_list.current_id = (map_list.current_id + 1) % map_list.maps.len();
        *config = GameConfig::new(&map_list.maps[map_list.current_id], Player::Player1);
        commands.insert_resource(NextState(GameState::Loading));
    }
}
