use std::time::Duration;

use axum::http::{HeaderMap, header};
use chrono::Utc;

use crate::error::{AppError, Result};

pub mod auth;
pub mod canvas;
pub mod nft;
pub mod pixel;

fn validate_wallet_address(wallet: &str) -> Result<()> {
    let decoded = bs58::decode(wallet)
        .into_vec()
        .map_err(|_| AppError::InvalidParams("Invalid wallet address format".into()))?;

    if decoded.len() != 32 {
        return Err(AppError::InvalidParams(
            "Invalid wallet address: must be 32 bytes".into(),
        ));
    }

    Ok(())
}

fn calculate_remaining_ttl(exp: u64) -> Option<Duration> {
    let now = Utc::now().timestamp() as u64;
    exp.checked_sub(now)
        .map(Duration::from_secs)
        .filter(|d| d.as_secs() > 0)
}

pub fn extract_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|val| val.to_str().ok())
        .and_then(|val| {
            val.split(';').find_map(|cookie| {
                cookie
                    .trim()
                    .strip_prefix(&format!("{}=", name))
                    .map(|cookie_str| cookie_str.to_string())
            })
        })
}
