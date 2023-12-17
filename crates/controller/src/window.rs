use bevy::{
    prelude::*,
    window::{CursorGrabMode, WindowMode},
};
use de_conf::Configuration;
use de_core::state::AppState;

pub struct WindowManagementPlugin;

impl Plugin for WindowManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnExit(AppState::AppLoading),
            update_window_mode_based_on_config,
        )
        .add_systems(Update, toggle_fullscreen);
    }
}

fn update_window_mode_based_on_config(mut windows: Query<&mut Window>, config: Res<Configuration>) {
    let window = &mut windows.single_mut();

    window.mode = config.window().mode();
    match window.mode {
        WindowMode::BorderlessFullscreen => {
            if cfg!(target_os = "macos") {
                window.cursor.grab_mode = CursorGrabMode::None;
            } else {
                window.cursor.grab_mode = CursorGrabMode::Confined;
            }
        }
        _ => {
            window.cursor.grab_mode = CursorGrabMode::None;
        }
    };
}

fn toggle_fullscreen(mut windows: Query<&mut Window>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::F11) {
        let window = &mut windows.single_mut();
        match window.mode {
            WindowMode::BorderlessFullscreen => {
                window.mode = WindowMode::Windowed;
                window.cursor.grab_mode = CursorGrabMode::None;
            }
            _ => {
                window.mode = WindowMode::BorderlessFullscreen;

                if cfg!(target_os = "macos") {
                    window.cursor.grab_mode = CursorGrabMode::None;
                } else {
                    window.cursor.grab_mode = CursorGrabMode::Confined;
                }
            }
        };
    }
}
