#[cfg(feature = "builtin-llm")]
use pocketflow_rs::ApiConfig;

#[cfg(feature = "builtin-llm")]
#[tokio::test]
async fn test_api_config_default() {
    let config = ApiConfig::default();
    assert_eq!(config.model, "gpt-3.5-turbo");
    assert_eq!(config.max_tokens, Some(1000));
    assert_eq!(config.temperature, Some(0.7));
    assert_eq!(config.timeout, Some(30));
}

#[cfg(not(feature = "builtin-llm"))]
#[tokio::test]
async fn test_llm_feature_not_enabled() {
    // This test runs when LLM features are not enabled
    assert!(true, "LLM features not enabled - this is expected");
}
