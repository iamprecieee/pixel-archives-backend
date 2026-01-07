use sea_orm::{
    DbErr, DeriveIden, DeriveMigrationName,
    prelude::Expr,
    sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Index, Table},
};
use sea_orm_migration::{MigrationTrait, SchemaManager, async_trait::async_trait};

use super::m20260106_000001_create_users::Users;

#[derive(DeriveIden)]
pub enum Canvases {
    Table,
    Id,
    OwnerId,
    Name,
    InviteCode,
    State,
    CanvasPda,
    MintAddress,
    TotalEscrowed,
    CreatedAt,
    PublishedAt,
    MintedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Canvases::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Canvases::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Canvases::OwnerId).uuid().not_null())
                    .col(ColumnDef::new(Canvases::Name).string_len(100).not_null())
                    .col(
                        ColumnDef::new(Canvases::InviteCode)
                            .string_len(12)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Canvases::State)
                            .string_len(20)
                            .not_null()
                            .default("draft"),
                    )
                    .col(ColumnDef::new(Canvases::CanvasPda).string_len(44))
                    .col(ColumnDef::new(Canvases::MintAddress).string_len(44))
                    .col(
                        ColumnDef::new(Canvases::TotalEscrowed)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Canvases::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Canvases::PublishedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Canvases::MintedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_canvases_owner")
                            .from(Canvases::Table, Canvases::OwnerId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_canvases_owner_id")
                    .table(Canvases::Table)
                    .col(Canvases::OwnerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_canvases_state")
                    .table(Canvases::Table)
                    .col(Canvases::State)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Canvases::Table).to_owned())
            .await
    }
}
