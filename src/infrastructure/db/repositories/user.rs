use crate::{
    error::Result,
    infrastructure::db::{
        Database,
        entities::{User, user},
    },
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
        Ok(User::find_by_id(id).one(db_connection).await?)
    }

    pub async fn find_user_by_wallet<C: ConnectionTrait>(
        db_connection: &C,
        wallet: &str,
    ) -> Result<Option<user::Model>> {
        Ok(User::find()
            .filter(user::Column::WalletAddress.eq(wallet))
            .one(db_connection)
            .await?)
    }

    pub async fn existing_user_by_wallet_or_username<C: ConnectionTrait + Send>(
        db_connection: &C,
        wallet: &str,
        username: Option<&str>,
    ) -> Result<(bool, bool)> {
        let wallet_exists = User::find()
            .filter(user::Column::WalletAddress.eq(wallet))
            .one(db_connection)
            .await?;

        let username_exists = if let Some(username) = username {
            User::find()
                .filter(user::Column::Username.eq(username))
                .one(db_connection)
                .await?
        } else {
            None
        };

        Ok((wallet_exists.is_some(), username_exists.is_some()))
    }

    pub async fn create_user(
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

        let created_user = user.insert(&db_transaction).await?;

        db_transaction.commit().await?;

        Ok(created_user)
    }
}
