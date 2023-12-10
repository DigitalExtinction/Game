use bevy::input::keyboard::KeyCode;
use bevy::prelude::Reflect;
use bevy::prelude::{MouseButton, Res};
use de_input::{AppKeybinding, DefaultKeybindings};
use leafwing_input_manager::prelude::{ActionState, InputMap, QwertyScanCode, UserInput};
use leafwing_input_manager::Actionlike;
use serde::{Deserialize, Serialize};

pub struct ActionPlugin;

impl bevy::app::Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_action_set::<Action>();
    }
}

use std::collections::HashMap;

use bevy::app::App;
use petitset::PetitSet;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Deserialize,
    Serialize,
    Actionlike,
    Reflect,
    PartialOrd,
    Ord,
)]
pub enum Action {
    Exit,
    SelectAllVisible,
    SelectAll,
    AddToSelection,
    ReplaceSelection,
    Up,
    Down,
    Left,
    Right,
    Pivot,
    BuildBase,
    BuildPowerHub,
}
impl DefaultKeybindings for Action {
    fn default_keybindings() -> InputMap<Self>
    where
        Self: Sized,
    {
        use Action::*;
        let keybindings = InputMap::<Self>::from(
            vec![
                (Exit, vec![KeyCode::Escape.into()]),
                (
                    SelectAllVisible,
                    vec![UserInput::chord(vec![
                        KeyCode::ControlLeft,
                        KeyCode::ShiftLeft,
                        KeyCode::A,
                    ])],
                ),
                (
                    SelectAll,
                    vec![UserInput::chord(vec![KeyCode::ControlLeft, KeyCode::A])],
                ),
                (
                    AddToSelection,
                    vec![
                        UserInput::Chord(PetitSet::from_iter(vec![
                            KeyCode::ControlLeft.into(),
                            MouseButton::Left.into(),
                        ])),
                        UserInput::Chord(PetitSet::from_iter(vec![
                            KeyCode::ControlRight.into(),
                            MouseButton::Left.into(),
                        ])),
                    ],
                ),
                (ReplaceSelection, vec![MouseButton::Left.into()]),
                (Up, vec![QwertyScanCode::W.into(), KeyCode::Up.into()]),
                (Down, vec![QwertyScanCode::S.into(), KeyCode::Down.into()]),
                (Left, vec![QwertyScanCode::A.into(), KeyCode::Left.into()]),
                (Right, vec![QwertyScanCode::D.into(), KeyCode::Right.into()]),
                (
                    Pivot,
                    vec![UserInput::Chord(PetitSet::from_iter(vec![
                        KeyCode::ControlLeft.into(),
                        MouseButton::Middle.into(),
                    ]))],
                ),
                (BuildBase, vec![KeyCode::B.into()]),
                (BuildPowerHub, vec![KeyCode::P.into()]),
            ]
            .into_iter()
            .collect::<HashMap<Self, Vec<UserInput>>>(),
        );
        println!("keybindings: {:?}", keybindings);
        keybindings
    }
}

impl Action {
    pub fn get_factory_actions() -> Vec<(Self, de_types::objects::BuildingType)> {
        use de_types::objects::BuildingType::*;
        use Action::*;

        vec![(BuildBase, Base), (BuildPowerHub, PowerHub)]
    }
}

pub(crate) fn action_just_pressed<A: Actionlike>(
    action: A,
) -> impl Fn(Res<ActionState<A>>) -> bool {
    move |action_state: Res<ActionState<A>>| action_state.just_pressed(action.clone())
}

pub(crate) fn action_pressed<A: Actionlike>(action: A) -> impl Fn(Res<ActionState<A>>) -> bool {
    move |action_state: Res<ActionState<A>>| action_state.pressed(action.clone())
}
