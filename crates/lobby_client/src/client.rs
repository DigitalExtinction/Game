use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use async_compat::Compat;
use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use reqwest::{header::HeaderValue, redirect::Policy, Client, Request};
use url::Url;

use crate::requestable::LobbyRequestCreator;

const USER_AGENT: &str = concat!("DigitalExtinction/", env!("CARGO_PKG_VERSION"));

#[derive(SystemParam)]
pub(super) struct AuthenticatedClient<'w> {
    auth: Res<'w, Authentication>,
    client: Option<Res<'w, LobbyClient>>,
}

impl<'w> AuthenticatedClient<'w> {
    pub(super) fn fire<T: LobbyRequestCreator>(
        &self,
        requestable: &T,
    ) -> Result<Task<Result<T::Response>>> {
        let Some(client) = self.client.as_ref() else {
            bail!("Client not yet set up.")
        };
        let request = client.create(self.auth.token(), requestable)?;
        Ok(client.fire::<T>(request))
    }
}

/// Lobby client authentication object. It should be used to get current
/// authentication state.
#[derive(Resource, Default)]
pub struct Authentication {
    token: Option<String>,
}

impl Authentication {
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub(super) fn set_token(&mut self, token: String) {
        self.token = Some(token)
    }
}

#[derive(Resource)]
pub(super) struct LobbyClient {
    server_url: Url,
    client: Client,
}

impl LobbyClient {
    pub(super) fn build(server_url: Url) -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .redirect(Policy::none())
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();

        Self { server_url, client }
    }

    fn create<T: LobbyRequestCreator>(
        &self,
        token: Option<&str>,
        requestable: &T,
    ) -> Result<Request> {
        let path = requestable.path();
        let url = self
            .server_url
            .join(path.as_ref())
            .context("Endpoint URL construction error")?;
        let mut request = requestable.create(url);

        // All authenticated endpoints start with /a all public endpoints start
        // with /p per DE Lobby API design.
        let authenticated = path.starts_with("/a");
        if authenticated {
            match token {
                Some(token) => {
                    let mut value = HeaderValue::try_from(format!("Bearer {token}"))
                        .context("Failed crate Authorization header value from the JWT")?;
                    value.set_sensitive(true);
                    request.headers_mut().insert("Authorization", value);
                }
                None => bail!("The client is not yet authenticated."),
            }
        }

        Ok(request)
    }

    fn fire<T: LobbyRequestCreator>(&self, request: Request) -> Task<Result<T::Response>> {
        info!("Requesting {} {}", request.method(), request.url());
        let client = self.client.clone();

        IoTaskPool::get().spawn(Compat::new(async move {
            let resonse = client
                .execute(request)
                .await
                .context("Failed to execute the request")?;

            let status = resonse.status();
            if status.is_success() {
                let text = resonse
                    .text()
                    .await
                    .context("Failed to load server response")?;
                let response = serde_json::from_str(text.as_str())
                    .context("Failed to parse server response")?;
                Ok(response)
            } else if status.is_server_error() {
                Err(anyhow!("Server side error occurred."))
            } else {
                let reason = status.canonical_reason().unwrap_or_else(|| status.as_str());
                let text = resonse
                    .text()
                    .await
                    .context("Failed to load server error response")?;
                Err(anyhow!("{}: {}", reason, text))
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use de_lobby_model::UsernameAndPassword;

    use super::*;
    use crate::{ListGamesRequest, SignInRequest};

    #[test]
    fn test_create() {
        let client = LobbyClient::build(Url::parse("https://example.com").unwrap());

        let sign_in = SignInRequest::new(UsernameAndPassword::new(
            "Indy".to_owned(),
            "123456".to_owned(),
        ));
        let request = client.create(None, &sign_in).unwrap();
        assert!(request.headers().get("Authorization").is_none());

        let request = client
            .create(Some("some-token"), &ListGamesRequest)
            .unwrap();
        assert_eq!(
            request
                .headers()
                .get("Authorization")
                .unwrap()
                .to_str()
                .unwrap(),
            "Bearer some-token"
        );
    }
}
