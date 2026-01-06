use std::{env, str::FromStr};

use crate::error::{AppError, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            server: ServerConfig {
                host: env_or_default("HOST", "127.0.0.1"),
                port: env_or_parse("PORT", 8080)?,
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
