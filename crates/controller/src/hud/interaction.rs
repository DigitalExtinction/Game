use bevy::{ecs::system::SystemParam, prelude::*};
use glam::Vec3Swizzles;

/// Top-level non-transparent UI node. All such nodes are marked with this component and no descendants have it attached
#[derive(Component)]
pub struct HudTopVisibleNode;

#[derive(SystemParam)]
pub(crate) struct HudNodes<'w, 's> {
    hud: Query<'w, 's, (&'static GlobalTransform, &'static Node), With<HudTopVisibleNode>>,
    windows: Res<'w, Windows>,
}

impl<'w, 's> HudNodes<'w, 's> {
    pub(crate) fn contains_point(&mut self, point: &Vec2) -> bool {
        let window = self.windows.get_primary().unwrap();
        let win_size = Vec2::new(window.width(), window.height());
        self.hud.iter().any(|(box_transform, node)| {
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
