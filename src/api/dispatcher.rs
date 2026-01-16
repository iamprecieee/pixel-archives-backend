use serde_json::Value;

use crate::{
    AppState,
    api::{
        methods::{
            auth::{authenticate_user, logout_user, refresh_user_token},
            canvas::{
                cancel_publish_canvas, confirm_publish_canvas, create_canvas, delete_canvas,
                get_canvas, join_canvas, list_canvas, publish_canvas,
            },
            nft::{
                announce_mint_countdown, cancel_mint, cancel_mint_countdown, confirm_mint, mint,
                prepare_metadata,
            },
            pixel::{cancel_pixel_bid, confirm_pixel_bid, paint_pixel, place_pixel_bid},
        },
        types::{
            AnnounceMintParams, AuthOperation, AuthParams, CancelMintCountdownParams,
            CancelMintParams, CancelPixelBidParams, CancelPublishCanvasParams,
            ConfirmNftMintParams, ConfirmPixelBidParams, ConfirmPublishCanvasParams,
            CreateCanvasParams, DeleteCanvasParams, GetCanvasParams, JoinCanvasParams,
            ListCanvasParams, MintNftParams, PaintPixelParams, PlacePixelBidParams,
            PrepareMetadataParams, PublishCanvasParams, SessionParams,
        },
    },
    error::AppError,
};

macro_rules! dispatch {
    ($param_type:ty, $handler:path, $params:expr, $state:expr) => {{
        let mut p: $param_type =
            serde_json::from_value($params).map_err(|e| AppError::InvalidParams(e.to_string()))?;

        p.state = Some($state);

        let result = $handler(p).await?;
        serde_json::to_value(result).map_err(AppError::from)
    }};

    ($param_ty:ty, $handler:path, $params:expr, $state:expr, $op_field:ident = $op_value:expr) => {{
        let mut p: $param_ty =
            serde_json::from_value($params).map_err(|e| AppError::InvalidParams(e.to_string()))?;
        p.state = Some($state);
        p.$op_field = Some($op_value);

        let result = $handler(p).await?;
        serde_json::to_value(result).map_err(AppError::from)
    }};
}

async fn dispatch_auth(method: &str, params: Value, state: AppState) -> Result<Value, AppError> {
    match method {
        "auth.register" => dispatch!(
            AuthParams,
            authenticate_user,
            params,
            state,
            operation = AuthOperation::Register
        ),
        "auth.login" => dispatch!(
            AuthParams,
            authenticate_user,
            params,
            state,
            operation = AuthOperation::Login
        ),
        "auth.logout" => dispatch!(SessionParams, logout_user, params, state),
        "auth.refresh" => dispatch!(SessionParams, refresh_user_token, params, state),
        _ => Err(AppError::MethodNotFound(method.to_string())),
    }
}

async fn dispatch_canvas(method: &str, params: Value, state: AppState) -> Result<Value, AppError> {
    match method {
        "canvas.create" => dispatch!(CreateCanvasParams, create_canvas, params, state),
        "canvas.list" => dispatch!(ListCanvasParams, list_canvas, params, state),
        "canvas.get" => dispatch!(GetCanvasParams, get_canvas, params, state),
        "canvas.join" => dispatch!(JoinCanvasParams, join_canvas, params, state),
        "canvas.publish" => dispatch!(PublishCanvasParams, publish_canvas, params, state),
        "canvas.confirmPublish" => {
            dispatch!(
                ConfirmPublishCanvasParams,
                confirm_publish_canvas,
                params,
                state
            )
        }
        "canvas.cancelPublish" => {
            dispatch!(
                CancelPublishCanvasParams,
                cancel_publish_canvas,
                params,
                state
            )
        }
        "canvas.delete" => dispatch!(DeleteCanvasParams, delete_canvas, params, state),
        _ => Err(AppError::MethodNotFound(method.to_string())),
    }
}

async fn dispatch_pixel(method: &str, params: Value, state: AppState) -> Result<Value, AppError> {
    match method {
        "pixel.place" => dispatch!(PlacePixelBidParams, place_pixel_bid, params, state),
        "pixel.confirm" => dispatch!(ConfirmPixelBidParams, confirm_pixel_bid, params, state),
        "pixel.paint" => dispatch!(PaintPixelParams, paint_pixel, params, state),
        "pixel.cancel" => dispatch!(CancelPixelBidParams, cancel_pixel_bid, params, state),
        _ => Err(AppError::MethodNotFound(method.to_string())),
    }
}

async fn dispatch_nft(method: &str, params: Value, state: AppState) -> Result<Value, AppError> {
    match method {
        "nft.mint" => dispatch!(MintNftParams, mint, params, state),
        "nft.cancelMint" => dispatch!(CancelMintParams, cancel_mint, params, state),
        "nft.announceMint" => dispatch!(AnnounceMintParams, announce_mint_countdown, params, state),
        "nft.cancelMintCountdown" => {
            dispatch!(
                CancelMintCountdownParams,
                cancel_mint_countdown,
                params,
                state
            )
        }
        "nft.confirmMint" => dispatch!(ConfirmNftMintParams, confirm_mint, params, state),
        "nft.prepareMetadata" => {
            dispatch!(PrepareMetadataParams, prepare_metadata, params, state)
        }
        _ => Err(AppError::MethodNotFound(method.to_string())),
    }
}

pub async fn dispatch_method(
    method: &str,
    params: Value,
    state: AppState,
    client_key: &str,
) -> Result<Value, AppError> {
    let limiter = match method {
        "auth.login" | "auth.register" | "auth.refresh" => Some(&state.rate_limiters.auth),
        "pixel.place" | "pixel.paint" => Some(&state.rate_limiters.pixel),
        "canvas.create" | "canvas.join" | "canvas.delete" => Some(&state.rate_limiters.canvas),
        "canvas.publish"
        | "canvas.confirmPublish"
        | "nft.announceMint"
        | "nft.mint"
        | "nft.confirmMint"
        | "nft.prepareMetadata" => Some(&state.rate_limiters.solana),
        _ => None,
    };

    if let Some(limiter) = limiter {
        let (allowed, _, _) = limiter.check(client_key).await?;
        if !allowed {
            return Err(AppError::RateLimitExceeded);
        }
    }

    if method.starts_with("auth.") {
        return dispatch_auth(method, params, state).await;
    }
    if method.starts_with("canvas.") {
        return dispatch_canvas(method, params, state).await;
    }
    if method.starts_with("pixel.") {
        return dispatch_pixel(method, params, state).await;
    }
    if method.starts_with("nft.") {
        return dispatch_nft(method, params, state).await;
    }
    Err(AppError::MethodNotFound(method.to_string()))
}
