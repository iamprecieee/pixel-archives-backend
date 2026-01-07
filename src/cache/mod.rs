pub mod keys;
pub mod local;
pub mod redis;

use crate::cache::local::LocalCache;
use crate::cache::redis::RedisCache;
use crate::config::Config;

pub struct Cache {
    pub local: LocalCache,
    pub redis: RedisCache,
}

impl Cache {
    pub async fn init(config: &Config) -> Self {
        Self {
            local: LocalCache::new(&config.cache),
            redis: RedisCache::connect(&config.cache).await.unwrap(),
        }
    }
}
