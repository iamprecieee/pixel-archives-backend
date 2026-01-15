use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::infrastructure::db::entities::canvas::CanvasState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatorOutput {
    pub address: String,
    pub share: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataResult {
    pub metadata_uri: String,
    pub image_uri: String,
    pub image_gateway_url: String,
    pub metadata_gateway_url: String,
    pub creators: Vec<CreatorOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintTransactionInfo {
    pub canvas_id: Uuid,
    pub canvas_pda: String,
    pub config_pda: String,
    pub program_id: String,
    pub blockhash: String,
    pub canvas_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintResult {
    pub canvas_id: Uuid,
    pub mint_address: Option<String>,
    pub state: CanvasState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftMetadata {
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub image: String,
    pub seller_fee_basis_points: u16,
    pub attributes: Vec<Attribute>,
    pub properties: Properties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub trait_type: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Properties {
    pub files: Vec<ImageFile>,
    pub category: String,
    pub creators: Vec<CreatorOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageFile {
    pub uri: String,

    #[serde(rename = "type")]
    pub file_type: String,
}
