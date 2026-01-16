use std::net::SocketAddr;

use axum::{
    Router,
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::{StatusCode, header},
    response::Response,
    routing::post,
};
use serde_json::Value;

use crate::{
    AppState,
    api::{
        dispatcher::dispatch_method,
        methods::extract_cookie,
        types::{JsonRpcRequest, JsonRpcResponse},
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

async fn rpc_handler(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
) -> Response {
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

    let mut client_key = String::new();

    // Inject tokens into params based on method
    if let Value::Object(map) = &mut params {
        if let Some(token) = &access_token {
            map.insert("access_token".to_string(), Value::String(token.clone()));

            client_key =
                if let Ok(claims) = state.jwt_service.validate_token(token, TokenType::Access) {
                    format!("user:{}", claims.sub)
                } else {
                    format!("ip:{}", addr.ip())
                };
        }
        if (method == "auth.refresh" || method == "auth.logout")
            && let Some(token) = &refresh_token
        {
            map.insert("refresh_token".to_string(), Value::String(token.clone()));
        }
    }

    let result = dispatch_method(&method, params, state.clone(), &client_key).await;

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
