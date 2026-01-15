use crate::{
    api::types::{
        CancelPublishCanvasParams, CancelPublishCanvasResponse, CanvasResponse,
        CanvasWithPixelsResponse, ConfirmPublishCanvasParams, ConfirmPublishCanvasResponse,
        CreateCanvasParams, DeleteCanvasParams, DeleteCanvasResponse, GetCanvasParams,
        JoinCanvasParams, JoinCanvasResponse, ListCanvasParams, ListCanvasResponse, OwnedPixelInfo,
        PublishCanvasParams, PublishCanvasResponse,
    },
    error::{AppError, Result},
    services::{auth::TokenType, canvas as canvas_service},
};

pub async fn create_canvas(params: CreateCanvasParams) -> Result<CanvasResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    let initial_color = params.initial_color.unwrap_or(0);
    let canvas =
        canvas_service::create_canvas(&app_state, user_id, &params.name, initial_color).await?;

    Ok(CanvasResponse {
        id: canvas.id.to_string(),
        name: canvas.name,
        invite_code: canvas.invite_code,
        state: format!("{:?}", canvas.state).to_lowercase(),
        owner_id: canvas.owner_id.to_string(),
        canvas_pda: canvas.canvas_pda,
        mint_address: canvas.mint_address,
    })
}

pub async fn get_canvas(params: GetCanvasParams) -> Result<CanvasWithPixelsResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    let result = canvas_service::get_canvas(&app_state, params.canvas_id, user_id).await?;

    Ok(CanvasWithPixelsResponse {
        canvas: CanvasResponse {
            id: result.canvas.id.to_string(),
            name: result.canvas.name,
            invite_code: result.canvas.invite_code,
            state: format!("{:?}", result.canvas.state).to_lowercase(),
            owner_id: result.canvas.owner_id.to_string(),
            canvas_pda: result.canvas.canvas_pda,
            mint_address: result.canvas.mint_address,
        },
        pixel_colors: result.pixel_colors,
        owned_pixels: result
            .owned_pixels
            .into_iter()
            .map(|p| OwnedPixelInfo {
                x: p.x,
                y: p.y,
                owner_id: p.owner_id,
                price_lamports: p.price_lamports,
            })
            .collect(),
    })
}

pub async fn list_canvas(params: ListCanvasParams) -> Result<ListCanvasResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    let canvases = canvas_service::list_canvases_by_user(&app_state, user_id).await?;

    Ok(ListCanvasResponse {
        owned: canvases
            .owned
            .into_iter()
            .map(|c| CanvasResponse {
                id: c.id.to_string(),
                name: c.name,
                invite_code: c.invite_code,
                state: format!("{:?}", c.state).to_lowercase(),
                owner_id: c.owner_id.to_string(),
                canvas_pda: c.canvas_pda,
                mint_address: c.mint_address,
            })
            .collect(),
        collaborating: canvases
            .collaborating
            .into_iter()
            .map(|c| CanvasResponse {
                id: c.id.to_string(),
                name: c.name,
                invite_code: c.invite_code,
                state: format!("{:?}", c.state).to_lowercase(),
                owner_id: c.owner_id.to_string(),
                canvas_pda: c.canvas_pda,
                mint_address: c.mint_address,
            })
            .collect(),
    })
}

pub async fn join_canvas(params: JoinCanvasParams) -> Result<JoinCanvasResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    let result = canvas_service::join_canvas(&app_state, user_id, &params.invite_code).await?;

    Ok(JoinCanvasResponse {
        success: true,
        canvas_id: result.canvas_id.to_string(),
    })
}

pub async fn publish_canvas(params: PublishCanvasParams) -> Result<PublishCanvasResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    let publish_info =
        canvas_service::initialize_canvas_publish(&app_state, params.canvas_id, user_id).await?;

    Ok(PublishCanvasResponse {
        success: true,
        state: "publishing".to_string(),
        pixel_colors_packed: publish_info.pixel_colors_packed,
    })
}

pub async fn confirm_publish_canvas(
    params: ConfirmPublishCanvasParams,
) -> Result<ConfirmPublishCanvasResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    let canvas = canvas_service::confirm_canvas_publish(
        &app_state,
        params.canvas_id,
        user_id,
        &params.signature,
        &params.canvas_pda,
    )
    .await?;

    Ok(ConfirmPublishCanvasResponse {
        success: true,
        state: "published".to_string(),
        canvas_pda: if let Some(value) = canvas.canvas_pda {
            value.to_string()
        } else {
            "".to_string()
        },
    })
}

pub async fn cancel_publish_canvas(
    params: CancelPublishCanvasParams,
) -> Result<CancelPublishCanvasResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    canvas_service::cancel_canvas_publish(&app_state, params.canvas_id, user_id).await?;

    Ok(CancelPublishCanvasResponse {
        success: true,
        state: "draft".to_string(),
    })
}

pub async fn delete_canvas(params: DeleteCanvasParams) -> Result<DeleteCanvasResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let user_id = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?
        .sub;

    canvas_service::delete_canvas(&app_state, params.canvas_id, user_id).await?;

    Ok(DeleteCanvasResponse { success: true })
}
