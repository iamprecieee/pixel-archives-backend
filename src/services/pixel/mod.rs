use std::time::{Duration, SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use crate::{
    AppState,
    error::{AppError, Result},
    infrastructure::{
        cache::keys::CacheKey,
        db::{
            entities::canvas::CanvasState,
            repositories::{CanvasRepository, PixelRepository, UserRepository},
        },
    },
    services::{
        pixel::types::{ConfirmPixelRequest, PixelInfo, PlacePixelResult},
        solana,
    },
};

pub mod types;

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

    if x < 0
        || x >= state.config.canvas.width as i16
        || y < 0
        || y >= state.config.canvas.height as i16
    {
        return Err(AppError::InvalidParams("Coordinates out of bounds".into()));
    }

    if color < 0 || color >= state.config.canvas.color_count as i16 {
        return Err(AppError::InvalidParams("Invalid color".into()));
    }

    let canvas = if let Some(cached) = state.cache.local.get_canvas(&canvas_id).await {
        (*cached).clone()
    } else {
        let canvas = CanvasRepository::find_canvas_by_id(state.db.get_connection(), canvas_id)
            .await?
            .ok_or(AppError::CanvasNotFound)?;
        state.cache.local.set_canvas(canvas.clone()).await;
        canvas
    };

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
    let lock_key = CacheKey::pixel_lock(&canvas_id, x as u8, y as u8);
    if let Some(lock_holder) = state.cache.redis.get::<String>(&lock_key).await?
        && lock_holder != user_id.to_string()
    {
        return Err(AppError::PixelLocked);
    }

    let cooldown_key = CacheKey::cooldown(&user_id);

    if let Some(last_time) = state.cache.redis.get::<u64>(&cooldown_key).await? {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_millis() as u64;

        let elapsed = now.saturating_sub(last_time);
        if elapsed < state.config.canvas.cooldown_ms {
            return Err(AppError::CooldownActive {
                remaining_ms: state.config.canvas.cooldown_ms - elapsed,
            });
        }
    }

    let pixel =
        PixelRepository::upsert_pixel(&state.db, canvas_id, x, y, Some(color), None, None).await?;

    // Invalidate pixel cache to ensure consistency on page refresh
    let cache_key = CacheKey::canvas_pixels(&canvas_id);
    let local_cache = state.cache.local.clone();
    let redis_cache = state.cache.redis.clone();
    let cooldown_cache = state.cache.redis.clone();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_millis() as u64;

    let _ = tokio::join!(
        local_cache.update_pixel(&canvas_id, x, y, color, None, 0),
        redis_cache.delete(&cache_key),
        cooldown_cache.set(
            &cooldown_key,
            &now,
            Duration::from_millis(state.config.canvas.cooldown_ms),
        ),
    );

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
    if (bid_lamports as u64) < state.config.canvas.min_bid_lamports {
        return Err(AppError::BidTooLow {
            min_lamports: state.config.canvas.min_bid_lamports,
        });
    }

    let current_pixel =
        PixelRepository::find_pixel(state.db.get_connection(), canvas_id, x, y).await?;

    let min_required_bid = if let Some(ref pixel) = current_pixel {
        pixel.price_lamports + 1
    } else {
        state.config.canvas.min_bid_lamports as i64
    };

    if bid_lamports < min_required_bid {
        return Err(AppError::BidTooLow {
            min_lamports: min_required_bid as u64,
        });
    }

    // Fetch previous owner's wallet if pixel is already claimed
    let previous_owner_wallet =
        if let Some(owner_id) = current_pixel.as_ref().and_then(|p| p.owner_id) {
            UserRepository::find_user_by_id(state.db.get_connection(), owner_id)
                .await?
                .map(|user| user.wallet_address)
        } else {
            None
        };

    // Locks pixel (Redis SETNX) to prevent race conditions.
    let lock_key = CacheKey::pixel_lock(&canvas_id, x as u8, y as u8);
    let acquired = state
        .cache
        .redis
        .setnx_with_value(
            &lock_key,
            &user_id.to_string(),
            Duration::from_millis(state.config.canvas.lock_ms),
        )
        .await?;
    if !acquired {
        return Err(AppError::PixelLocked);
    }

    let lock_expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_millis() as u64
        + state.config.canvas.lock_ms;

    Ok(PlacePixelResult {
        x,
        y,
        color,
        requires_confirmation: true,
        lock_expires_at: Some(lock_expires_at),
        previous_owner_wallet,
    })
}

pub async fn confirm_pixel_bid(
    state: &AppState,
    confirm_request: ConfirmPixelRequest,
) -> Result<PixelInfo> {
    if (confirm_request.bid_lamports as u64) < state.config.canvas.min_bid_lamports {
        return Err(AppError::BidTooLow {
            min_lamports: state.config.canvas.min_bid_lamports,
        });
    }

    let lock_key = CacheKey::pixel_lock(
        &confirm_request.canvas_id,
        confirm_request.x as u8,
        confirm_request.y as u8,
    );

    if let Some(lock_holder) = state.cache.redis.get::<String>(&lock_key).await? {
        if lock_holder != confirm_request.user_id.to_string() {
            return Err(AppError::InvalidParams(
                "This pixel is locked by another user".into(),
            ));
        }
    } else {
        return Err(AppError::InvalidParams(
            "No pending bid for this pixel".into(),
        ));
    }

    if let Some(current) = PixelRepository::find_pixel(
        state.db.get_connection(),
        confirm_request.canvas_id,
        confirm_request.x,
        confirm_request.y,
    )
    .await?
        && confirm_request.bid_lamports <= current.price_lamports
    {
        return Err(AppError::BidTooLow {
            min_lamports: (current.price_lamports + 1) as u64,
        });
    }

    // Verify transaction on Solana
    let is_valid_transaction = solana::verify_program_transaction(
        state.solana_client.get_client(),
        confirm_request.signature.as_str(),
        state.solana_client.get_program_id(),
    )
    .await?;

    if !is_valid_transaction {
        return Err(AppError::TransactionFailed(
            "Transaction verification failed".into(),
        ));
    }

    let pixel = PixelRepository::upsert_pixel(
        &state.db,
        confirm_request.canvas_id,
        confirm_request.x,
        confirm_request.y,
        Some(confirm_request.color),
        Some(confirm_request.user_id),
        Some(confirm_request.bid_lamports),
    )
    .await?;

    let _ = tokio::join!(
        state.cache.local.update_pixel(
            &confirm_request.canvas_id,
            confirm_request.x,
            confirm_request.y,
            confirm_request.color,
            Some(confirm_request.user_id),
            confirm_request.bid_lamports
        ),
        state.cache.redis.delete(&lock_key),
    );

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
    let lock_key = CacheKey::pixel_lock(&canvas_id, x as u8, y as u8);

    if let Some(lock_holder) = state.cache.redis.get::<String>(&lock_key).await? {
        if lock_holder != user_id.to_string() {
            return Err(AppError::InvalidParams(
                "Cannot cancel another user's bid".into(),
            ));
        }
    } else {
        return Err(AppError::InvalidParams(
            "No pending bid for this pixel".into(),
        ));
    }

    state.cache.redis.delete(&lock_key).await?;

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

    let is_valid_transaction = solana::verify_program_transaction(
        state.solana_client.get_client(),
        signature,
        state.solana_client.get_program_id(),
    )
    .await?;

    if !is_valid_transaction {
        return Err(AppError::TransactionFailed(
            "Transaction verification failed".into(),
        ));
    }

    let updated_pixel =
        PixelRepository::upsert_pixel(&state.db, canvas_id, x, y, Some(color), None, None).await?;

    let cache_key = CacheKey::canvas_pixels(&canvas_id);
    let price = updated_pixel.price_lamports;

    let _ = tokio::join!(
        state
            .cache
            .local
            .update_pixel(&canvas_id, x, y, color, Some(user_id), price),
        state.cache.redis.delete(&cache_key),
    );

    Ok(PixelInfo {
        x: updated_pixel.x,
        y: updated_pixel.y,
        color: updated_pixel.color,
        owner_id: updated_pixel.owner_id,
        price_lamports: price,
    })
}
