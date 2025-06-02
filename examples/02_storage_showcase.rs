//! 🗄️ PocketFlow-rs Storage Systems Showcase
//!
//! Comprehensive tour of all storage backends with performance comparisons.
//! This example demonstrates in-memory, file-based, Redis, and database storage.

use pocketflow_rs::prelude::*;
use serde_json::json;
use std::time::Instant;

#[cfg(feature = "storage-redis")]
use pocketflow_rs::RedisStorage;

#[cfg(feature = "storage-database")]
use pocketflow_rs::{DatabaseStorage, storage::AsyncStorageBackend};

#[cfg(feature = "storage-file")]
use pocketflow_rs::FileStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗄️ PocketFlow-rs Storage Systems Showcase");
    println!("Exploring different storage backends and their use cases\n");

    // Example data for testing
    let test_data = json!({
        "user_id": "user_123",
        "preferences": {
            "theme": "dark",
            "language": "en",
            "notifications": true
        },
        "workflow_state": {
            "current_step": "processing",
            "progress": 0.75,
            "estimated_completion": "2024-01-01T12:00:00Z"
        }
    });

    // 1. In-Memory Storage (Default)
    println!("📝 1. In-Memory Storage (Default)");
    println!("   💡 Best for: Fast access, temporary data, development");
    let start = Instant::now();

    let mut memory_store = SharedStore::new();
    memory_store.set("test_data".to_string(), test_data.clone())?;
    memory_store.set("counter".to_string(), json!(42))?;

    // Test retrieval
    let _retrieved = memory_store.get("test_data")?;
    let memory_time = start.elapsed();

    println!("   ✅ Operations completed in {:?}", memory_time);
    println!("   📊 Store size: {} items", memory_store.len()?);
    println!("   🧠 Memory usage: Low (data lives in RAM)");
    println!();

    // 2. File-Based Storage
    #[cfg(feature = "storage-file")]
    {
        println!("💾 2. File-Based Storage");
        println!("   💡 Best for: Persistence, single-machine deployment, simple backup");

        let start = Instant::now();
        let temp_path = std::env::temp_dir().join("cosmoflow_demo.json");

        let file_storage = FileStorage::new(&temp_path)?;
        let mut file_store = SharedStore::with_storage(file_storage);

        file_store.set("test_data".to_string(), test_data.clone())?;
        file_store.set(
            "persistent_config".to_string(),
            json!({
                "app_version": "1.0.0",
                "deployment_env": "production"
            }),
        )?;

        // Test persistence by creating new store instance
        let file_storage2 = FileStorage::new(&temp_path)?;
        let file_store2 = SharedStore::with_storage(file_storage2);
        let _retrieved = file_store2.get("test_data")?;

        let file_time = start.elapsed();
        println!("   ✅ Operations completed in {:?}", file_time);
        println!("   📁 File location: {:?}", temp_path);
        println!("   💾 Persistent: Data survives application restarts");
        println!("   🔄 Performance: Good for moderate data sizes");

        // Cleanup
        let _ = std::fs::remove_file(temp_path);
        println!();
    }

    #[cfg(not(feature = "storage-file"))]
    {
        println!("💾 2. File-Based Storage");
        println!("   ⚠️  Feature not enabled. Run with: --features storage-file");
        println!();
    }

    // 3. Redis Distributed Storage
    #[cfg(feature = "storage-redis")]
    {
        println!("🔴 3. Redis Distributed Storage");
        println!("   💡 Best for: Distributed systems, high-performance caching, real-time data");

        if let Ok(redis_storage) = RedisStorage::new("redis://127.0.0.1:6379/") {
            let start = Instant::now();
            let mut redis_store = SharedStore::with_storage(redis_storage);

            // Test Redis operations
            redis_store.set("test_data".to_string(), test_data.clone())?;
            redis_store.set(
                "session_data".to_string(),
                json!({
                    "session_id": "sess_abc123",
                    "user_agent": "PocketFlow-rs/1.0",
                    "ip_address": "192.168.1.100"
                }),
            )?;

            let _retrieved = redis_store.get("test_data")?;
            let redis_time = start.elapsed();

            println!("   ✅ Operations completed in {:?}", redis_time);
            println!("   🌐 Distributed: Accessible from multiple applications");
            println!("   ⚡ Performance: Excellent for high-throughput scenarios");
            println!("   🔄 Persistence: Configurable (memory-only or disk-backed)");

            // Cleanup
            redis_store.remove("test_data")?;
            redis_store.remove("session_data")?;
        } else {
            println!("   ⚠️  Redis server not available at redis://127.0.0.1:6379/");
            println!("   💡 Start Redis: docker run -d -p 6379:6379 redis:latest");
        }
        println!();
    }

    #[cfg(not(feature = "storage-redis"))]
    {
        println!("🔴 3. Redis Distributed Storage");
        println!("   ⚠️  Feature not enabled. Run with: --features storage-redis");
        println!();
    }

    // 4. Database Storage (PostgreSQL/MySQL)
    #[cfg(feature = "storage-database")]
    {
        println!("🗃️ 4. Database Storage (PostgreSQL/MySQL)");
        println!("   💡 Best for: Enterprise applications, complex queries, ACID compliance");

        // Try PostgreSQL connection
        let postgres_url = "postgresql://postgres:password@localhost:5432/pocketflow_rs";
        match DatabaseStorage::new(postgres_url).await {
            Ok(mut db_storage) => {
                let start = Instant::now();

                // Database storage uses async operations
                db_storage
                    .set("test_data".to_string(), test_data.clone())
                    .await?;
                db_storage
                    .set(
                        "audit_log".to_string(),
                        json!({
                            "action": "data_insert",
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "user_id": "system"
                        }),
                    )
                    .await?;

                let _retrieved = db_storage.get("test_data").await?;
                let db_time = start.elapsed();

                println!("   ✅ Operations completed in {:?}", db_time);
                println!("   🛡️  ACID Transactions: Data integrity guaranteed");
                println!("   📊 Complex Queries: SQL support for advanced analytics");
                println!("   📈 Scalability: Handles large datasets efficiently");
                println!("   ⚡ Async I/O: Non-blocking operations for better performance");

                // Cleanup
                db_storage.remove("test_data").await?;
                db_storage.remove("audit_log").await?;
            }
            Err(_) => {
                println!("   ⚠️  Database not available at {}", postgres_url);
                println!(
                    "   💡 Start PostgreSQL: docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=password postgres:latest"
                );
            }
        }
        println!();
    }

    #[cfg(not(feature = "storage-database"))]
    {
        println!("🗃️ 4. Database Storage");
        println!("   ⚠️  Feature not enabled. Run with: --features storage-database");
        println!();
    }

    // Performance Comparison Summary
    println!("📊 Performance & Use Case Summary:");
    println!("┌─────────────────┬──────────────┬─────────────┬────────────────┬─────────────────┐");
    println!("│ Storage Type    │ Speed        │ Persistence │ Distribution   │ Best Use Case   │");
    println!("├─────────────────┼──────────────┼─────────────┼────────────────┼─────────────────┤");
    println!(
        "│ In-Memory       │ ⚡⚡⚡⚡⚡      │ ❌          │ ❌             │ Dev/Testing     │"
    );
    println!(
        "│ File-Based      │ ⚡⚡⚡        │ ✅          │ ❌             │ Single Machine  │"
    );
    println!(
        "│ Redis           │ ⚡⚡⚡⚡       │ ⚙️          │ ✅             │ Distributed     │"
    );
    println!(
        "│ Database        │ ⚡⚡          │ ✅          │ ✅             │ Enterprise      │"
    );
    println!("└─────────────────┴──────────────┴─────────────┴────────────────┴─────────────────┘");

    println!("\n💡 Choosing the Right Storage:");
    println!("  🚀 Development: In-Memory (fast iteration)");
    println!("  🏠 Single App: File-Based (simple persistence)");
    println!("  🌐 Microservices: Redis (shared state)");
    println!("  🏢 Enterprise: Database (full features)");

    println!("\n🎯 What's Next?");
    println!("  📚 Try: cargo run --example 03_node_showcase");
    println!("  🔧 Configure your preferred storage backend for your use case!");

    Ok(())
}
