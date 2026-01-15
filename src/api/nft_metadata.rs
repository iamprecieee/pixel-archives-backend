use axum::{
    Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use uuid::Uuid;

use crate::{
    AppState,
    services::nft::{self as nft_service, image::generate_png_from_colors},
};

pub async fn get_nft_image(State(state): State<AppState>, Path(canvas_id): Path<Uuid>) -> Response {
    let (canvas_pda, _) = state.solana_client.derive_canvas_pda_from_uuid(&canvas_id);

    let account_data = match state.solana_client.get_account_data(&canvas_pda).await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to fetch canvas from blockchain: {}", e);
            return (StatusCode::NOT_FOUND, "Canvas not found on-chain").into_response();
        }
    };

    // Parse pixel_colors directly from blockchain account data.
    // Ensures strictly trustless representation.
    // CanvasMetadata layout (Total 835 bytes):
    //   0-7:    Discriminator (8 bytes)
    //   8-39:   Owner Pubkey (32 bytes)
    //   40-55:  ID (16 bytes)
    //   56:     State (1 byte)
    //   57:     Width (1 byte)
    //   58:     Height (1 byte)
    //   59-66:  Total Escrow (8 bytes)
    //   67-834: Pixel Colors (768 bytes) - 6-bit packed, 4 pixels/3 bytes
    const PIXEL_COLORS_OFFSET: usize = 67;
    const PIXEL_COLORS_SIZE: usize = 768;

    if account_data.len() < PIXEL_COLORS_OFFSET + PIXEL_COLORS_SIZE {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Invalid canvas account data",
        )
            .into_response();
    }

    let pixel_colors = &account_data[PIXEL_COLORS_OFFSET..PIXEL_COLORS_OFFSET + PIXEL_COLORS_SIZE];

    let image_data = match generate_png_from_colors(pixel_colors) {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to generate image: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate image",
            )
                .into_response();
        }
    };

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        image_data,
    )
        .into_response()
}

pub async fn get_metadata(State(state): State<AppState>, Path(canvas_id): Path<Uuid>) -> Response {
    let metadata_result = match nft_service::get_nft_metadata(&state, canvas_id).await {
        Ok(metadata) => metadata,
        Err(_) => return (StatusCode::NOT_FOUND, "Metadata not found").into_response(),
    };

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        serde_json::to_string_pretty(&metadata_result).unwrap_or_default(),
    )
        .into_response()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{canvas_id}/image", get(get_nft_image))
        .route("/{canvas_id}/image.png", get(get_nft_image))
        .route("/{canvas_id}/metadata", get(get_metadata))
        .route("/{canvas_id}/metadata.json", get(get_metadata))
}
