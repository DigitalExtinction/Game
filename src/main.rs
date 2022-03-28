use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::WindowMode,
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use de::{camera, terrain};
use std::f32::consts::{FRAC_PI_2, PI};

// u32 needs to be converted to usize at various places. Make sure that this
// module is not complided for samller pointer width.
#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("`target_pointer_width` has to be at least 32 bits.");

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Digital Extinction".to_string(),
            mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EguiPlugin)
        .add_startup_system(setup)
        .add_startup_system(spawn_tree)
        .add_startup_system(camera::setup)
        .add_system(camera::mouse_movement.label("mouse.movement"))
        .add_system(camera::mouse_wheel.label("mouse.wheel"))
        .add_system(
            camera::move_camera
                .after("mouse.movement")
                .after("mouse.wheel"),
        )
        .add_system(ui_example)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.6,
    });

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(terrain::mesh::build_mesh()),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.1, 0.8, 0.3),
                perceptual_roughness: 1.0,
                ..Default::default()
            }),
            ..Default::default()
        })
        .insert(terrain::components::Terrain);

    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });

    let map_size = 10.0;
    let midmap = Vec3::new(5., 0., 5.);
    let sun_elevation: f32 = 0.6 * 3.14 / 2.;
    let sun_transform = Transform {
        translation: midmap,
        rotation: Quat::from_euler(EulerRot::YXZ, FRAC_PI_2, -sun_elevation, 0.),
        ..Default::default()
    };

    println!("{:?} {:?}", sun_transform.forward(), sun_transform.up());

    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 30000.,
            shadow_projection: OrthographicProjection {
                left: -map_size / 2.,
                right: map_size / 2.,
                bottom: -map_size / 2.,
                top: map_size / 2.,
                near: -map_size,
                far: map_size,
                ..Default::default()
            },
            shadow_depth_bias: 0.2,
            shadow_normal_bias: 0.2,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: sun_transform,
        ..Default::default()
    });
}

fn spawn_tree(mut commands: Commands, ass: Res<AssetServer>) {
    // note that we have to include the `Scene0` label
    let my_gltf = ass.load("tree01-v001.glb#Scene0");

    // to be able to position our 3d model:
    // spawn a parent entity with a Transform and GlobalTransform
    // and spawn our gltf as a scene under it
    commands
        .spawn_bundle((
            Transform::from_xyz(2.0, 0.0, 5.0),
            GlobalTransform::identity(),
        ))
        .with_children(|parent| {
            parent.spawn_scene(my_gltf.clone());
        });

    let mut second_tree_transform = Transform::from_xyz(4.7, 0.0, 4.1);
    second_tree_transform.rotate(Quat::from_rotation_y(0.4 * PI));
    commands
        .spawn_bundle((second_tree_transform, GlobalTransform::identity()))
        .with_children(|parent| {
            parent.spawn_scene(my_gltf);
        });
}

fn ui_example(mut egui_context: ResMut<EguiContext>) {
    egui::Window::new("Hello").show(egui_context.ctx_mut(), |ui| {
        ui.label("world");
    });
}
