//! Authentication and user management related API objects.

use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

const MIN_PASSWORD_LEN: usize = 6;
const MAX_PASSWORD_LEN: usize = 30;
pub(super) const MAX_USERNAME_LEN: usize = 32;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TokenResponse {
    token: String,
}

impl TokenResponse {
    pub(super) fn new(token: String) -> Self {
        Self { token }
    }
}

/// Username & password to be used while signing in.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UsernameAndPassword {
    username: String,
    password: String,
}

impl UsernameAndPassword {
    pub(super) fn username(&self) -> &str {
        self.username.as_str()
    }

    pub(super) fn password(&self) -> &str {
        self.password.as_str()
    }
}

/// User object combined with a password. To be used while signing up.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UserWithPassword {
    password: String,
    user: User,
}

impl UserWithPassword {
    pub(super) fn user(&self) -> &User {
        &self.user
    }

    pub(super) fn password(&self) -> &str {
        self.password.as_str()
    }

    pub(super) fn validate(&self) -> Result<()> {
        self.user.validate()?;

        ensure!(
            self.password.len() >= MIN_PASSWORD_LEN,
            "Password must have at least {} characters.",
            MIN_PASSWORD_LEN
        );

        ensure!(
            self.password.len() <= MAX_PASSWORD_LEN,
            "Password must have at most {} bytes.",
            MAX_PASSWORD_LEN
        );

        Ok(())
    }
}

/// A complete user info.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    username: String,
}

impl User {
    pub(super) fn new(username: String) -> Self {
        Self { username }
    }

    pub fn username(&self) -> &str {
        self.username.as_str()
    }

    pub(super) fn validate(&self) -> Result<()> {
        ensure!(!self.username.is_empty(), "Empty username is not allowed.");
        ensure!(
            self.username.trim().len() == self.username.len(),
            "Username starting or ending with whitespace is not allowed."
        );
        ensure!(
            self.username.len() <= MAX_USERNAME_LEN,
            "Username has {} characters, which is more than the limit of {} characters.",
            self.username.len(),
            MAX_USERNAME_LEN
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_user() {
        let mut user = UserWithPassword {
            password: "short".to_owned(),
            user: User {
                username: "Indy".to_owned(),
            },
        };
        assert_eq!(
            user.validate().err().unwrap().to_string(),
            "Password must have at least 6 characters."
        );

        user.password = "Long-enough-pwd".to_owned();
        assert!(user.validate().is_ok());

        user.user.username = "Indy ".to_owned();
        assert_eq!(
            user.validate().err().unwrap().to_string(),
            "Username starting or ending with whitespace is not allowed."
        );
    }
}
