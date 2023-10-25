use std::io::Read;

use de_core::fs::conf_dir;
use leafwing_input_manager::prelude::InputMap;
use ron::ser::PrettyConfig;

use crate::BindableActionlike;

pub(crate) fn get_keybindings<A: BindableActionlike>(
    action_set_name: String,
    default_keybindings: InputMap<A>,
) -> InputMap<A> {
    let mut file = match std::fs::File::open(
        conf_dir()
            .expect("Could not get config dir")
            .join(format!("keybindings.{}.ron", action_set_name)),
    ) {
        Ok(file) => file,
        Err(_) => {
            std::fs::write(
                conf_dir()
                    .expect("Could not get config dir")
                    .join(format!("keybindings.{}.ron", action_set_name)),
                ron::ser::to_string_pretty(&default_keybindings, PrettyConfig::new()).unwrap(),
            )
            .unwrap();
            return default_keybindings;
        }
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let mut keybindings: InputMap<A> = ron::from_str(&contents).unwrap();

    // fill unset keys with default keybindings
    keybindings.merge(&default_keybindings);

    keybindings
}
