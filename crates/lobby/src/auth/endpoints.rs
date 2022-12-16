use actix_web::{post, web, HttpResponse, Responder};
use de_lobby_model::{Token, UserWithPassword, UsernameAndPassword};
use log::{error, info, warn};

use super::{
    db::{RegistrationError, Users},
    token::{Claims, Tokens},
};

/// Registers all authentication endpoints.
pub(super) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").service(sign_up).service(sign_in));
}

#[post("/sign-up")]
async fn sign_up(
    tokens: web::Data<Tokens>,
    users: web::Data<Users>,
    user: web::Json<UserWithPassword>,
) -> impl Responder {
    if let Err(error) = user.validate() {
        warn!("Invalid sing-up request: {}", error);
        return HttpResponse::BadRequest().json(error.to_string());
    }

    match users.register(&user.0).await {
        Ok(_) => {
            let token = match tokens.encode(&Claims::standard(user.0.user().username())) {
                Ok(token) => token,
                Err(error) => {
                    error!("Token encoding error: {:?}", error);
                    return HttpResponse::InternalServerError().finish();
                }
            };
            info!(
                "Registration of user {} was successful.",
                user.user().username()
            );
            HttpResponse::Ok().json(Token::new(token))
        }
        Err(RegistrationError::UsernameTaken) => {
            warn!("Username {} is already taken.", user.user().username());
            HttpResponse::Conflict().json("The username is already taken.")
        }
        Err(error) => {
            error!("Registration error: {:?}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/sign-in")]
async fn sign_in(
    tokens: web::Data<Tokens>,
    users: web::Data<Users>,
    user: web::Json<UsernameAndPassword>,
) -> impl Responder {
    match users.login(&user.0).await {
        Ok(false) => {
            warn!("Signing in of user {} was unsuccessful.", user.username());
            HttpResponse::Unauthorized().finish()
        }
        Ok(true) => {
            let token = match tokens.encode(&Claims::standard(user.0.username())) {
                Ok(token) => token,
                Err(error) => {
                    error!("Token encoding error: {:?}", error);
                    return HttpResponse::InternalServerError().finish();
                }
            };
            info!("Signing in of user {} was successful.", user.username());
            HttpResponse::Ok().json(Token::new(token))
        }
        Err(error) => {
            error!("Sign-in error: {:?}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}
