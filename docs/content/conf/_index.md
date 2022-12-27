+++
title = "DE Configuration"
weight = 4
sort_by = "weight"
+++

Game configuration is stored into and loaded from a YAML file located at
`{user_conf_dir}/DigitalExtinction/conf.yaml`. `{user_conf_dir}` is obtained with
[dirs::config_dir()](https://docs.rs/dirs/4.0.0/dirs/fn.config_dir.html) and
conforms to the following table:

|Platform | Value                                 | Example                                  |
| ------- | ------------------------------------- | ---------------------------------------- |
| Linux   | `$XDG_CONFIG_HOME` or `$HOME`/.config | /home/alice/.config                      |
| macOS   | `$HOME`/Library/Application Support   | /Users/Alice/Library/Application Support |
| Windows | `{FOLDERID_RoamingAppData}`           | C:\Users\Alice\AppData\Roaming           |

# Configuration YAML

All properties in the YAML tree are optional, default values are used instead.
Missing configuration YAML file is treated equally to an empty YAML file, id
est as if all properties are missing.

* `multiplayer` (object) – multiplayer and network configuration.
  * `server` (string; default: `http://lobby.de-game.org`) – lobby server base URL.
