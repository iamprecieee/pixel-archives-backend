pub mod canvas;
pub mod pixel;
pub mod user;

pub use canvas::CanvasRepository;
pub use pixel::PixelRepository;
use rand::Rng;
pub use user::UserRepository;

pub fn generate_invite_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNOPQRSTUVWXYZ0123456789";
    (0..8)
        .map(|_| {
            let mut rng = rand::rng();
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
