use std::time::Duration;

use chrono::Utc;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::{
    error::{AppError, Result},
    infrastructure::cache::Cache,
    services::MESSAGE_VALIDITY_SECS,
};

pub struct AuthMessage {
    pub wallet: String,
    pub timestamp: u64,
    pub nonce: String,
}

pub fn verify_signature(wallet: &str, message: &str, signature: &str) -> Result<()> {
    let pubkey_bytes = bs58::decode(wallet)
        .into_vec()
        .map_err(|_| AppError::InvalidSignature)?;

    if pubkey_bytes.len() != 32 {
        return Err(AppError::InvalidSignature);
    }

    let pubkey_array: [u8; 32] = pubkey_bytes
        .try_into()
        .map_err(|_| AppError::InvalidSignature)?;

    let verifying_key =
        VerifyingKey::from_bytes(&pubkey_array).map_err(|_| AppError::InvalidSignature)?;

    let signature_bytes = bs58::decode(signature)
        .into_vec()
        .map_err(|_| AppError::InvalidSignature)?;

    if signature_bytes.len() != 64 {
        return Err(AppError::InvalidSignature);
    }

    let signature_array: [u8; 64] = signature_bytes
        .try_into()
        .map_err(|_| AppError::InvalidSignature)?;

    let byte_signature = Signature::from_bytes(&signature_array);

    verifying_key
        .verify(message.as_bytes(), &byte_signature)
        .map_err(|_| AppError::InvalidSignature)?;

    Ok(())
}

pub fn parse_auth_message(message: &str) -> Result<AuthMessage> {
    let parts: Vec<&str> = message.split(':').collect();

    if parts.len() != 4 || parts[0] != "pixel" {
        return Err(AppError::InvalidSignature);
    }

    let wallet = parts[1].to_string();
    let timestamp: u64 = parts[2].parse().map_err(|_| AppError::InvalidSignature)?;
    let nonce = parts[3].to_string();

    let now = Utc::now().timestamp() as u64;

    if now.abs_diff(timestamp) > MESSAGE_VALIDITY_SECS {
        return Err(AppError::InvalidSignature);
    }

    Ok(AuthMessage {
        wallet,
        timestamp,
        nonce,
    })
}

pub async fn check_and_consume_nonce(cache: &Cache, wallet: &str, nonce: &str) -> Result<()> {
    let nonce_key = format!("auth:nonce:{}:{}", wallet, nonce);

    let is_new = cache
        .redis
        .setnx(&nonce_key, Duration::from_secs(MESSAGE_VALIDITY_SECS + 60))
        .await?;

    if !is_new {
        return Err(AppError::InvalidSignature);
    }

    Ok(())
}
