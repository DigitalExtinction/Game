use std::future::{ready, Ready};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    http::header::Header,
    web, Error, HttpMessage,
};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use futures_util::future::LocalBoxFuture;
use log::warn;

use super::token::Tokens;

pub struct AuthMiddlewareFactory;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware { service }))
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let tokens = req.app_data::<web::Data<Tokens>>().unwrap().as_ref();

        match Authorization::<Bearer>::parse(&req) {
            Ok(auth) => match tokens.decode(auth.as_ref().token()) {
                Ok(claims) => {
                    let previous = req.extensions_mut().insert(claims);
                    assert!(previous.is_none());
                }
                Err(error) => {
                    warn!("JWT decoding error: {:?}", error);
                    return Box::pin(async move {
                        Err(ErrorUnauthorized(format!(
                            "Invalid Bearer token provided: {}",
                            error
                        )))
                    });
                }
            },
            Err(error) => {
                warn!("JWT extraction error: {:?}", error);
                return Box::pin(async move {
                    Err(ErrorUnauthorized(
                        "Authorization header with Bearer token not provided.",
                    ))
                });
            }
        }

        let fut = self.service.call(req);
        Box::pin(async move { fut.await })
    }
}
