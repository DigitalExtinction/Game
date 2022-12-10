use actix_web::{get, post, web, HttpResponse, Responder};
use log::{error, warn};

use super::{
    db::{AdditionError, CreationError, Games},
    model::{Game, GameConfig},
};
use crate::auth::Claims;

/// Registers all authentication endpoints.
pub(super) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/games").service(create).service(list));
}

#[post("/")]
async fn create(
    claims: web::ReqData<Claims>,
    games: web::Data<Games>,
    game_config: web::Json<GameConfig>,
) -> impl Responder {
    let game_config = game_config.into_inner();
    if let Err(error) = game_config.validate() {
        warn!("Invalid game configuration: {:?}", error);
        return HttpResponse::BadRequest().json(format!("{}", error));
    }

    let game = Game::new(game_config, claims.username().to_owned());
    match games.create(game).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(CreationError::NameTaken) => {
            warn!("Game creation error: game name is already taken.");
            HttpResponse::Conflict().json("Game name is already taken.")
        }
        Err(CreationError::AdditionError(AdditionError::AlreadyInAGame)) => {
            warn!("Game creation error: a user is already in a different game.");
            HttpResponse::Forbidden().json("User is already in different game.")
        }
        Err(error) => {
            error!("Game creation error: {:?}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/")]
async fn list(games: web::Data<Games>) -> impl Responder {
    match games.list().await {
        Ok(games) => HttpResponse::Ok().json(games),
        Err(error) => {
            error!("Game listing error: {:?}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}
