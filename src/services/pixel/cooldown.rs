use std::time::Duration;

use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    infrastructure::cache::{keys::CacheKey, redis::RedisCache},
};

pub fn get_current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_millis() as u64
}

/// Checks if the user is on cooldown. Returns error with remaining time if active.
pub async fn check_cooldown_state(
    redis: &RedisCache,
    user_id: &Uuid,
    cooldown_ms: u64,
) -> Result<()> {
    let key = CacheKey::cooldown(user_id);
    if let Some(last_time) = redis.get::<u64>(&key).await? {
        let now = get_current_time_ms();
        let elapsed = now.saturating_sub(last_time);
        if elapsed < cooldown_ms {
            return Err(AppError::CooldownActive {
                remaining_ms: cooldown_ms - elapsed,
            });
        }
    }
    Ok(())
}

/// Records the current time as the user's last action for cooldown purposes.
pub async fn record_cooldown_state(
    redis: &RedisCache,
    user_id: &Uuid,
    cooldown_ms: u64,
) -> Result<()> {
    let key = CacheKey::cooldown(user_id);
    let now = get_current_time_ms();
    redis
        .set(&key, &now, Duration::from_millis(cooldown_ms))
        .await
}
