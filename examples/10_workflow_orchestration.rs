//! ⚙️ PocketFlow-rs Workflow Orchestration
//!
//! Production-ready multi-step workflows with comprehensive examples.
//! Learn advanced patterns for enterprise workflow automation.

use pocketflow_rs::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚙️ PocketFlow-rs Workflow Orchestration");
    println!("Building production-ready enterprise workflows\n");

    // Example 1: Document Processing Pipeline
    document_processing_workflow().await?;

    println!("\n{}\n", "=".repeat(60));

    // Example 2: Customer Onboarding Automation
    customer_onboarding_workflow().await?;

    println!("\n{}\n", "=".repeat(60));

    // Example 3: Multi-stage Data Pipeline
    data_pipeline_workflow().await?;

    Ok(())
}

/// Example 1: Document Processing Workflow
async fn document_processing_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Document Processing Pipeline");
    println!("   🎯 Pattern: Upload → Validate → Process → Store → Notify");

    let mut workflow = FlowBuilder::new()
        .start_node("upload")
        .terminal_action("complete")
        .terminal_action("failed")
        .max_steps(20)
        // Stage 1: Document Upload
        .node(
            "upload",
            Node::new(SetValueNode::new(
                "document".to_string(),
                json!({
                    "id": "doc_001",
                    "name": "contract.pdf",
                    "size": "2.5MB",
                    "type": "pdf",
                    "uploaded_at": chrono::Utc::now().to_rfc3339()
                }),
                Action::simple("uploaded"),
            )),
        )
        // Stage 2: Validation
        .node(
            "validate",
            Node::new(ConditionalNode::new(
                |store| {
                    if let Ok(Some(doc)) = store.get("document") {
                        let size_str = doc["size"].as_str().unwrap_or("0MB");
                        let doc_type = doc["type"].as_str().unwrap_or("unknown");

                        println!(
                            "   📋 Validating: {} ({})",
                            doc["name"].as_str().unwrap_or("unknown"),
                            size_str
                        );

                        if doc_type == "pdf" && !size_str.starts_with("0") {
                            println!("   ✅ Document validation passed");
                            true
                        } else {
                            println!("   ❌ Document validation failed");
                            false
                        }
                    } else {
                        false
                    }
                },
                Action::simple("valid"),
                Action::simple("invalid"),
            )),
        )
        // Stage 3: Processing
        .node(
            "process",
            Node::new(SetValueNode::new(
                "processing_result".to_string(),
                json!({
                    "status": "processed",
                    "pages_extracted": 15,
                    "text_sections": 8,
                    "images_found": 3,
                    "metadata": {
                        "author": "Legal Team",
                        "created": "2024-01-10",
                        "modified": "2024-01-15"
                    },
                    "processing_time_ms": 1250,
                    "processed_at": chrono::Utc::now().to_rfc3339()
                }),
                Action::simple("processed"),
            )),
        )
        // Stage 4: Storage
        .node(
            "store",
            Node::new(LogNode::new(
                "💾 Document stored in database with full-text search indexing",
                Action::simple("stored"),
            )),
        )
        // Stage 5: Notification
        .node(
            "notify",
            Node::new(LogNode::new(
                "📧 Notification sent: Document processing completed successfully",
                Action::simple("complete"),
            )),
        )
        // Error handling
        .node(
            "validation_error",
            Node::new(LogNode::new(
                "⚠️ Document validation failed - unsupported format or size",
                Action::simple("failed"),
            )),
        )
        // Routing
        .route("upload", "uploaded", "validate")
        .route("validate", "valid", "process")
        .route("validate", "invalid", "validation_error")
        .route("process", "processed", "store")
        .route("store", "stored", "notify")
        .build();

    let mut store = SharedStore::new();
    let result = workflow.execute(&mut store).await?;

    println!(
        "   📊 Pipeline Result: {} → {} (Status: {})",
        result.execution_path.join(" → "),
        result.last_node_id,
        if result.success {
            "✅ Success"
        } else {
            "❌ Failed"
        }
    );

    Ok(())
}

/// Example 2: Customer Onboarding Workflow
async fn customer_onboarding_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("👤 Customer Onboarding Automation");
    println!("   🎯 Pattern: Register → Verify → Setup → Welcome → Track");

    let mut workflow = FlowBuilder::new()
        .start_node("registration")
        .terminal_action("onboarded")
        .terminal_action("rejected")
        .max_steps(15)
        
        // Stage 1: Customer Registration
        .node("registration", Node::new(SetValueNode::new(
            "customer".to_string(),
            json!({
                "id": "cust_001",
                "email": "alice@company.com",
                "company": "TechCorp Inc.",
                "plan": "premium",
                "industry": "technology",
                "team_size": 25,
                "registration_date": chrono::Utc::now().to_rfc3339()
            }),
            Action::simple("registered")
        )))
        
        // Stage 2: Verification
        .node("verification", Node::new(ConditionalNode::new(
            |store| {
                if let Ok(Some(customer)) = store.get("customer") {
                    let email = customer["email"].as_str().unwrap_or("");
                    let company = customer["company"].as_str().unwrap_or("");
                    
                    println!("   🔍 Verifying: {} from {}", email, company);
                    
                    if email.contains("@") && !company.is_empty() {
                        println!("   ✅ Customer verification successful");
                        true
                    } else {
                        println!("   ❌ Customer verification failed");
                        false
                    }
                } else {
                    false
                }
            },
            Action::simple("verified"),
            Action::simple("verification_failed")
        )))
        
        // Stage 3: Account Setup
        .node("setup", Node::new(SetValueNode::new(
            "account_setup".to_string(),
            json!({
                "workspace_created": true,
                "api_keys_generated": true,
                "team_invites_sent": true,
                "billing_configured": true,
                "features_enabled": ["advanced_analytics", "custom_workflows", "priority_support"],
                "setup_completed_at": chrono::Utc::now().to_rfc3339()
            }),
            Action::simple("setup_complete")
        )))
        
        // Stage 4: Welcome & Training
        .node("welcome", Node::new(LogNode::new(
            "🎉 Welcome package sent: Tutorial videos, documentation, and support contacts",
            Action::simple("welcomed")
        )))
        
        // Stage 5: Tracking Setup
        .node("tracking", Node::new(LogNode::new(
            "📊 Analytics tracking initialized for onboarding metrics",
            Action::simple("onboarded")
        )))
        
        // Error handling
        .node("rejection", Node::new(LogNode::new(
            "❌ Customer onboarding rejected due to verification failure",
            Action::simple("rejected")
        )))
        
        // Routing
        .route("registration", "registered", "verification")
        .route("verification", "verified", "setup")
        .route("verification", "verification_failed", "rejection")
        .route("setup", "setup_complete", "welcome")
        .route("welcome", "welcomed", "tracking")
        .build();

    let mut store = SharedStore::new();
    let result = workflow.execute(&mut store).await?;

    println!(
        "   📊 Onboarding Result: {} → {} (Status: {})",
        result.execution_path.join(" → "),
        result.last_node_id,
        if result.success {
            "✅ Success"
        } else {
            "❌ Failed"
        }
    );

    Ok(())
}

/// Example 3: Multi-stage Data Pipeline
async fn data_pipeline_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Multi-stage Data Pipeline");
    println!("   🎯 Pattern: Extract → Transform → Validate → Load → Monitor");

    let mut workflow = FlowBuilder::new()
        .start_node("extract")
        .terminal_action("success")
        .terminal_action("failed")
        .max_steps(20)
        // Stage 1: Data Extraction
        .node(
            "extract",
            Node::new(SetValueNode::new(
                "raw_data".to_string(),
                json!({
                    "source": "production_database",
                    "records_extracted": 10000,
                    "tables": ["users", "orders", "products"],
                    "extraction_time": "2.3s",
                    "data_quality": "good",
                    "extracted_at": chrono::Utc::now().to_rfc3339()
                }),
                Action::simple("extracted"),
            )),
        )
        // Stage 2: Data Transformation
        .node(
            "transform",
            Node::new(SetValueNode::new(
                "transformed_data".to_string(),
                json!({
                    "transformations_applied": [
                        "data_cleaning",
                        "normalization",
                        "enrichment",
                        "aggregation"
                    ],
                    "output_records": 9850,
                    "data_quality": "high",
                    "transformation_time": "5.7s",
                    "transformed_at": chrono::Utc::now().to_rfc3339()
                }),
                Action::simple("transformed"),
            )),
        )
        // Stage 3: Data Validation
        .node(
            "validate",
            Node::new(ConditionalNode::new(
                |store| {
                    if let Ok(Some(transformed)) = store.get("transformed_data") {
                        let output_records = transformed["output_records"].as_i64().unwrap_or(0);
                        let quality = transformed["data_quality"].as_str().unwrap_or("unknown");

                        println!(
                            "   🔍 Validating: {} records with {} quality",
                            output_records, quality
                        );

                        if output_records > 5000 && quality == "high" {
                            println!("   ✅ Data validation successful");
                            true
                        } else {
                            println!("   ❌ Data validation failed");
                            false
                        }
                    } else {
                        false
                    }
                },
                Action::simple("validation_passed"),
                Action::simple("validation_failed"),
            )),
        )
        // Stage 4: Data Loading
        .node(
            "load",
            Node::new(LogNode::new(
                "💾 Data loaded to data warehouse with indexing and partitioning",
                Action::simple("loaded"),
            )),
        )
        // Stage 5: Monitoring Setup
        .node(
            "monitor",
            Node::new(LogNode::new(
                "📈 Monitoring alerts configured for data freshness and quality",
                Action::simple("success"),
            )),
        )
        // Error handling
        .node(
            "validation_error",
            Node::new(LogNode::new(
                "⚠️ Data pipeline failed validation - manual review required",
                Action::simple("failed"),
            )),
        )
        // Routing
        .route("extract", "extracted", "transform")
        .route("transform", "transformed", "validate")
        .route("validate", "validation_passed", "load")
        .route("validate", "validation_failed", "validation_error")
        .route("load", "loaded", "monitor")
        .build();

    let mut store = SharedStore::new();
    let result = workflow.execute(&mut store).await?;

    println!(
        "   📊 Pipeline Result: {} → {} (Status: {})",
        result.execution_path.join(" → "),
        result.last_node_id,
        if result.success {
            "✅ Success"
        } else {
            "❌ Failed"
        }
    );

    // Summary and Best Practices
    println!("\n💡 Enterprise Workflow Best Practices:");
    println!("  🏗️ Decomposition: Break complex processes into clear stages");
    println!("  🔍 Validation: Add validation nodes at critical decision points");
    println!("  🛡️ Error Handling: Design graceful failure and recovery paths");
    println!("  📊 Monitoring: Include tracking and observability throughout");
    println!("  🔄 Idempotency: Ensure operations can be safely retried");
    println!("  📝 Documentation: Use descriptive node names and log messages");

    println!("\n🎯 Production Workflow Checklist:");
    println!("  ✅ Clear input/output contracts");
    println!("  ✅ Comprehensive error handling");
    println!("  ✅ Performance monitoring points");
    println!("  ✅ Data validation at each stage");
    println!("  ✅ Rollback and recovery procedures");
    println!("  ✅ Security and compliance checks");

    Ok(())
}
