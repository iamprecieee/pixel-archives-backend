use sea_orm_migration::{MigrationTrait, MigratorTrait, async_trait::async_trait};

mod m20260106_000001_create_users;
mod m20260106_000002_create_canvases;
mod m20260106_000003_create_pixels;
mod m20260106_000004_create_collaborators;

pub struct Migrator;

#[async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260106_000001_create_users::Migration),
            Box::new(m20260106_000002_create_canvases::Migration),
            Box::new(m20260106_000003_create_pixels::Migration),
            Box::new(m20260106_000004_create_collaborators::Migration),
        ]
    }
}
