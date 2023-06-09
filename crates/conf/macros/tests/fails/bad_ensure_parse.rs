use conf_macros::Config;

#[derive(Config)]
pub struct Camera {
    #[is_finite]
    #[ensure(*move_margin > 0., "`move_margin` must be positive.")]
    pub move_margin: f32,

    #[ensure(*min_distance >= 10., "`min_distance` must be larger or equal to 10.0.")]
    pub min_distance: f32,

    #[ensure(Man I love rust!)]
    pub max_distance: f32,

    #[ensure(*wheel_zoom_sensitivity > 1., "`wheel_zoom_sensitivity` must be greater than 1.0.")]
    pub wheel_zoom_sensitivity: f32,

    #[ensure(*touchpad_zoom_sensitivity > 1., "`touchpad_zoom_sensitivity` must be greater than 1.0.")]
    pub touchpad_zoom_sensitivity: f32,

    #[ensure(*rotation_sensitivity > 0., "`rotation_sensitivity` must be greater than 0.0.")]
    pub rotation_sensitivity: f32,
}


fn main() {}
