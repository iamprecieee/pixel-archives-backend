use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;

use super::common::{PixelCoords, SuccessResponse};

#[derive(Deserialize)]
pub struct PlacePixelBidParams {
    pub access_token: String,
    pub canvas_id: Uuid,
    #[serde(flatten)]
    pub coords: PixelCoords,
    pub bid_lamports: Option<i64>,

    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Debug, Serialize)]
pub struct PlacePixelBidResponse {
    pub success: bool,
    #[serde(flatten)]
    pub coords: PixelCoords,
    pub requires_confirmation: bool,
    pub previous_owner_wallet: Option<String>,
}

#[derive(Deserialize)]
pub struct ConfirmPixelBidParams {
    pub access_token: String,
    pub canvas_id: Uuid,
    #[serde(flatten)]
    pub coords: PixelCoords,
    pub bid_lamports: Option<i64>,
    pub signature: String,

    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Debug, Serialize)]
pub struct ConfirmPixelBidResponse {
    pub success: bool,
    #[serde(flatten)]
    pub coords: PixelCoords,
    pub owner_id: Option<String>,
    pub price_lamports: i64,
}

#[derive(Deserialize)]
pub struct CancelPixelBidParams {
    pub access_token: String,
    pub canvas_id: Uuid,
    pub x: i16,
    pub y: i16,

    #[serde(skip)]
    pub state: Option<AppState>,
}

pub type CancelPixelBidResponse = SuccessResponse;

#[derive(Deserialize)]
pub struct PaintPixelParams {
    pub access_token: String,
    pub canvas_id: Uuid,
    #[serde(flatten)]
    pub coords: PixelCoords,
    pub signature: String,

    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Debug, Serialize)]
pub struct PaintPixelResponse {
    pub success: bool,
    #[serde(flatten)]
    pub coords: PixelCoords,
}
