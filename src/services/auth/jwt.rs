use std::time::Duration;

use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::JwtConfig,
    error::{AppError, Result},
    services::auth::TokenType,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: Uuid,
    pub wallet: String,
    pub exp: u64,
    pub iat: u64,
    pub jti: String,
    pub token_type: TokenType,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_ttl: Duration,
    refresh_ttl: Duration,
}

impl JwtService {
    pub fn new(jwt_config: &JwtConfig) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(jwt_config.secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(jwt_config.secret.as_bytes()),
            access_ttl: jwt_config.access_token_ttl,
            refresh_ttl: jwt_config.refresh_token_ttl,
        }
    }

    fn create_token(
        &self,
        user_id: Uuid,
        wallet: &str,
        token_type: TokenType,
        ttl: Duration,
    ) -> Result<String> {
        let now = Utc::now().timestamp() as u64;

        let claims = JwtClaims {
            sub: user_id,
            wallet: wallet.to_string(),
            exp: now + ttl.as_secs(),
            iat: now,
            jti: Uuid::new_v4().to_string(),
            token_type,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::InternalServerError(e.to_string()))
    }

    pub fn create_access_token(&self, user_id: Uuid, wallet: &str) -> Result<String> {
        self.create_token(user_id, wallet, TokenType::Access, self.access_ttl)
    }

    pub fn create_refresh_token(&self, user_id: Uuid, wallet: &str) -> Result<String> {
        self.create_token(user_id, wallet, TokenType::Refresh, self.refresh_ttl)
    }

    pub fn validate_token(&self, token: &str, expected_type: TokenType) -> Result<JwtClaims> {
        let validation = Validation::default();

        let data =
            decode::<JwtClaims>(token, &self.decoding_key, &validation).map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
                    _ => AppError::Unauthorized,
                }
            })?;

        let claims = data.claims;
        if claims.token_type != expected_type {
            return Err(AppError::Unauthorized);
        }
        Ok(claims)
    }
}
