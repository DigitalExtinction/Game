#![allow(unused_variables)]
#![allow(clippy::disallowed_names)]

use anyhow::ensure;
use conf_macros::Config;
use serde::Deserialize;

#[derive(Config, Default)]
pub struct TestConfig {
    pub field1: u32,
    #[check(|v: u32| Ok(ensure!(*v > 0, "foo")))]
    #[check(|v: u32| Ok(ensure!(*v < 10, "bar")))]
    pub field2: u32,
    #[check(|v: u32| Ok(ensure!(*v > 0, "baz")))]
    pub field3: u32,
}

#[derive(Deserialize, Default, Config, Debug)]
pub struct Camera {
    #[is_finite]
    #[ensure(*move_margin > 0., "`move_margin` must be positive.")]
    pub move_margin: f32,

    #[ensure(*min_distance >= 10., "`min_distance` must be larger or equal to 10.0.")]
    pub min_distance: f32,

    #[ensure(*max_distance <= 300., "`max_distance` must be smaller or equal to 300.0.")]
    pub max_distance: f32,

    #[ensure(*wheel_zoom_sensitivity > 1., "`wheel_zoom_sensitivity` must be greater than 1.0.")]
    pub wheel_zoom_sensitivity: f32,

    #[ensure(*touchpad_zoom_sensitivity > 1., "`touchpad_zoom_sensitivity` must be greater than 1.0.")]
    pub touchpad_zoom_sensitivity: f32,

    #[ensure(*rotation_sensitivity > 0., "`rotation_sensitivity` must be greater than 0.0.")]
    pub rotation_sensitivity: f32,
}

#[derive(Config, Default)]
pub struct ComplexConfig {
    #[ensure(foo <= bar, "foo")]
    #[ensure(foo <= baz, "bar")]
    pub foo: u32,
    #[ensure(bar >= foo, "baz")]
    pub bar: u32,
    #[ensure(baz >= foo, "qux")]
    pub baz: u32,
}

#[test]
fn test_derive_config() {
    let config = TestConfig {
        field1: 1,
        field2: 2,
        field3: 3,
    };
    assert!(dbg!(config.check().is_ok()));
}

#[test]
fn test_derive_config_fail() {
    let config = TestConfig {
        field1: 1,
        field2: 20,
        field3: 3,
    };
    let check = config.check();
    assert!(dbg!(check.is_err()));
    // assert_eq!(
    //     dbg!(check.unwrap_err().to_string()),
    //     Err(vec![
    //         ("{} failed check. value {:?} did not pass closure {}, Error: {}",),
    //     ])
    // );
}

#[test]
fn test_derive_config_default_camera() {
    let config = Camera {
        move_margin: 1.,
        min_distance: 10.,
        max_distance: 80.,
        wheel_zoom_sensitivity: 1.1,
        touchpad_zoom_sensitivity: 1.01,
        rotation_sensitivity: 0.008,
    };
    assert!(dbg!(config.check()).is_ok());
}

#[test]
fn test_derive_config_default_camera_fail() {
    let config = Camera {
        move_margin: 1.,
        min_distance: 10.,
        max_distance: 80.,
        wheel_zoom_sensitivity: 1.1,
        touchpad_zoom_sensitivity: 1.01,
        rotation_sensitivity: 0.,
    };
    assert_eq!(&config.check().unwrap_err()[0].0,
                "rotation_sensitivity failed check. value 0.0 did not pass closure (| rotation_sensitivity : & f32 |\n{\n    ensure!\n    (* rotation_sensitivity > 0.,\n    \"`rotation_sensitivity` must be greater than 0.0.\") ; Ok(())\n}), Error: `rotation_sensitivity` must be greater than 0.0.",);
}

#[test]
fn test_derive_config_default_camera_fail_finite() {
    let mut config = Camera {
        move_margin: 1.,
        min_distance: 10.,
        max_distance: 80.,
        wheel_zoom_sensitivity: 1.1,
        touchpad_zoom_sensitivity: 1.01,
        rotation_sensitivity: f32::INFINITY,
    };
    config.max_distance = 300.;
    assert!(dbg!(config.check()).is_err());
}

#[test]
fn test_derive_config_default_complex() {
    let config = ComplexConfig {
        foo: 1,
        bar: 2,
        baz: 3,
    };
    assert!(dbg!(config.check()).is_ok());
}

#[test]
fn fails() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fails/*.rs");
}
