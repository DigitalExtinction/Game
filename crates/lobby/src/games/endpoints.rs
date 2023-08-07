use actix_web::{get, post, put, web, HttpResponse, Responder};
use de_lobby_model::{Game, GamePlayer, GamePlayerInfo, GameSetup, Validatable};
use log::{error, warn};

use super::db::{AdditionError, CreationError, Games, RemovalError};
use crate::auth::Claims;

/// Registers all authentication endpoints.
pub(super) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/games")
            .service(create)
            .service(get)
            .service(list)
            .service(join)
            .service(leave),
    );
}

#[post("")]
async fn create(
    claims: web::ReqData<Claims>,
    games: web::Data<Games>,
    game_setup: web::Json<GameSetup>,
) -> impl Responder {
    let game_setup = game_setup.into_inner();
    if let Err(error) = game_setup.validate() {
        warn!("Invalid game setup: {:?}", error);
        return HttpResponse::BadRequest().json(format!("{error}"));
    }

    let game = Game::from_author(game_setup, claims.username().to_owned());
    match games.create(game).await {
        Ok(_) => HttpResponse::Ok().json(()),
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

#[get("/{name}")]
async fn get(path: web::Path<String>, games: web::Data<Games>) -> impl Responder {
    let name = path.into_inner();
    match games.get(&name).await {
        Ok(Some(game)) => HttpResponse::Ok().json(game),
        Ok(None) => HttpResponse::NotFound().json("Game not found"),
        Err(error) => {
            error!("Game get error: {:?}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("")]
async fn list(games: web::Data<Games>) -> impl Responder {
    match games.list().await {
        Ok(games) => HttpResponse::Ok().json(games),
        Err(error) => {
            error!("Game listing error: {:?}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[put("/{name}/join")]
async fn join(
    claims: web::ReqData<Claims>,
    games: web::Data<Games>,
    path: web::Path<String>,
    player_info: web::Json<GamePlayerInfo>,
) -> impl Responder {
    let name = path.into_inner();

    // TODO error code for conflict
    // TODO error if ordinal >= max players

    let player = GamePlayer::new(claims.username().to_owned(), player_info.0);
    match games.add_player(&player, name.as_str()).await {
        Ok(_) => HttpResponse::Ok().json(()),
        Err(AdditionError::AlreadyInAGame) => {
            warn!("Game joining error: a user is already in a different game.");
            HttpResponse::Forbidden().json("User is already in a different game.")
        }
        Err(AdditionError::UserOrGameDoesNotExist) => {
            warn!("Game joining error: the game or the user does not exist");
            HttpResponse::NotFound().json("Game not found.")
        }
        Err(error) => {
            error!("Error while adding a player to a game: {:?}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[put("/{name}/leave")]
async fn leave(
    claims: web::ReqData<Claims>,
    games: web::Data<Games>,
    path: web::Path<String>,
) -> impl Responder {
    let name = path.into_inner();

    match games.remove_player(claims.username(), name.as_str()).await {
        Ok(_) => HttpResponse::Ok().json(()),
        Err(RemovalError::NotInTheGame) => {
            warn!("Game leaving error: the user is not in the game.");
            HttpResponse::Forbidden().json("The user is not in the game.")
        }
        Err(error) => {
            error!("Error while removing a player from a game: {:?}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}
