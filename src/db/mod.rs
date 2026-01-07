pub mod entities;
pub mod migrations;
pub mod repositories;

use sea_orm::{ConnectOptions, DatabaseConnection, DatabaseTransaction, TransactionTrait};
use sea_orm_migration::MigratorTrait;

use crate::{config::DatabaseConfig, error::Result};

use migrations::Migrator;

#[derive(Debug, Clone)]
pub struct Database {
    connection: DatabaseConnection,
}

impl Database {
    pub async fn init_db(config: &DatabaseConfig) -> Result<Self> {
        let mut options = ConnectOptions::new(&config.url);

        options
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect_timeout(config.connect_timeout)
            .idle_timeout(config.idle_timeout)
            .sqlx_logging(false);

        let connection = sea_orm::Database::connect(options).await?;

        Ok(Self { connection })
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub async fn begin_transaction(&self) -> Result<DatabaseTransaction> {
        Ok(self.connection.begin().await?)
    }

    pub async fn run_migrations(&self) -> Result<()> {
        Ok(Migrator::up(&self.connection, None).await?)
    }
}
