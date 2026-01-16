use axum::extract::Request;
use tracing::{Span, info_span};

use crate::utils::security::mask_uri_token;

pub fn make_log_span(request: &Request) -> Span {
    let masked_uri = mask_uri_token(&request.uri().to_string());

    info_span!(
        "request",
        method = ?request.method(),
        uri = ?masked_uri,
        version = ?request.version(),
    )
}
