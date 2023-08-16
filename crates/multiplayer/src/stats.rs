use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use bevy::prelude::*;
use de_core::schedule::PreMovement;
use de_messages::{FromGame, ToGame};
use tracing::{debug, info, trace};

use crate::{
    messages::{FromGameServerEvent, MessagesSet, ToGameServerEvent},
    netstate::NetState,
};

const RELIABLE_PING_INTERVAL: Duration = Duration::from_secs(10);
const RELIABLE_HISTORY: usize = 10;
const UNRELIABLE_PING_INTERVAL: Duration = Duration::from_secs(1);
const UNRELIABLE_HISTORY: usize = 100;
const STATS_INTERVAL: Duration = Duration::from_secs(10);
const STATS_OFFSET: Duration = Duration::from_secs(10);

pub(crate) struct StatsPlugin;

impl StatsPlugin {
    fn build_spec<const R: bool>(app: &mut App) {
        app.add_systems(OnEnter(NetState::Joined), setup_spec::<R>)
            .add_systems(OnExit(NetState::Joined), cleanup_spec::<R>)
            .add_systems(
                PostUpdate,
                ping::<R>
                    .run_if(in_state(NetState::Joined))
                    .before(MessagesSet::SendMessages),
            )
            .add_systems(
                PreMovement,
                (
                    pong::<R>
                        .run_if(on_event::<FromGameServerEvent>())
                        .in_set(StatsSet::Pong)
                        .after(MessagesSet::RecvMessages),
                    unresolved::<R>
                        .in_set(StatsSet::Unresolved)
                        .after(StatsSet::Pong),
                )
                    .run_if(in_state(NetState::Joined)),
            );
    }
}

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        Self::build_spec::<false>(app);
        Self::build_spec::<true>(app);

        app.add_systems(OnEnter(NetState::Joined), setup)
            .add_systems(OnExit(NetState::Joined), cleanup)
            .add_systems(
                PreMovement,
                (
                    stats_tick.in_set(StatsSet::StatsTick),
                    delivery_rate
                        .after(StatsSet::StatsTick)
                        .after(StatsSet::Unresolved),
                )
                    .run_if(in_state(NetState::Joined)),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum StatsSet {
    Pong,
    Unresolved,
    StatsTick,
}

#[derive(Resource)]
struct PingTimer<const R: bool>(Timer);

#[derive(Resource)]
struct StatsTimer(Timer);

#[derive(Resource)]
struct Counter(u32);

impl Counter {
    fn new() -> Self {
        Self(0)
    }

    /// Returns a new unique ID (wrapping) for a ping.
    fn next(&mut self) -> u32 {
        let id = self.0;
        self.0 = id.wrapping_add(1);
        id
    }
}

#[derive(Resource)]
struct PingTracker<const R: bool> {
    times: VecDeque<PingRecord>,
}

struct PingRecord {
    resolved: bool,
    id: u32,
    time: Instant,
}

impl<const R: bool> PingTracker<R> {
    fn new() -> Self {
        Self {
            times: VecDeque::new(),
        }
    }

    /// Register a new ping send time.
    fn register(&mut self, id: u32, time: Instant) {
        self.times.push_back(PingRecord {
            resolved: false,
            id,
            time,
        });
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

    /// Returns fraction of ping records marked as resolved.
    ///
    /// # Arguments
    ///
    /// * `cutoff` - pings sent after this time are excluded from the
    ///   statistics.
    fn resolution_rate(&self, cutoff: Instant) -> Option<f32> {
        let mut sample_size = 0;
        let mut resolved_count = 0;

        for record in self.times.iter() {
            if record.time > cutoff {
                continue;
            }

            sample_size += 1;
            if record.resolved {
                resolved_count += 1;
            }
        }

        if sample_size == 0 {
            None
        } else {
            Some(resolved_count as f32 / sample_size as f32)
        }
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
    commands.insert_resource(Counter::new());
    commands.insert_resource(StatsTimer(Timer::new(STATS_INTERVAL, TimerMode::Repeating)));
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<Counter>();
    commands.remove_resource::<StatsTimer>();
}

fn setup_spec<const R: bool>(mut commands: Commands) {
    let interval = if R {
        RELIABLE_PING_INTERVAL
    } else {
        UNRELIABLE_PING_INTERVAL
    };

    commands.insert_resource(PingTimer::<R>(Timer::new(interval, TimerMode::Repeating)));
    commands.insert_resource(PingTracker::<R>::new());
}

fn cleanup_spec<const R: bool>(mut commands: Commands) {
    commands.remove_resource::<PingTimer<R>>();
    commands.remove_resource::<PingTracker<R>>();
}

fn ping<const R: bool>(
    time: Res<Time>,
    mut timer: ResMut<PingTimer<R>>,
    mut counter: ResMut<Counter>,
    mut tracker: ResMut<PingTracker<R>>,
    mut messages: EventWriter<ToGameServerEvent<R>>,
) {
    timer.0.tick(time.delta());

    let time = Instant::now();
    for _ in 0..timer.0.times_finished_this_tick() {
        let id = counter.next();
        tracker.register(id, time);
        if R {
            info!("Sending reliable Ping({id}).",);
        } else {
            trace!("Sending unreliable Ping({id}).",);
        }
        messages.send(ToGame::Ping(id).into());
    }
}

fn pong<const R: bool>(
    mut tracker: ResMut<PingTracker<R>>,
    mut messages: EventReader<FromGameServerEvent>,
) {
    for event in messages.iter() {
        if let FromGame::Pong(id) = event.message() {
            if let Some(send_time) = tracker.resolve(*id) {
                let time = Instant::now();
                let system_time = time - send_time;
                let network_time = event.time() - send_time;

                if R {
                    info!(
                        "Received reliable Pong({}) with {{ system: {}ms, network: {}ms }} round trip.",
                        *id,
                        system_time.as_millis(),
                        network_time.as_millis(),
                    );
                } else {
                    debug!(
                        "Received unreliable Pong({}) with {{ system: {}ms, network: {}ms }} round trip.",
                        *id,
                        system_time.as_millis(),
                        network_time.as_millis(),
                    );
                }
            }
        }
    }
}

fn unresolved<const R: bool>(mut buffer: Local<Vec<u32>>, mut tracker: ResMut<PingTracker<R>>) {
    let count = if R {
        RELIABLE_HISTORY
    } else {
        UNRELIABLE_HISTORY
    };

    buffer.clear();
    tracker.trim(count, &mut buffer);

    if R {
        for &id in buffer.iter() {
            error!("Ping({id}) was not responded in time.");
        }
    }
}

fn stats_tick(time: Res<Time>, mut timer: ResMut<StatsTimer>) {
    timer.0.tick(time.delta());
}

fn delivery_rate(timer: ResMut<StatsTimer>, tracker: Res<PingTracker<false>>) {
    if timer.0.just_finished() {
        let Some(rate) = tracker.resolution_rate(Instant::now() - STATS_OFFSET) else {
            return;
        };

        let rate_percentage = rate * 100.;
        let rate_sqrt_percentage = rate.sqrt() * 100.;
        info!(
            "End-to-end unreliable ping success rate: {:.1}%; One way estimate: {:.1}%",
            rate_percentage, rate_sqrt_percentage
        );

        if rate < 0.95 {
            warn!("Low ping delivery reliability: {:.1}%", rate_percentage);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker() {
        let mut tracker = PingTracker::<true>::new();

        let time_a = Instant::now();
        tracker.register(0, time_a);
        let time_b = time_a + Duration::from_millis(100);
        tracker.register(1, time_b);
        let time_c = time_a + Duration::from_millis(200);
        tracker.register(2, time_c);

        assert_eq!(tracker.resolve(2).unwrap(), time_c);
        tracker.register(3, Instant::now());
        assert_eq!(tracker.resolve(1).unwrap(), time_b);
        tracker.register(4, Instant::now());
        tracker.register(5, Instant::now());

        let mut ids = Vec::new();
        tracker.trim(2, &mut ids);
        assert_eq!(ids, vec![0, 3]);
    }
}
