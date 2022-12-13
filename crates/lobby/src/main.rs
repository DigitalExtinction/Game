use actix_web::{middleware::Logger, web, App, HttpServer};
use anyhow::{Context, Result};
use auth::{Auth, AuthMiddlewareFactory};
use games::GamesService;
use log::info;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

mod auth;
mod conf;
mod db;
mod games;

const JSON_PAYLOAD_LIMIT: usize = 10 * 1024;
const DB_URL_VAR_NAME: &str = "DE_DB_URL";
const HTTP_PORT_VAR_NAME: &str = "DE_HTTP_PORT";
const DEFAULT_HTTP_PORT: u16 = 8080;

macro_rules! handle_error {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(error) => {
                log::error!("{}", error);
                panic!("{:?}", error);
            }
        }
    };
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    handle_error!(env_logger::try_init().context("Failed to init the logger"));

    let json_cfg = web::JsonConfig::default()
        .limit(JSON_PAYLOAD_LIMIT)
        .content_type(|mime| mime == mime::APPLICATION_JSON)
        .content_type_required(true);

    let http_port: u16 = handle_error!(conf::optional(HTTP_PORT_VAR_NAME, DEFAULT_HTTP_PORT));
    info!("HTTP port set to {}", http_port);

    let db_pool = handle_error!(db_pool().await);
    let auth = handle_error!(Auth::setup(db_pool).await);
    let games = handle_error!(GamesService::setup(db_pool).await);

    HttpServer::new(move || {
        let public_scope = web::scope("/p").configure(|c| auth.configure_public(c));
        let authenticated_scope = web::scope("/a")
            .wrap(AuthMiddlewareFactory)
            .configure(|c| games.configure(c));

        App::new()
            .wrap(Logger::default())
            .app_data(json_cfg.clone())
            .configure(|c| auth.configure_root(c))
            .service(public_scope)
            .service(authenticated_scope)
    })
    .bind(("0.0.0.0", http_port))?
    .run()
    .await
}

/// Loads DB configuration and setup SQLite DB pool.
async fn db_pool() -> Result<&'static Pool<Sqlite>> {
    let db_url: String = conf::mandatory(DB_URL_VAR_NAME)?;
    let pool = SqlitePoolOptions::new()
        .connect(db_url.as_str())
        .await
        .with_context(|| format!("Failed to connect to the SQLite DB with URL {}", db_url))?;
    Ok(Box::leak(Box::new(pool)))
}
