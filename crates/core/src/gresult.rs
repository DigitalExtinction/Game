use bevy::prelude::Resource;

#[derive(Resource)]
pub enum GameResult {
    /// Game finished normally with the player either loosing or winning.
    Finished(NormalResult),
    /// The game finished due to an error.
    Error(String),
}

impl GameResult {
    /// Create new normally finished game result.
    pub fn finished(won: bool) -> Self {
        Self::Finished(NormalResult::new(won))
    }

    /// Create game result from an error.
    pub fn error(message: impl ToString) -> Self {
        Self::Error(message.to_string())
    }
}

pub struct NormalResult {
    won: bool,
}

impl NormalResult {
    fn new(won: bool) -> Self {
        Self { won }
    }

    pub fn won(&self) -> bool {
        self.won
    }
}
