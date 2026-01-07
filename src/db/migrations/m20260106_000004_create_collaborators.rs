use sea_orm::{
    DbErr, DeriveIden, DeriveMigrationName,
    prelude::Expr,
    sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Index, Table},
};
use sea_orm_migration::{MigrationTrait, SchemaManager, async_trait::async_trait};

use super::m20260106_000001_create_users::Users;
use super::m20260106_000002_create_canvases::Canvases;

#[derive(DeriveIden)]
pub enum CanvasCollaborators {
    Table,
    CanvasId,
    UserId,
    JoinedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CanvasCollaborators::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CanvasCollaborators::CanvasId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CanvasCollaborators::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CanvasCollaborators::JoinedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(CanvasCollaborators::CanvasId)
                            .col(CanvasCollaborators::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_collaborators_canvas")
                            .from(CanvasCollaborators::Table, CanvasCollaborators::CanvasId)
                            .to(Canvases::Table, Canvases::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_collaborators_user")
                            .from(CanvasCollaborators::Table, CanvasCollaborators::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_collaborators_canvas_id")
                    .table(CanvasCollaborators::Table)
                    .col(CanvasCollaborators::CanvasId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_collaborators_user_id")
                    .table(CanvasCollaborators::Table)
                    .col(CanvasCollaborators::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CanvasCollaborators::Table).to_owned())
            .await
    }
}
