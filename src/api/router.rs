use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header},
    response::Response,
    routing::post,
};
use serde_json::Value;

use crate::{
    AppState,
    api::{
        methods::{self, extract_cookie},
        types::{
            AuthOperation, AuthParams, CancelPixelBidParams, CancelPublishCanvasParams,
            ConfirmPixelBidParams, ConfirmPublishCanvasParams, CreateCanvasParams,
            DeleteCanvasParams, GetCanvasParams, JoinCanvasParams, JsonRpcRequest, JsonRpcResponse,
            ListCanvasParams, PaintPixelParams, PlacePixelBidParams, PublishCanvasParams,
            SessionParams,
        },
    },
    error::{AppError, JsonRpcErrorResponse},
    services::auth::{
        TokenType,
        cookie::{clear_cookie, create_cookie},
    },
};

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(rpc_handler))
}

async fn rpc_handler(State(state): State<AppState>, request: Request<Body>) -> Response {
    let (parts, body) = request.into_parts();
    let headers = parts.headers;

    let access_token = extract_cookie(&headers, "access_token");
    let refresh_token = extract_cookie(&headers, "refresh_token");

    const MAX_BODY_SIZE: usize = 1024 * 1024;

    let body_bytes = match axum::body::to_bytes(body, MAX_BODY_SIZE).await {
        Ok(b) => b,
        Err(e) => {
            let msg = if e.to_string().contains("length limit") {
                "Request body too large (max 1MB)"
            } else {
                "Parse error"
            };
            return build_json_response(
                JsonRpcErrorResponse::from_error(&AppError::InvalidParams(msg.into()), None),
                vec![],
            );
        }
    };

    let request: JsonRpcRequest = match serde_json::from_slice(&body_bytes) {
        Ok(req) => req,
        Err(_) => {
            return build_json_response(
                JsonRpcErrorResponse::from_error(
                    &AppError::InvalidParams("Parse error".into()),
                    None,
                ),
                vec![],
            );
        }
    };

    if request.jsonrpc != "2.0" {
        return build_json_response(
            JsonRpcErrorResponse::from_error(
                &AppError::InvalidParams("Invalid JSON-RPC version".into()),
                request.id,
            ),
            vec![],
        );
    }

    let mut params = request.params;
    let method = request.method.clone();

    // Inject tokens into params based on method
    if let Value::Object(map) = &mut params {
        if let Some(t) = &access_token {
            map.insert("access_token".to_string(), Value::String(t.clone()));
        }
        if (method == "auth.refresh" || method == "auth.logout")
            && let Some(t) = &refresh_token
        {
            map.insert("refresh_token".to_string(), Value::String(t.clone()));
        }
    }

    let result = dispatch_method(&method, params, state.clone()).await;

    let secure = state
        .config
        .server
        .server_public_url
        .starts_with("https://");

    let cookies: Vec<_> = match method.as_str() {
        "auth.login" | "auth.register" | "auth.refresh" => {
            if let Ok(ref value) = result {
                let mut cookies = vec![];
                if let Some(t) = value.get("access_token").and_then(|t| t.as_str()) {
                    cookies.push(create_cookie(
                        TokenType::Access,
                        t,
                        state.config.jwt.access_token_ttl.as_secs() as i64,
                        secure,
                    ));
                }
                if let Some(t) = value.get("refresh_token").and_then(|t| t.as_str()) {
                    cookies.push(create_cookie(
                        TokenType::Refresh,
                        t,
                        state.config.jwt.refresh_token_ttl.as_secs() as i64,
                        secure,
                    ));
                }
                cookies
            } else {
                vec![]
            }
        }
        "auth.logout" => vec![
            clear_cookie(TokenType::Access, secure),
            clear_cookie(TokenType::Refresh, secure),
        ],
        _ => vec![],
    };

    match result {
        Ok(value) => {
            let response_value = match method.as_str() {
                "auth.login" | "auth.register" | "auth.refresh" => {
                    let user_value = value.get("user").cloned().unwrap_or(Value::Null);
                    serde_json::to_value(JsonRpcResponse::new(
                        serde_json::json!({ "user": user_value }),
                        request.id,
                    ))
                    .expect("JsonRpcResponse serialization failed")
                }
                _ => serde_json::to_value(JsonRpcResponse::new(value, request.id))
                    .expect("JsonRpcResponse serialization failed"),
            };
            build_json_response(response_value, cookies)
        }
        Err(err) => build_json_response(JsonRpcErrorResponse::from_error(&err, request.id), vec![]),
    }
}

fn build_json_response(
    value: Value,
    cookies: Vec<(header::HeaderName, header::HeaderValue)>,
) -> Response {
    let body = serde_json::to_string(&value).expect("JSON serialization failed");
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json");

    for (name, val) in cookies {
        response = response.header(name, val);
    }

    response.body(Body::from(body)).unwrap()
}

async fn dispatch_method(method: &str, params: Value, state: AppState) -> Result<Value, AppError> {
    if method.starts_with("auth.") {
        return dispatch_auth(method, params, state).await;
    }
    if method.starts_with("canvas.") {
        return dispatch_canvas(method, params, state).await;
    }
    if method.starts_with("pixel.") {
        return dispatch_pixel(method, params, state).await;
    }
    Err(AppError::MethodNotFound(method.to_string()))
}

async fn dispatch_auth(method: &str, params: Value, state: AppState) -> Result<Value, AppError> {
    match method {
        "auth.register" => {
            let mut auth_params: AuthParams = serde_json::from_value(params)
                .map_err(|e| AppError::InvalidParams(e.to_string()))?;
            auth_params.state = Some(state);
            auth_params.operation = Some(AuthOperation::Register);

            let result = methods::auth::authenticate_user(auth_params).await?;
            serde_json::to_value(result).map_err(AppError::from)
        }
        "auth.login" => {
            let mut auth_params: AuthParams = serde_json::from_value(params)
                .map_err(|e| AppError::InvalidParams(e.to_string()))?;
            auth_params.state = Some(state);
            auth_params.operation = Some(AuthOperation::Login);

            let result = methods::auth::authenticate_user(auth_params).await?;
            serde_json::to_value(result).map_err(AppError::from)
        }
        "auth.logout" => {
            let mut session_params: SessionParams = serde_json::from_value(params)
                .map_err(|e| AppError::InvalidParams(e.to_string()))?;
            session_params.state = Some(state);

            let result = methods::auth::logout_user(session_params).await?;
            serde_json::to_value(result).map_err(AppError::from)
        }
        "auth.refresh" => {
            let mut session_params: SessionParams = serde_json::from_value(params)
                .map_err(|e| AppError::InvalidParams(e.to_string()))?;
            session_params.state = Some(state);

            let result = methods::auth::refresh_user_token(session_params).await?;
            serde_json::to_value(result).map_err(AppError::from)
        }
        _ => Err(AppError::MethodNotFound(method.to_string())),
    }
}

async fn dispatch_canvas(method: &str, params: Value, state: AppState) -> Result<Value, AppError> {
    match method {
        "canvas.create" => {
            let mut create_params: CreateCanvasParams = serde_json::from_value(params)
                .map_err(|e| AppError::InvalidParams(e.to_string()))?;
            create_params.state = Some(state);

            let result = methods::canvas::create_canvas(create_params).await?;
            serde_json::to_value(result).map_err(AppError::from)
        }
        "canvas.list" => {
            let mut list_params: ListCanvasParams = serde_json::from_value(params)
                .map_err(|e| AppError::InvalidParams(e.to_string()))?;
            list_params.state = Some(state);

            let result = methods::canvas::list_canvas(list_params).await?;
            serde_json::to_value(result).map_err(AppError::from)
        }
        "canvas.get" => {
            let mut get_params: GetCanvasParams = serde_json::from_value(params)
                .map_err(|e| AppError::InvalidParams(e.to_string()))?;
            get_params.state = Some(state);

            let result = methods::canvas::get_canvas(get_params).await?;
            serde_json::to_value(result).map_err(AppError::from)
        }
        "canvas.join" => {
            let mut join_params: JoinCanvasParams = serde_json::from_value(params)
                .map_err(|e| AppError::InvalidParams(e.to_string()))?;
            join_params.state = Some(state);

            let result = methods::canvas::join_canvas(join_params).await?;
            serde_json::to_value(result).map_err(AppError::from)
        }
        "canvas.publish" => {
            let mut publish_params: PublishCanvasParams = serde_json::from_value(params)
                .map_err(|e| AppError::InvalidParams(e.to_string()))?;
            publish_params.state = Some(state);

            let result = methods::canvas::publish_canvas(publish_params).await?;
            serde_json::to_value(result).map_err(AppError::from)
        }
        "canvas.confirmPublish" => {
            let mut confirm_params: ConfirmPublishCanvasParams = serde_json::from_value(params)
                .map_err(|e| AppError::InvalidParams(e.to_string()))?;
            confirm_params.state = Some(state);

            let result = methods::canvas::confirm_publish_canvas(confirm_params).await?;
            serde_json::to_value(result).map_err(AppError::from)
        }
        "canvas.cancelPublish" => {
            let mut cancel_params: CancelPublishCanvasParams = serde_json::from_value(params)
                .map_err(|e| AppError::InvalidParams(e.to_string()))?;
            cancel_params.state = Some(state);

            let result = methods::canvas::cancel_publish_canvas(cancel_params).await?;
            serde_json::to_value(result).map_err(AppError::from)
        }
        "canvas.delete" => {
            let mut delete_params: DeleteCanvasParams = serde_json::from_value(params)
                .map_err(|e| AppError::InvalidParams(e.to_string()))?;
            delete_params.state = Some(state);

            let result = methods::canvas::delete_canvas(delete_params).await?;
            serde_json::to_value(result).map_err(AppError::from)
        }
        _ => Err(AppError::MethodNotFound(method.to_string())),
    }
}

async fn dispatch_pixel(method: &str, params: Value, state: AppState) -> Result<Value, AppError> {
    match method {
        "pixel.place" => {
            let mut place_params: PlacePixelBidParams = serde_json::from_value(params)
                .map_err(|e| AppError::InvalidParams(e.to_string()))?;
            place_params.state = Some(state);

            let result = methods::pixel::place_pixel_bid(place_params).await?;
            serde_json::to_value(result).map_err(AppError::from)
        }
        "pixel.confirm" => {
            let mut confirm_params: ConfirmPixelBidParams = serde_json::from_value(params)
                .map_err(|e| AppError::InvalidParams(e.to_string()))?;
            confirm_params.state = Some(state);

            let result = methods::pixel::confirm_pixel_bid(confirm_params).await?;
            serde_json::to_value(result).map_err(AppError::from)
        }
        "pixel.paint" => {
            let mut paint_params: PaintPixelParams = serde_json::from_value(params)
                .map_err(|e| AppError::InvalidParams(e.to_string()))?;
            paint_params.state = Some(state);

            let result = methods::pixel::paint_pixel(paint_params).await?;
            serde_json::to_value(result).map_err(AppError::from)
        }
        "pixel.cancel" => {
            let mut cancel_params: CancelPixelBidParams = serde_json::from_value(params)
                .map_err(|e| AppError::InvalidParams(e.to_string()))?;
            cancel_params.state = Some(state);

            let result = methods::pixel::cancel_pixel_bid(cancel_params).await?;
            serde_json::to_value(result).map_err(AppError::from)
        }
        _ => Err(AppError::MethodNotFound(method.to_string())),
    }
}
