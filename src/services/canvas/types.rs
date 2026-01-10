use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::infrastructure::db::entities::canvas::{self, CanvasState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasInfo {
    pub id: Uuid,
    pub name: String,
    pub invite_code: String,
    pub state: CanvasState,
    pub owner_id: Uuid,
    pub canvas_pda: Option<String>,
    pub mint_address: Option<String>,
    pub total_escrowed: i64,
}

impl From<canvas::Model> for CanvasInfo {
    fn from(value: canvas::Model) -> Self {
        CanvasInfo {
            id: value.id,
            name: value.name,
            invite_code: value.invite_code,
            state: value.state,
            owner_id: value.owner_id,
            canvas_pda: value.canvas_pda,
            mint_address: value.mint_address,
            total_escrowed: value.total_escrowed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasWithPixels {
    pub canvas: CanvasInfo,
    pub pixel_colors: String,
    pub owned_pixels: Vec<OwnedCanvasPixelInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OwnedCanvasPixelInfo {
    pub x: i16,
    pub y: i16,
    pub owner_id: String,
    pub price_lamports: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CachedPixelData {
    pub pixel_colors: String,
    pub owned_pixels: Vec<OwnedCanvasPixelInfo>,
}

pub struct UserCanvases {
    pub owned: Vec<CanvasInfo>,
    pub collaborating: Vec<CanvasInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishTransactionInfo {
    pub canvas_id: Uuid,
    pub canvas_pda: String,
    pub config_pda: String,
    pub program_id: String,
    pub blockhash: String,
    pub canvas_id_bytes: [u8; 16],

    /// Base64 encoded 768 bytes of 6-bit packed pixel colors from database.
    pub pixel_colors_packed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinCanvasResult {
    pub canvas_id: Uuid,
    pub already_member: bool,
}
