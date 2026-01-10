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

    pub fn canvas_lock(canvas_id: &Uuid) -> String {
        format!("lock:canvas:{canvas_id}")
    }

    pub fn cooldown(user_id: &Uuid) -> String {
        format!("cooldown:{user_id}")
    }

    pub fn pixel_lock(canvas_id: &Uuid, x: u8, y: u8) -> String {
        format!("lock:pixel:{canvas_id}:{x}:{y}")
    }
}
