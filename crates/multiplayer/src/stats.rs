use std::{collections::VecDeque, time::Duration};

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
    time: Duration,
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
    fn start(&mut self, time: Duration) -> u32 {
        let id = self.counter;
        self.counter = id.wrapping_add(1);
        self.times.push_back(PingRecord {
            resolved: false,
            id,
            time,
        });
        id
    }

    /// Marks a ping record as resolved and returns round-trip time.
    fn resolve(&mut self, id: u32, time: Duration) -> Option<Duration> {
        for record in self.times.iter_mut() {
            if record.id == id {
                if record.resolved {
                    return None;
                } else {
                    record.resolved = true;
                    return Some(time - record.time);
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
    mut messages: EventWriter<ToGameServerEvent>,
) {
    timer.0.tick(time.delta());
    for _ in 0..timer.0.times_finished_this_tick() {
        let id = tracker.start(time.elapsed());
        info!("Sending Ping({id}).");
        messages.send(ToGame::Ping(id).into());
    }
}

fn pong(
    time: Res<Time>,
    mut tracker: ResMut<PingTracker>,
    mut messages: EventReader<FromGameServerEvent>,
) {
    for event in messages.iter() {
        if let FromGame::Pong(id) = event.message() {
            match tracker.resolve(*id, time.elapsed()) {
                Some(round_trip) => {
                    info!(
                        "Received Pong({}) with {}ms round trip.",
                        *id,
                        round_trip.as_millis()
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

        assert_eq!(tracker.start(Duration::from_millis(500)), 0);
        assert_eq!(tracker.start(Duration::from_millis(800)), 1);
        assert_eq!(tracker.start(Duration::from_millis(900)), 2);

        assert_eq!(
            tracker.resolve(2, Duration::from_millis(910)).unwrap(),
            Duration::from_millis(10)
        );
        assert_eq!(tracker.start(Duration::from_millis(1100)), 3);
        assert_eq!(
            tracker.resolve(1, Duration::from_millis(1005)).unwrap(),
            Duration::from_millis(205)
        );
        assert_eq!(tracker.start(Duration::from_millis(1300)), 4);
        assert_eq!(tracker.start(Duration::from_millis(1800)), 5);

        let mut ids = Vec::new();
        tracker.trim(2, &mut ids);
        assert_eq!(ids, vec![0, 3]);
    }
}
