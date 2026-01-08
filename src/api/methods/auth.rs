use crate::{
    api::{
        methods::{calculate_remaining_ttl, validate_wallet_address},
        types::{
            AuthOperation, AuthParams, AuthResponse, LogoutResponse, SessionParams, UserResponse,
        },
    },
    error::{AppError, Result},
    infrastructure::{cache::keys::CacheKey, db::repositories::UserRepository},
    services::auth::{TokenType, check_and_consume_nonce, parse_auth_message, verify_signature},
};

pub async fn authenticate_user(params: AuthParams) -> Result<AuthResponse> {
    validate_wallet_address(&params.wallet)?;

    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let auth_msg = parse_auth_message(&params.message)?;
    if auth_msg.wallet != params.wallet {
        return Err(AppError::InvalidParams("Wallet mismatch in message".into()));
    }

    verify_signature(&params.wallet, &params.message, &params.signature)?;

    check_and_consume_nonce(&app_state.cache, &params.wallet, &auth_msg.nonce).await?;

    let operation = params.operation.ok_or(AppError::InternalServerError(
        "Failed to get method operation".to_string(),
    ))?;

    let user = match operation {
        AuthOperation::Login => {
            UserRepository::find_user_by_wallet(app_state.db.get_connection(), &params.wallet)
                .await?
                .ok_or(AppError::UserNotFound)?
        }
        AuthOperation::Register => {
            let (wallet_exists, username_exists) =
                UserRepository::existing_user_by_wallet_or_username(
                    app_state.db.get_connection(),
                    &params.wallet,
                    params.username.as_deref(),
                )
                .await?;

            if wallet_exists {
                return Err(AppError::UserExists);
            }
            if username_exists {
                return Err(AppError::UsernameExists);
            }

            UserRepository::create_user(&app_state.db, &params.wallet, params.username).await?
        }
    };

    let access_token = app_state
        .jwt_service
        .create_access_token(user.id, &user.wallet_address)?;
    let refresh_token = app_state
        .jwt_service
        .create_refresh_token(user.id, &user.wallet_address)?;

    let user_response = UserResponse {
        id: user.id.to_string(),
        wallet_address: user.wallet_address,
        username: user.username,
    };

    let session_key = CacheKey::user_session(&user.id);
    let session_ttl = app_state.config.jwt.refresh_token_ttl;

    if let Err(e) = app_state
        .cache
        .redis
        .set(&session_key, &user_response, session_ttl)
        .await
    {
        tracing::warn!(error = ?e, "Failed to cache user session");
    }

    Ok(AuthResponse {
        access_token,
        refresh_token,
        user: user_response,
    })
}

pub async fn logout_user(params: SessionParams) -> Result<LogoutResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    let access_token_claims = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)?;
    let access_token_blacklist_key = CacheKey::token_blacklist(&access_token_claims.jti);
    let access_token_remaining_ttl = calculate_remaining_ttl(access_token_claims.exp);

    if let Some(ttl) = access_token_remaining_ttl {
        app_state
            .cache
            .redis
            .set(&access_token_blacklist_key, &true, ttl)
            .await?;
    }

    if let Some(ref refresh_token) = params.refresh_token
        && let Ok(refresh_token_claims) = app_state
            .jwt_service
            .validate_token(refresh_token, TokenType::Refresh)
    {
        let refresh_token_blacklist_key = CacheKey::token_blacklist(&refresh_token_claims.jti);
        let refresh_token_remaining_ttl = calculate_remaining_ttl(refresh_token_claims.exp);

        if let Some(ttl) = refresh_token_remaining_ttl {
            app_state
                .cache
                .redis
                .set(&refresh_token_blacklist_key, &true, ttl)
                .await?;
        }
    }

    let session_key = CacheKey::user_session(&access_token_claims.sub);
    if let Err(e) = app_state.cache.redis.delete(&session_key).await {
        tracing::warn!(error = ?e, "Failed to delete user session during logout");
    }

    Ok(LogoutResponse { success: true })
}

pub async fn refresh_user_token(params: SessionParams) -> Result<AuthResponse> {
    let app_state = params.state.ok_or(AppError::InternalServerError(
        "Failed to get app state".to_string(),
    ))?;

    if let Ok(access_token_claims) = app_state
        .jwt_service
        .validate_token(&params.access_token, TokenType::Access)
    {
        let access_token_blacklist_key = CacheKey::token_blacklist(&access_token_claims.jti);
        let access_token_remaining_ttl = calculate_remaining_ttl(access_token_claims.exp);

        if let Some(ttl) = access_token_remaining_ttl {
            let _ = app_state
                .cache
                .redis
                .set(&access_token_blacklist_key, &true, ttl)
                .await;
        }
    }

    let refresh_token = params
        .refresh_token
        .ok_or(AppError::InvalidParams("refresh_token is required".into()))?;

    let refresh_token_claims = app_state
        .jwt_service
        .validate_token(&refresh_token, TokenType::Refresh)?;

    let refresh_token_blacklist_key = CacheKey::token_blacklist(&refresh_token_claims.jti);
    if let Some(true) = app_state
        .cache
        .redis
        .get::<bool>(&refresh_token_blacklist_key)
        .await?
    {
        return Err(AppError::Unauthorized);
    }

    let refresh_token_remaining_ttl = calculate_remaining_ttl(refresh_token_claims.exp);
    if let Some(ttl) = refresh_token_remaining_ttl {
        let _ = app_state
            .cache
            .redis
            .set(&refresh_token_blacklist_key, &true, ttl)
            .await;
    }

    let session_key = CacheKey::user_session(&refresh_token_claims.sub);
    let user_response: UserResponse = match app_state.cache.redis.get(&session_key).await? {
        Some(cached) => cached,
        None => {
            let user = UserRepository::find_user_by_id(
                app_state.db.get_connection(),
                refresh_token_claims.sub,
            )
            .await?
            .ok_or(AppError::UserNotFound)?;
            UserResponse {
                id: user.id.to_string(),
                wallet_address: user.wallet_address,
                username: user.username,
            }
        }
    };

    let access_token = app_state
        .jwt_service
        .create_access_token(refresh_token_claims.sub, &refresh_token_claims.wallet)?;

    let refresh_token = app_state
        .jwt_service
        .create_refresh_token(refresh_token_claims.sub, &refresh_token_claims.wallet)?;

    let session_ttl = app_state.config.jwt.refresh_token_ttl;
    let _ = app_state
        .cache
        .redis
        .set(&session_key, &user_response, session_ttl)
        .await;

    Ok(AuthResponse {
        access_token,
        refresh_token,
        user: user_response,
    })
}
