#[cfg(feature = "storage-database")]
use crate::storage::AsyncStorageBackend;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Database, DatabaseConnection, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter,
};
use sea_orm_migration::MigratorTrait;
use serde_json::Value;

pub mod entities;
pub mod migration;

use entities::key_value_store::{ActiveModel, Column, Entity as KeyValueStore};
pub use migration::Migrator;

#[derive(Debug, Clone)]
pub struct DatabaseStorage {
    connection: DatabaseConnection,
    prefix: String,
}

impl DatabaseStorage {
    /// Create a new database storage with default prefix
    pub async fn new(database_url: &str) -> Result<Self, DbErr> {
        Self::new_with_prefix(database_url, "pocketflow").await
    }

    /// Create a new database storage with custom prefix
    pub async fn new_with_prefix(database_url: &str, prefix: &str) -> Result<Self, DbErr> {
        let connection = Database::connect(database_url).await?;
        Ok(Self {
            connection,
            prefix: prefix.to_string(),
        })
    }

    /// Run migrations to set up the database schema
    pub async fn migrate(&self) -> Result<(), DbErr> {
        Migrator::up(&self.connection, None).await
    }

    /// Get the database connection
    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    /// Get the key prefix
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Create the full key with prefix
    fn full_key(&self, key: &str) -> String {
        format!("{}:{}", self.prefix, key)
    }

    /// Remove prefix from full key
    fn strip_prefix<'a>(&self, full_key: &'a str) -> Option<&'a str> {
        let prefix_with_colon = format!("{}:", self.prefix);
        full_key.strip_prefix(&prefix_with_colon)
    }
}

#[cfg(feature = "storage-database")]
#[async_trait::async_trait]
impl AsyncStorageBackend for DatabaseStorage {
    type Error = DbErr;

    async fn set(&mut self, key: String, value: Value) -> Result<(), Self::Error> {
        let full_key = self.full_key(&key);
        let value_str = serde_json::to_string(&value)
            .map_err(|e| DbErr::Custom(format!("Failed to serialize value: {}", e)))?;

        // Try to find existing record
        if let Some(existing) = KeyValueStore::find_by_id(&full_key)
            .one(&self.connection)
            .await?
        {
            // Update existing record
            let mut active_model: ActiveModel = existing.into();
            active_model.value = Set(value_str);
            active_model.updated_at = Set(chrono::Utc::now());
            active_model.update(&self.connection).await?;
        } else {
            // Insert new record
            let new_model = ActiveModel {
                key: Set(full_key),
                value: Set(value_str),
                prefix: Set(Some(self.prefix.clone())),
                created_at: Set(chrono::Utc::now()),
                updated_at: Set(chrono::Utc::now()),
            };
            new_model.insert(&self.connection).await?;
        }

        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Value>, Self::Error> {
        let full_key = self.full_key(key);

        let result = KeyValueStore::find_by_id(&full_key)
            .one(&self.connection)
            .await?;

        if let Some(model) = result {
            let value = serde_json::from_str(&model.value)
                .map_err(|e| DbErr::Custom(format!("Failed to deserialize value: {}", e)))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    async fn remove(&mut self, key: &str) -> Result<Option<Value>, Self::Error> {
        let full_key = self.full_key(key);

        // Get the value before deletion
        let existing_value = self.get(key).await?;

        // Delete the record
        KeyValueStore::delete_by_id(&full_key)
            .exec(&self.connection)
            .await?;

        Ok(existing_value)
    }

    async fn contains_key(&self, key: &str) -> Result<bool, Self::Error> {
        let full_key = self.full_key(key);

        let count = KeyValueStore::find_by_id(&full_key)
            .count(&self.connection)
            .await?;

        Ok(count > 0)
    }

    async fn keys(&self) -> Result<Vec<String>, Self::Error> {
        let prefix_filter = format!("{}:", self.prefix);

        let records = KeyValueStore::find()
            .filter(Column::Key.starts_with(&prefix_filter))
            .all(&self.connection)
            .await?;

        let keys = records
            .into_iter()
            .filter_map(|model| self.strip_prefix(&model.key).map(String::from))
            .collect();

        Ok(keys)
    }

    async fn clear(&mut self) -> Result<(), Self::Error> {
        let prefix_filter = format!("{}:", self.prefix);

        KeyValueStore::delete_many()
            .filter(Column::Key.starts_with(&prefix_filter))
            .exec(&self.connection)
            .await?;

        Ok(())
    }

    async fn len(&self) -> Result<usize, Self::Error> {
        let prefix_filter = format!("{}:", self.prefix);

        let count = KeyValueStore::find()
            .filter(Column::Key.starts_with(&prefix_filter))
            .count(&self.connection)
            .await? as usize;

        Ok(count)
    }

    async fn is_empty(&self) -> Result<bool, Self::Error> {
        let len = self.len().await?;
        Ok(len == 0)
    }
}
