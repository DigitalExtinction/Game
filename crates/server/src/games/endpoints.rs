use actix_web::{post, web, HttpResponse, Responder};
use log::{error, warn};

use super::{
    db::{AdditionError, CreationError, Games},
    model::{Game, GameConfig},
};

/// Registers all authentication endpoints.
pub(super) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/a/games").service(create));
}

#[post("/")]
async fn create(games: web::Data<Games>, game_config: web::Json<GameConfig>) -> impl Responder {
    let game_config = game_config.into_inner();
    if let Err(error) = game_config.validate() {
        warn!("Invalid game configuration: {:?}", error);
        return HttpResponse::BadRequest().json(format!("{}", error));
    }

    let game = Game::new(game_config, "Indy".to_owned());

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
