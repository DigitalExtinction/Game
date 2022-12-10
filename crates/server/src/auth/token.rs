use anyhow::{Context, Result};
use jsonwebtoken::{
    decode, encode, get_current_timestamp, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

const TOKEN_LIFETIME: u64 = 86400;

/// Client authentication token claims.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: u64,
}

impl Claims {
    /// Creates and returns new claims for a particular user. The expiration is
    /// set to now + a fixed offset.
    pub(super) fn standard<U: Into<String>>(username: U) -> Self {
        Self {
            sub: username.into(),
            exp: get_current_timestamp() + TOKEN_LIFETIME,
        }
    }
}

#[derive(Clone)]
pub(super) struct Tokens {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl Tokens {
    pub(super) fn new(secret: &str) -> Result<Self> {
        let secret = base64::decode(secret).context("Failed to decode JWT secret")?;
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        Ok(Self {
            encoding_key,
            decoding_key,
        })
    }

    /// Encodes a user ID into a new JWT.
    pub(super) fn encode(&self, claims: &Claims) -> Result<String> {
        encode(&Header::default(), &claims, &self.encoding_key).context("Failed to encode JWT")
    }

    /// Decodes and validates a JWT.
    pub fn decode(&self, token: &str) -> Result<Claims> {
        decode(token, &self.decoding_key, &Validation::default())
            .context("Failed to decode JWT")
            .map(|t| t.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokens() {
        let secret_base64 = "eHg=";
        let tokens = Tokens::new(secret_base64).unwrap();
        let token_a = tokens.encode(&Claims::standard("Indy")).unwrap();
        let token_b = tokens.encode(&Claims::standard("Indy2")).unwrap();
        assert_ne!(token_a, token_b);
        assert_eq!(tokens.decode(&token_a).unwrap().sub, "Indy");
        assert_eq!(tokens.decode(&token_b).unwrap().sub, "Indy2");
    }
}
