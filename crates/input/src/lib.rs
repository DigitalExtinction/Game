use bevy::prelude::App;
use bevy::prelude::Reflect;
use std::collections::HashMap;
use leafwing_input_manager::prelude::{ActionState, InputMap};
use leafwing_input_manager::{Actionlike, InputManagerBundle};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use serde::de::DeserializeOwned;
use crate::plugin::InputManagerPlugin;

mod io;
mod plugin;


pub trait BindableActionlike:
    Actionlike + DeserializeOwned + Serialize + Clone + Send + Sync + Eq + Hash + Ord
{
}

impl<T: Actionlike + DeserializeOwned + Serialize + Clone + Send + Sync + Eq + Hash + Ord>
    BindableActionlike for T
{
}

pub trait DefaultKeybindings: BindableActionlike {
    fn default_keybindings() -> InputMap<Self> where Self: Sized;
}

pub trait AppKeybinding {
    /// Add a keybinding with config to the app.
    fn add_action_set<A: BindableActionlike + DefaultKeybindings>(
        &mut self,
        config_name: impl Into<String>,
    ) -> &mut Self;
}

impl AppKeybinding for App {
    fn add_action_set<A: BindableActionlike + DefaultKeybindings>(
        &mut self,
        config_name: impl Into<String>,
    ) -> &mut Self {
        let keybindings: InputMap<A> = io::get_keybindings(config_name.into(), A::default_keybindings());
        self.world.insert_resource(keybindings);
        self.world.insert_resource(ActionState::<A>::default());

        self.add_plugins(InputManagerPlugin::<A>::default());

        self
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::KeyCode;
    use leafwing_input_manager::prelude::UserInput;
    use leafwing_input_manager::user_input::InputKind::Keyboard;

    #[test]
    fn test_keybindings() {
        let mut app = App::new();
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, Actionlike, Reflect, PartialOrd, Ord
        )]
        enum PlayerAction {
            // Movement
            Up,
            Down,
            Left,
            Right,
            // Abilities
            Ability1,
            Ability2,
            Ability3,
            Ability4,
            Ultimate,
        }

        impl DefaultKeybindings for PlayerAction {
            fn default_keybindings() -> InputMap<Self> where Self: Sized {
                InputMap::from(
                    vec![
                        (Self::Up, vec![UserInput::Single(Keyboard(KeyCode::W))]),
                        (Self::Down, vec![UserInput::Single(Keyboard(KeyCode::S))]),
                        (Self::Left, vec![UserInput::Single(Keyboard(KeyCode::A))]),
                        (Self::Right, vec![UserInput::Single(Keyboard(KeyCode::D))]),
                        (Self::Ability1, vec![UserInput::Single(Keyboard(KeyCode::Q))]),
                        (Self::Ability2, vec![UserInput::Single(Keyboard(KeyCode::E))]),
                        (Self::Ability3, vec![UserInput::Single(Keyboard(KeyCode::F))]),
                        (Self::Ability4, vec![UserInput::Single(Keyboard(KeyCode::R))]),
                        (Self::Ultimate, vec![UserInput::Single(Keyboard(KeyCode::Space))]),
                    ].into_iter().collect::<HashMap<Self, Vec<UserInput>>>()
                )
            }
        }

        app.add_action_set::<PlayerAction>(
            "player".to_string(),
        );

        app.update();
    }
}
