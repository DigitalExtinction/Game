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

/// This represents visibility flags. An object is visible if at least one flag
/// is set to true. The individual flags can be controlled independently.
///
/// The system [`VisibilityLabels::Update`] executed during
/// [`GameStage::PostUpdate`] automatically updates
/// [`bevy::render::prelude::Visibility`] of entities with this component.
#[derive(Component, Default)]
pub struct VisibilityFlags(u32);

impl VisibilityFlags {
    pub fn update(&mut self, bit: u32, value: bool) {
        let mask = 1 << bit;
        if value {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }
    }

    pub fn visible(&self) -> bool {
        self.0 > 0
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

        flags.update(1, true);
        assert!(flags.visible());
        flags.update(3, true);
        assert!(flags.visible());
        flags.update(1, false);
        assert!(flags.visible());
        flags.update(3, false);
        assert!(!flags.visible());
    }
}
