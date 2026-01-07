pub mod keys;
pub mod local;
pub mod redis;

use crate::config::Config;
use crate::error::Result;
use crate::infrastructure::cache::local::LocalCache;
use crate::infrastructure::cache::redis::RedisCache;

pub struct Cache {
    pub local: LocalCache,
    pub redis: RedisCache,
}

impl Cache {
    pub async fn init(config: &Config) -> Result<Self> {
        Ok(Self {
            local: LocalCache::new(&config.cache),
            redis: RedisCache::connect(&config.cache).await?,
        })
    }
}
