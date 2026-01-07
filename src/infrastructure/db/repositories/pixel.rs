use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, prelude::Expr, sea_query::Alias,
};
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    infrastructure::db::{
        Database,
        entities::{Pixel, pixel},
    },
};

pub struct PixelRepository;

impl PixelRepository {
    pub async fn find_pixel<C: ConnectionTrait>(
        db_connection: &C,
        canvas_id: Uuid,
        x: i16,
        y: i16,
    ) -> Result<Option<pixel::Model>> {
        Pixel::find()
            .filter(pixel::Column::CanvasId.eq(canvas_id))
            .filter(pixel::Column::X.eq(x))
            .filter(pixel::Column::Y.eq(y))
            .one(db_connection)
            .await
            .map_err(AppError::DatabaseError)
    }

    // when we need to fetch all pixels of a canvas
    pub async fn find_pixels_by_canvas<C: ConnectionTrait>(
        db_connection: &C,
        canvas_id: Uuid,
    ) -> Result<Vec<pixel::Model>> {
        Pixel::find()
            .filter(pixel::Column::CanvasId.eq(canvas_id))
            .all(db_connection)
            .await
            .map_err(AppError::DatabaseError)
    }

    pub async fn upsert_pixel(
        db: &Database,
        canvas_id: Uuid,
        x: i16,
        y: i16,
        color: Option<i16>,
        owner_id: Option<Uuid>,
        price_lamports: Option<i64>,
    ) -> Result<pixel::Model> {
        let db_connection = db.get_connection();
        let db_transaction = db.begin_transaction().await?;
        let now = Utc::now();
        let existing_pixel = Self::find_pixel(db_connection, canvas_id, x, y).await?;

        if let Some(existing_pixel) = existing_pixel {
            let mut active: pixel::ActiveModel = existing_pixel.into();

            if let Some(color) = color {
                active.color = Set(color);
            }
            if let Some(owner_id) = owner_id {
                active.owner_id = Set(Some(owner_id));
            }
            if let Some(price_lamports) = price_lamports {
                active.price_lamports = Set(price_lamports);
            }
            active.updated_at = Set(now);

            let updated_pixel = active
                .update(&db_transaction)
                .await
                .map_err(AppError::DatabaseError)?;

            db_transaction
                .commit()
                .await
                .map_err(AppError::DatabaseError)?;

            Ok(updated_pixel)
        } else {
            // validating required fields for new insert
            let color = color
                .ok_or_else(|| AppError::InvalidParams("Color required for new pixel".into()))?;
            let price_lamports = price_lamports.unwrap_or(0);

            let pixel = pixel::ActiveModel {
                canvas_id: Set(canvas_id),
                x: Set(x),
                y: Set(y),
                color: Set(color),
                owner_id: Set(owner_id),
                price_lamports: Set(price_lamports),
                updated_at: Set(now),
            };

            let inserted_pixel = pixel
                .insert(&db_transaction)
                .await
                .map_err(AppError::DatabaseError)?;

            db_transaction
                .commit()
                .await
                .map_err(AppError::DatabaseError)?;

            Ok(inserted_pixel)
        }
    }

    pub async fn initialize_canvas_pixels(
        db: &Database,
        canvas_id: Uuid,
        width: u8,
        height: u8,
        initial_color: i16,
    ) -> Result<()> {
        let db_transaction = db.begin_transaction().await?;

        let now = Utc::now();
        let mut pixels = Vec::with_capacity((width as usize) * (height as usize));

        for y in 0..height {
            for x in 0..width {
                let pixel = pixel::ActiveModel {
                    canvas_id: Set(canvas_id),
                    x: Set(x as i16),
                    y: Set(y as i16),
                    color: Set(initial_color),
                    owner_id: Set(None),
                    price_lamports: Set(0),
                    updated_at: Set(now),
                };
                pixels.push(pixel);
            }
        }

        if !pixels.is_empty() {
            Pixel::insert_many(pixels)
                .exec(&db_transaction)
                .await
                .map_err(AppError::DatabaseError)?;
        }

        db_transaction
            .commit()
            .await
            .map_err(AppError::DatabaseError)?;

        Ok(())
    }

    pub async fn find_top_pixel_owners<C: ConnectionTrait>(
        db_connection: &C,
        canvas_id: Uuid,
        limit: usize,
    ) -> Result<Vec<(Uuid, i64)>> {
        let results = Pixel::find()
            .select_only()
            .column(pixel::Column::OwnerId)
            .column_as(
                Expr::col(pixel::Column::PriceLamports).sum(),
                "total_lamports",
            )
            .filter(pixel::Column::CanvasId.eq(canvas_id))
            .filter(pixel::Column::OwnerId.is_not_null())
            .group_by(pixel::Column::OwnerId)
            .order_by_desc(Expr::col(Alias::new("total_lamports")))
            .limit(limit as u64)
            .into_tuple::<(Uuid, i64)>()
            .all(db_connection)
            .await?;

        Ok(results)
    }
}
