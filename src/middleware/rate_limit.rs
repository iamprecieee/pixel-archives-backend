use std::{sync::Arc, time::Duration};

use chrono::Utc;

use crate::{error::AppError, infrastructure::cache::redis::RedisCache};

#[derive(Debug, Clone)]
pub struct SlidingWindowConfig {
    pub max_requests_per_window: u32,
    pub window_duration_secs: u64,
    pub key_prefix: String,
}

#[derive(Clone)]
pub struct RateLimiter {
    redis_cache: Arc<RedisCache>,
    config: SlidingWindowConfig,
}

impl RateLimiter {
    pub fn new(redis_cache: Arc<RedisCache>, config: SlidingWindowConfig) -> Self {
        Self {
            redis_cache,
            config,
        }
    }

    /// Returns (allowed, remaining, reset_at) tuple.
    pub async fn check(&self, key: &str) -> Result<(bool, u32, u64), AppError> {
        let window_secs = self.config.window_duration_secs;
        let now = Utc::now().timestamp() as u64;

        // Calculate current and previous window keys
        let current_window = now / window_secs;
        let previous_window = current_window.saturating_sub(1);

        let current_key = format!("{}:{}:{}", self.config.key_prefix, key, current_window);
        let previous_key = format!("{}:{}:{}", self.config.key_prefix, key, previous_window);

        // Get counts from both windows
        let current_count: u32 = self
            .redis_cache
            .get::<u32>(&current_key)
            .await?
            .unwrap_or(0);

        let previous_count: u32 = self
            .redis_cache
            .get::<u32>(&previous_key)
            .await?
            .unwrap_or(0);

        // Calculate weight of previous window (how much of it overlaps with our sliding window)
        let seconds_into_current = now % window_secs;
        let previous_weight = 1.0 - (seconds_into_current as f64 / window_secs as f64);

        // Weighted count: previous * weight + current
        let weighted_count =
            (previous_count as f64 * previous_weight + current_count as f64).ceil() as u32;

        let reset_at = (current_window + 1) * window_secs;

        if weighted_count >= self.config.max_requests_per_window {
            return Ok((false, 0, reset_at));
        }

        // Increment current window counter
        let new_count = current_count + 1;
        self.redis_cache
            .set(
                &current_key,
                &new_count,
                Duration::from_secs(window_secs * 2),
            )
            .await?;

        let remaining = self
            .config
            .max_requests_per_window
            .saturating_sub(weighted_count + 1);

        Ok((true, remaining, reset_at))
    }
}

pub fn create_limiter(redis_cache: Arc<RedisCache>, limit: u32, prefix: &str) -> RateLimiter {
    RateLimiter::new(
        redis_cache,
        SlidingWindowConfig {
            max_requests_per_window: limit,
            window_duration_secs: 60,
            key_prefix: format!("rate:{}", prefix),
        },
    )
}
