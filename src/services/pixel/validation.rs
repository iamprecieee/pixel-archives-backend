use crate::{
    config::CanvasConfig,
    error::{AppError, Result},
};

pub fn validate_pixel_coordinates(config: &CanvasConfig, x: i16, y: i16) -> Result<()> {
    if x < 0 || x >= config.width as i16 || y < 0 || y >= config.height as i16 {
        return Err(AppError::InvalidParams("Coordinates out of bounds".into()));
    }
    Ok(())
}

pub fn validate_pixel_color(config: &CanvasConfig, color: i16) -> Result<()> {
    if color < 0 || color >= config.color_count as i16 {
        return Err(AppError::InvalidParams("Invalid color".into()));
    }
    Ok(())
}

pub fn validate_min_bid(config: &CanvasConfig, bid_lamports: i64) -> Result<()> {
    if (bid_lamports as u64) < config.min_bid_lamports {
        return Err(AppError::BidTooLow {
            min_lamports: config.min_bid_lamports,
        });
    }
    Ok(())
}

pub fn validate_outbid(current_price: i64, bid_lamports: i64) -> Result<()> {
    let min_required = current_price + 1;
    if bid_lamports < min_required {
        return Err(AppError::BidTooLow {
            min_lamports: min_required as u64,
        });
    }
    Ok(())
}
