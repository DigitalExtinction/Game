use bevy::prelude::Resource;

#[derive(Resource)]
pub struct GameResult {
    won: bool,
}

impl GameResult {
    pub fn new(won: bool) -> Self {
        Self { won }
    }

    pub fn won(&self) -> bool {
        self.won
    }
}
