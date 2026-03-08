//! MCP Server — seven xore tools + ServerHandler

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Implementation, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router, ErrorData, ServerHandler,
};
use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use tokio::task::spawn_blocking;

use xore_config::{Config, XorePaths};
use xore_process::{DataParser, DataProfiler, SqlEngine};
use xore_search::{FileScanner, FileTypeFilter, ScanConfig, Searcher};

use crate::error::into_mcp_error;
use crate::helpers::{dataframe_to_json, smart_sample, strip_ansi, text_result};

// ─── Argument structs ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FindFilesArgs {
    /// Root directory to search in
    pub path: String,
    /// File type filter: csv, json, log, code, text, parquet, or comma-separated extensions
    pub file_type: Option<String>,
    /// Maximum directory depth (None = unlimited)
    pub max_depth: Option<usize>,
    /// Include hidden files/directories (default false)
    pub include_hidden: Option<bool>,
    /// Maximum results to return (default 200)
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchIndexArgs {
    /// Full-text search query
    pub query: String,
    /// Path to xore index directory (uses default if omitted)
    pub index_path: Option<String>,
    /// File type filter (e.g. "rs", "py", "csv")
    pub file_type: Option<String>,
    /// Maximum results to return (default 50)
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetSchemaArgs {
    /// Path to the data file (CSV, Parquet, JSON, JSONL)
    pub path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct QueryDataArgs {
    /// Path to the data file (CSV, Parquet, JSON, JSONL)
    pub path: String,
    /// SQL query — use table name "this"
    pub sql: String,
    /// Maximum rows to return (default 1000)
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SampleDataArgs {
    /// Path to the data file (CSV, Parquet, JSON, JSONL)
    pub path: String,
    /// Number of rows (default 20)
    pub rows: Option<usize>,
    /// Strategy: "smart" (uniform, default), "head", or "tail"
    pub strategy: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct QualityCheckArgs {
    /// Path to the data file (CSV, Parquet, JSON, JSONL)
    pub path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetConfigArgs {}

// ─── Server ────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct XoreMcpServer {
    tool_router: ToolRouter<Self>,
}

impl XoreMcpServer {
    pub fn new() -> Self {
        Self { tool_router: Self::tool_router() }
    }
}

#[tool_router]
impl XoreMcpServer {
    #[tool(description = "Find files in the local filesystem using path, type, and depth filters")]
    async fn find_files(
        &self,
        Parameters(args): Parameters<FindFilesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        spawn_blocking(move || {
            let root = PathBuf::from(&args.path);
            let file_type = args
                .file_type
                .as_deref()
                .map(FileTypeFilter::parse)
                .transpose()
                .map_err(|e| into_mcp_error(anyhow::anyhow!("invalid file_type: {}", e)))?;

            let config = ScanConfig {
                root,
                file_type,
                max_depth: args.max_depth,
                include_hidden: args.include_hidden.unwrap_or(false),
                ..Default::default()
            };
            let limit = args.limit.unwrap_or(200);
            let (files, stats) = FileScanner::new(config)
                .scan()
                .map_err(|e| into_mcp_error(anyhow::anyhow!("scan failed: {}", e)))?;

            let file_list: Vec<_> = files
                .iter()
                .take(limit)
                .map(|f| {
                    json!({
                        "path": f.path.display().to_string(),
                        "size": f.size,
                        "is_dir": f.is_dir,
                        "modified_secs": f.modified
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs()),
                    })
                })
                .collect();

            let returned = file_list.len();
            text_result(json!({
                "files": file_list,
                "returned": returned,
                "total_matched": stats.matched_files,
                "total_scanned": stats.total_files,
            }))
        })
        .await
        .map_err(|e| into_mcp_error(anyhow::anyhow!("task panicked: {}", e)))?
    }

    #[tool(description = "Search xore's full-text index for files matching a query")]
    async fn search_index(
        &self,
        Parameters(args): Parameters<SearchIndexArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        spawn_blocking(move || {
            let index_path = match args.index_path.as_deref() {
                Some(p) => PathBuf::from(p),
                None => XorePaths::new()
                    .map_err(|e| into_mcp_error(anyhow::anyhow!("paths init failed: {}", e)))?
                    .default_index_dir(),
            };

            let limit = args.limit.unwrap_or(50);
            let searcher = Searcher::new(&index_path)
                .map_err(|e| into_mcp_error(anyhow::anyhow!("searcher init failed: {}", e)))?;

            let results = match args.file_type.as_deref() {
                Some(ft) => searcher
                    .search_with_filter(&args.query, Some(ft), limit)
                    .map_err(|e| into_mcp_error(anyhow::anyhow!("search failed: {}", e)))?,
                None => searcher
                    .search_smart(&args.query, limit)
                    .map_err(|e| into_mcp_error(anyhow::anyhow!("search failed: {}", e)))?,
            };

            let count = results.len();
            let items: Vec<_> = results
                .iter()
                .map(|r| {
                    json!({
                        "path": r.path.display().to_string(),
                        "score": r.score,
                        "line": r.line,
                        "snippet": r.snippet.as_deref().map(strip_ansi),
                    })
                })
                .collect();

            text_result(json!({ "results": items, "count": count }))
        })
        .await
        .map_err(|e| into_mcp_error(anyhow::anyhow!("task panicked: {}", e)))?
    }

    #[tool(description = "Get schema and column statistics of a data file (CSV, Parquet, JSON)")]
    async fn get_schema(
        &self,
        Parameters(args): Parameters<GetSchemaArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        spawn_blocking(move || {
            let path = PathBuf::from(&args.path);
            let df = DataParser::new()
                .read(&path)
                .map_err(|e| into_mcp_error(anyhow::anyhow!("read failed: {}", e)))?;

            let report = DataProfiler::new()
                .profile(&df)
                .map_err(|e| into_mcp_error(anyhow::anyhow!("profile failed: {}", e)))?;

            let schema: Vec<_> = df
                .schema()
                .iter()
                .map(|(name, dtype)| {
                    let n = name.as_str();
                    json!({
                        "column": n,
                        "dtype": format!("{:?}", dtype),
                        "type_label": report.column_types.get(n).cloned().unwrap_or_default(),
                        "missing_count": report.missing_values.get(n).map(|m| m.count),
                        "missing_pct": report.missing_values.get(n).map(|m| m.percentage),
                    })
                })
                .collect();

            text_result(json!({
                "path": args.path,
                "rows": report.total_rows,
                "columns": report.total_columns,
                "schema": schema,
            }))
        })
        .await
        .map_err(|e| into_mcp_error(anyhow::anyhow!("task panicked: {}", e)))?
    }

    #[tool(description = "Run a SQL query on a data file. Use 'this' as the table name")]
    async fn query_data(
        &self,
        Parameters(args): Parameters<QueryDataArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        spawn_blocking(move || {
            let path = PathBuf::from(&args.path);
            let limit = args.limit.unwrap_or(1000);
            let mut engine = SqlEngine::new();
            engine
                .register_table("this", &path)
                .map_err(|e| into_mcp_error(anyhow::anyhow!("register table failed: {}", e)))?;

            let df = engine
                .execute(&args.sql)
                .map_err(|e| into_mcp_error(anyhow::anyhow!("query failed: {}", e)))?;

            let df_limited = df.head(Some(limit));
            let col_names: Vec<String> =
                df_limited.get_column_names().iter().map(|s| s.to_string()).collect();
            let rows = dataframe_to_json(&df_limited)
                .map_err(|e| into_mcp_error(anyhow::anyhow!("serialization failed: {}", e)))?;
            let row_count = rows.len();

            text_result(json!({
                "rows": rows,
                "row_count": row_count,
                "columns": col_names,
            }))
        })
        .await
        .map_err(|e| into_mcp_error(anyhow::anyhow!("task panicked: {}", e)))?
    }

    #[tool(description = "Sample rows from a data file. Strategies: smart (uniform), head, tail")]
    async fn sample_data(
        &self,
        Parameters(args): Parameters<SampleDataArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        spawn_blocking(move || {
            let path = PathBuf::from(&args.path);
            let n = args.rows.unwrap_or(20);
            let strategy = args.strategy.as_deref().unwrap_or("smart").to_string();

            let df = DataParser::new()
                .read(&path)
                .map_err(|e| into_mcp_error(anyhow::anyhow!("read failed: {}", e)))?;

            let total = df.height();
            let sampled = match strategy.as_str() {
                "head" => df.head(Some(n)),
                "tail" => df.tail(Some(n)),
                _ => smart_sample(&df, n)
                    .map_err(|e| into_mcp_error(anyhow::anyhow!("sample failed: {}", e)))?,
            };

            let rows = dataframe_to_json(&sampled)
                .map_err(|e| into_mcp_error(anyhow::anyhow!("serialization failed: {}", e)))?;
            let sampled_count = rows.len();

            text_result(json!({
                "rows": rows,
                "sampled": sampled_count,
                "total": total,
                "strategy": strategy,
            }))
        })
        .await
        .map_err(|e| into_mcp_error(anyhow::anyhow!("task panicked: {}", e)))?
    }

    #[tool(description = "Data quality check: missing values, duplicates, outliers, suggestions")]
    async fn quality_check(
        &self,
        Parameters(args): Parameters<QualityCheckArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        spawn_blocking(move || {
            let path = PathBuf::from(&args.path);
            let df = DataParser::new()
                .read(&path)
                .map_err(|e| into_mcp_error(anyhow::anyhow!("read failed: {}", e)))?;

            let report = DataProfiler::new()
                .profile(&df)
                .map_err(|e| into_mcp_error(anyhow::anyhow!("profile failed: {}", e)))?;

            let missing: Vec<_> = report
                .missing_values
                .iter()
                .map(|(col, m)| json!({ "column": col, "count": m.count, "pct": m.percentage }))
                .collect();

            let outliers: Vec<_> = report
                .outliers
                .iter()
                .map(|(col, o)| {
                    let end = o.indices.len().min(5);
                    json!({
                        "column": col,
                        "count": o.count,
                        "pct": o.percentage,
                        "sample_indices": &o.indices[..end],
                    })
                })
                .collect();

            let suggestions: Vec<_> = report
                .suggestions
                .iter()
                .map(|s| {
                    json!({
                        "type": format!("{:?}", s.suggestion_type),
                        "message": s.message,
                        "column": s.column,
                        "severity": format!("{:?}", s.severity),
                    })
                })
                .collect();

            text_result(json!({
                "path": args.path,
                "total_rows": report.total_rows,
                "total_columns": report.total_columns,
                "duplicate_rows": report.duplicate_rows,
                "missing_values": missing,
                "outliers": outliers,
                "suggestions": suggestions,
            }))
        })
        .await
        .map_err(|e| into_mcp_error(anyhow::anyhow!("task panicked: {}", e)))?
    }

    #[tool(description = "Get the current xore configuration (index path, history path, etc.)")]
    async fn get_config(
        &self,
        Parameters(_args): Parameters<GetConfigArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        spawn_blocking(move || {
            let config = Config::load_with_defaults();
            text_result(json!({
                "index_path":   config.paths.index.display().to_string(),
                "history_path": config.paths.history.display().to_string(),
                "logs_path":    config.paths.logs.display().to_string(),
                "models_path":  config.paths.models.display().to_string(),
            }))
        })
        .await
        .map_err(|e| into_mcp_error(anyhow::anyhow!("task panicked: {}", e)))?
    }
}

#[tool_handler]
impl ServerHandler for XoreMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("xore-mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions(
                "XORE file search and data processing tools. \
                Use find_files/search_index for file discovery, \
                get_schema/sample_data/query_data/quality_check for data analysis.",
            )
    }
}
