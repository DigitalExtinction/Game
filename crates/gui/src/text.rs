use bevy::prelude::*;

pub(crate) struct TextPlugin;

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.run_if(not(resource_exists::<TextProps>())));
    }
}

/// Resource handling text properties throughout the app.
#[derive(Resource)]
pub struct TextProps(pub Handle<Font>);

impl TextProps {
    pub(crate) fn button_text_style(&self) -> TextStyle {
        TextStyle {
            font: self.font(),
            font_size: 40.0,
            color: Color::rgb(0.9, 0.9, 0.9),
        }
    }

    pub fn label_text_style(&self) -> TextStyle {
        TextStyle {
            font: self.font(),
            font_size: 35.0,
            color: Color::rgb(0.9, 0.9, 0.9),
        }
    }

    pub(crate) fn input_text_style(&self) -> TextStyle {
        TextStyle {
            font: self.font(),
            font_size: 30.0,
            color: Color::BLACK,
        }
    }

    pub(crate) fn toast_text_style(&self) -> TextStyle {
        TextStyle {
            font: self.font(),
            font_size: 30.0,
            color: Color::BLACK,
        }
    }

    fn font(&self) -> Handle<Font> {
        self.0.clone()
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Fira_Mono/FiraMono-Medium.ttf");
    commands.insert_resource(TextProps(font));
}
