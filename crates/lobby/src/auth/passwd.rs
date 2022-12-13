use anyhow::{Context, Error, Result};
use pbkdf2::{
    password_hash::{Output, PasswordHasher, Salt, SaltString},
    Pbkdf2,
};
use rand_core::OsRng;
use subtle::ConstantTimeEq;

pub(super) const MAX_PASS_HASH_LEN: usize = Output::B64_MAX_LENGTH;
// `RECOMMENDED_LENGTH` bytes are B64 encoded by `SaltString`
pub(super) const MAX_PASS_SALT_LEN: usize = ((Salt::RECOMMENDED_LENGTH * 4) / 3) + 1;

/// Representation of a user password which can be safely loaded from and
/// stored to a database.
pub(super) struct DbPassword(Output, SaltString);

impl DbPassword {
    /// Create new DB password from a non-hashed original password and a random
    /// salt.
    pub(super) fn generate(password: &str) -> Result<Self> {
        let salt = SaltString::generate(&mut OsRng);
        let hashed = Self::hash(password, &salt)?;
        Ok(Self::new(hashed, salt))
    }

    fn new(hashed: Output, salt: SaltString) -> Self {
        Self(hashed, salt)
    }

    fn hash(password: &str, salt: &SaltString) -> Result<Output> {
        Pbkdf2
            .hash_password(password.as_bytes(), salt)
            .context("Failed to hash the password")?
            .hash
            .context("Password hash could not be retrieved")
    }

    /// Returns Base64 encoded, hashed & salted password.
    pub(super) fn b64_encode_pwd_hash(&self) -> Result<String> {
        let mut output = [0; MAX_PASS_HASH_LEN];
        Ok(self
            .0
            .b64_encode(&mut output)
            .context("Failed to encode password hash")?
            .to_owned())
    }

    /// Returns password salt.
    pub(super) fn salt_str(&self) -> &str {
        self.1.as_str()
    }

    /// Securely check that a given password corresponds to the password
    /// represented by `self`.
    pub(super) fn check(&self, password: &str) -> bool {
        let Ok(hashed) = Self::hash(password, &self.1) else { return false };
        self.0.ct_eq(&hashed).into()
    }
}

impl TryFrom<(&str, &str)> for DbPassword {
    type Error = Error;

    fn try_from(values: (&str, &str)) -> Result<Self> {
        let hashed = Output::b64_decode(values.0).context("Failed to decode password hash")?;
        let salt = SaltString::new(values.1).context("Invalid password salt loaded from the DB")?;
        Ok(Self::new(hashed, salt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tast_password() {
        let first = DbPassword::generate("heslo123").unwrap();
        let second = DbPassword::generate("heslo123").unwrap();

        assert_ne!(first.salt_str(), second.salt_str());
        assert_ne!(
            first.b64_encode_pwd_hash().unwrap(),
            second.b64_encode_pwd_hash().unwrap()
        );

        assert!(first.check("heslo123"));
        assert!(second.check("heslo123"));
        assert!(!first.check("heslo12"));
        assert!(!second.check("heslo1234"));

        let pwd_hash = first.b64_encode_pwd_hash().unwrap();
        let pwd_salt = first.salt_str();
        let end2end = DbPassword::try_from((pwd_hash.as_str(), pwd_salt)).unwrap();
        assert!(end2end.check("heslo123"));
        assert!(!end2end.check("heslo12"));
    }
}
