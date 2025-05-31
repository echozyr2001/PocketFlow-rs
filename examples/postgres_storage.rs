use pocketflow_rs::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), sea_orm::DbErr> {
    println!("🐘 PocketFlow-RS PostgreSQL Example");
    println!("🔧 This example demonstrates PostgreSQL-specific features");
    
    // Check for PostgreSQL connection string
    let postgres_url = std::env::var("DATABASE_POSTGRES_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/pocketflow".to_string());
    
    println!("📡 Connecting to PostgreSQL: {}", 
        postgres_url.split('@').last().unwrap_or("localhost"));
    
    match DatabaseStorage::new(&postgres_url).await {
        Ok(postgres_storage) => {
            println!("✅ Connected to PostgreSQL database");
            
            // Run migrations
            postgres_storage.migrate().await?;
            println!("🔄 Migrations completed");
            
            let store = AsyncSharedStore::new(postgres_storage);
            
            // PostgreSQL JSON features demonstration
            let postgres_features = json!({
                "jsonb_support": true,
                "advanced_indexing": ["gin", "gist", "btree"],
                "full_text_search": true,
                "array_types": true,
                "custom_types": true,
                "parallel_queries": true,
                "partitioning": true,
                "replication": ["streaming", "logical"],
                "acid_compliance": "full"
            });
            
            store.set("postgres_features".to_string(), postgres_features).await?;
            println!("✅ Stored PostgreSQL feature set");
            
            // Complex data with PostgreSQL-specific advantages
            let complex_query_demo = json!({
                "query_optimization": {
                    "cost_based_optimizer": true,
                    "parallel_execution": true,
                    "adaptive_plans": true
                },
                "data_types": [
                    "uuid", "jsonb", "arrays", "hstore", "ltree",
                    "geometry", "timestamp_with_timezone", "interval"
                ],
                "extensions": [
                    "postgis", "pg_crypto", "pg_stat_statements", 
                    "pg_trgm", "fuzzystrmatch", "unaccent"
                ],
                "performance": {
                    "concurrent_users": "hundreds to thousands",
                    "data_size": "terabytes",
                    "transaction_rate": "very_high"
                }
            });
            
            store.set("postgres_advantages".to_string(), complex_query_demo).await?;
            println!("✅ Demonstrated PostgreSQL advanced capabilities");
            
            // Performance test
            let start_time = std::time::Instant::now();
            for i in 0..50 {
                store.set(format!("postgres_perf_{}", i), json!({
                    "index": i,
                    "timestamp": chrono::Utc::now(),
                    "data": format!("PostgreSQL performance test {}", i)
                })).await?;
            }
            let duration = start_time.elapsed();
            println!("⚡ PostgreSQL: 50 inserts in {:?}", duration);
            
            // Retrieve and display
            if let Some(features) = store.get("postgres_features").await? {
                println!("🔍 PostgreSQL Features Retrieved:");
                println!("  - JSONB Support: {}", features["jsonb_support"]);
                println!("  - Full Text Search: {}", features["full_text_search"]);
                println!("  - Array Types: {}", features["array_types"]);
            }
            
            // Final stats
            let total_keys = store.len().await?;
            println!("📊 Total records in PostgreSQL: {}", total_keys);
            
            println!("🎉 PostgreSQL example completed successfully!");
        }
        Err(e) => {
            println!("❌ Failed to connect to PostgreSQL: {}", e);
            println!("💡 Make sure PostgreSQL is running and accessible");
            println!("🔧 Set DATABASE_POSTGRES_URL environment variable:");
            println!("   export DATABASE_POSTGRES_URL=\"postgres://user:password@localhost:5432/database\"");
            return Err(e);
        }
    }
    
    Ok(())
}