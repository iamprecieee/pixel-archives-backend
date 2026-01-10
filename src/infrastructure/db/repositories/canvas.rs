use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, JoinType,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    infrastructure::db::{
        Database,
        entities::{Canvas, CanvasCollaborator, Pixel, canvas, canvas_collaborator, pixel},
        repositories::generate_invite_code,
    },
};

pub struct CanvasRepository;

impl CanvasRepository {
    pub async fn find_canvas_by_id<C: ConnectionTrait>(
        db_connection: &C,
        id: Uuid,
    ) -> Result<Option<canvas::Model>> {
        Canvas::find_by_id(id)
            .one(db_connection)
            .await
            .map_err(AppError::DatabaseError)
    }

    pub async fn find_canvas_by_invite_code<C: ConnectionTrait>(
        db_connection: &C,
        code: &str,
    ) -> Result<Option<canvas::Model>> {
        Canvas::find()
            .filter(canvas::Column::InviteCode.eq(code))
            .one(db_connection)
            .await
            .map_err(AppError::DatabaseError)
    }

    pub async fn list_canvases_by_owner<C: ConnectionTrait>(
        conn: &C,
        owner_id: Uuid,
    ) -> Result<Vec<canvas::Model>> {
        Canvas::find()
            .filter(canvas::Column::OwnerId.eq(owner_id))
            .order_by_desc(canvas::Column::CreatedAt)
            .all(conn)
            .await
            .map_err(AppError::DatabaseError)
    }

    pub async fn list_canvases_by_collaborator<C: ConnectionTrait>(
        db_connection: &C,
        user_id: Uuid,
    ) -> Result<Vec<canvas::Model>> {
        let canvases = Canvas::find()
            .join(
                JoinType::InnerJoin,
                canvas::Relation::CanvasCollaborator.def(),
            )
            .filter(canvas_collaborator::Column::UserId.eq(user_id))
            .filter(canvas::Column::OwnerId.ne(user_id))
            .order_by_desc(canvas::Column::CreatedAt)
            .all(db_connection)
            .await?;

        Ok(canvases)
    }

    pub async fn exists_by_name_and_owner<C: ConnectionTrait>(
        db_connection: &C,
        owner_id: Uuid,
        canvas_name: &str,
    ) -> Result<bool> {
        let count = Canvas::find()
            .filter(canvas::Column::OwnerId.eq(owner_id))
            .filter(canvas::Column::Name.eq(canvas_name))
            .count(db_connection)
            .await?;

        Ok(count > 0)
    }

    pub async fn create_canvas<C: ConnectionTrait>(
        db_connection: &C,
        owner_id: Uuid,
        name: &str,
    ) -> Result<canvas::Model> {
        let now = Utc::now();
        let invite_code = generate_invite_code();

        let canvas = canvas::ActiveModel {
            id: Set(Uuid::new_v4()),
            owner_id: Set(owner_id),
            name: Set(name.to_string()),
            invite_code: Set(invite_code),
            state: Set(canvas::CanvasState::Draft),
            canvas_pda: Set(None),
            mint_address: Set(None),
            total_escrowed: Set(0),
            created_at: Set(now),
            published_at: Set(None),
            minted_at: Set(None),
        };

        Ok(canvas.insert(db_connection).await?)
    }

    pub async fn add_canvas_collaborator<C: ConnectionTrait>(
        db_connection: &C,
        canvas_id: Uuid,
        user_id: Uuid,
    ) -> Result<()> {
        let now = Utc::now();
        let collaborator = canvas_collaborator::ActiveModel {
            canvas_id: Set(canvas_id),
            user_id: Set(user_id),
            joined_at: Set(now),
        };

        collaborator.insert(db_connection).await?;

        Ok(())
    }

    pub async fn is_canvas_collaborator<C: ConnectionTrait>(
        db_connection: &C,
        canvas_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool> {
        let count = CanvasCollaborator::find()
            .filter(canvas_collaborator::Column::CanvasId.eq(canvas_id))
            .filter(canvas_collaborator::Column::UserId.eq(user_id))
            .count(db_connection)
            .await?;
        Ok(count > 0)
    }

    pub async fn update_canvas_state<F>(
        db: &Database,
        id: Uuid,
        state: canvas::CanvasState,
        updater: F,
    ) -> Result<canvas::Model>
    where
        F: FnOnce(&mut canvas::ActiveModel),
    {
        let db_transaction = db.begin_transaction().await?;

        let canvas = Canvas::find_by_id(id)
            .lock_exclusive()
            .one(&db_transaction)
            .await?
            .ok_or(AppError::CanvasNotFound)?;

        if !canvas.state.is_valid_transition(&state) {
            db_transaction.rollback().await?;
            return Err(AppError::InvalidCanvasStateTransition);
        }

        let mut active: canvas::ActiveModel = canvas.into();
        active.state = Set(state);

        updater(&mut active);

        let updated_canvas = active.update(&db_transaction).await?;

        db_transaction.commit().await?;

        Ok(updated_canvas)
    }

    pub async fn update_canvas_escrow(
        db: &Database,
        id: Uuid,
        escrow_lamports: i64,
    ) -> Result<canvas::Model> {
        let db_transaction = db.begin_transaction().await?;

        let canvas = Canvas::find_by_id(id)
            .lock_exclusive()
            .one(&db_transaction)
            .await?
            .ok_or(AppError::CanvasNotFound)?;

        let mut active: canvas::ActiveModel = canvas.into();
        active.total_escrowed = Set(escrow_lamports);

        let updated_canvas = active.update(&db_transaction).await?;

        db_transaction.commit().await?;

        Ok(updated_canvas)
    }

    pub async fn delete_canvas(db: &Database, id: Uuid) -> Result<()> {
        let db_transaction = db.begin_transaction().await?;

        Pixel::delete_many()
            .filter(pixel::Column::CanvasId.eq(id))
            .exec(&db_transaction)
            .await?;

        CanvasCollaborator::delete_many()
            .filter(canvas_collaborator::Column::CanvasId.eq(id))
            .exec(&db_transaction)
            .await?;

        Canvas::delete_by_id(id).exec(&db_transaction).await?;

        db_transaction.commit().await?;

        Ok(())
    }
}
