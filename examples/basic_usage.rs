#[cfg(feature = "storage-file")]
use pocketflow_rs::FileStorage;
use pocketflow_rs::{Action, InMemorySharedStore, SharedStore};
use serde_json::json;
#[cfg(feature = "storage-file")]
use tempfile::tempdir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 PocketFlow-RS Enhanced Storage Example");

    // Example 1: In-memory storage (default)
    println!("\n📝 Example 1: In-Memory Storage");
    let mut memory_store = InMemorySharedStore::new();

    // Set some values
    memory_store.set("user_input".to_string(), json!("Hello, PocketFlow!"))?;
    memory_store.set("temperature".to_string(), json!(0.7))?;
    memory_store.set("max_tokens".to_string(), json!(100))?;

    println!("✅ Added values to in-memory store");

    // Get values back
    if let Some(input) = memory_store.get("user_input")? {
        println!("📝 User input: {}", input);
    }

    // Use serializable convenience methods
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct LLMConfig {
        model: String,
        temperature: f64,
        max_tokens: u32,
    }

    let config = LLMConfig {
        model: "gpt-4".to_string(),
        temperature: 0.7,
        max_tokens: 100,
    };

    memory_store
        .set_serializable("llm_config".to_string(), &config)
        .unwrap();
    println!("✅ Stored LLM config");

    // Retrieve and deserialize
    if let Some(retrieved_config) = memory_store
        .get_deserializable::<LLMConfig>("llm_config")
        .unwrap()
    {
        println!("🔧 Retrieved config: {:?}", retrieved_config);
    }

    // Show store statistics
    println!("📊 Memory store stats: {} items", memory_store.len()?);

    // Example 2: File-based storage
    #[cfg(feature = "storage-file")]
    {
        println!("\n💾 Example 2: File-Based Storage");
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("pocketflow_data.json");

        let file_storage = FileStorage::new(&file_path)?;
        let mut file_store = SharedStore::with_storage(file_storage);

        // Store data that will persist to file
        file_store.set(
            "persistent_data".to_string(),
            json!({
                "message": "This data is saved to a file!",
                "timestamp": "2024-01-01T00:00:00Z"
            }),
        )?;

        println!("✅ Saved data to file: {:?}", file_path);

        // Create a new store instance with the same file to demonstrate persistence
        let file_storage2 = FileStorage::new(&file_path)?;
        let file_store2 = SharedStore::with_storage(file_storage2);

        if let Some(persistent_data) = file_store2.get("persistent_data")? {
            println!("🔄 Retrieved persistent data: {}", persistent_data);
        }
    }

    #[cfg(not(feature = "storage-file"))]
    {
        println!("\n💾 Example 2: File-Based Storage (Feature not enabled)");
        println!("Enable with: cargo run --features storage-file --example basic_usage");
    }

    // Example 3: Working with Actions
    println!("\n🎯 Example 3: Actions for Flow Control");
    let action1: Action = "continue".into();
    let action2: Action = "retry".into();
    let action3: Action = "finish".into();

    println!("Available actions: {}, {}, {}", action1, action2, action3);

    // Simulate a decision flow based on store content
    let current_action = if memory_store.contains_key("user_input")? {
        "process_input"
    } else {
        "request_input"
    };

    println!("🔄 Next action: {}", current_action);

    // Example 4: Store operations
    println!("\n🛠️  Example 4: Advanced Store Operations");

    // List all keys
    let keys = memory_store.keys()?;
    println!("🔑 All keys in memory store: {:?}", keys);

    // Check if keys exist
    println!(
        "🔍 Contains 'temperature': {}",
        memory_store.contains_key("temperature")?
    );
    println!(
        "🔍 Contains 'nonexistent': {}",
        memory_store.contains_key("nonexistent")?
    );

    // Remove a value
    let removed_value = memory_store.remove("temperature")?;
    println!("🗑️  Removed temperature: {:?}", removed_value);

    // Final state
    println!("📊 Final memory store state:");
    println!("  - Size: {} items", memory_store.len()?);
    println!("  - Is empty: {}", memory_store.is_empty()?);

    // Clean up (optional, temp dir will be cleaned automatically)
    memory_store.clear()?;
    println!("🧹 Cleared memory store");

    Ok(())
}
