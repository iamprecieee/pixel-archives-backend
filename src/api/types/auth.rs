use serde::{Deserialize, Serialize};

use crate::AppState;

use super::common::SuccessResponse;

pub enum AuthOperation {
    Login,
    Register,
}

#[derive(Deserialize)]
pub struct AuthParams {
    pub wallet: String,
    pub message: String,
    pub signature: String,
    pub username: Option<String>,

    #[serde(skip)]
    pub state: Option<AppState>,

    #[serde(skip)]
    pub operation: Option<AuthOperation>,
}

#[derive(Deserialize)]
pub struct SessionParams {
    pub access_token: String,
    pub refresh_token: Option<String>,

    #[serde(skip)]
    pub state: Option<AppState>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponse {
    pub id: String,
    pub wallet_address: String,
    pub username: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
}

pub type LogoutResponse = SuccessResponse;
