//! 配置管理模块单元测试

use std::path::PathBuf;
use tempfile::TempDir;
use xore_core::Config;

#[test]
fn test_default_config() {
    let config = Config::default();

    // 验证默认值
    assert!(config.search.num_threads > 0);
    assert_eq!(config.search.auto_rebuild_days, 30);
    assert_eq!(config.search.max_index_size_gb, 10);

    assert!(config.process.lazy_execution);
    assert_eq!(config.process.chunk_size_mb, 64);
    assert_eq!(config.process.cache_size_mb, 512);

    assert!(config.ai.enable_semantic);
    assert_eq!(config.ai.embedding_dim, 384);

    assert_eq!(config.limits.max_file_size_mb, 100);
    assert_eq!(config.limits.max_memory_mb, 2048);
    assert_eq!(config.limits.max_query_time_ms, 5000);
    assert_eq!(config.limits.max_results, 1000);

    assert_eq!(config.ui.theme, "dark");
    assert!(config.ui.progress_bar);
    assert!(config.ui.color_output);

    assert!(!config.exclude.patterns.is_empty());
    assert!(config.exclude.patterns.contains(&".git".to_string()));
}

#[test]
fn test_save_and_load_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // 创建并保存配置
    let mut config = Config::default();
    config.search.num_threads = 8;
    config.process.chunk_size_mb = 128;
    config.ui.theme = "light".to_string();

    config.save(&config_path).unwrap();

    // 加载配置并验证
    let loaded_config = Config::load(&config_path).unwrap();

    assert_eq!(loaded_config.search.num_threads, 8);
    assert_eq!(loaded_config.process.chunk_size_mb, 128);
    assert_eq!(loaded_config.ui.theme, "light");
}

#[test]
fn test_load_nonexistent_config() {
    let result = Config::load(&PathBuf::from("/nonexistent/config.toml"));
    assert!(result.is_err());
}

#[test]
fn test_save_invalid_path() {
    let config = Config::default();
    let result = config.save(&PathBuf::from("/invalid/path/config.toml"));
    assert!(result.is_err());
}

#[test]
fn test_config_serialization() {
    let config = Config::default();
    let toml_str = toml::to_string(&config).unwrap();

    // 验证序列化包含关键字段
    assert!(toml_str.contains("num_threads"));
    assert!(toml_str.contains("chunk_size_mb"));
    assert!(toml_str.contains("enable_semantic"));
    assert!(toml_str.contains("max_memory_mb"));
}

#[test]
fn test_config_deserialization() {
    let toml_str = r#"
        [search]
        global_index_path = "/tmp/index"
        use_project_index = false
        project_index_path = ".xore/index"
        num_threads = 4
        auto_rebuild_days = 7
        max_index_size_gb = 5
        max_file_size_mb = 100
        writer_buffer_mb = 50

        [process]
        lazy_execution = false
        chunk_size_mb = 32
        cache_size_mb = 256

        [ai]
        model_path = "/tmp/model.onnx"
        enable_semantic = false
        embedding_dim = 768

        [limits]
        max_file_size_mb = 50
        max_memory_mb = 1024
        max_query_time_ms = 3000
        max_results = 500

        [ui]
        theme = "light"
        progress_bar = false
        color_output = false

        [exclude]
        patterns = ["*.log", "*.tmp"]
    "#;

    let config: Config = toml::from_str(toml_str).unwrap();

    assert_eq!(config.search.num_threads, 4);
    assert_eq!(config.search.auto_rebuild_days, 7);
    assert!(!config.search.use_project_index);
    assert!(!config.process.lazy_execution);
    assert_eq!(config.process.chunk_size_mb, 32);
    assert!(!config.ai.enable_semantic);
    assert_eq!(config.ai.embedding_dim, 768);
    assert_eq!(config.limits.max_file_size_mb, 50);
    assert_eq!(config.ui.theme, "light");
    assert!(!config.ui.progress_bar);
    assert_eq!(config.exclude.patterns.len(), 2);
}

#[test]
fn test_config_clone() {
    let config = Config::default();
    let cloned = config.clone();

    assert_eq!(config.search.num_threads, cloned.search.num_threads);
    assert_eq!(config.ui.theme, cloned.ui.theme);
}

#[test]
fn test_search_config() {
    let config = Config::default();
    assert!(config.search.num_threads > 0);
    assert!(config.search.auto_rebuild_days > 0);
}

#[test]
fn test_limits_config() {
    let config = Config::default();
    assert!(config.limits.max_file_size_mb > 0);
    assert!(config.limits.max_memory_mb > 0);
    assert!(config.limits.max_query_time_ms > 0);
    assert!(config.limits.max_results > 0);
}

#[test]
fn test_exclude_patterns() {
    let config = Config::default();
    assert!(config.exclude.patterns.contains(&".git".to_string()));
    assert!(config.exclude.patterns.contains(&"node_modules".to_string()));
    assert!(config.exclude.patterns.contains(&"target".to_string()));
}

#[test]
fn test_config_debug_format() {
    let config = Config::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("Config"));
    assert!(debug_str.contains("search"));
}
