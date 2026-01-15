pub mod image;
pub mod types;

use std::collections::HashMap;

use base64::Engine;
use sea_orm::ActiveValue::Set;
use uuid::Uuid;

use crate::{
    AppState,
    error::{AppError, Result},
    infrastructure::{
        cache::keys::CacheKey,
        db::{
            entities::canvas::CanvasState,
            repositories::{CanvasRepository, PixelRepository, UserRepository},
        },
    },
    services::{
        nft::types::{
            Attribute, CreatorOutput, ImageFile, MetadataResult, MintResult, MintTransactionInfo,
            NftMetadata, Properties,
        },
        solana,
    },
    ws::types::RoomCanvasUpdate,
};

pub async fn prepare_metadata(state: &AppState, canvas_id: Uuid) -> Result<MetadataResult> {
    let pixels =
        PixelRepository::find_pixels_by_canvas(state.db.get_connection(), canvas_id).await?;
    let image_data = image::generate_png(&pixels)?;

    let image_base64 = base64::engine::general_purpose::STANDARD.encode(&image_data);
    let image_data_uri = format!("data:image/png;base64,{}", image_base64);

    let canvas = CanvasRepository::find_canvas_by_id(state.db.get_connection(), canvas_id)
        .await?
        .ok_or(AppError::CanvasNotFound)?;

    let canvas_owner = UserRepository::find_user_by_id(state.db.get_connection(), canvas.owner_id)
        .await?
        .ok_or(AppError::UserNotFound)?;

    let top_pixel_owners =
        PixelRepository::find_top_pixel_owners(state.db.get_connection(), canvas_id, 4).await?;

    let total_sol_invested: i64 = top_pixel_owners.iter().map(|(_, amount)| amount).sum();

    let mut creators_list = Vec::new();

    // Owner gets min 10%.
    let canvas_owner_base_share: u8 = if top_pixel_owners.is_empty() { 100 } else { 10 };
    let remaining_share: u8 = 100 - canvas_owner_base_share;

    // Add owner as first creator
    creators_list.push(serde_json::json!({
        "address": canvas_owner.wallet_address,
        "share": canvas_owner_base_share
    }));

    // Batch fetch users.
    let other_owner_ids: Vec<Uuid> = top_pixel_owners
        .iter()
        .filter(|(id, _)| *id != canvas.owner_id)
        .map(|(id, _)| *id)
        .collect();

    let users_map: HashMap<Uuid, _> =
        UserRepository::find_users_by_ids(state.db.get_connection(), &other_owner_ids)
            .await?
            .into_iter()
            .map(|u| (u.id, u))
            .collect();

    // Add top pixel claimers (excluding owner)
    for (owner_id, amount) in &top_pixel_owners {
        if *owner_id == canvas.owner_id {
            continue;
        }

        if let Some(user) = users_map.get(owner_id) {
            let share = if total_sol_invested > 0 {
                ((*amount as f64 / total_sol_invested as f64) * remaining_share as f64).round()
                    as u8
            } else {
                0
            };
            if share > 0 {
                creators_list.push(serde_json::json!({
                    "address": user.wallet_address,
                    "share": share
                }));
            }
        }
    }

    // Ensure shares sum to 100.
    let total_shares: u8 = creators_list
        .iter()
        .filter_map(|creator| creator["share"].as_u64().map(|s| s as u8))
        .sum();
    if total_shares != 100
        && let Some(first) = creators_list.first_mut()
    {
        let first_share = first["share"].as_u64().unwrap_or(0) as i16;
        let adjustment = 100i16 - total_shares as i16;
        let new_share = (first_share + adjustment).max(1) as u64;
        first["share"] = serde_json::json!(new_share);
    }

    let creators_output: Vec<CreatorOutput> = creators_list
        .iter()
        .filter_map(|creator| {
            creator["address"].as_str().map(|addr| CreatorOutput {
                address: addr.to_string(),
                share: creator["share"].as_u64().unwrap_or(0) as u8,
            })
        })
        .collect();

    let metadata_uri = format!(
        "{}/nft/{}/metadata.json",
        state.config.server.server_public_url, canvas_id
    );

    Ok(MetadataResult {
        metadata_uri,
        image_uri: image_data_uri.clone(),
        image_gateway_url: image_data_uri,
        metadata_gateway_url: String::new(),
        creators: creators_output,
    })
}

pub async fn initiate_nft_mint(
    state: &AppState,
    canvas_id: Uuid,
    user_id: Uuid,
) -> Result<MintTransactionInfo> {
    let canvas = CanvasRepository::find_canvas_by_id(state.db.get_connection(), canvas_id)
        .await?
        .ok_or(AppError::CanvasNotFound)?;

    if canvas.owner_id != user_id {
        return Err(AppError::NotCanvasOwner);
    }

    // Verify canvas is in MintPending state (lock was set by announceMint)
    if canvas.state != CanvasState::MintPending {
        return Err(AppError::InvalidCanvasStateTransition);
    }

    CanvasRepository::update_canvas_state(&state.db, canvas_id, CanvasState::Minting, |_active| {})
        .await?;

    state
        .ws_rooms
        .broadcast(&canvas_id, RoomCanvasUpdate::MintingStarted)
        .await;

    let canvas_pda_string = canvas.canvas_pda.ok_or(AppError::InvalidParams(
        "Canvas not published on-chain".into(),
    ))?;

    let (config_pda, _) = state.solana_client.derive_config_pda();

    let blockhash = state
        .solana_client
        .get_recent_blockhash()
        .await
        .map_err(|e| AppError::SolanaRpc(e.to_string()))?;

    Ok(MintTransactionInfo {
        canvas_id,
        canvas_pda: canvas_pda_string,
        config_pda: config_pda.to_string(),
        program_id: state.solana_client.get_program_id().to_string(),
        blockhash: blockhash.to_string(),
        canvas_name: canvas.name,
    })
}

pub async fn confirm_nft_mint(
    state: &AppState,
    canvas_id: Uuid,
    user_id: Uuid,
    signature: &str,
    mint_address: &str,
) -> Result<MintResult> {
    let canvas = CanvasRepository::find_canvas_by_id(state.db.get_connection(), canvas_id)
        .await?
        .ok_or(AppError::CanvasNotFound)?;

    if canvas.owner_id != user_id {
        return Err(AppError::NotCanvasOwner);
    }

    let tx_valid = solana::verify_program_transaction(
        state.solana_client.get_client(),
        signature,
        state.solana_client.get_program_id(),
    )
    .await?;

    if !tx_valid {
        return Err(AppError::TransactionFailed(
            "Transaction verification failed".into(),
        ));
    }

    let canvas = CanvasRepository::update_canvas_state(
        &state.db,
        canvas_id,
        CanvasState::Minted,
        |active| {
            active.mint_address = Set(Some(mint_address.to_string()));
        },
    )
    .await?;

    let lock_key = CacheKey::canvas_lock(&canvas_id);
    state.cache.redis.delete(&lock_key).await?;

    state
        .ws_rooms
        .broadcast(
            &canvas_id,
            RoomCanvasUpdate::Minted {
                mint_address: mint_address.to_string(),
            },
        )
        .await;

    Ok(MintResult {
        canvas_id,
        mint_address: canvas.mint_address,
        state: canvas.state,
    })
}

pub async fn cancel_mint(state: &AppState, canvas_id: Uuid, user_id: Uuid) -> Result<()> {
    let canvas = CanvasRepository::find_canvas_by_id(state.db.get_connection(), canvas_id)
        .await?
        .ok_or(AppError::CanvasNotFound)?;

    if canvas.owner_id != user_id {
        return Err(AppError::NotCanvasOwner);
    }

    CanvasRepository::update_canvas_state(
        &state.db,
        canvas_id,
        CanvasState::Published,
        |_active| {},
    )
    .await?;

    let lock_key = CacheKey::canvas_lock(&canvas_id);
    state.cache.redis.delete(&lock_key).await?;

    state
        .ws_rooms
        .broadcast(
            &canvas_id,
            RoomCanvasUpdate::MintingFailed {
                reason: "Cancelled by user".into(),
            },
        )
        .await;

    Ok(())
}

pub async fn get_nft_metadata(state: &AppState, canvas_id: Uuid) -> Result<NftMetadata> {
    let canvas = CanvasRepository::find_canvas_by_id(state.db.get_connection(), canvas_id)
        .await?
        .ok_or(AppError::CanvasNotFound)?;

    if canvas.state != CanvasState::Minted {
        return Err(AppError::InvalidParams("Canvas is not minted".into()));
    }

    let owner = UserRepository::find_user_by_id(state.db.get_connection(), canvas.owner_id)
        .await?
        .ok_or(AppError::UserNotFound)?;

    let pixels =
        PixelRepository::find_pixels_by_canvas(state.db.get_connection(), canvas_id).await?;

    let claimed_count = pixels
        .iter()
        .filter(|pixel| pixel.owner_id.is_some())
        .count();

    let base_url = &state.config.server.server_public_url;
    let image_url = format!("{}/nft/{}/image.png", base_url, canvas_id);

    Ok(NftMetadata {
        name: canvas.name.clone(),
        symbol: "PIXEL".into(),
        description: format!("{}: 32x32 collaborative pixel art canvas.", canvas.name),
        image: image_url.clone(),
        seller_fee_basis_points: 500,
        attributes: vec![
            Attribute {
                trait_type: "Width".into(),
                value: "32".into(),
            },
            Attribute {
                trait_type: "Height".into(),
                value: "32".into(),
            },
            Attribute {
                trait_type: "Pixels Claimed".into(),
                value: claimed_count.to_string(),
            },
        ],
        properties: Properties {
            files: vec![ImageFile {
                uri: image_url,
                file_type: "image/png".into(),
            }],
            category: "image".into(),
            creators: vec![CreatorOutput {
                address: owner.wallet_address,
                share: 100,
            }],
        },
    })
}
