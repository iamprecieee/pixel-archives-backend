use std::{env, str::FromStr, time::Duration};

use crate::error::{AppError, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            server: ServerConfig {
                host: env_or_default("HOST", "127.0.0.1"),
                port: env_or_parse("PORT", 8080)?,
            },
            database: DatabaseConfig {
                url: env_required("DATABASE_URL")?,
                max_connections: env_or_parse("DB_MAX_CONNECTIONS", 10)?,
                min_connections: env_or_parse("DB_MIN_CONNECTIONS", 5)?,
                connect_timeout: Duration::from_secs(env_or_parse("DB_CONNECT_TIMEOUT_SECS", 10)?),
                idle_timeout: Duration::from_secs(env_or_parse("DB_IDLE_TIMEOUT_SECS", 300)?),
            },
        })
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
