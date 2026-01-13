use std::time::Duration;

use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    infrastructure::cache::{keys::CacheKey, redis::RedisCache},
};

/// Checks if the pixel is locked by another user.
/// Returns Ok(()) if unlocked or locked by the same user.
pub async fn assert_not_locked_by_other(
    redis: &RedisCache,
    canvas_id: &Uuid,
    x: u8,
    y: u8,
    user_id: &Uuid,
) -> Result<()> {
    let lock_key = CacheKey::pixel_lock(canvas_id, x, y);
    if let Some(lock_holder) = redis.get::<String>(&lock_key).await?
        && lock_holder != user_id.to_string()
    {
        return Err(AppError::PixelLocked);
    }
    Ok(())
}

/// Asserts the user owns the lock. Returns error if no lock or different owner.
pub async fn assert_lock_owned(
    redis: &RedisCache,
    canvas_id: &Uuid,
    x: u8,
    y: u8,
    user_id: &Uuid,
) -> Result<()> {
    let lock_key = CacheKey::pixel_lock(canvas_id, x, y);
    match redis.get::<String>(&lock_key).await? {
        Some(holder) if holder == user_id.to_string() => Ok(()),
        Some(_) => Err(AppError::InvalidParams(
            "This pixel is locked by another user".into(),
        )),
        None => Err(AppError::InvalidParams(
            "No pending bid for this pixel".into(),
        )),
    }
}

/// Attempts to acquire a lock on the pixel. Returns true if acquired.
pub async fn acquire_pixel_lock(
    redis: &RedisCache,
    canvas_id: &Uuid,
    x: u8,
    y: u8,
    user_id: &Uuid,
    ttl: Duration,
) -> Result<bool> {
    let lock_key = CacheKey::pixel_lock(canvas_id, x, y);
    redis
        .setnx_with_value(&lock_key, &user_id.to_string(), ttl)
        .await
}

/// Releases the lock on the pixel.
pub async fn release_pixel_lock(redis: &RedisCache, canvas_id: &Uuid, x: u8, y: u8) -> Result<()> {
    let lock_key = CacheKey::pixel_lock(canvas_id, x, y);
    redis.delete(&lock_key).await
}
