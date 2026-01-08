use std::{env, str::FromStr, time::Duration};

use crate::error::{AppError, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub jwt: JwtConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_allowed_origins: Vec<String>,
    pub max_concurrent_requests: usize,
    pub server_public_url: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub url: String,
    pub pool_size: usize,
    pub connect_timeout: Duration,
    pub local_canvas_max_capacity: u64,
    pub local_canvas_short_ttl: u64,
    pub local_canvas_mid_ttl: u64,
    pub local_pixels_max_capacity: u64,
    pub local_pixels_short_ttl: u64,
    pub local_pixels_mid_ttl: u64,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            server: ServerConfig {
                host: env_or_default("HOST", "127.0.0.1"),
                port: env_or_parse("PORT", 8080)?,
                cors_allowed_origins: env_list("CORS_ALLOWED_ORIGINS", vec!["".into()]),
                max_concurrent_requests: env_or_parse("SERVER_MAX_CONCURRENT_REQUESTS", 100)?,
                server_public_url: env_required("SERVER_PUBLIC_URL")?,
            },
            database: DatabaseConfig {
                url: env_required("DATABASE_URL")?,
                max_connections: env_or_parse("DB_MAX_CONNECTIONS", 10)?,
                min_connections: env_or_parse("DB_MIN_CONNECTIONS", 5)?,
                connect_timeout: Duration::from_secs(env_or_parse("DB_CONNECT_TIMEOUT_SECS", 10)?),
                idle_timeout: Duration::from_secs(env_or_parse("DB_IDLE_TIMEOUT_SECS", 300)?),
            },
            cache: CacheConfig {
                url: env_required("CACHE_URL")?,
                pool_size: env_or_parse("CACHE_POOL_SIZE", 10)?,
                connect_timeout: Duration::from_secs(env_or_parse(
                    "CACHE_CONNECT_TIMEOUT_SECS",
                    10,
                )?),
                local_canvas_max_capacity: env_or_parse("CACHE_LOCAL_CANVAS_MAX_CAPACITY", 500)?,
                local_canvas_short_ttl: env_or_parse("CACHE_LOCAL_CANVAS_SHORT_TTL", 15)?,
                local_canvas_mid_ttl: env_or_parse("CACHE_LOCAL_CANVAS_MID_TTL", 30)?,
                local_pixels_max_capacity: env_or_parse("CACHE_LOCAL_PIXELS_MAX_CAPACITY", 100)?,
                local_pixels_short_ttl: env_or_parse("CACHE_LOCAL_PIXELS_SHORT_TTL", 5)?,
                local_pixels_mid_ttl: env_or_parse("CACHE_LOCAL_PIXELS_MID_TTL", 10)?,
            },
            jwt: JwtConfig {
                secret: env_required("JWT_SECRET")?,
                access_token_ttl: Duration::from_secs(
                    env_or_parse("JWT_ACCESS_TTL_SECS", 900)?, // 15mins
                ),
                refresh_token_ttl: Duration::from_secs(
                    env_or_parse("JWT_REFRESH_TTL_SECS", 3600)?, // 1hr
                ),
            },
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.jwt.secret.len() < 32 {
            return Err(AppError::InvalidParams(
                "JWT_SECRET must be at least 32 characters".into(),
            ));
        }

        Ok(())
    }
}

fn env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_or_parse<T: FromStr>(key: &str, default: T) -> Result<T> {
    match env::var(key) {
        Ok(val) => val
            .parse()
            .map_err(|_| AppError::InvalidParams(format!("Invalid value for {key}"))),
        Err(_) => Ok(default),
    }
}

fn env_required(key: &str) -> Result<String> {
    env::var(key).map_err(|_| AppError::InvalidParams(format!("{key} is required")))
}

fn env_list(key: &str, default: Vec<String>) -> Vec<String> {
    env::var(key)
        .map(|val| {
            val.split(',')
                .map(|str_val| str_val.trim().to_string())
                .collect()
        })
        .unwrap_or(default)
}
