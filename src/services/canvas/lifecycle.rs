use base64::Engine;
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use std::time::Duration;
use uuid::Uuid;

use crate::{
    AppState,
    error::{AppError, Result},
    infrastructure::{
        cache::keys::CacheKey,
        db::{
            entities::canvas::CanvasState,
            repositories::{CanvasRepository, PixelRepository},
        },
    },
    services::{
        canvas::{
            pack_pixels_to_colors,
            types::{CanvasInfo, PublishTransactionInfo},
        },
        solana,
    },
    ws::types::RoomCanvasUpdate,
};

pub async fn create_canvas(
    state: &AppState,
    owner_id: Uuid,
    name: &str,
    initial_color: i16,
) -> Result<CanvasInfo> {
    let max_name_length = state.config.canvas.max_name_length;
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() || trimmed_name.len() > max_name_length as usize {
        return Err(AppError::InvalidParams(format!(
            "Canvas name cannot be empty or exceed {} characters",
            max_name_length
        )));
    }

    let db_transaction = state.db.begin_transaction().await?;

    if CanvasRepository::exists_by_name_and_owner(&db_transaction, owner_id, name).await? {
        db_transaction.rollback().await?;
        return Err(AppError::CanvasNameExists);
    }

    let canvas = CanvasRepository::create_canvas(&db_transaction, owner_id, name).await?;

    CanvasRepository::add_canvas_collaborator(&db_transaction, canvas.id, owner_id).await?;

    PixelRepository::initialize_canvas_pixels(
        &db_transaction,
        canvas.id,
        state.config.canvas.width,
        state.config.canvas.height,
        initial_color,
    )
    .await?;

    db_transaction.commit().await?;

    Ok(CanvasInfo::from(canvas))
}

pub async fn initialize_canvas_publish(
    state: &AppState,
    canvas_id: Uuid,
    user_id: Uuid,
) -> Result<PublishTransactionInfo> {
    state.cache.local.invalidate_canvas(&canvas_id).await;

    let canvas = CanvasRepository::find_canvas_by_id(state.db.get_connection(), canvas_id)
        .await?
        .ok_or(AppError::CanvasNotFound)?;

    if canvas.owner_id != user_id {
        return Err(AppError::NotCanvasOwner);
    }

    let lock_key = CacheKey::canvas_lock(&canvas_id);
    let acquired = state
        .cache
        .redis
        .setnx(
            &lock_key,
            Duration::from_secs(state.config.cache.redis_cache_short_ttl),
        )
        .await?;

    if !acquired {
        return Err(AppError::PixelLocked);
    }

    let pixels =
        PixelRepository::find_pixels_by_canvas(state.db.get_connection(), canvas_id).await?;

    let pixel_colors_packed = pack_pixels_to_colors(
        &pixels,
        state.config.canvas.width,
        state.config.canvas.height,
    );
    let pixel_colors_base64 = base64::engine::general_purpose::STANDARD.encode(pixel_colors_packed);

    CanvasRepository::update_canvas_state(
        &state.db,
        canvas_id,
        CanvasState::Publishing,
        |_active| {},
    )
    .await?;

    let canvas_id_bytes = canvas_id.as_bytes();
    let (canvas_pda, _bump) = state.solana_client.derive_canvas_pda(canvas_id_bytes);
    let (config_pda, _) = state.solana_client.derive_config_pda();

    let blockhash = state
        .solana_client
        .get_recent_blockhash()
        .await
        .map_err(|e| AppError::SolanaRpc(e.to_string()))?;

    Ok(PublishTransactionInfo {
        canvas_id,
        canvas_pda: canvas_pda.to_string(),
        config_pda: config_pda.to_string(),
        program_id: state.solana_client.get_program_id().to_string(),
        blockhash: blockhash.to_string(),
        canvas_id_bytes: *canvas_id_bytes,
        pixel_colors_packed: pixel_colors_base64,
    })
}

pub async fn confirm_canvas_publish(
    state: &AppState,
    canvas_id: Uuid,
    user_id: Uuid,
    signature: &str,
    canvas_pda: &str,
) -> Result<CanvasInfo> {
    let canvas = if let Some(cached) = state.cache.local.get_canvas(&canvas_id).await {
        (*cached).clone()
    } else {
        let canvas = CanvasRepository::find_canvas_by_id(state.db.get_connection(), canvas_id)
            .await?
            .ok_or(AppError::CanvasNotFound)?;
        state.cache.local.set_canvas(canvas.clone()).await;
        canvas
    };

    if canvas.owner_id != user_id {
        return Err(AppError::NotCanvasOwner);
    }

    let tx_valid = solana::verify_program_transaction(
        state.solana_client.get_client(),
        signature,
        state.solana_client.get_program_id(),
    )
    .await?;

    if !tx_valid {
        return Err(AppError::TransactionFailed(
            "Transaction verification failed".into(),
        ));
    }

    let canvas = CanvasRepository::update_canvas_state(
        &state.db,
        canvas_id,
        CanvasState::Published,
        |active| {
            active.published_at = Set(Some(Utc::now()));
            active.canvas_pda = Set(Some(canvas_pda.to_string()));
        },
    )
    .await?;

    let lock_key = CacheKey::canvas_lock(&canvas_id);

    let _ = tokio::join!(
        state.cache.local.invalidate_canvas(&canvas_id),
        state.cache.redis.delete(&lock_key),
    );

    state
        .ws_rooms
        .broadcast(
            &canvas_id,
            RoomCanvasUpdate::Published {
                pda: canvas_pda.to_string(),
            },
        )
        .await;

    Ok(CanvasInfo::from(canvas))
}

pub async fn cancel_canvas_publish(state: &AppState, canvas_id: Uuid, user_id: Uuid) -> Result<()> {
    let canvas = if let Some(cached) = state.cache.local.get_canvas(&canvas_id).await {
        (*cached).clone()
    } else {
        let canvas = CanvasRepository::find_canvas_by_id(state.db.get_connection(), canvas_id)
            .await?
            .ok_or(AppError::CanvasNotFound)?;
        state.cache.local.set_canvas(canvas.clone()).await;
        canvas
    };

    if canvas.owner_id != user_id {
        return Err(AppError::NotCanvasOwner);
    }

    CanvasRepository::update_canvas_state(&state.db, canvas_id, CanvasState::Draft, |_active| {})
        .await?;

    let lock_key = CacheKey::canvas_lock(&canvas_id);
    let _ = tokio::join!(
        state.cache.local.invalidate_canvas(&canvas_id),
        state.cache.redis.delete(&lock_key),
    );

    Ok(())
}

pub async fn delete_canvas(state: &AppState, canvas_id: Uuid, user_id: Uuid) -> Result<()> {
    let canvas = if let Some(cached) = state.cache.local.get_canvas(&canvas_id).await {
        (*cached).clone()
    } else {
        CanvasRepository::find_canvas_by_id(state.db.get_connection(), canvas_id)
            .await?
            .ok_or(AppError::CanvasNotFound)?
    };

    if canvas.owner_id != user_id {
        return Err(AppError::NotCanvasOwner);
    }

    if canvas.state != CanvasState::Draft {
        return Err(AppError::InvalidParams(
            "Only Draft canvases can be deleted".into(),
        ));
    }

    CanvasRepository::delete_canvas(&state.db, canvas_id).await?;

    let redis_pixel_key = CacheKey::canvas_pixels(&canvas_id);
    let redis_lock_key = CacheKey::canvas_lock(&canvas_id);

    let _ = tokio::join!(
        state.cache.local.invalidate_canvas(&canvas_id),
        state.cache.local.invalidate_pixels(&canvas_id),
        state.cache.redis.delete(&redis_pixel_key),
        state.cache.redis.delete(&redis_lock_key)
    );

    Ok(())
}
