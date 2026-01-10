use uuid::Uuid;

use crate::{
    api::types::{
        CancelPixelBidParams, CancelPixelBidResponse, ConfirmPixelBidParams,
        ConfirmPixelBidResponse, PaintPixelParams, PaintPixelResponse, PlacePixelBidParams,
        PlacePixelBidResponse,
    },
    error::{AppError, Result},
    services::{
        auth::TokenType,
        pixel::{self as pixel_service, types::ConfirmPixelRequest},
    },
};

pub async fn place_pixel_bid(params: PlacePixelBidParams) -> Result<PlacePixelBidResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    let canvas_id: Uuid = params
        .canvas_id
        .parse()
        .map_err(|_| AppError::InvalidParams("Invalid canvas_id".into()))?;

    let result = pixel_service::place_pixel(
        &app_state,
        canvas_id,
        user_id,
        params.x,
        params.y,
        params.color,
        params.bid_lamports,
    )
    .await?;

    Ok(PlacePixelBidResponse {
        success: true,
        x: result.x,
        y: result.y,
        color: result.color,
        requires_confirmation: result.requires_confirmation,
        previous_owner_wallet: result.previous_owner_wallet,
    })
}

pub async fn confirm_pixel_bid(params: ConfirmPixelBidParams) -> Result<ConfirmPixelBidResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    let canvas_id: Uuid = params
        .canvas_id
        .parse()
        .map_err(|_| AppError::InvalidParams("Invalid canvas_id".into()))?;

    let pixel_info = pixel_service::confirm_pixel_bid(
        &app_state,
        ConfirmPixelRequest {
            canvas_id,
            user_id,
            x: params.x,
            y: params.y,
            color: params.color,
            bid_lamports: params.bid_lamports.unwrap_or(0),
            signature: params.signature,
        },
    )
    .await?;

    Ok(ConfirmPixelBidResponse {
        success: true,
        x: pixel_info.x,
        y: pixel_info.y,
        color: pixel_info.color,
        owner_id: pixel_info.owner_id.map(|id| id.to_string()),
        price_lamports: pixel_info.price_lamports,
    })
}

pub async fn cancel_pixel_bid(params: CancelPixelBidParams) -> Result<CancelPixelBidResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    let canvas_id: Uuid = params
        .canvas_id
        .parse()
        .map_err(|_| AppError::InvalidParams("Invalid canvas_id".into()))?;

    pixel_service::cancel_pixel_bid(&app_state, canvas_id, user_id, params.x, params.y).await?;

    Ok(CancelPixelBidResponse { success: true })
}

pub async fn paint_pixel(params: PaintPixelParams) -> Result<PaintPixelResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    let canvas_id: Uuid = params
        .canvas_id
        .parse()
        .map_err(|_| AppError::InvalidParams("Invalid canvas_id".into()))?;

    let updated_pixel = pixel_service::paint_pixel(
        &app_state,
        canvas_id,
        user_id,
        params.x,
        params.y,
        params.color,
        &params.signature,
    )
    .await?;

    Ok(PaintPixelResponse {
        success: true,
        x: updated_pixel.x,
        y: updated_pixel.y,
        color: updated_pixel.color,
    })
}
