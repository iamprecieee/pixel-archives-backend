use std::time::Duration;

use deadpool_redis::{
    Config as PoolConfig, Pool, Runtime,
    redis::{self, AsyncCommands},
};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    config::CacheConfig,
    error::{AppError, Result},
};

#[derive(Clone)]
pub struct RedisCache {
    pool: Pool,
}

impl RedisCache {
    pub async fn connect(cache_config: &CacheConfig) -> Result<Self> {
        let pool_config = PoolConfig::from_url(&cache_config.url);
        let pool = pool_config
            .builder()
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .max_size(cache_config.pool_size)
            .wait_timeout(Some(cache_config.connect_timeout))
            .runtime(Runtime::Tokio1)
            .build()
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let mut redis_connection = pool
            .get()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let _: String = redis::cmd("PING")
            .query_async(&mut *redis_connection)
            .await?;

        Ok(Self { pool })
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut redis_connection = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let value: Option<String> = redis_connection.get(key).await?;
        match value {
            Some(val) => Ok(Some(serde_json::from_str(&val)?)),
            None => Ok(None),
        }
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<()> {
        let mut redis_connection = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let serialized = serde_json::to_string(value)?;
        redis_connection
            .set_ex::<_, _, ()>(key, serialized, ttl.as_secs())
            .await?;
        Ok(())
    }

    pub async fn setnx(&self, key: &str, ttl: Duration) -> Result<bool> {
        let mut redis_connection = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let result: Option<String> = redis::cmd("SET")
            .arg(key)
            .arg("true")
            .arg("NX")
            .arg("EX")
            .arg(ttl.as_secs())
            .query_async(&mut *redis_connection)
            .await?;

        Ok(result.is_some())
    }

    pub async fn setnx_with_value(&self, key: &str, value: &str, ttl: Duration) -> Result<bool> {
        let mut redis_connection = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let result: Option<String> = redis::cmd("SET")
            .arg(key)
            .arg(value)
            .arg("NX")
            .arg("EX")
            .arg(ttl.as_secs())
            .query_async(&mut *redis_connection)
            .await?;

        Ok(result.is_some())
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut redis_connection = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        redis_connection.del::<_, ()>(key).await?;
        Ok(())
    }
}
