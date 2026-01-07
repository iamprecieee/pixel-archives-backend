use crate::services::auth::TokenType;
use axum::http::{HeaderName, HeaderValue};

pub fn create_cookie(
    cookie_type: TokenType,
    token: &str,
    max_age_secs: i64,
    secure: bool,
) -> (HeaderName, HeaderValue) {
    let secure_flag = if secure { "; Secure" } else { "" };
    let cookie_value = format!(
        "{}={}; Path=/; Max-Age={}; HttpOnly; SameSite=Lax{}",
        cookie_type.name(),
        token,
        max_age_secs,
        secure_flag
    );

    (
        HeaderName::from_static("set-cookie"),
        HeaderValue::from_str(&cookie_value).expect("Invalid cookie value"),
    )
}

pub fn clear_cookie(cookie_type: TokenType, secure: bool) -> (HeaderName, HeaderValue) {
    let secure_flag = if secure { "; Secure" } else { "" };
    let cookie_value = format!(
        "{}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax{}",
        cookie_type.name(),
        secure_flag
    );

    (
        HeaderName::from_static("set-cookie"),
        HeaderValue::from_str(&cookie_value).expect("Invalid cookie value"),
    )
}
