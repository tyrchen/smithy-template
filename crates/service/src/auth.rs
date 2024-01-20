use derive_more::Debug;
use echo_server_sdk::error::{ForbiddenError, SigninError};
use jwt_simple::prelude::*;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub sk: String,
    pub pk: String,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("jwt error: {0}")]
    JWTError(#[from] jwt_simple::Error),
}

type Result<T> = std::result::Result<T, AuthError>;

const TOKEN_DURATION: u64 = 14;

#[derive(Serialize, Deserialize)]
pub struct CustomClaims {
    pub data: String,
}

#[derive(Debug, Clone)]
pub struct AuthSigner {
    provider: String,
    #[debug(skip)]
    key: Ed25519KeyPair,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuthVerifier {
    provider: String,
    key: Ed25519PublicKey,
}

impl AuthSigner {
    pub fn try_new(provider: impl Into<String>, key: impl AsRef<str>) -> Result<Self> {
        let key = Ed25519KeyPair::from_pem(key.as_ref())?;

        Ok(Self {
            provider: provider.into(),
            key,
        })
    }

    pub fn sign(&self, data: String) -> Result<String> {
        let claims =
            Claims::with_custom_claims(CustomClaims { data }, Duration::from_days(TOKEN_DURATION))
                .with_issuer(&self.provider)
                .with_subject("auth");
        let token = self.key.sign(claims)?;
        Ok(token)
    }
}

impl AuthVerifier {
    pub fn try_new(provider: impl Into<String>, key: impl AsRef<str>) -> Result<Self> {
        let key = Ed25519PublicKey::from_pem(key.as_ref())?;
        Ok(Self {
            provider: provider.into(),
            key,
        })
    }

    pub fn verify(&self, token: impl AsRef<str>) -> Result<JWTClaims<CustomClaims>> {
        let token = token.as_ref();
        let claims = self
            .key
            .verify_token::<CustomClaims>(token, Some(VerificationOptions::default()))?;
        Ok(claims)
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        let sk = Ed25519KeyPair::generate();
        Self {
            sk: sk
                .to_pem()
                .split("-----BEGIN PUBLIC KEY-----")
                .next()
                .unwrap()
                .to_string(),
            pk: sk.public_key().to_pem(),
        }
    }
}

impl From<AuthError> for SigninError {
    fn from(e: AuthError) -> Self {
        Self::ForbiddenError(ForbiddenError {
            message: e.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {}
