use bevy::{
    ecs::{query::ReadOnlyWorldQuery, system::SystemParam},
    prelude::*,
};
use glam::Vec3Swizzles;

/// Top-level non-transparent or otherwise interaction blocking Node. All such
/// nodes are marked with this component and no descendants have it attached.
///
/// These nodes block mouse based interaction with the 3D world behind them.
/// These nodes do not block UI interaction: if desired, this must be done via
/// native bevi_ui mechanisms.
#[derive(Component)]
pub(crate) struct InteractionBlocker;

#[derive(SystemParam)]
pub(crate) struct HudNodes<'w, 's, F = With<InteractionBlocker>>
where
    F: ReadOnlyWorldQuery + Sync + Send + 'static,
{
    hud: Query<
        'w,
        's,
        (
            &'static GlobalTransform,
            &'static ComputedVisibility,
            &'static Node,
        ),
        F,
    >,
    windows: Res<'w, Windows>,
}

impl<'w, 's, F> HudNodes<'w, 's, F>
where
    F: ReadOnlyWorldQuery + Sync + Send + 'static,
{
    pub(crate) fn contains_point(&self, point: Vec2) -> bool {
        self.relative_position(point).is_some()
    }

    /// Returns relative position of `point` to the fist Node which contains
    /// the point.
    ///
    /// The returned point is between (0, 0) (top-left corner) and (1, 1)
    /// (bottom-right corner).
    pub(crate) fn relative_position(&self, point: Vec2) -> Option<Vec2> {
        let window = self.windows.get_primary().unwrap();
        // This is because screen y starts on bottom, GlobalTransform on top.
        let point = Vec2::new(point.x, window.height() - point.y);

        self.hud
            .iter()
            .filter_map(|(box_transform, visibility, node)| {
                if !visibility.is_visible() {
                    return None;
                }

                let box_size = node.size();
                let box_transform: Vec3 = box_transform.translation();
                // GlobalTransform is centered, width/2 to left and to right,
                // same on vertical.
                let box_position = box_transform.xy() - box_size / 2.;
                let relative = (point - box_position) / box_size;
                if relative.cmpge(Vec2::ZERO).all() && relative.cmple(Vec2::ONE).all() {
                    Some(relative)
                } else {
                    None
                }
            })
            .next()
    }
}
