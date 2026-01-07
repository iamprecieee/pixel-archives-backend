use crate::{
    db::{
        Database,
        entities::{User, user},
    },
    error::{AppError, Result},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
};
use uuid::Uuid;

pub struct UserRepository;

impl UserRepository {
    pub async fn find_user_by_id<C: ConnectionTrait>(
        db_connection: &C,
        id: Uuid,
    ) -> Result<Option<user::Model>> {
        User::find_by_id(id)
            .one(db_connection)
            .await
            .map_err(AppError::DatabaseError)
    }

    pub async fn find_user_by_wallet<C: ConnectionTrait>(
        db_connection: &C,
        wallet: &str,
    ) -> Result<Option<user::Model>> {
        User::find()
            .filter(user::Column::WalletAddress.eq(wallet))
            .one(db_connection)
            .await
            .map_err(AppError::DatabaseError)
    }

    pub async fn create_user<C: ConnectionTrait>(
        db: &Database,
        wallet: &str,
        username: Option<String>,
    ) -> Result<user::Model> {
        let db_transaction = db.begin_transaction().await?;

        let now = Utc::now();
        let user = user::ActiveModel {
            id: Set(Uuid::new_v4()),
            wallet_address: Set(wallet.to_string()),
            username: Set(username),
            created_at: Set(now),
        };

        let created_user = user
            .insert(&db_transaction)
            .await
            .map_err(AppError::DatabaseError)?;

        db_transaction
            .commit()
            .await
            .map_err(AppError::DatabaseError)?;

        Ok(created_user)
    }
}
