use std::time::Duration;

use crate::{
    api::types::{
        AnnounceMintParams, AnnounceMintResponse, CancelMintCountdownParams,
        CancelMintCountdownResponse, CancelMintParams, CancelMintResponse, ConfirmNftMintParams,
        ConfirmNftMintResponse, MintNftParams, MintNftResponse, PrepareMetadataParams,
        PrepareMetadataResponse,
    },
    error::{AppError, Result},
    infrastructure::{
        cache::keys::CacheKey,
        db::{entities::canvas::CanvasState, repositories::CanvasRepository},
    },
    services::{auth::TokenType, nft as nft_service},
    ws::types::RoomCanvasUpdate,
};

pub async fn announce_mint_countdown(params: AnnounceMintParams) -> Result<AnnounceMintResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    let canvas =
        CanvasRepository::find_canvas_by_id(app_state.db.get_connection(), params.canvas_id)
            .await?
            .ok_or(AppError::CanvasNotFound)?;

    if canvas.owner_id != user_id {
        return Err(AppError::NotCanvasOwner);
    }

    if canvas.state != CanvasState::Published {
        return Err(AppError::InvalidCanvasStateTransition);
    }

    let lock_key = CacheKey::canvas_lock(&params.canvas_id);
    let is_locked: Option<bool> = app_state.cache.redis.get(&lock_key).await?;

    if is_locked.is_some() {
        return Err(AppError::PixelLocked);
    }

    // Set lock for countdown duration + buffer
    app_state
        .cache
        .redis
        .set(&lock_key, &true, Duration::from_secs(60))
        .await?;

    CanvasRepository::update_canvas_state(
        &app_state.db,
        params.canvas_id,
        CanvasState::MintPending,
        |_active| {},
    )
    .await?;

    app_state
        .ws_rooms
        .broadcast(
            &params.canvas_id,
            RoomCanvasUpdate::MintCountdown {
                seconds: app_state.config.canvas.mint_countdown_secs,
            },
        )
        .await;

    Ok(AnnounceMintResponse {
        success: true,
        state: "mint_pending".to_string(),
        countdown_seconds: app_state.config.canvas.mint_countdown_secs,
    })
}

pub async fn cancel_mint_countdown(
    params: CancelMintCountdownParams,
) -> Result<CancelMintCountdownResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    let canvas =
        CanvasRepository::find_canvas_by_id(app_state.db.get_connection(), params.canvas_id)
            .await?
            .ok_or(AppError::CanvasNotFound)?;

    if canvas.owner_id != user_id {
        return Err(AppError::NotCanvasOwner);
    }

    if canvas.state != CanvasState::MintPending {
        return Err(AppError::InvalidParams(
            "Canvas not in mint pending state".into(),
        ));
    }

    // Clear lock
    let lock_key = CacheKey::canvas_lock(&canvas.id);
    app_state.cache.redis.delete(&lock_key).await?;

    // Revert to published state
    CanvasRepository::update_canvas_state(
        &app_state.db,
        canvas.id,
        CanvasState::Published,
        |_active| {},
    )
    .await?;

    // Broadcast cancellation
    app_state
        .ws_rooms
        .broadcast(&canvas.id, RoomCanvasUpdate::MintCountdownCancelled)
        .await;

    Ok(CancelMintCountdownResponse { success: true })
}

pub async fn prepare_metadata(params: PrepareMetadataParams) -> Result<PrepareMetadataResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    let canvas =
        CanvasRepository::find_canvas_by_id(app_state.db.get_connection(), params.canvas_id)
            .await?
            .ok_or(AppError::CanvasNotFound)?;

    if canvas.owner_id != user_id {
        return Err(AppError::NotCanvasOwner);
    }

    if canvas.state != CanvasState::Published && canvas.state != CanvasState::MintPending {
        return Err(AppError::InvalidParams(
            "Canvas must be published to prepare metadata".into(),
        ));
    }

    let result = nft_service::prepare_metadata(&app_state, params.canvas_id).await?;

    Ok(PrepareMetadataResponse {
        success: true,
        metadata_uri: result.metadata_uri,
        image_uri: result.image_uri,
        image_gateway_url: result.image_gateway_url,
        metadata_gateway_url: result.metadata_gateway_url,
        creators: result.creators,
    })
}

pub async fn mint(params: MintNftParams) -> Result<MintNftResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    let _ = nft_service::initiate_nft_mint(&app_state, params.canvas_id, user_id).await?;

    Ok(MintNftResponse {
        success: true,
        state: "minting".to_string(),
    })
}

pub async fn confirm_mint(params: ConfirmNftMintParams) -> Result<ConfirmNftMintResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    let _ = nft_service::confirm_nft_mint(
        &app_state,
        params.canvas_id,
        user_id,
        params.signature.as_str(),
        params.mint_address.as_str(),
    )
    .await?;

    Ok(ConfirmNftMintResponse {
        success: true,
        state: "minted".to_string(),
    })
}

pub async fn cancel_mint(params: CancelMintParams) -> Result<CancelMintResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    nft_service::cancel_mint(&app_state, params.canvas_id, user_id).await?;

    Ok(CancelMintResponse {
        success: true,
        state: "published".to_string(),
    })
}
