use bevy::ui::{UiRect, Val};

#[derive(Default)]
pub struct OuterStyle {
    pub width: Val,
    pub height: Val,
    pub margin: UiRect,
}
