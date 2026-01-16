pub fn mask_uri_token(uri: &str) -> String {
    if uri.contains("token=") {
        let mut masked = uri.to_string();
        if let Some(start) = masked.find("token=") {
            let rest = &masked[start + 6..];

            if let Some(end) = rest.find('&') {
                masked.replace_range(start + 6..start + end + 6, "***");
            } else {
                masked.replace_range(start + 6.., "***");
            }
        }
        masked
    } else {
        uri.to_string()
    }
}