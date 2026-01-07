use serde::{Deserialize, Serialize};

pub mod cookie;
pub mod jwt;
pub mod signature;

pub use jwt::{JwtClaims, JwtService};
pub use signature::{check_and_consume_nonce, parse_auth_message, verify_signature};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

impl TokenType {
    pub fn name(&self) -> &'static str {
        match self {
            TokenType::Access => "access_token",
            TokenType::Refresh => "refresh_token",
        }
    }
}
