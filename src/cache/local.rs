use std::sync::Arc;

use moka::future::Cache;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{config::CacheConfig, db::entities::canvas};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CachedPixel {
    pub x: i16,
    pub y: i16,
    pub color: i16,
    pub owner_id: Option<Uuid>,
    pub price_lamports: i64,
}

pub struct LocalCache {
    canvas_cache: Cache<Uuid, Arc<canvas::Model>>,
    pixels_cache: Cache<Uuid, Arc<RwLock<Vec<CachedPixel>>>>,
}

impl LocalCache {
    pub fn new(cache_config: &CacheConfig) -> Self {
        Self {
            canvas_cache: Cache::builder()
                .max_capacity(cache_config.local_canvas_max_capacity)
                .time_to_live(Duration::from_secs(cache_config.local_canvas_mid_ttl))
                .time_to_idle(Duration::from_secs(cache_config.local_canvas_short_ttl))
                .build(),

            pixels_cache: Cache::builder()
                .max_capacity(cache_config.local_pixels_max_capacity)
                .time_to_live(Duration::from_secs(cache_config.local_pixels_mid_ttl))
                .time_to_idle(Duration::from_secs(cache_config.local_pixels_short_ttl))
                .build(),
        }
    }

    pub async fn get_canvas(&self, id: &Uuid) -> Option<Arc<canvas::Model>> {
        self.canvas_cache.get(id).await
    }

    pub async fn set_canvas(&self, canvas: canvas::Model) {
        self.canvas_cache.insert(canvas.id, Arc::new(canvas)).await;
    }

    pub async fn invalidate_canvas(&self, id: &Uuid) {
        self.canvas_cache.invalidate(id).await;
    }

    pub async fn invalidate_pixels(&self, canvas_id: &Uuid) {
        self.pixels_cache.invalidate(canvas_id).await;
    }

    pub async fn update_pixel(
        &self,
        canvas_id: &Uuid,
        x: i16,
        y: i16,
        color: i16,
        owner_id: Option<Uuid>,
        price: i64,
    ) {
        if let Some(pixels) = self.pixels_cache.get(canvas_id).await {
            let mut pixels = pixels.write().await;

            if let Some(pixel) = pixels.iter_mut().find(|p| p.x == x && p.y == y) {
                pixel.color = color;
                pixel.owner_id = owner_id;
                pixel.price_lamports = price;
            } else {
                pixels.push(CachedPixel {
                    x,
                    y,
                    color,
                    owner_id,
                    price_lamports: price,
                });
            }
        }
    }
}
