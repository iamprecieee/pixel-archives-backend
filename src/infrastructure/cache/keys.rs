use uuid::Uuid;

pub struct CacheKey;

impl CacheKey {
    pub fn canvas_pixels(id: &Uuid) -> String {
        format!("canvas:{id}:pixels")
    }

    pub fn user_session(user_id: &Uuid) -> String {
        format!("user:session:{user_id}")
    }

    pub fn token_blacklist(jti: &str) -> String {
        format!("token:blacklist:{jti}")
    }
}
