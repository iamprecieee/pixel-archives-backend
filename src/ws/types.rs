use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum RoomCanvasUpdate {
    Pixel(RoomPixelUpdate),
    PixelLocked { x: u8, y: u8, user_id: Uuid },
    PixelUnlocked { x: u8, y: u8 },
    PublishingStarted,
    Published { pda: String },
    PublishingFailed { reason: String },
    MintingStarted,
    Minted { mint_address: String },
    MintingFailed { reason: String },
    MintCountdown { seconds: u8 },
    MintCountdownCancelled,
    UserJoined { user_id: Uuid },
    UserLeft { user_id: Uuid },
    ConnectionCount { count: usize },
    Finalized,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RoomPixelUpdate {
    pub x: u8,
    pub y: u8,
    pub color: u8,
    pub owner_id: Option<Uuid>,
    pub price_lamports: Option<u64>,
}

#[derive(Deserialize)]
pub struct WsQuery {
    pub canvas_id: Uuid,
    pub token: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Ping,
    Subscribe { x: u8, y: u8 },
    Unsubscribe { x: u8, y: u8 },
}
