use std::{collections::VecDeque, time::Duration};

use bevy::prelude::*;
use de_core::state::AppState;

use crate::text::TextProps;

const MIN_TOAST_DURATION: Duration = Duration::from_secs(2);
const PER_BYTE_TOAST_DURATION: Duration = Duration::from_nanos(84000000);

pub(crate) struct ToastPlugin;

impl Plugin for ToastPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToastQueue>()
            .add_event::<ToastEvent>()
            .add_systems(
                PostUpdate,
                (
                    process_events.in_set(ToastSet::ProcessEvents),
                    spawn_and_despawn
                        .run_if(not(in_state(AppState::AppLoading)))
                        .after(ToastSet::ProcessEvents),
                ),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum ToastSet {
    ProcessEvents,
}

/// Send this event to briefly display a UI toast.
#[derive(Event)]
pub struct ToastEvent(String);

impl ToastEvent {
    /// Creates a new toast event. Text is automatically converted to string
    /// and only first line is taken.
    pub fn new(text: impl ToString) -> Self {
        Self(text.to_string().lines().next().unwrap().into())
    }

    fn text(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Resource, Default)]
struct ToastQueue {
    current: Option<CurrentToast>,
    queue: VecDeque<String>,
}

impl ToastQueue {
    fn push(&mut self, text: String) {
        self.queue.push_front(text);
    }

    fn pop(&mut self) -> Option<String> {
        self.queue.pop_back()
    }

    fn current(&self) -> Option<&CurrentToast> {
        self.current.as_ref()
    }

    fn set_current(&mut self, toast: Option<CurrentToast>) {
        self.current = toast;
    }
}

struct CurrentToast {
    expiration: Duration,
    entity: Entity,
}

impl CurrentToast {
    fn new(expiration: Duration, entity: Entity) -> Self {
        Self { expiration, entity }
    }

    fn entity(&self) -> Entity {
        self.entity
    }

    fn expired(&self, now: Duration) -> bool {
        now >= self.expiration
    }
}

fn process_events(mut events: EventReader<ToastEvent>, mut queue: ResMut<ToastQueue>) {
    for event in events.iter() {
        info!("Enqueuing a toast: {}", event.text());
        queue.push(event.text().to_owned())
    }
}

fn spawn_and_despawn(
    mut commands: Commands,
    time: Res<Time>,
    text_props: Res<TextProps>,
    mut queue: ResMut<ToastQueue>,
) {
    let now = time.elapsed();
    if queue.current().map_or(false, |c| !c.expired(now)) {
        return;
    }

    if let Some(entity) = queue.current().map(|c| c.entity()) {
        commands.entity(entity).despawn_recursive();
    }

    let current = match queue.pop() {
        Some(text) => {
            let duration = (text.len() as f32) * PER_BYTE_TOAST_DURATION.as_secs_f32();
            let duration = Duration::from_secs_f32(duration).max(MIN_TOAST_DURATION);
            let entity = spawn(&mut commands, text_props.as_ref(), text);
            Some(CurrentToast::new(now + duration, entity))
        }
        None => None,
    };
    queue.set_current(current);
}

fn spawn(commands: &mut Commands, text_props: &TextProps, text: String) -> Entity {
    let text_style = text_props.toast_text_style();

    let mut commands = commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            left: Val::Percent(20.),
            right: Val::Percent(20.),
            top: Val::Percent(5.),
            bottom: Val::Percent(85.),
            padding: UiRect::all(Val::Percent(1.)),
            ..default()
        },
        background_color: Color::RED.into(),
        z_index: ZIndex::Local(10000),
        ..default()
    });

    commands.with_children(|builder| {
        builder.spawn(TextBundle::from_section(text, text_style));
    });

    commands.id()
}
