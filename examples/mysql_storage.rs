use pocketflow_rs::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), sea_orm::DbErr> {
    println!("🐬 PocketFlow-RS MySQL Example");
    println!("🔧 This example demonstrates MySQL-specific features");

    // Check for MySQL connection string
    let mysql_url = std::env::var("DATABASE_MYSQL_URL")
        .unwrap_or_else(|_| "mysql://root:password@localhost:3306/pocketflow".to_string());

    println!(
        "📡 Connecting to MySQL: {}",
        mysql_url.split('@').last().unwrap_or("localhost")
    );

    match DatabaseStorage::new(&mysql_url).await {
        Ok(mysql_storage) => {
            println!("✅ Connected to MySQL database");

            // Run migrations
            mysql_storage.migrate().await?;
            println!("🔄 Migrations completed");

            let store = AsyncSharedStore::new(mysql_storage);

            // MySQL features demonstration
            let mysql_features = json!({
                "storage_engines": ["InnoDB", "MyISAM", "Memory", "Archive"],
                "replication": {
                    "master_slave": true,
                    "master_master": true,
                    "group_replication": true
                },
                "clustering": "MySQL Cluster (NDB)",
                "partitioning": true,
                "full_text_indexing": true,
                "json_support": "since_5.7",
                "performance": {
                    "query_cache": true,
                    "connection_pooling": true,
                    "thread_per_connection": true
                },
                "high_availability": ["MySQL Router", "ProxySQL", "HAProxy"]
            });

            store
                .set("mysql_features".to_string(), mysql_features)
                .await?;
            println!("✅ Stored MySQL feature set");

            // Web application optimization demo
            let webapp_optimization = json!({
                "read_replicas": {
                    "purpose": "Scale read operations",
                    "latency": "low",
                    "consistency": "eventual"
                },
                "connection_pooling": {
                    "max_connections": 1000,
                    "idle_timeout": "8_hours",
                    "pool_recycling": true
                },
                "caching": {
                    "query_cache": "deprecated_in_8.0",
                    "external_cache": ["Redis", "Memcached"],
                    "application_cache": true
                },
                "indexing_strategy": {
                    "primary_key": "clustered_index",
                    "secondary_indexes": "non_clustered",
                    "covering_indexes": "performance_optimization"
                }
            });

            store
                .set("mysql_webapp_optimization".to_string(), webapp_optimization)
                .await?;
            println!("✅ Demonstrated MySQL web application optimizations");

            // E-commerce specific features
            let ecommerce_features = json!({
                "transactions": {
                    "isolation_levels": ["READ_UNCOMMITTED", "READ_COMMITTED", "REPEATABLE_READ", "SERIALIZABLE"],
                    "deadlock_detection": true,
                    "row_level_locking": true
                },
                "data_consistency": {
                    "foreign_keys": true,
                    "check_constraints": "since_8.0.16",
                    "triggers": true
                },
                "scalability": {
                    "horizontal_scaling": "sharding",
                    "vertical_scaling": "hardware_upgrade",
                    "read_scaling": "read_replicas"
                }
            });

            store
                .set("mysql_ecommerce".to_string(), ecommerce_features)
                .await?;
            println!("✅ Stored e-commerce specific MySQL features");

            // Performance test with bulk data
            let start_time = std::time::Instant::now();
            for i in 0..100 {
                store
                    .set(
                        format!("mysql_perf_{}", i),
                        json!({
                            "order_id": i,
                            "customer_id": i % 10,
                            "product_ids": [i * 2, i * 2 + 1],
                            "total_amount": (i as f64) * 29.99,
                            "timestamp": chrono::Utc::now(),
                            "status": if i % 3 == 0 { "completed" } else { "processing" }
                        }),
                    )
                    .await?;
            }
            let duration = start_time.elapsed();
            println!("⚡ MySQL: 100 e-commerce records in {:?}", duration);

            // Query performance demonstration
            let query_start = std::time::Instant::now();
            let all_keys = store.keys().await?;
            let query_duration = query_start.elapsed();
            println!(
                "🔍 MySQL: Listed {} keys in {:?}",
                all_keys.len(),
                query_duration
            );

            // Retrieve and display features
            if let Some(features) = store.get("mysql_features").await? {
                println!("🔍 MySQL Features Retrieved:");
                if let Some(engines) = features["storage_engines"].as_array() {
                    println!("  - Storage Engines: {:?}", engines);
                }
                println!("  - JSON Support: {}", features["json_support"]);
                println!("  - Clustering: {}", features["clustering"]);
            }

            // Connection info
            println!("📊 MySQL Connection Statistics:");
            println!("  - Total records: {}", store.len().await?);
            println!("  - Storage engine: InnoDB (default)");
            println!("  - Character set: utf8mb4 (recommended)");

            println!("🎉 MySQL example completed successfully!");
        }
        Err(e) => {
            println!("❌ Failed to connect to MySQL: {}", e);
            println!("💡 Make sure MySQL is running and accessible");
            println!("🔧 Set DATABASE_MYSQL_URL environment variable:");
            println!(
                "   export DATABASE_MYSQL_URL=\"mysql://user:password@localhost:3306/database\""
            );
            return Err(e);
        }
    }

    Ok(())
}
