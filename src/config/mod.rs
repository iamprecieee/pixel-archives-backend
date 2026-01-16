use std::{env, str::FromStr, time::Duration};

use crate::error::{AppError, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub jwt: JwtConfig,
    pub canvas: CanvasConfig,
    pub solana: SolanaConfig,
    pub rate_limit: RateLimitConfig,
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
    pub redis_cache_mid_ttl: u64,
    pub redis_cache_short_ttl: u64,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct CanvasConfig {
    pub max_name_length: u8,
    pub width: u8,
    pub height: u8,
    pub color_count: u8,
    pub min_bid_lamports: u64,
    pub cooldown_ms: u64,
    pub max_collaborators: usize,
    pub lock_ms: u64,
    pub mint_countdown_secs: u8,
}

#[derive(Debug, Clone)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub program_id: String,
    pub commitment: String,
    pub blockhash_ttl: u64,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub auth_limit: u32,
    pub pixel_limit: u32,
    pub canvas_limit: u32,
    pub solana_limit: u32,
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
                redis_cache_short_ttl: env_or_parse("CACHE_REDIS_SHORT_TTL", 120)?,
                redis_cache_mid_ttl: env_or_parse("CACHE_REDIS_MID_TTL", 300)?,
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
            canvas: CanvasConfig {
                max_name_length: env_or_parse("MAX_CANVAS_NAME_LENGTH", 32)?,
                width: env_or_parse("CANVAS_WIDTH", 32)?,
                height: env_or_parse("CANVAS_HEIGHT", 32)?,
                color_count: env_or_parse("CANVAS_COLORS", 64)?,
                min_bid_lamports: env_or_parse("MIN_BID_LAMPORTS", 1_000_000)?, // 0.001 SOL
                cooldown_ms: env_or_parse("PIXEL_COOLDOWN_MS", 5000)?,
                max_collaborators: env_or_parse("MAX_COLLABORATORS", 50)?,
                lock_ms: env_or_parse("PIXEL_LOCK_MS", 60000)?,
                mint_countdown_secs: env_or_parse("MINT_COUNTDOWN_SECS", 30)?,
            },
            solana: SolanaConfig {
                rpc_url: env_required("SOLANA_RPC_URL")?,
                program_id: env_required("SOLANA_PROGRAM_ID")?,
                commitment: env_or("SOLANA_COMMITMENT", "confirmed"),
                blockhash_ttl: env_or_parse("SOLANA_BLOCKHASH_TTL", 15)?,
            },
            rate_limit: RateLimitConfig {
                auth_limit: env_or_parse("RATE_LIMIT_AUTH", 10)?,
                pixel_limit: env_or_parse("RATE_LIMIT_PIXEL", 30)?,
                canvas_limit: env_or_parse("RATE_LIMIT_CANVAS", 5)?,
                solana_limit: env_or_parse("RATE_LIMIT_SOLANA", 20)?,
            },
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.jwt.secret.len() < 32 {
            return Err(AppError::InvalidParams(
                "JWT_SECRET must be at least 32 characters".into(),
            ));
        }

        if self.canvas.width == 0 || self.canvas.height == 0 {
            return Err(AppError::InvalidParams(
                "Canvas dimensions must be positive".into(),
            ));
        }

        if self.canvas.color_count == 0 {
            return Err(AppError::InvalidParams(
                "Color count must be positive".into(),
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

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
