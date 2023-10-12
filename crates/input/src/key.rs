use std::hash::Hash;
use std::marker::PhantomData;

use bevy::app::PreUpdate;
use bevy::input::InputSystem;
use bevy::prelude::{App, Input, IntoSystemConfigs, Plugin, Res, ResMut, Resource, SystemSet};

use crate::ActionTrait;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, SystemSet)]
pub struct ActionSet;

#[derive(Resource)]
pub struct Action<A: ActionTrait> {
    inputs: Vec<A::InputType>,
    is_currently_pressed: bool,
    just_pressed: bool,
    just_released: bool,
    _marker: PhantomData<A>,
}

impl<A: ActionTrait> Action<A> {
    pub fn inputs(&self) -> &Vec<A::InputType> {
        &self.inputs
    }
}

pub struct KeyPlugin<A: ActionTrait + Send + Sync + 'static> {
    keys: Vec<A::InputType>,
    #[allow(dead_code)] // TODO: Remove this and use this field
    config_name: String,
    _marker: PhantomData<A>,
}

impl<A: ActionTrait + Send + Sync + 'static> KeyPlugin<A> {
    pub fn new(keys: Vec<A::InputType>, config_name: String) -> Self {
        Self {
            keys,
            config_name,
            _marker: PhantomData,
        }
    }
}

impl<A: ActionTrait + Send + Sync + 'static> Plugin for KeyPlugin<A> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            keyboard_input::<A>.after(InputSystem).in_set(ActionSet),
        )
        .insert_resource(Action::<A> {
            inputs: self.keys.clone(),
            is_currently_pressed: false,
            just_pressed: false,
            just_released: false,
            _marker: PhantomData,
        });
    }
}

fn keyboard_input<A: ActionTrait + Send + Sync + 'static>(
    input: Res<Input<A::InputType>>,
    mut action: ResMut<Action<A>>,
) {
    let mut just_pressed = false;
    let mut just_released = false;
    let mut is_currently_pressed = false;

    for key in action.inputs() {
        if input.just_pressed(*key) {
            just_pressed = true;
        }

        if input.just_released(*key) {
            just_released = true;
        }

        if input.pressed(*key) {
            is_currently_pressed = true;
        }
    }

    action.just_pressed = just_pressed;
    action.just_released = just_released;
    action.is_currently_pressed = is_currently_pressed;
}

#[cfg(test)]
mod tests {
    use bevy::input::InputPlugin;
    use bevy::prelude::KeyCode;
    use bevy::prelude::KeyCode::Key0;

    use super::*;
    use crate::AppKeybinding;

    #[test]
    fn test_keybinding() {
        struct TestAction;

        impl ActionTrait for TestAction {
            type InputType = KeyCode;
        }

        let mut app = App::new();

        app.add_plugins(InputPlugin);

        app.add_keybinding::<TestAction>(Key0, "test_key".to_string());

        app.update();
        let action = app.world.get_resource::<Action<TestAction>>().unwrap();

        assert!(!action.just_pressed);
        assert!(!action.just_released);
        assert!(!action.is_currently_pressed);

        fn test_press_key(mut keyboard_input_events: ResMut<Input<KeyCode>>) {
            keyboard_input_events.press(Key0);
        }

        app.add_systems(
            PreUpdate,
            test_press_key.before(ActionSet).after(InputSystem),
        );

        app.update();

        let action = app.world.get_resource::<Action<TestAction>>().unwrap();
        let keyboard_input_events = app.world.get_resource::<Input<KeyCode>>().unwrap();

        println!("{:?}", keyboard_input_events);

        assert!(action.just_pressed);
        assert!(!action.just_released);
        assert!(action.is_currently_pressed);
    }
}
