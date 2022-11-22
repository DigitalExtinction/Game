use std::marker::PhantomData;

use bevy::{
    ecs::{
        event::{Event, Events},
        system::Resource,
    },
    prelude::*,
};
use iyes_loopless::prelude::*;

use crate::state::GameState;

pub struct ResendEventPlugin<T: Event> {
    _marker: PhantomData<T>,
}

impl<T: Event> Default for ResendEventPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T: Event> Plugin for ResendEventPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Loading, setup::<T>)
            .add_system(enqueue_events::<T>.run_in_state(GameState::Loading))
            .add_enter_system(GameState::Playing, resend_events::<T>);
    }
}

#[derive(Resource)]
struct EventQueue<T: Event>(Vec<T>);

fn setup<T: Event>(mut commands: Commands) {
    commands.insert_resource(EventQueue::<T>(Vec::new()));
}

fn enqueue_events<T: Event>(mut queue: ResMut<EventQueue<T>>, mut events: ResMut<Events<T>>) {
    queue.0.extend(events.drain());
}

fn resend_events<T: Event>(
    mut commands: Commands,
    mut queue: ResMut<EventQueue<T>>,
    mut events: EventWriter<T>,
) {
    for event in queue.0.drain(..) {
        events.send(event);
    }
    commands.remove_resource::<EventQueue<T>>();
}
