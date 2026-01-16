use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppState, services::nft::types::CreatorOutput};

use super::common::{CanvasActionParams, StateChangeResponse, SuccessResponse};

// Type aliases for NFT action params (all share CanvasActionParams structure)
pub type AnnounceMintParams = CanvasActionParams;
pub type CancelMintCountdownParams = CanvasActionParams;
pub type PrepareMetadataParams = CanvasActionParams;
pub type MintNftParams = CanvasActionParams;
pub type CancelMintParams = CanvasActionParams;

#[derive(Debug, Serialize)]
pub struct AnnounceMintResponse {
    pub success: bool,
    pub state: String,
    pub countdown_seconds: u8,
}

pub type CancelMintCountdownResponse = SuccessResponse;

#[derive(Debug, Serialize)]
pub struct PrepareMetadataResponse {
    pub success: bool,
    pub metadata_uri: String,
    pub image_uri: String,
    pub image_gateway_url: String,
    pub metadata_gateway_url: String,
    pub creators: Vec<CreatorOutput>,
}

pub type MintNftResponse = StateChangeResponse;

#[derive(Deserialize)]
pub struct ConfirmNftMintParams {
    pub access_token: String,
    pub canvas_id: Uuid,
    pub signature: String,
    pub mint_address: String,

    #[serde(skip)]
    pub state: Option<AppState>,
}

pub type ConfirmNftMintResponse = StateChangeResponse;
pub type CancelMintResponse = StateChangeResponse;
