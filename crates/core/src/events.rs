use std::marker::PhantomData;

use bevy::{
    ecs::{event::Events, system::Resource},
    prelude::*,
};
use iyes_loopless::prelude::*;

use crate::state::GameState;

pub struct ResendEventPlugin<T: Resource> {
    _marker: PhantomData<T>,
}

impl<T: Resource> Default for ResendEventPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T: Resource> Plugin for ResendEventPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Loading, setup::<T>)
            .add_system(enqueue_events::<T>.run_in_state(GameState::Loading))
            .add_enter_system(GameState::Playing, resend_events::<T>);
    }
}

struct EventQueue<T: Resource>(Vec<T>);

fn setup<T: Resource>(mut commands: Commands) {
    commands.insert_resource(EventQueue::<T>(Vec::new()));
}

fn enqueue_events<T: Resource>(mut queue: ResMut<EventQueue<T>>, mut events: ResMut<Events<T>>) {
    queue.0.extend(events.drain());
}

fn resend_events<T: Resource>(
    mut commands: Commands,
    mut queue: ResMut<EventQueue<T>>,
    mut events: EventWriter<T>,
) {
    for event in queue.0.drain(..) {
        events.send(event);
    }
    commands.remove_resource::<EventQueue<T>>();
}
