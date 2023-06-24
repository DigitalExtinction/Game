use bevy::prelude::*;
use de_core::baseset::GameSet;
use de_core::{cleanup::DespawnOnGameExit, gamestate::GameState};
use de_energy::Battery;
use de_gui::{BodyTextCommands, BodyTextOps, GuiCommands, OuterStyle};

use super::{interaction::InteractionBlocker, HUD_COLOR};
use crate::selection::Selected;

const PREFIXES: [&'static str; 5] = ["T", "G", "M", "k", ""];

pub(crate) struct DetailsPlugin;

impl Plugin for DetailsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(GameState::Playing)))
            .add_system(
                update
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Resource)]
struct DetailsText(Entity);

fn setup(mut commands: GuiCommands) {
    let node = commands
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
    let details_text = commands
        .spawn_body_text(
            OuterStyle {
                size: Size {
                    width: Val::Percent(20.),
                    height: Val::Percent(30.),
                },
                margin: UiRect::all(Val::Percent(5.)),
            },
            "",
        )
        .id();
    commands.entity(node).add_child(details_text);

    commands.insert_resource(DetailsText(details_text));
}

fn format_units(value: f64, units: &str) -> String {
    for i in (0..PREFIXES.len()).rev() {
        let coeff = 1000f64.powi(i as i32);
        if value > coeff {
            let rounded = most_significant(value / coeff);
            return format!("{}{}{units}", rounded, PREFIXES[i]);
        }
    }

    let rounded = most_significant(value);
    format!("{}{}", rounded, units)
}

fn most_significant(value: f64) -> f64 {
    if value == 0.0 {
        0.0
    } else {
        let d = 3 - value.abs().log10().ceil() as i32;
        let p = 10f64.powi(d);
        (value * p).round() / p
    }
}

fn update(
    ui: Res<DetailsText>,
    selected: Query<Entity, With<Selected>>,
    battery: Query<&Battery>,
    mut text_ops: BodyTextOps,
) {
    let mut battery_total = 0.;
    let mut battery_max = 0.;
    let mut selected_count = 0;

    for entity in selected.iter() {
        if let Ok(battery) = battery.get(entity) {
            selected_count += 1;

            battery_total += battery.energy();
            battery_max += battery.capacity();
        }
    }

    if battery_max == 0. {
        text_ops
            .set_text(ui.0, "")
            .expect("Failed to set text of details");
        return;
    }

    let text = format!(
        "Battery: Battery: {} / {} ({:.1}%)\nSelected {}",
        format_units(battery_total, "J"),
        format_units(battery_max, "J"),
        battery_total * 100. / battery_max,
        selected_count,
    );

    text_ops
        .set_text(ui.0, text)
        .expect("Failed to set text of details");
}
