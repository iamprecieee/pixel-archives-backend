use base64::Engine;
use std::time::Duration;
use uuid::Uuid;

use crate::{
    AppState,
    error::{AppError, Result},
    infrastructure::{
        cache::keys::CacheKey,
        db::repositories::{CanvasRepository, PixelRepository},
    },
    services::canvas::types::{
        CachedPixelData, CanvasInfo, CanvasWithPixels, JoinCanvasResult, OwnedCanvasPixelInfo,
        UserCanvases,
    },
};

pub async fn join_canvas(
    state: &AppState,
    user_id: Uuid,
    invite_code: &str,
) -> Result<JoinCanvasResult> {
    let canvas =
        CanvasRepository::find_canvas_by_invite_code(state.db.get_connection(), invite_code)
            .await?
            .ok_or(AppError::CanvasNotFound)?;

    if CanvasRepository::is_canvas_collaborator(state.db.get_connection(), canvas.id, user_id)
        .await?
    {
        return Ok(JoinCanvasResult {
            canvas_id: canvas.id,
            already_member: true,
        });
    }

    CanvasRepository::add_canvas_collaborator(state.db.get_connection(), canvas.id, user_id)
        .await?;

    Ok(JoinCanvasResult {
        canvas_id: canvas.id,
        already_member: false,
    })
}

pub async fn get_canvas(
    state: &AppState,
    canvas_id: Uuid,
    user_id: Uuid,
) -> Result<CanvasWithPixels> {
    let db_connection = state.db.get_connection();

    if !CanvasRepository::is_canvas_collaborator(db_connection, canvas_id, user_id).await? {
        return Err(AppError::NotCanvasCollaborator);
    }
    let canvas = if let Some(cached) = state.cache.local.get_canvas(&canvas_id).await {
        (*cached).clone()
    } else {
        let canvas = CanvasRepository::find_canvas_by_id(db_connection, canvas_id)
            .await?
            .ok_or(AppError::CanvasNotFound)?;

        state.cache.local.set_canvas(canvas.clone()).await;
        canvas
    };

    // for redis (l2) cache
    let cache_key = CacheKey::canvas_pixels(&canvas_id);

    let pixel_data =
        if let Ok(Some(cached)) = state.cache.redis.get::<CachedPixelData>(&cache_key).await {
            cached.clone()
        } else {
            let pixels = PixelRepository::find_pixels_by_canvas(db_connection, canvas_id).await?;

            let width = state.config.canvas.width as usize;
            let height = state.config.canvas.height as usize;

            let mut color_bytes = vec![0u8; width * height];
            let mut owned_pixels = Vec::new();

            for pixel in pixels {
                let index = (pixel.y as usize) * (width) + (pixel.x as usize);
                if index < color_bytes.len() {
                    color_bytes[index] = pixel.color as u8;
                }

                if let Some(owner_id) = pixel.owner_id {
                    owned_pixels.push(OwnedCanvasPixelInfo {
                        x: pixel.x,
                        y: pixel.y,
                        owner_id: owner_id.to_string(),
                        price_lamports: pixel.price_lamports,
                    });
                }
            }

            let pixel_colors = base64::engine::general_purpose::STANDARD.encode(&color_bytes);

            let cached_pixels = CachedPixelData {
                pixel_colors,
                owned_pixels,
            };

            state
                .cache
                .redis
                .set(
                    &cache_key,
                    &cached_pixels,
                    Duration::from_secs(state.config.cache.redis_cache_mid_ttl),
                )
                .await?;

            cached_pixels
        };

    Ok(CanvasWithPixels {
        canvas: CanvasInfo {
            id: canvas.id,
            name: canvas.name,
            invite_code: canvas.invite_code,
            state: canvas.state,
            owner_id: canvas.owner_id,
            canvas_pda: canvas.canvas_pda,
            mint_address: canvas.mint_address,
            total_escrowed: canvas.total_escrowed,
        },
        pixel_colors: pixel_data.pixel_colors,
        owned_pixels: pixel_data.owned_pixels,
    })
}

pub async fn list_canvases_by_user(state: &AppState, user_id: Uuid) -> Result<UserCanvases> {
    let db_connection = state.db.get_connection();

    let (owned, collaborating) = tokio::join!(
        CanvasRepository::list_canvases_by_owner(db_connection, user_id),
        CanvasRepository::list_canvases_by_collaborator(db_connection, user_id)
    );

    Ok(UserCanvases {
        owned: owned?.into_iter().map(CanvasInfo::from).collect(),
        collaborating: collaborating?
            .into_iter()
            .filter(|canvas| canvas.owner_id != user_id)
            .map(CanvasInfo::from)
            .collect(),
    })
}
