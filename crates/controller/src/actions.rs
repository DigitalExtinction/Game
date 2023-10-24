use bevy::input::keyboard::KeyCode;
use bevy::prelude::Reflect;
use bevy::prelude::{Commands, MouseButton, Res, Startup};
use de_input::{AppKeybinding, DefaultKeybindings};
use leafwing_input_manager::prelude::DualAxis;
use leafwing_input_manager::prelude::{ActionState, InputMap, UserInput};
use leafwing_input_manager::Actionlike;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

pub struct ActionPlugin;

impl bevy::app::Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_action_set::<Action>("actions");
    }
}

/// make actoinlike enum that has normal actions and factory actions. an action is A vareint followed by a KeyConfig<A>
macro_rules! make_actions {
    {$($action:ident, ($($keybind:expr),*)),*; $($mouse_action:ident, ($($mouse_keybind:expr),*)),*; $($building_action:ident, $building_type:ident, ($($building_key:expr),*)),*} => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, Actionlike, Reflect, PartialOrd, Ord)]
        pub enum Action {
            $($action,)*
            $($mouse_action,)*
            $($building_action,)*
        }

        impl DefaultKeybindings for Action {
            fn default_keybindings() -> InputMap<Self> where Self: Sized {
                use Action::*;
                let keybindings = InputMap::<Self>::from(
                    vec![
                        $(($action, vec![$($keybind.into()),*])),*,
                        $(($mouse_action, vec![$($mouse_keybind.into()),*])),*,
                        $(($building_action, vec![$($building_key.into()),*])),*
                    ].into_iter().collect::<HashMap<Self, Vec<UserInput>>>()
                );
                keybindings
            }
        }

        impl Action {
            pub fn get_factory_actions() -> Vec<(Self, de_types::objects::BuildingType)> {
                use Action::*;
                use de_types::objects::BuildingType::*;

                vec![$(($building_action, $building_type)),*]
            }

            pub fn get_mouse_actions() -> Vec<Self> {
                use Action::*;
                vec![$($mouse_action),*]
            }
        }
    }
}

use bevy::app::App;
use petitset::PetitSet;
use std::collections::HashMap;

make_actions! {
    // --- general actions ---
    // keyboard actions
    Exit, (KeyCode::Escape),
    SelectAllVisible, (UserInput::chord(vec![KeyCode::ControlLeft, KeyCode::ShiftLeft, KeyCode::A])),
    SelectAll, (UserInput::chord(vec![KeyCode::ControlLeft, KeyCode::A])),
    // mouse selections
    AddToSelection, (
        UserInput::Chord(PetitSet::from_iter(vec![KeyCode::ControlLeft.into(),MouseButton::Left.into()])),
        UserInput::Chord(PetitSet::from_iter(vec![KeyCode::ControlRight.into(), MouseButton::Left.into()]))),
    ReplaceSelection, (MouseButton::Left),
    // camera controls
    Up, (KeyCode::W, KeyCode::Up),
    Down, (KeyCode::S, KeyCode::Down),
    Left, (KeyCode::A, KeyCode::Left),
    Right, (KeyCode::D, KeyCode::Right),
    Pivot, (UserInput::Chord(PetitSet::from_iter(vec![KeyCode::ControlLeft.into(), MouseButton::Middle.into()])));
    // --- mouse actions (these will trigger the drag logic) ---
    PrimaryClick, (MouseButton::Left),
    SecondaryClick, (MouseButton::Right);
    //  --- building actions ---
    BuildBase, Base, (KeyCode::B),
    BuildPowerHub, PowerHub, (KeyCode::P)
}

pub(crate) fn action_pressed<A: Actionlike>(action: A) -> impl Fn(Res<ActionState<A>>) -> bool {
    move |action_state: Res<ActionState<A>>| action_state.pressed(action.clone())
}
