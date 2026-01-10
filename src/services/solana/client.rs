use std::{
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use solana_client::{client_error::ClientError, nonblocking::rpc_client::RpcClient};
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{hash::Hash, pubkey::Pubkey};
use tokio::sync::RwLock;

use crate::config::SolanaConfig;

struct CachedBlockhash {
    hash: Hash,
    fetched_at: Instant,
}

pub struct SolanaClient {
    client: RpcClient,
    program_id: Pubkey,
    program_id_str: String,
    blockhash_cache: Arc<RwLock<Option<CachedBlockhash>>>,
    blockhash_ttl: Duration,
}

impl SolanaClient {
    pub fn initialize(config: &SolanaConfig) -> Self {
        let commitment = match config.commitment.as_str() {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => CommitmentConfig::confirmed(),
        };

        let client = RpcClient::new_with_commitment(config.rpc_url.clone(), commitment);
        let program_id =
            Pubkey::from_str(&config.program_id).expect("Invalid program ID in config");

        Self {
            client,
            program_id,
            program_id_str: config.program_id.clone(),
            blockhash_cache: Arc::new(RwLock::new(None)),
            blockhash_ttl: Duration::from_secs(config.blockhash_ttl),
        }
    }

    pub fn get_program_id(&self) -> &str {
        &self.program_id_str
    }

    pub fn get_client(&self) -> &RpcClient {
        &self.client
    }

    pub fn derive_canvas_pda(&self, canvas_id: &[u8; 16]) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"canvas", canvas_id], &self.program_id)
    }

    pub fn derive_config_pda(&self) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"config"], &self.program_id)
    }

    pub async fn get_recent_blockhash(&self) -> Result<Hash, ClientError> {
        {
            let cache = self.blockhash_cache.read().await;
            if let Some(ref cached) = *cache
                && cached.fetched_at.elapsed() < self.blockhash_ttl
            {
                return Ok(cached.hash);
            }
        }

        let hash = self.client.get_latest_blockhash().await?;

        {
            let mut cache = self.blockhash_cache.write().await;
            *cache = Some(CachedBlockhash {
                hash,
                fetched_at: Instant::now(),
            });
        }

        Ok(hash)
    }
}
