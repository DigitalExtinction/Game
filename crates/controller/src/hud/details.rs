use bevy::prelude::*;
use de_core::baseset::GameSet;
use de_core::{cleanup::DespawnOnGameExit, gamestate::GameState};
use de_energy::Battery;
use de_gui::{BodyTextCommands, BodyTextOps, GuiCommands, OuterStyle};

use super::{interaction::InteractionBlocker, HUD_COLOR};
use crate::selection::Selected;

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
                margin: UiRect {
                    left: Val::Px(10.),
                    right: Val::Px(10.),
                    top: Val::Px(10.),
                    bottom: Val::Px(10.),
                },
            },
            "",
        )
        .id();
    commands.entity(node).add_child(details_text);

    commands.insert_resource(DetailsText(details_text));
}

fn update(
    ui: Res<DetailsText>,
    selected: Query<Entity, With<Selected>>,
    battery: Query<&Battery>,
    mut text_ops: BodyTextOps,
) {
    if selected.is_empty() {
        text_ops
            .set_text(ui.0, "".into())
            .expect("Failed to set text of details");
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

    let text = format!(
        "Battery: {}/{} ({}%)\nSelected {}",
        battery_total,
        battery_max,
        battery_total * 100 / battery_max,
        selected.iter().count()
    );

    text_ops
        .set_text(ui.0, text)
        .expect("Failed to set text of details");
    #[derive(Resource)]
    struct DetailsText(Entity);
}
