use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacePixelResult {
    pub x: i16,
    pub y: i16,
    pub color: i16,
    pub requires_confirmation: bool,
    pub lock_expires_at: Option<u64>,
    pub previous_owner_wallet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelInfo {
    pub x: i16,
    pub y: i16,
    pub color: i16,
    pub owner_id: Option<Uuid>,
    pub price_lamports: i64,
}

pub struct ConfirmPixelRequest {
    pub canvas_id: Uuid,
    pub user_id: Uuid,
    pub x: i16,
    pub y: i16,
    pub color: i16,
    pub bid_lamports: i64,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidPixelInfo {
    pub program_id: String,
    pub config_pda: String,
    pub canvas_pda: String,
    pub pixel_pda: String,
    pub pixel_bump: u8,
    pub x: i16,
    pub y: i16,
    pub current_price: i64,
    pub min_bid: i64,
    pub blockhash: String,
}
