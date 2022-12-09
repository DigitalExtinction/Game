use actix_web::web;
use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};

use self::db::Games;

mod db;
mod endpoints;
mod model;

#[derive(Clone)]
pub struct GamesService {
    games: Games,
}

impl GamesService {
    /// Setup games DB and endpoints.
    ///
    /// This should be called after [`crate::auth::Auth`].
    pub async fn setup(pool: &'static Pool<Sqlite>) -> Result<Self> {
        Ok(Self {
            games: db::Games::init(pool)
                .await
                .context("Failed to initialize games")?,
        })
    }

    /// Configure actix-web application.
    pub fn configure(&self, cfg: &mut web::ServiceConfig) {
        cfg.app_data(web::Data::new(self.games.clone()));
        endpoints::configure(cfg);
    }
}
