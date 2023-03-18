use bevy::prelude::*;
use de_construction::EnqueueAssemblyEvent;
use de_core::{
    baseset::GameSet,
    cleanup::DespawnOnGameExit,
    gamestate::GameState,
    objects::{ObjectType, UnitType},
};
use de_gui::{ButtonCommands, GuiCommands, OuterStyle};
use de_objects::ObjectCache;

use super::{interaction::InteractionBlocker, HUD_COLOR};
use crate::selection::Selected;

pub(crate) struct ActionBarPlugin;

impl Plugin for ActionBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(GameState::Playing)))
            .add_system(cleanup.in_schedule(OnExit(GameState::Playing)))
            .add_system(
                detect_update
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(GameState::Playing))
                    .in_set(ActionBarSet::DetectUpdate),
            )
            .add_system(
                update
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(GameState::Playing))
                    .run_if(resource_exists_and_changed::<ActiveEntity>())
                    .after(ActionBarSet::DetectUpdate),
            )
            .add_system(
                button_system
                    .in_base_set(GameSet::Input)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum ActionBarSet {
    DetectUpdate,
}

#[derive(Resource)]
struct ActionBarNode(Entity);

#[derive(Resource, Default)]
struct ActiveEntity(Option<Entity>);

/// An entity attached to every "manufacture this" button in the action bar.
#[derive(Component)]
struct ButtonAction(UnitType);

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<ActionBarNode>();
    commands.remove_resource::<ActiveEntity>();
}

fn setup(mut commands: Commands) {
    let entity = commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size {
                        width: Val::Percent(60.),
                        height: Val::Percent(15.),
                    },
                    position_type: PositionType::Absolute,
                    position: UiRect::new(
                        Val::Percent(20.),
                        Val::Percent(80.),
                        Val::Percent(85.),
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

    commands.insert_resource(ActionBarNode(entity));
    commands.init_resource::<ActiveEntity>();
}

fn detect_update(mut active: ResMut<ActiveEntity>, selected: Query<Entity, With<Selected>>) {
    let new = selected.get_single().ok();
    if active.0 != new {
        active.0 = new;
    }
}

fn update(
    mut commands: GuiCommands,
    cache: Res<ObjectCache>,
    bar_node: Res<ActionBarNode>,
    active: Res<ActiveEntity>,
    objects: Query<&ObjectType>,
) {
    commands.entity(bar_node.0).despawn_descendants();

    let Some(active) = active.0 else {return };
    let object_type = *objects.get(active).unwrap();

    if let Some(factory) = cache.get(object_type).factory() {
        for &unit in factory.products() {
            spawn_button(&mut commands, bar_node.0, unit);
        }
    }
}

fn spawn_button(commands: &mut GuiCommands, parent: Entity, unit: UnitType) {
    let button = commands
        .spawn_button(
            OuterStyle {
                size: Size::new(Val::Percent(10.), Val::Percent(80.)),
                margin: UiRect::new(
                    Val::Percent(0.),
                    Val::Percent(0.),
                    Val::Percent(2.),
                    Val::Percent(2.),
                ),
            },
            unit.to_string().chars().next().unwrap(),
        )
        .insert(ButtonAction(unit))
        .id();
    commands.entity(parent).add_child(button);
}

fn button_system(
    active: Res<ActiveEntity>,
    interactions: Query<(&Interaction, &ButtonAction), Changed<Interaction>>,
    mut events: EventWriter<EnqueueAssemblyEvent>,
) {
    for (&interaction, action) in interactions.iter() {
        if let Interaction::Clicked = interaction {
            events.send(EnqueueAssemblyEvent::new(active.0.unwrap(), action.0));
        }
    }
}
