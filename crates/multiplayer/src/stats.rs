use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use bevy::prelude::*;
use de_core::baseset::GameSet;
use de_net::{FromGame, ToGame};

use crate::{
    messages::{FromGameServerEvent, MessagesSet, ToGameServerEvent},
    netstate::NetState,
};

const PING_INTERVAL: Duration = Duration::from_secs(10);
const MAX_DELAY_INTERVALS: usize = 10;

pub(crate) struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(NetState::Joined)))
            .add_system(cleanup.in_schedule(OnExit(NetState::Joined)))
            .add_system(
                ping.in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(NetState::Joined))
                    .before(MessagesSet::SendMessages),
            )
            .add_system(
                pong.in_base_set(GameSet::PreMovement)
                    .run_if(in_state(NetState::Joined))
                    .run_if(on_event::<FromGameServerEvent>())
                    .in_set(StatsSet::Pong)
                    .after(MessagesSet::RecvMessages),
            )
            .add_system(
                unresolved
                    .in_base_set(GameSet::PreMovement)
                    .run_if(in_state(NetState::Joined))
                    .after(StatsSet::Pong),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum StatsSet {
    Pong,
}

#[derive(Resource)]
struct PingTimer(Timer);

#[derive(Resource)]
struct PingTracker {
    counter: u32,
    times: VecDeque<PingRecord>,
}

struct PingRecord {
    resolved: bool,
    id: u32,
    time: Instant,
}

impl PingTracker {
    fn new() -> Self {
        Self {
            counter: 0,
            times: VecDeque::new(),
        }
    }

    /// Register a new ping send time and returns a new unique ID (wrapping)
    /// for the ping.
    fn start(&mut self, time: Instant) -> u32 {
        let id = self.counter;
        self.counter = id.wrapping_add(1);
        self.times.push_back(PingRecord {
            resolved: false,
            id,
            time,
        });
        id
    }

    /// Marks a ping record as resolved and returns ping send time.
    fn resolve(&mut self, id: u32) -> Option<Instant> {
        for record in self.times.iter_mut() {
            if record.id == id {
                if record.resolved {
                    return None;
                } else {
                    record.resolved = true;
                    return Some(record.time);
                }
            }
        }

        None
    }

    /// Trims the history of sent pings and pushes non-resolved trimmed ping
    /// IDs to `ids`.
    ///
    /// # Arguments
    ///
    /// * `len` - maximum number of pings (resolved and unresolved) to
    ///   keep.
    ///
    /// * `ids` - unresolved trimmed pings will be pushed to this Vec.
    fn trim(&mut self, len: usize, ids: &mut Vec<u32>) {
        while self.times.len() > len {
            let record = self.times.pop_front().unwrap();
            if !record.resolved {
                ids.push(record.id);
            }
        }
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(PingTimer(Timer::new(PING_INTERVAL, TimerMode::Repeating)));
    commands.insert_resource(PingTracker::new());
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<PingTimer>();
    commands.remove_resource::<PingTracker>();
}

fn ping(
    time: Res<Time>,
    mut timer: ResMut<PingTimer>,
    mut tracker: ResMut<PingTracker>,
    mut messages: EventWriter<ToGameServerEvent<true>>,
) {
    timer.0.tick(time.delta());

    let time = Instant::now();
    for _ in 0..timer.0.times_finished_this_tick() {
        let id = tracker.start(time);
        info!("Sending Ping({id}).");
        messages.send(ToGame::Ping(id).into());
    }
}

fn pong(mut tracker: ResMut<PingTracker>, mut messages: EventReader<FromGameServerEvent>) {
    for event in messages.iter() {
        if let FromGame::Pong(id) = event.message() {
            match tracker.resolve(*id) {
                Some(send_time) => {
                    let time = Instant::now();
                    let system_time = time - send_time;
                    let network_time = event.time() - send_time;

                    info!(
                        "Received Pong({}) with {{ system: {}ms, network: {}ms }} round trip.",
                        *id,
                        system_time.as_millis(),
                        network_time.as_millis(),
                    );
                }
                None => {
                    warn!("Receive non-registered Pong({}).", *id);
                }
            }
        }
    }
}

fn unresolved(mut buffer: Local<Vec<u32>>, mut tracker: ResMut<PingTracker>) {
    buffer.clear();
    tracker.trim(MAX_DELAY_INTERVALS, &mut buffer);

    for &id in buffer.iter() {
        error!("Ping({id}) was not responded in time.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker() {
        let mut tracker = PingTracker::new();

        let time_a = Instant::now();
        assert_eq!(tracker.start(time_a), 0);
        let time_b = time_a + Duration::from_millis(100);
        assert_eq!(tracker.start(time_b), 1);
        let time_c = time_a + Duration::from_millis(200);
        assert_eq!(tracker.start(time_c), 2);

        assert_eq!(tracker.resolve(2).unwrap(), time_c);
        assert_eq!(tracker.start(Instant::now()), 3);
        assert_eq!(tracker.resolve(1).unwrap(), time_b);
        assert_eq!(tracker.start(Instant::now()), 4);
        assert_eq!(tracker.start(Instant::now()), 5);

        let mut ids = Vec::new();
        tracker.trim(2, &mut ids);
        assert_eq!(ids, vec![0, 3]);
    }
}
