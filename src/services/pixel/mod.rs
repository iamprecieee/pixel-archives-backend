use std::time::Duration;
use uuid::Uuid;

use crate::{
    AppState,
    error::{AppError, Result},
    infrastructure::{
        cache::keys::CacheKey,
        db::{
            entities::canvas::{self, CanvasState},
            repositories::{CanvasRepository, PixelRepository, UserRepository},
        },
    },
    services::{
        pixel::{cooldown::*, lock::*, types::*, validation::*},
        solana,
    },
    ws::types::{RoomCanvasUpdate, RoomPixelUpdate},
};

pub mod cooldown;
pub mod lock;
pub mod types;
pub mod validation;

/// Fetches canvas from local cache or database.
async fn get_cached_canvas(state: &AppState, canvas_id: Uuid) -> Result<canvas::Model> {
    if let Some(cached) = state.cache.local.get_canvas(&canvas_id).await {
        return Ok((*cached).clone());
    }
    let canvas = CanvasRepository::find_canvas_by_id(state.db.get_connection(), canvas_id)
        .await?
        .ok_or(AppError::CanvasNotFound)?;
    state.cache.local.set_canvas(canvas.clone()).await;
    Ok(canvas)
}

async fn invalidate_pixel_caches(
    state: &AppState,
    canvas_id: &Uuid,
    x: i16,
    y: i16,
    color: i16,
    owner_id: Option<Uuid>,
    price: i64,
) {
    let cache_key = CacheKey::canvas_pixels(canvas_id);
    let _ = tokio::join!(
        state
            .cache
            .local
            .update_pixel(canvas_id, x, y, color, owner_id, price),
        state.cache.redis.delete(&cache_key),
    );
}

async fn broadcast_pixel_update(
    state: &AppState,
    canvas_id: &Uuid,
    x: i16,
    y: i16,
    color: i16,
    owner_id: Option<Uuid>,
    price: Option<u64>,
) {
    let update = RoomPixelUpdate {
        x: x as u8,
        y: y as u8,
        color: color as u8,
        owner_id,
        price_lamports: price,
    };
    state
        .ws_rooms
        .broadcast(canvas_id, RoomCanvasUpdate::Pixel(update))
        .await;
}

pub async fn place_pixel(
    state: &AppState,
    canvas_id: Uuid,
    user_id: Uuid,
    x: i16,
    y: i16,
    color: i16,
    bid_lamports: Option<i64>,
) -> Result<PlacePixelResult> {
    if !CanvasRepository::is_canvas_collaborator(state.db.get_connection(), canvas_id, user_id)
        .await?
    {
        return Err(AppError::NotCollaborator);
    }

    validate_pixel_coordinates(&state.config.canvas, x, y)?;
    validate_pixel_color(&state.config.canvas, color)?;

    let canvas = get_cached_canvas(state, canvas_id).await?;

    match canvas.state {
        CanvasState::Draft => place_pixel_draft(state, canvas_id, user_id, x, y, color).await,
        CanvasState::Published => {
            let bid = bid_lamports.unwrap_or(0);
            place_pixel_bid(state, canvas_id, user_id, x, y, color, bid).await
        }
        CanvasState::MintPending => Err(AppError::InvalidParams(
            "Canvas is preparing to mint. Pixel operations are temporarily blocked.".into(),
        )),
        _ => Err(AppError::InvalidParams(
            "Canvas not in a state that allows pixel placement".into(),
        )),
    }
}

async fn place_pixel_draft(
    state: &AppState,
    canvas_id: Uuid,
    user_id: Uuid,
    x: i16,
    y: i16,
    color: i16,
) -> Result<PlacePixelResult> {
    assert_not_locked_by_other(&state.cache.redis, &canvas_id, x as u8, y as u8, &user_id).await?;

    check_cooldown_state(
        &state.cache.redis,
        &user_id,
        state.config.canvas.cooldown_ms,
    )
    .await?;

    let pixel =
        PixelRepository::upsert_pixel(&state.db, canvas_id, x, y, Some(color), None, None).await?;

    let _ = tokio::join!(
        async { invalidate_pixel_caches(state, &canvas_id, x, y, color, None, 0).await },
        record_cooldown_state(
            &state.cache.redis,
            &user_id,
            state.config.canvas.cooldown_ms
        ),
    );

    broadcast_pixel_update(state, &canvas_id, x, y, color, None, None).await;

    Ok(PlacePixelResult {
        x: pixel.x,
        y: pixel.y,
        color: pixel.color,
        requires_confirmation: false,
        lock_expires_at: None,
        previous_owner_wallet: None,
    })
}

async fn place_pixel_bid(
    state: &AppState,
    canvas_id: Uuid,
    user_id: Uuid,
    x: i16,
    y: i16,
    color: i16,
    bid_lamports: i64,
) -> Result<PlacePixelResult> {
    validate_min_bid(&state.config.canvas, bid_lamports)?;

    let current_pixel =
        PixelRepository::find_pixel(state.db.get_connection(), canvas_id, x, y).await?;

    if let Some(ref pixel) = current_pixel {
        validate_outbid(pixel.price_lamports, bid_lamports)?;
    }

    let previous_owner_wallet =
        if let Some(owner_id) = current_pixel.as_ref().and_then(|p| p.owner_id) {
            UserRepository::find_user_by_id(state.db.get_connection(), owner_id)
                .await?
                .map(|user| user.wallet_address)
        } else {
            None
        };

    let lock_ttl = Duration::from_millis(state.config.canvas.lock_ms);
    let is_acquired = acquire_pixel_lock(
        &state.cache.redis,
        &canvas_id,
        x as u8,
        y as u8,
        &user_id,
        lock_ttl,
    )
    .await?;
    if !is_acquired {
        return Err(AppError::PixelLocked);
    }

    state
        .ws_rooms
        .broadcast(
            &canvas_id,
            RoomCanvasUpdate::PixelLocked {
                x: x as u8,
                y: y as u8,
                user_id,
            },
        )
        .await;

    let lock_expires_at = get_current_time_ms() + state.config.canvas.lock_ms;

    Ok(PlacePixelResult {
        x,
        y,
        color,
        requires_confirmation: true,
        lock_expires_at: Some(lock_expires_at),
        previous_owner_wallet,
    })
}

pub async fn confirm_pixel_bid(state: &AppState, req: ConfirmPixelRequest) -> Result<PixelInfo> {
    validate_min_bid(&state.config.canvas, req.bid_lamports)?;
    assert_lock_owned(
        &state.cache.redis,
        &req.canvas_id,
        req.x as u8,
        req.y as u8,
        &req.user_id,
    )
    .await?;

    if let Some(current) =
        PixelRepository::find_pixel(state.db.get_connection(), req.canvas_id, req.x, req.y).await?
    {
        validate_outbid(current.price_lamports, req.bid_lamports)?;
    }

    let is_valid = solana::verify_program_transaction(
        state.solana_client.get_client(),
        &req.signature,
        state.solana_client.get_program_id(),
    )
    .await?;

    if !is_valid {
        return Err(AppError::TransactionFailed(
            "Transaction verification failed".into(),
        ));
    }

    let pixel = PixelRepository::upsert_pixel(
        &state.db,
        req.canvas_id,
        req.x,
        req.y,
        Some(req.color),
        Some(req.user_id),
        Some(req.bid_lamports),
    )
    .await?;

    let _ = tokio::join!(
        async {
            invalidate_pixel_caches(
                state,
                &req.canvas_id,
                req.x,
                req.y,
                req.color,
                Some(req.user_id),
                req.bid_lamports,
            )
            .await
        },
        release_pixel_lock(&state.cache.redis, &req.canvas_id, req.x as u8, req.y as u8),
    );

    broadcast_pixel_update(
        state,
        &req.canvas_id,
        req.x,
        req.y,
        req.color,
        Some(req.user_id),
        Some(pixel.price_lamports as u64),
    )
    .await;

    state
        .ws_rooms
        .broadcast(
            &req.canvas_id,
            RoomCanvasUpdate::PixelUnlocked {
                x: req.x as u8,
                y: req.y as u8,
            },
        )
        .await;

    Ok(PixelInfo {
        x: pixel.x,
        y: pixel.y,
        color: pixel.color,
        owner_id: pixel.owner_id,
        price_lamports: pixel.price_lamports,
    })
}

pub async fn cancel_pixel_bid(
    state: &AppState,
    canvas_id: Uuid,
    user_id: Uuid,
    x: i16,
    y: i16,
) -> Result<()> {
    assert_lock_owned(&state.cache.redis, &canvas_id, x as u8, y as u8, &user_id).await?;
    release_pixel_lock(&state.cache.redis, &canvas_id, x as u8, y as u8).await?;

    state
        .ws_rooms
        .broadcast(
            &canvas_id,
            RoomCanvasUpdate::PixelUnlocked {
                x: x as u8,
                y: y as u8,
            },
        )
        .await;

    Ok(())
}

pub async fn paint_pixel(
    state: &AppState,
    canvas_id: Uuid,
    user_id: Uuid,
    x: i16,
    y: i16,
    color: i16,
    signature: &str,
) -> Result<PixelInfo> {
    let pixel = PixelRepository::find_pixel(state.db.get_connection(), canvas_id, x, y)
        .await?
        .ok_or(AppError::InvalidParams("Pixel not found".into()))?;

    if pixel.owner_id != Some(user_id) {
        return Err(AppError::Unauthorized);
    }

    let is_valid = solana::verify_program_transaction(
        state.solana_client.get_client(),
        signature,
        state.solana_client.get_program_id(),
    )
    .await?;

    if !is_valid {
        return Err(AppError::TransactionFailed(
            "Transaction verification failed".into(),
        ));
    }

    let updated =
        PixelRepository::upsert_pixel(&state.db, canvas_id, x, y, Some(color), None, None).await?;

    invalidate_pixel_caches(
        state,
        &canvas_id,
        x,
        y,
        color,
        Some(user_id),
        updated.price_lamports,
    )
    .await;
    broadcast_pixel_update(
        state,
        &canvas_id,
        x,
        y,
        color,
        updated.owner_id,
        Some(updated.price_lamports as u64),
    )
    .await;

    Ok(PixelInfo {
        x: updated.x,
        y: updated.y,
        color: updated.color,
        owner_id: updated.owner_id,
        price_lamports: updated.price_lamports,
    })
}
