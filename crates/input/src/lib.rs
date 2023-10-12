mod key;

use std::hash::Hash;

use bevy::prelude::App;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::key::KeyPlugin;

pub trait KeyBinding:
    Copy + Eq + Hash + Send + Sync + Serialize + DeserializeOwned + 'static
{
}

impl<T: Copy + Eq + Hash + Send + Sync + Serialize + DeserializeOwned + 'static> KeyBinding for T {}

pub trait ActionTrait {
    type InputType: KeyBinding;
}

pub trait AppKeybinding {
    /// Add a keybinding to the app.
    ///
    /// # Arguments
    /// * `E` - The event type to be sent when the keybinding is pressed.
    /// * `I` - The type of Input.
    /// * `default` - The default keybinding.
    /// * `config_name` - The name of the keybinding in the config file.
    fn add_keybinding<K: ActionTrait + Send + Sync + 'static>(
        &mut self,
        default_keys: impl IntoKeys<K::InputType>,
        config_name: String,
    ) -> &mut Self;
}

pub trait IntoKeys<T: KeyBinding> {
    fn into_keys(self) -> Vec<T>;
}

impl<T: KeyBinding> IntoKeys<T> for T {
    fn into_keys(self) -> Vec<T> {
        vec![self]
    }
}

impl<T: KeyBinding> IntoKeys<T> for Vec<T> {
    fn into_keys(self) -> Vec<T> {
        self
    }
}

impl AppKeybinding for App {
    fn add_keybinding<K: ActionTrait + Send + Sync + 'static>(
        &mut self,
        default_keys: impl IntoKeys<K::InputType>,
        config_name: String,
    ) -> &mut Self {
        self.add_plugins(KeyPlugin::<K>::new(default_keys.into_keys(), config_name))
    }
}
