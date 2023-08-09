use std::borrow::Cow;

use de_lobby_model::{
    Game, GameListing, GamePlayerInfo, GameSetup, Token, UserWithPassword, UsernameAndPassword,
};
use reqwest::{header::HeaderValue, Method, Request};
use serde::Serialize;
use url::Url;

use crate::requestable::{LobbyRequest, LobbyRequestCreator};

pub struct SignUpRequest(UserWithPassword);

impl SignUpRequest {
    pub fn new(params: UserWithPassword) -> Self {
        Self(params)
    }
}

impl LobbyRequest for SignUpRequest {
    type Response = Token;
}

impl LobbyRequestCreator for SignUpRequest {
    fn path(&self) -> Cow<str> {
        "/p/auth/sign-up".into()
    }

    fn create(&self, url: Url) -> Request {
        let mut request = Request::new(Method::POST, url);
        json(&mut request, &self.0);
        request
    }
}

pub struct SignInRequest(UsernameAndPassword);

impl SignInRequest {
    pub fn new(params: UsernameAndPassword) -> Self {
        Self(params)
    }
}

impl LobbyRequest for SignInRequest {
    type Response = Token;
}

impl LobbyRequestCreator for SignInRequest {
    fn path(&self) -> Cow<str> {
        "/p/auth/sign-in".into()
    }

    fn create(&self, url: Url) -> Request {
        let mut request = Request::new(Method::POST, url);
        json(&mut request, &self.0);
        request
    }
}

pub struct CreateGameRequest(GameSetup);

impl CreateGameRequest {
    pub fn new(setup: GameSetup) -> Self {
        Self(setup)
    }
}

impl LobbyRequest for CreateGameRequest {
    type Response = ();
}

impl LobbyRequestCreator for CreateGameRequest {
    fn path(&self) -> Cow<str> {
        "/a/games".into()
    }

    fn create(&self, url: Url) -> Request {
        let mut request = Request::new(Method::POST, url);
        json(&mut request, &self.0);
        request
    }
}

pub struct ListGamesRequest;

impl LobbyRequest for ListGamesRequest {
    type Response = GameListing;
}

impl LobbyRequestCreator for ListGamesRequest {
    fn path(&self) -> Cow<str> {
        "/a/games".into()
    }

    fn create(&self, url: Url) -> Request {
        Request::new(Method::GET, url)
    }
}

pub struct GetGameRequest(String);

impl GetGameRequest {
    pub fn new(id: impl ToString) -> Self {
        Self(id.to_string())
    }
}

impl LobbyRequest for GetGameRequest {
    type Response = Game;
}

impl LobbyRequestCreator for GetGameRequest {
    fn path(&self) -> Cow<str> {
        encode(&["a", "games", self.0.as_str()])
    }

    fn create(&self, url: Url) -> Request {
        Request::new(Method::GET, url)
    }
}

pub struct JoinGameRequest {
    game: String,
    player: GamePlayerInfo,
}

impl JoinGameRequest {
    pub fn new(game: String, player: GamePlayerInfo) -> Self {
        Self { game, player }
    }
}

impl LobbyRequest for JoinGameRequest {
    type Response = ();
}

impl LobbyRequestCreator for JoinGameRequest {
    fn path(&self) -> Cow<str> {
        encode(&["a", "games", self.game.as_str(), "join"])
    }

    fn create(&self, url: Url) -> Request {
        let mut request = Request::new(Method::PUT, url);
        json(&mut request, &self.player);
        request
    }
}

pub struct LeaveGameRequest(String);

impl LeaveGameRequest {
    pub fn new(name: String) -> Self {
        Self(name)
    }
}

impl LobbyRequest for LeaveGameRequest {
    type Response = ();
}

impl LobbyRequestCreator for LeaveGameRequest {
    fn path(&self) -> Cow<str> {
        encode(&["a", "games", self.0.as_str(), "leave"])
    }

    fn create(&self, url: Url) -> Request {
        Request::new(Method::PUT, url)
    }
}

fn json<T: Serialize>(request: &mut Request, content: &T) {
    request.headers_mut().insert(
        "Content-Type",
        HeaderValue::try_from("application/json").unwrap(),
    );
    *request.body_mut() = Some(serde_json::to_string(&content).unwrap().into());
}

fn encode(parts: &[&str]) -> Cow<'static, str> {
    let mut result = String::new();
    for part in parts {
        result.push('/');
        result.push_str(urlencoding::encode(part).as_ref());
    }
    result.into()
}

#[cfg(test)]
mod tests {
    use de_lobby_model::{GameConfig, GameMap, User};

    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(encode(&["ahoj", "svete"]), "/ahoj/svete");
        assert_eq!(encode(&["ahoj", "velky svete"]), "/ahoj/velky%20svete");
    }

    #[test]
    fn test_sign_up() {
        let request = SignUpRequest::new(UserWithPassword::new(
            "Obviously 123456".to_owned(),
            User::new("Indy".to_owned()),
        ));
        assert_eq!(request.path().as_ref(), "/p/auth/sign-up");

        let request = request.create(Url::parse("https://example.com/p/auth/sign-up").unwrap());
        assert_eq!(request.method().as_str(), "POST");
        assert_eq!(request.url().as_str(), "https://example.com/p/auth/sign-up");

        let body = String::from_utf8(request.body().unwrap().as_bytes().unwrap().to_vec()).unwrap();
        let expected_body = r#"{"password":"Obviously 123456","user":{"username":"Indy"}}"#;
        assert_eq!(body, expected_body);
    }

    #[test]
    fn test_sign_in() {
        let request = SignInRequest::new(UsernameAndPassword::new(
            "Martin Indra".to_owned(),
            "Obviously 123456".to_owned(),
        ));
        assert_eq!(request.path().as_ref(), "/p/auth/sign-in");

        let request = request.create(Url::parse("https://example.com/p/auth/sign-in").unwrap());
        assert_eq!(request.method().as_str(), "POST");
        assert_eq!(request.url().as_str(), "https://example.com/p/auth/sign-in");

        let body = String::from_utf8(request.body().unwrap().as_bytes().unwrap().to_vec()).unwrap();
        let expected_body = r#"{"username":"Martin Indra","password":"Obviously 123456"}"#;
        assert_eq!(body, expected_body);
    }

    #[test]
    fn test_create() {
        let config = GameConfig::new(
            "Druhá Hra".to_owned(),
            2,
            GameMap::new(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_owned(),
                "custom".to_owned(),
            ),
        );
        let request =
            CreateGameRequest::new(GameSetup::new("127.0.0.1:8082".parse().unwrap(), config));
        assert_eq!(request.path().as_ref(), "/a/games");

        let request = request.create(Url::parse("http://example.com/a/games").unwrap());
        assert_eq!(request.method().as_str(), "POST");
        assert_eq!(request.url().as_str(), "http://example.com/a/games");

        let body = String::from_utf8(request.body().unwrap().as_bytes().unwrap().to_vec()).unwrap();
        let expected_body = concat!(
            r#"{"server":"127.0.0.1:8082","config":{"name":"Druhá Hra","maxPlayers":2,"#,
            r#""map":{"hash":"#,
            r#""0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef","#,
            r#""name":"custom"}}}"#
        );

        assert_eq!(body, expected_body);
    }

    #[test]
    fn test_join() {
        let request = JoinGameRequest::new("Cool Game".to_owned(), GamePlayerInfo::new(2));
        assert_eq!(request.path().as_ref(), "/a/games/Cool%20Game/join");

        let request = request.create(Url::parse("http://example.com/a/games/123/join").unwrap());
        let body = String::from_utf8(request.body().unwrap().as_bytes().unwrap().to_vec()).unwrap();
        assert_eq!(body, r#"{"ordinal":2}"#);
    }

    #[test]
    fn test_leave() {
        let request = LeaveGameRequest::new("První Hra".to_owned());
        assert_eq!(request.path().as_ref(), "/a/games/Prvn%C3%AD%20Hra/leave");
    }
}
