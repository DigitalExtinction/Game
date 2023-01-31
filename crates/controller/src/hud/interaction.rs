use bevy::{ecs::system::SystemParam, prelude::*};
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
pub(crate) struct HudNodes<'w, 's> {
    hud: Query<
        'w,
        's,
        (
            &'static GlobalTransform,
            &'static ComputedVisibility,
            &'static Node,
        ),
        With<InteractionBlocker>,
    >,
    windows: Res<'w, Windows>,
}

impl<'w, 's> HudNodes<'w, 's> {
    pub(crate) fn contains_point(&mut self, point: &Vec2) -> bool {
        let window = self.windows.get_primary().unwrap();
        let win_size = Vec2::new(window.width(), window.height());
        self.hud.iter().any(|(box_transform, visibility, node)| {
            if !visibility.is_visible() {
                return false;
            }

            // WARNING: This is because mouse y starts on bottom, GlobalTransform on top
            let mouse_position = Vec2::new(point.x, win_size.y - point.y);

            let box_size = node.size();
            let box_transform: Vec3 = box_transform.translation();
            // WARNING: This is because GlobalTransform is centered, width/2 to left and to right, same on vertical
            let box_position = box_transform.xy() - box_size / 2.;

            mouse_position.cmpge(box_position).all()
                && mouse_position.cmple(box_position + box_size).all()
        })
    }
}
