use bevy::{
    ecs::{query::QueryFilter, system::SystemParam},
    prelude::*,
};

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
    F: QueryFilter + Sync + Send + 'static,
{
    hud: Query<
        'w,
        's,
        (
            &'static GlobalTransform,
            &'static ViewVisibility,
            &'static Node,
        ),
        F,
    >,
}

impl<'w, 's, F> HudNodes<'w, 's, F>
where
    F: QueryFilter + Sync + Send + 'static,
{
    /// See [`Self::relative_position`].
    pub(crate) fn contains_point(&self, point: Vec2) -> bool {
        self.relative_position(point).is_some()
    }

    /// Returns relative position of `point` to the fist Node which contains
    /// the point.
    ///
    /// The returned point is between (0, 0) (top-left corner) and (1, 1)
    /// (bottom-right corner).
    ///
    /// The method relies on [`ViewVisibility`], therefore the results are
    /// accurate with respect to the last rendered frame only iff called before
    /// [`bevy::render::view::VisibilitySystems::VisibilityPropagate`] (during
    /// `PostUpdate` schedule).
    pub(crate) fn relative_position(&self, point: Vec2) -> Option<Vec2> {
        self.hud
            .iter()
            .filter_map(|(box_transform, visibility, node)| {
                if !visibility.get() {
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
