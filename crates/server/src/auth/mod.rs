use std::sync::Arc;

use actix_web::web;
use anyhow::{ensure, Context, Result};
use sqlx::{Pool, Sqlite};

use self::{db::Users, token::Tokens};
use crate::conf;

mod db;
mod endpoints;
mod model;
mod passwd;
mod token;

const JWT_SECRET_VAR_NAME: &str = "DE_JWT_SECRET";
const MIN_SECRET_LEN: usize = 12;
const MAX_SECRET_LEN: usize = 86;

/// This struct can be used to setup authentication on an actix-web App.
#[derive(Clone)]
pub struct Auth {
    tokens: Tokens,
    users: Users,
}

impl Auth {
    /// Creates and sets up new authentication object. This method should be
    /// called only once during the application startup.
    ///
    /// The resulting object can be repeatedly used to configure an actix-web
    /// App.
    pub async fn setup(pool: Arc<Pool<Sqlite>>) -> Result<Self> {
        let jwt_secret: String = conf::mandatory(JWT_SECRET_VAR_NAME)?;

        ensure!(
            jwt_secret.len() >= MIN_SECRET_LEN,
            "JWT secret is too short: {} < {}",
            jwt_secret.len(),
            MIN_SECRET_LEN
        );
        ensure!(
            jwt_secret.len() <= MAX_SECRET_LEN,
            "JWT secret is too long: {} > {}",
            jwt_secret.len(),
            MAX_SECRET_LEN
        );

        Ok(Self {
            tokens: Tokens::new(jwt_secret.as_str()).context("Failed to initialize tokens")?,
            users: Users::init(pool)
                .await
                .context("Failed to initialize users")?,
        })
    }

    /// Configure actix-web application.
    pub fn configure(&self, cfg: &mut web::ServiceConfig) {
        cfg.app_data(web::Data::new(self.tokens.clone()));
        cfg.app_data(web::Data::new(self.users.clone()));
        endpoints::configure(cfg);
    }
}
