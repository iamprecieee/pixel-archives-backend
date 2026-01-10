use std::{str::FromStr, time::Duration};

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_transaction_status::UiTransactionEncoding;

use crate::error::{AppError, Result};

pub async fn verify_program_transaction(
    client: &RpcClient,
    signature: &str,
    program_id: &str,
) -> Result<bool> {
    let transaction_signature = Signature::from_str(signature)
        .map_err(|_| AppError::InvalidParams("Invalid transaction signature".into()))?;

    let program_pubkey = Pubkey::from_str(program_id)
        .map_err(|_| AppError::InvalidParams("Invalid program ID".into()))?;

    let mut last_status_err = None;

    // Retry for up to 30 seconds
    for _ in 0..30 {
        match client
            .get_signature_statuses(&[transaction_signature])
            .await
        {
            Ok(response) => {
                if let Some(Some(status)) = response.value.first() {
                    if status.err.is_some() {
                        return Ok(false);
                    }

                    if matches!(
                        status.confirmation_status,
                        Some(solana_transaction_status::TransactionConfirmationStatus::Processed)
                            | Some(
                                solana_transaction_status::TransactionConfirmationStatus::Confirmed
                            )
                            | Some(
                                solana_transaction_status::TransactionConfirmationStatus::Finalized
                            )
                    ) {
                        break;
                    }
                }
            }
            Err(e) => last_status_err = Some(e.to_string()),
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    if last_status_err.is_some() {
        return Err(AppError::SolanaRpc(format!(
            "Transaction not confirmed after 30s. Last error: {:?}",
            last_status_err
        )));
    }

    // If not confirmed after 30 retries
    let transaction_response = client
        .get_transaction_with_config(
            &transaction_signature,
            solana_client::rpc_config::RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Json),
                commitment: Some(solana_commitment_config::CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        )
        .await
        .map_err(|e| AppError::SolanaRpc(format!("Failed to fetch transaction: {}", e)))?;

    let message = match transaction_response.transaction.transaction {
        solana_transaction_status::EncodedTransaction::Json(ui_transaction) => {
            ui_transaction.message
        }
        _ => {
            return Err(AppError::SolanaRpc(
                "Unexpected transaction encoding".into(),
            ));
        }
    };

    let account_keys: Vec<Pubkey> = match message {
        solana_transaction_status::UiMessage::Parsed(parsed) => parsed
            .account_keys
            .iter()
            .filter_map(|key| Pubkey::from_str(&key.pubkey).ok())
            .collect(),
        solana_transaction_status::UiMessage::Raw(raw) => raw
            .account_keys
            .iter()
            .filter_map(|key| Pubkey::from_str(key).ok())
            .collect(),
    };

    if !account_keys.contains(&program_pubkey) {
        return Err(AppError::InvalidParams(
            "Transaction does not involve our program".into(),
        ));
    }

    Ok(true)
}
