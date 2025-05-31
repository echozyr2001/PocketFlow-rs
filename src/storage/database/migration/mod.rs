pub use sea_orm_migration::prelude::*;

mod m20250531_000001_create_key_value_store;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20250531_000001_create_key_value_store::Migration)]
    }
}
