use bevy::prelude::*;
use de_core::{cleanup::DespawnOnGameExit, gamestate::GameState};
use de_energy::Battery;
use de_gui::{BodyTextCommands, BodyTextOps, GuiCommands, OuterStyle};

use super::{interaction::InteractionBlocker, HUD_COLOR};
use crate::selection::Selected;

const PREFIXES: [&str; 5] = ["", "k", "M", "G", "T"];

pub(crate) struct DetailsPlugin;

impl Plugin for DetailsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup)
            .add_systems(PostUpdate, update.run_if(in_state(GameState::Playing)))
            .add_systems(OnExit(GameState::Playing), clean_up);
    }
}

#[derive(Resource)]
struct DetailsText(Entity);

fn setup(mut commands: GuiCommands) {
    let node = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(20.),
                    height: Val::Percent(30.),
                    position_type: PositionType::Absolute,
                    left: Val::Percent(0.),
                    right: Val::Percent(20.),
                    top: Val::Percent(70.),
                    bottom: Val::Percent(100.),
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
                margin: UiRect::all(Val::Percent(5.)),
                ..default()
            },
            "",
        )
        .id();
    commands.entity(node).add_child(details_text);

    commands.insert_resource(DetailsText(details_text));
}

fn format_units(value: f64, units: &str) -> String {
    let mut value = value;
    let mut i = 0;

    while value >= 1000.0 && i < PREFIXES.len() - 1 {
        value /= 1000.0;
        i += 1;
    }

    format!("{}{}{}", most_significant(value), PREFIXES[i], units)
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
        "Battery: {} / {} ({:.1}%)\nSelected {}",
        format_units(battery_total, "J"),
        format_units(battery_max, "J"),
        battery_total * 100. / battery_max,
        selected_count,
    );

    text_ops
        .set_text(ui.0, text)
        .expect("Failed to set text of details");
}

fn clean_up(mut commands: Commands) {
    // remove DetailsText
    commands.remove_resource::<DetailsText>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_units() {
        assert_eq!(format_units(-1.0, "J"), "-1J");
        assert_eq!(format_units(0.0, "J"), "0J");
        assert_eq!(format_units(1.0, "J"), "1J");
        assert_eq!(format_units(10.0, "J"), "10J");
        assert_eq!(format_units(100.0, "J"), "100J");
        assert_eq!(format_units(673.0, "J"), "673J");
        assert_eq!(format_units(1000.0, "J"), "1kJ");
        assert_eq!(format_units(10000.0, "J"), "10kJ");
        assert_eq!(format_units(100000.0, "J"), "100kJ");
        assert_eq!(format_units(590385.0, "J"), "590kJ");
        assert_eq!(format_units(1000000.0, "J"), "1MJ");
        assert_eq!(format_units(10000000.0, "J"), "10MJ");
        assert_eq!(format_units(100000000.0, "J"), "100MJ");
        assert_eq!(format_units(339484857.0, "J"), "339MJ");
        assert_eq!(format_units(1000000000.0, "J"), "1GJ");
        assert_eq!(format_units(10000000000.0, "J"), "10GJ");
        assert_eq!(format_units(100000000000.0, "J"), "100GJ");
        assert_eq!(format_units(1000000000000.0, "J"), "1TJ");
        assert_eq!(format_units(10000000000000.0, "J"), "10TJ");
        assert_eq!(format_units(100000000000000.0, "J"), "100TJ");
    }
}
