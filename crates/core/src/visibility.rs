use bevy::prelude::*;

use crate::stages::GameStage;

pub(crate) struct VisibilityPlugin;

impl Plugin for VisibilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            GameStage::PostUpdate,
            update.label(VisibilityLabels::Update),
        );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub enum VisibilityLabels {
    Update,
}

/// This represents visibility flags. An object is visible if at least one
/// "visible" flag is set to true and none of "invisible" flag is true. The
/// individual flags can be controlled independently.
///
/// The system [`VisibilityLabels::Update`] executed during
/// [`GameStage::PostUpdate`] automatically updates
/// [`bevy::render::prelude::Visibility`] of entities with this component.
#[derive(Component, Default)]
pub struct VisibilityFlags {
    visible: u32,
    invisible: u32,
}

impl VisibilityFlags {
    pub fn update_visible(&mut self, bit: u32, value: bool) {
        Self::update(&mut self.visible, bit, value);
    }

    pub fn update_invisible(&mut self, bit: u32, value: bool) {
        Self::update(&mut self.invisible, bit, value);
    }

    fn update(flags: &mut u32, bit: u32, value: bool) {
        let mask = 1 << bit;
        if value {
            *flags |= mask;
        } else {
            *flags &= !mask;
        }
    }

    /// Returns value of a specific "invisible" flag.
    pub fn invisible_value(&self, bit: u32) -> bool {
        self.invisible & (1 << bit) != 0
    }

    pub fn visible(&self) -> bool {
        self.invisible == 0 && self.visible > 0
    }
}

fn update(mut entities: Query<(&VisibilityFlags, &mut Visibility), Changed<VisibilityFlags>>) {
    for (flags, mut visibility) in entities.iter_mut() {
        visibility.is_visible = flags.visible();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_flags() {
        let mut flags = VisibilityFlags::default();
        assert!(!flags.visible());

        flags.update_visible(1, true);
        assert!(flags.visible());
        flags.update_visible(3, true);
        assert!(flags.visible());
        flags.update_visible(1, false);
        assert!(flags.visible());
        flags.update_visible(3, false);
        assert!(!flags.visible());

        assert!(!flags.invisible_value(1));
        flags.update_invisible(1, true);
        assert!(!flags.visible());
        assert!(flags.invisible_value(1));
        flags.update_visible(1, true);
        assert!(!flags.visible());
        assert!(flags.invisible_value(1));
        flags.update_invisible(1, false);
        assert!(flags.visible());
        assert!(!flags.invisible_value(1));
    }
}
