use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};

/// Builder of keyboard events & state based system execution condition.
#[derive(Copy, Clone)]
pub(crate) struct KeyCondition {
    control: bool,
    shift: bool,
    key: KeyCode,
}

impl KeyCondition {
    /// Run if a key is pressed and control is not.
    pub(crate) fn single(key: KeyCode) -> Self {
        Self {
            control: false,
            shift: false,
            key,
        }
    }

    /// Run if a key is pressed together with control.
    pub(crate) fn with_ctrl(mut self) -> Self {
        self.control = true;
        self
    }

    /// Run if a key is pressed together with shift.
    pub(crate) fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub(crate) fn build(self) -> impl Fn(Res<Input<KeyCode>>, EventReader<KeyboardInput>) -> bool {
        move |keys: Res<Input<KeyCode>>, mut events: EventReader<KeyboardInput>| {
            let proper_key = events
                .iter()
                .filter(|k| {
                    k.state == ButtonState::Pressed && k.key_code.map_or(false, |c| c == self.key)
                })
                .count()
                > 0;

            let control = keys.pressed(KeyCode::LControl) || keys.pressed(KeyCode::RControl);
            let shift = keys.pressed(KeyCode::LShift) || keys.pressed(KeyCode::RShift);
            self.control == control && shift == self.shift && proper_key
        }
    }
}
