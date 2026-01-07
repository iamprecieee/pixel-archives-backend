use sea_orm::{
    ConnectionTrait, DbErr, DeriveIden, DeriveMigrationName,
    prelude::Expr,
    sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Index, Table},
};
use sea_orm_migration::{MigrationTrait, SchemaManager, async_trait::async_trait};

use super::m20260106_000001_create_users::Users;
use super::m20260106_000002_create_canvases::Canvases;

#[derive(DeriveIden)]
pub enum Pixels {
    Table,
    CanvasId,
    X,
    Y,
    Color,
    OwnerId,
    PriceLamports,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Pixels::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Pixels::CanvasId).uuid().not_null())
                    .col(ColumnDef::new(Pixels::X).small_integer().not_null())
                    .col(ColumnDef::new(Pixels::Y).small_integer().not_null())
                    .col(
                        ColumnDef::new(Pixels::Color)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Pixels::OwnerId).uuid())
                    .col(
                        ColumnDef::new(Pixels::PriceLamports)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Pixels::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(Pixels::CanvasId)
                            .col(Pixels::X)
                            .col(Pixels::Y),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_pixels_canvas")
                            .from(Pixels::Table, Pixels::CanvasId)
                            .to(Canvases::Table, Canvases::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_pixels_owner")
                            .from(Pixels::Table, Pixels::OwnerId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_pixels_canvas_id")
                    .table(Pixels::Table)
                    .col(Pixels::CanvasId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_pixels_owner_id")
                    .table(Pixels::Table)
                    .col(Pixels::OwnerId)
                    .to_owned(),
            )
            .await?;

        // Add check constraints
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE pixels
                ADD CONSTRAINT chk_pixels_x CHECK (x >= 0 AND x < 32),
                ADD CONSTRAINT chk_pixels_y CHECK (y >= 0 AND y < 32),
                ADD CONSTRAINT chk_pixels_color CHECK (color >= 0 AND color < 64)
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Pixels::Table).to_owned())
            .await
    }
}
