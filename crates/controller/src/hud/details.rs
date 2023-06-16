use crate::selection::Selected;
use bevy::prelude::*;
use bevy::text::BreakLineOn;
use de_core::{cleanup::DespawnOnGameExit, gamestate::GameState};
use de_core::baseset::GameSet;
use de_energy::Battery;
use de_gui::TextProps;


use super::{interaction::InteractionBlocker, HUD_COLOR};

pub(crate) struct DetailsPlugin;

impl Plugin for DetailsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(GameState::Playing))).add_system(
            update
                .in_base_set(GameSet::PostUpdate)
                .run_if(in_state(GameState::Playing))
        );
    }
}

#[derive(Resource)]
struct DetailsNode(Entity);

fn setup(mut commands: Commands) {
    let entity = commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size {
                        width: Val::Percent(20.),
                        height: Val::Percent(30.),
                    },
                    position_type: PositionType::Absolute,
                    position: UiRect::new(
                        Val::Percent(0.),
                        Val::Percent(20.),
                        Val::Percent(70.),
                        Val::Percent(100.),
                    ),
                    ..default()
                },
                background_color: HUD_COLOR.into(),
                ..default()
            },
            DespawnOnGameExit,
            InteractionBlocker,
        ))
        .id();

    commands.insert_resource(DetailsNode(entity));
}

fn update(
    ui: Res<DetailsNode>,
    selected: Query<Entity, With<Selected>>,
    battery: Query<&Battery>,
    font: Res<TextProps>,
    mut commands: Commands,
) {
    if selected.is_empty() {
        commands.entity(ui.0).despawn_descendants();
        return;
    }

    let mut battery_total = 0;
    let mut battery_max = 0;

    for entity in selected.iter() {
        if let Ok(battery) = battery.get(entity) {
            battery_total += battery.energy().floor() as i64;
            battery_max += battery.capacity().floor() as i64;
        }
    }

    if battery_max == 0 {
        return;
    }

    // despawn old text
    commands.entity(ui.0).despawn_descendants();

    let text = format!(
        "Battery: {}/{} ({}%)\nSelected {}",
        battery_total,
        battery_max,
        battery_total * 100 / battery_max,
        selected.iter().count()
    );

    commands.entity(ui.0).with_children(|parent| {
        parent.spawn(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: text,
                    style: font.label_text_style(),
                }],
                alignment: TextAlignment::Left,
                linebreak_behaviour: BreakLineOn::WordBoundary,
            },
            ..Default::default()
        });
    });
}
