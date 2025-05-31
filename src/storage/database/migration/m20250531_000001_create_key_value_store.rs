use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(KeyValueStore::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(KeyValueStore::Key)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(KeyValueStore::Value).text().not_null())
                    .col(ColumnDef::new(KeyValueStore::Prefix).string())
                    .col(
                        ColumnDef::new(KeyValueStore::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(KeyValueStore::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(KeyValueStore::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum KeyValueStore {
    Table,
    Key,
    Value,
    Prefix,
    CreatedAt,
    UpdatedAt,
}