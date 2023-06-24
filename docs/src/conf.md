# Configuration

Game configuration is stored into and loaded from a YAML file located at
`{user_conf_dir}/DigitalExtinction/conf.yaml`. `{user_conf_dir}` is obtained
with [dirs::config_dir()](https://docs.rs/dirs/latest/dirs/fn.config_dir.html)
and conforms to the following table:

|Platform | Value                                 | Example                                  |
| ------- | ------------------------------------- | ---------------------------------------- |
| Linux   | `$XDG_CONFIG_HOME` or `$HOME`/.config | /home/alice/.config                      |
| macOS   | `$HOME`/Library/Application Support   | /Users/Alice/Library/Application Support |
| Windows | `{FOLDERID_RoamingAppData}`           | C:\Users\Alice\AppData\Roaming           |

## Configuration YAML

All properties in the YAML tree are optional, default values are used instead.
Missing configuration YAML file is treated equally to an empty YAML file, id
est as if all properties are missing.

* `multiplayer` (object) – multiplayer and network configuration.
  * `server` (string; default: `http://lobby.de-game.org`) – lobby server base URL.
* `camera` (object) – in-game camera configuration.
  * `move_margin` (f32; default: `40.0`) – horizontal camera movement is
    initiated if mouse is withing this distance in logical pixels to a window
    edge. It must be a finite positive number.
  * `min_distance` (f32; default: `20.0`) – minimum camera distance from the
    terrain. It must be a finite number larger or equal to `10.0`.
  * `max_distance` (f32; default: `80.0`) – maximum camera distance from the
    terrain. It must be a finite number larger or equal to `min_distance` and
    smaller or equal to `300.0`.
  * `wheel_zoom_sensitivity` (f32; default: `1.1`) – camera to terrain distance
    scale factor used during mouse wheel zooming. The distance is changed to
    `current_distance * wheel_zoom_sensitivity` (zoom out) or `current_distance
    / wheel_zoom_sensitivity` (zoom in) with each mouse wheel tick. It must be
    a finite number larger than `1.0`.
  * `touchpad_zoom_sensitivity` (f32; default: `1.01`) – camera to terrain
    distance scale factor used during touchpad zooming. The distance is changed
    to `current_distance * touchpad_zoom_sensitivity` (zoom out) or
    `current_distance / touchpad_zoom_sensitivity` (zoom in) with each pixel
    movement. It must be a finite number larger than `1.0`.
  * `rotation_sensitivity` (f32; default: `0.008`) – multiplicative factor used
    during camera tilting and rotation. Mouse drag by `delta` logical pixels
    leads to the change of elevation and azimuth by `delta *
    rotation_sensitivity` radians. It must be a positive finite number.
  * `scroll_inverted` (bool; default: `false`) – if `true`, mouse wheel and
    touchpad scrolling is inverted.
* `audio` (object) – audio configuration.
  * `music_volume` (f32; default: `1.0`) – sets the music volume. It must be a finite
    number between `0.0` and `1.0`. If set to 0 music will not play.