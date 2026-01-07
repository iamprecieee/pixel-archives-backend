use uuid::Uuid;

pub struct CacheKey;

impl CacheKey {
    pub fn canvas_pixels(id: &Uuid) -> String {
        format!("canvas:{id}:pixels")
    }
}
