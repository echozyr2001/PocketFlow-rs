use pocketflow_rs::prelude::*;
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), sea_orm::DbErr> {
    println!("🚀 PocketFlow-RS Multi-Database Support Example");
    
    // Example 1: SQLite Database (always available)
    println!("\n📝 Example 1: SQLite Database");
    
    let sqlite_storage = DatabaseStorage::new("sqlite::memory:").await?;
    sqlite_storage.migrate().await?;
    let sqlite_store = AsyncSharedStore::new(sqlite_storage);
    
    // Test basic operations with SQLite
    sqlite_store.set("sqlite_test".to_string(), json!({
        "database": "SQLite",
        "type": "in-memory",
        "features": ["lightweight", "serverless", "zero-config"]
    })).await?;
    
    println!("✅ SQLite: Stored test data");
    
    if let Some(data) = sqlite_store.get("sqlite_test").await? {
        println!("📄 SQLite Data: {}", data["database"]);
    }
    
    // Example 2: PostgreSQL Database (if available)
    println!("\n📝 Example 2: PostgreSQL Database");
    
    // Check if PostgreSQL URL is provided via environment variable
    if let Ok(postgres_url) = env::var("DATABASE_POSTGRES_URL") {
        match DatabaseStorage::new(&postgres_url).await {
            Ok(postgres_storage) => {
                if let Ok(_) = postgres_storage.migrate().await {
                    let postgres_store = AsyncSharedStore::new(postgres_storage);
                    
                    postgres_store.set("postgres_test".to_string(), json!({
                        "database": "PostgreSQL",
                        "type": "relational",
                        "features": ["ACID", "transactions", "JSON support", "scalable"]
                    })).await?;
                    
                    println!("✅ PostgreSQL: Connected and stored test data");
                    
                    if let Some(data) = postgres_store.get("postgres_test").await? {
                        println!("📄 PostgreSQL Data: {}", data["database"]);
                    }
                } else {
                    println!("⚠️ PostgreSQL: Failed to run migrations");
                }
            }
            Err(e) => {
                println!("⚠️ PostgreSQL: Connection failed - {}", e);
            }
        }
    } else {
        println!("💡 PostgreSQL: Set DATABASE_POSTGRES_URL environment variable to test");
        println!("   Example: export DATABASE_POSTGRES_URL=\"postgres://user:pass@localhost:5432/pocketflow\"");
    }
    
    // Example 3: MySQL Database (if available)
    println!("\n📝 Example 3: MySQL Database");
    
    // Check if MySQL URL is provided via environment variable
    if let Ok(mysql_url) = env::var("DATABASE_MYSQL_URL") {
        match DatabaseStorage::new(&mysql_url).await {
            Ok(mysql_storage) => {
                if let Ok(_) = mysql_storage.migrate().await {
                    let mysql_store = AsyncSharedStore::new(mysql_storage);
                    
                    mysql_store.set("mysql_test".to_string(), json!({
                        "database": "MySQL",
                        "type": "relational",
                        "features": ["high-performance", "replication", "clustering"]
                    })).await?;
                    
                    println!("✅ MySQL: Connected and stored test data");
                    
                    if let Some(data) = mysql_store.get("mysql_test").await? {
                        println!("📄 MySQL Data: {}", data["database"]);
                    }
                } else {
                    println!("⚠️ MySQL: Failed to run migrations");
                }
            }
            Err(e) => {
                println!("⚠️ MySQL: Connection failed - {}", e);
            }
        }
    } else {
        println!("💡 MySQL: Set DATABASE_MYSQL_URL environment variable to test");
        println!("   Example: export DATABASE_MYSQL_URL=\"mysql://user:pass@localhost:3306/pocketflow\"");
    }
    
    // Example 4: Database Performance Comparison
    println!("\n📊 Example 4: Performance Comparison");
    
    let start_time = std::time::Instant::now();
    
    // Perform bulk operations on SQLite
    for i in 0..100 {
        sqlite_store.set(format!("bulk_test_{}", i), json!({
            "index": i,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "data": format!("test_data_{}", i)
        })).await?;
    }
    
    let sqlite_time = start_time.elapsed();
    println!("⚡ SQLite: Inserted 100 records in {:?}", sqlite_time);
    
    // Example 5: Complex Queries and Data Structures
    println!("\n🔍 Example 5: Complex Data Operations");
    
    // Store complex workflow configuration
    let workflow_config = json!({
        "workflow_id": "complex_pipeline",
        "version": "2.1.0",
        "stages": [
            {
                "name": "data_ingestion",
                "type": "batch",
                "config": {
                    "batch_size": 1000,
                    "timeout": 300,
                    "retry_count": 3
                },
                "dependencies": []
            },
            {
                "name": "data_processing",
                "type": "stream",
                "config": {
                    "window_size": "5m",
                    "watermark": "10s",
                    "parallelism": 4
                },
                "dependencies": ["data_ingestion"]
            },
            {
                "name": "ai_inference",
                "type": "ml",
                "config": {
                    "model": "gpt-4",
                    "temperature": 0.7,
                    "max_tokens": 2048
                },
                "dependencies": ["data_processing"]
            },
            {
                "name": "result_storage",
                "type": "persistent",
                "config": {
                    "format": "parquet",
                    "compression": "snappy",
                    "partitioning": ["date", "category"]
                },
                "dependencies": ["ai_inference"]
            }
        ],
        "scheduling": {
            "type": "cron",
            "expression": "0 */6 * * *",
            "timezone": "UTC"
        },
        "monitoring": {
            "metrics": ["throughput", "latency", "error_rate"],
            "alerts": {
                "error_rate_threshold": 5.0,
                "latency_threshold": "30s"
            }
        }
    });
    
    sqlite_store.set("complex_workflow".to_string(), workflow_config).await?;
    
    if let Some(config) = sqlite_store.get("complex_workflow").await? {
        println!("✅ Complex workflow stored and retrieved");
        println!("📋 Workflow ID: {}", config["workflow_id"]);
        println!("📋 Version: {}", config["version"]);
        println!("📋 Stages: {}", config["stages"].as_array().unwrap().len());
    }
    
    // Example 6: Database Feature Matrix
    println!("\n📈 Example 6: Database Feature Comparison");
    
    let database_features = json!({
        "SQLite": {
            "pros": [
                "Zero configuration",
                "Serverless",
                "Cross-platform",
                "Small footprint",
                "ACID compliant"
            ],
            "cons": [
                "Single writer limitation",
                "No network access",
                "Limited concurrency"
            ],
            "use_cases": [
                "Development and testing",
                "Desktop applications",
                "IoT devices",
                "Prototyping"
            ]
        },
        "PostgreSQL": {
            "pros": [
                "Full ACID compliance",
                "Extensible",
                "Standards compliant",
                "Strong data integrity",
                "Advanced features"
            ],
            "cons": [
                "More complex setup",
                "Higher memory usage",
                "Steeper learning curve"
            ],
            "use_cases": [
                "Web applications",
                "Data warehousing",
                "Financial systems",
                "Enterprise applications"
            ]
        },
        "MySQL": {
            "pros": [
                "High performance",
                "Mature ecosystem",
                "Wide adoption", 
                "Good replication",
                "Easy to scale"
            ],
            "cons": [
                "Less feature-rich than PostgreSQL",
                "Some compliance limitations",
                "License considerations"
            ],
            "use_cases": [
                "Web applications",
                "E-commerce",
                "Social media platforms",
                "Content management"
            ]
        }
    });
    
    sqlite_store.set("database_features".to_string(), database_features).await?;
    println!("✅ Database feature comparison stored");
    
    // Final statistics
    let total_keys = sqlite_store.keys().await?;
    println!("\n📊 Final Statistics:");
    println!("  - Total keys stored: {}", total_keys.len());
    println!("  - Storage backend: SQLite (in-memory)");
    println!("  - All operations completed successfully");
    
    println!("\n🎉 Multi-database support example completed!");
    println!("💡 PocketFlow-rs supports SQLite, PostgreSQL, and MySQL");
    println!("🔧 Use environment variables to test different databases:");
    println!("   - DATABASE_POSTGRES_URL for PostgreSQL");
    println!("   - DATABASE_MYSQL_URL for MySQL");
    
    Ok(())
}

// Helper function to demonstrate database-specific configurations
#[allow(dead_code)]
async fn database_specific_examples() -> Result<(), sea_orm::DbErr> {
    // SQLite with file persistence
    let _sqlite_file = DatabaseStorage::new("sqlite:pocketflow.db").await?;
    
    // PostgreSQL with connection pool
    let _postgres = DatabaseStorage::new(
        "postgres://user:password@localhost:5432/pocketflow?max_connections=10"
    ).await?;
    
    // MySQL with SSL
    let _mysql = DatabaseStorage::new(
        "mysql://user:password@localhost:3306/pocketflow?ssl-mode=required"
    ).await?;
    
    Ok(())
}