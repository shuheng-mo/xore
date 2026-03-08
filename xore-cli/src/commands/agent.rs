//! Agent 命令实现
//!
//! 提供 Agent-Native 优化功能，包括初始化、Schema 获取、智能采样和查询。

use anyhow::{Context, Result};
use colored::*;
use serde_json::{json, Value};
use std::path::Path;
use xore_core::{ContextOperation, SessionContext};
use xore_process::{AnyValue, DataFrame, DataParser, DataProfiler, SqlEngine};

use crate::ui::{Column, Table, TableStyle, ICON_INFO, ICON_SUCCESS, ICON_TIP};

/// Agent 子命令参数
pub struct AgentArgs {
    pub subcommand: AgentSubcommand,
}

/// Agent 子命令枚举
pub enum AgentSubcommand {
    Init { model: String, format: String },
    Schema { file: String, histogram: bool, json: bool, minify: bool, with_context: bool },
    Sample { file: String, n: usize, strategy: SampleStrategy, json: bool, with_context: bool },
    Query { file: String, sql: String, format: String, minify: bool, limit: Option<usize>, with_context: bool },
    Explain { sql: String },
    Context { subcommand: ContextSubcommand },
}

/// Context 子命令枚举
pub enum ContextSubcommand {
    Get { level: String, session_id: String },
    Clear { session_id: String },
    Export { session_id: String },
    Set { custom: String, session_id: String },
}

/// 采样策略
#[derive(Debug, Clone)]
pub enum SampleStrategy {
    Random,
    Head,
    Tail,
    Smart,
}

impl std::str::FromStr for SampleStrategy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "random" => Ok(SampleStrategy::Random),
            "head" => Ok(SampleStrategy::Head),
            "tail" => Ok(SampleStrategy::Tail),
            "smart" => Ok(SampleStrategy::Smart),
            _ => Err(anyhow::anyhow!("无效的采样策略: {}", s)),
        }
    }
}

/// 执行 Agent 命令
pub fn execute(args: AgentArgs) -> Result<()> {
    match args.subcommand {
        AgentSubcommand::Init { model, format } => execute_init(&model, &format),
        AgentSubcommand::Schema { file, histogram, json, minify, with_context } => {
            execute_schema(&file, histogram, json, minify, with_context)
        }
        AgentSubcommand::Sample { file, n, strategy, json, with_context } => {
            execute_sample(&file, n, strategy, json, with_context)
        }
        AgentSubcommand::Query { file, sql, format, minify, limit, with_context } => {
            execute_query(&file, &sql, &format, minify, limit, with_context)
        }
        AgentSubcommand::Explain { sql } => execute_explain(&sql),
        AgentSubcommand::Context { subcommand } => execute_context(subcommand),
    }
}

/// 执行 init 命令
fn execute_init(model: &str, format: &str) -> Result<()> {
    match format.to_lowercase().as_str() {
        "openai" => {
            let schema = get_openai_tools_schema();
            println!("{}", serde_json::to_string_pretty(&schema)?);
        }
        "langchain" => {
            let code = get_langchain_tool_code(model);
            println!("{}", code);
        }
        "openapi" => {
            let spec = get_openapi_spec();
            println!("{}", serde_json::to_string_pretty(&spec)?);
        }
        "mcp" => {
            let desc = get_mcp_tool_description();
            println!("{}", serde_json::to_string_pretty(&desc)?);
        }
        _ => {
            let template = get_prompt_template(model)?;
            println!("{}", template);
        }
    }
    Ok(())
}

/// 获取 MCP 工具完整描述（自描述能力增强）
fn get_mcp_tool_description() -> serde_json::Value {
    json!({
        "tool": "xore",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "高性能本地文件搜索与数据处理工具，通过计算下推和结构化摘要可降低 90%+ Token 消耗",
        "commands": {
            "find": "全文/语义文件搜索 (alias: f)",
            "process": "数据文件 SQL 查询与格式转换 (alias: p)",
            "agent schema": "获取数据结构（不读全文）",
            "agent sample": "智能采样数据",
            "agent query": "SQL 查询（计算下推）",
            "agent explain": "SQL 错误分析",
            "agent context": "会话上下文管理"
        },
        "error_handling": {
            "retryable_errors": ["文件锁冲突", "索引正在构建", "临时IO错误"],
            "max_retries": 2,
            "fatal_errors": ["文件不存在", "格式不支持", "SQL语法错误", "权限不足"],
            "exit_code_map": {
                "0": "成功",
                "1": "参数错误",
                "2": "文件不存在",
                "3": "SQL语法错误"
            }
        },
        "output_contract": {
            "json_common_fields": ["success", "data", "error", "suggestion", "token_saved"],
            "schema_output_fields": ["rows", "columns", "null_count", "duplicate_rows"],
            "sample_output_fields": ["total", "samples"],
            "query_output_fields": ["total", "rows", "execution_time_ms"]
        },
        "limits": {
            "max_file_size_mb": 100,
            "supported_formats": ["csv", "parquet", "json", "jsonl"],
            "unsupported_operations": ["修改文件内容", "执行系统命令"]
        },
        "token_estimate": {
            "schema": "50-200 token/次",
            "sample": "20 token/行",
            "query": "10 token/行 + 50 token固定开销"
        },
        "syntax_diff": {
            "agent query": "表名固定为 this",
            "process query": "表名为文件名（不含扩展名）"
        },
        "recommended_workflow": [
            "1. xore agent schema <file>  # 了解数据结构",
            "2. xore agent sample <file>  # 获取代表性数据",
            "3. xore agent query <file> \"SELECT ...\"  # 精确查询"
        ]
    })
}

/// 获取 OpenAI Tools Schema
fn get_openai_tools_schema() -> serde_json::Value {
    json!({
        "type": "function",
        "function": {
            "name": "xore",
            "description": "高性能本地文件搜索与数据处理工具，可降低90%+ Token消耗",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "enum": ["find", "process", "agent"],
                        "description": "要执行的命令"
                    },
                    "query": {
                        "type": "string",
                        "description": "搜索查询字符串"
                    },
                    "path": {
                        "type": "string",
                        "description": "搜索路径，默认为当前目录"
                    },
                    "file_type": {
                        "type": "string",
                        "description": "文件类型过滤"
                    },
                    "sql": {
                        "type": "string",
                        "description": "SQL 查询语句"
                    },
                    "file": {
                        "type": "string",
                        "description": "数据文件路径"
                    }
                },
                "required": ["command"]
            }
        }
    })
}

/// 获取 LangChain Tool 代码
fn get_langchain_tool_code(model: &str) -> String {
    format!(
        r#"# XORE LangChain Tools
# Target model: {model}
# Generated by: xore agent init --format langchain
#
# Install: pip install langchain pydantic
# Usage: import this file and add the tools to your agent

from __future__ import annotations
import subprocess
from typing import List, Optional
from langchain.tools import StructuredTool
from pydantic import BaseModel, Field


def _run_xore(*args: str) -> str:
    """Execute xore CLI and return stdout or stderr."""
    result = subprocess.run(["xore", *args], capture_output=True, text=True)
    return result.stdout if result.returncode == 0 else f"ERROR: {{result.stderr}}"


# ── Tool 1: File Search ───────────────────────────────────────────────────────

class XoreFindInput(BaseModel):
    query: str = Field(description="搜索关键词")
    path: str = Field(default=".", description="搜索路径（默认当前目录）")
    file_type: Optional[str] = Field(default=None, description="文件类型过滤，如 csv / json / code")
    output: str = Field(default="agent-json", description="输出格式: raw | json | agent-json | agent-md")
    max_tokens: Optional[int] = Field(default=None, description="限制输出 token 数量")

def xore_find(query: str, path: str = ".", file_type: Optional[str] = None,
              output: str = "agent-json", max_tokens: Optional[int] = None) -> str:
    args = ["find", query, "--path", path, "--output", output]
    if file_type:
        args += ["--type", file_type]
    if max_tokens:
        args += ["--max-tokens", str(max_tokens)]
    return _run_xore(*args)

xore_find_tool = StructuredTool.from_function(
    func=xore_find,
    name="xore_find",
    description="高性能本地文件搜索，支持全文检索和语义搜索，返回结构化结果以节省 Token",
    args_schema=XoreFindInput,
)


# ── Tool 2: Schema（数据结构）────────────────────────────────────────────────

class XoreSchemaInput(BaseModel):
    file: str = Field(description="数据文件路径（CSV / Parquet / JSON / JSONL）")

def xore_schema(file: str) -> str:
    return _run_xore("agent", "schema", file, "--json")

xore_schema_tool = StructuredTool.from_function(
    func=xore_schema,
    name="xore_schema",
    description="获取数据文件的列名、类型、缺失值等结构信息，不读取全部数据，极省 Token",
    args_schema=XoreSchemaInput,
)


# ── Tool 3: Sample（智能采样）───────────────────────────────────────────────

class XoreSampleInput(BaseModel):
    file: str = Field(description="数据文件路径")
    rows: int = Field(default=5, description="采样行数")
    strategy: str = Field(default="smart", description="采样策略: smart | head | tail | random")

def xore_sample(file: str, rows: int = 5, strategy: str = "smart") -> str:
    return _run_xore("agent", "sample", file, "-n", str(rows), "--strategy", strategy, "--json")

xore_sample_tool = StructuredTool.from_function(
    func=xore_sample,
    name="xore_sample",
    description="对数据文件进行智能采样，返回代表性数据行，适合快速了解数据内容",
    args_schema=XoreSampleInput,
)


# ── Tool 4: Query（SQL 查询）────────────────────────────────────────────────

class XoreQueryInput(BaseModel):
    file: str = Field(description="数据文件路径")
    sql: str = Field(description="SQL 查询语句，表名固定为 this，例如: SELECT * FROM this LIMIT 10")
    limit: Optional[int] = Field(default=100, description="最大返回行数")

def xore_query(file: str, sql: str, limit: int = 100) -> str:
    return _run_xore("agent", "query", file, sql, "--format", "json", "--limit", str(limit))

xore_query_tool = StructuredTool.from_function(
    func=xore_query,
    name="xore_query",
    description=(
        "对数据文件执行 SQL 查询（计算下推），表名固定为 this。"
        "示例: SELECT col, COUNT(*) FROM this GROUP BY col。"
        "适合大文件分析，仅返回查询结果，节省 90%+ Token"
    ),
    args_schema=XoreQueryInput,
)


# ── All Tools ────────────────────────────────────────────────────────────────

XORE_TOOLS: List[StructuredTool] = [
    xore_find_tool,
    xore_schema_tool,
    xore_sample_tool,
    xore_query_tool,
]

# Usage example:
# from langchain.agents import initialize_agent, AgentType
# from langchain_openai import ChatOpenAI
# llm = ChatOpenAI(model="{model}")
# agent = initialize_agent(XORE_TOOLS, llm, agent=AgentType.STRUCTURED_CHAT_ZERO_SHOT_REACT_DESCRIPTION)
# agent.run("分析 data.csv 文件的结构，然后查询前10行")
"#,
        model = model
    )
}

/// 获取 OpenAPI Spec
fn get_openapi_spec() -> serde_json::Value {
    json!({
        "openapi": "3.0.0",
        "info": {
            "title": "XORE API",
            "description": "高性能本地文件搜索与数据处理工具",
            "version": "1.1.0"
        },
        "paths": {
            "/find": {
                "get": {
                    "summary": "文件搜索",
                    "parameters": [
                        {"name": "query", "in": "query", "schema": {"type": "string"}},
                        {"name": "path", "in": "query", "schema": {"type": "string", "default": "."}}
                    ]
                }
            }
        }
    })
}

/// 获取 Prompt 模板
fn get_prompt_template(model: &str) -> Result<String> {
    let template = match model.to_lowercase().as_str() {
        "gpt-4" | "gpt-4o" | "gpt-3.5-turbo" | "openai" => {
            r#"# XORE Agent 初始化信息

你现在拥有 xore 工具，可用于高性能本地数据检索和分析。

## 核心命令

1. 获取数据结构（不读全文）：
   xore agent schema <file>           # 获取列名、类型、分布
   xore agent schema /logs/*.json     # 支持 glob 模式

2. 智能采样（获取代表性数据）：
   xore agent sample <file>           # 默认 5 行
   xore agent sample <file> -n 10     # 指定行数

3. 结构化查询（计算下推）：
   xore agent query <file> "SELECT * FROM this WHERE condition"
   xore p <file> "SQL" --format json  # JSON 输出

4. 全文/语义搜索：
   xore f "query" --index             # 全文搜索
   xore f "query" --semantic          # 语义搜索

## 使用原则

- 先 schema 了解结构，再 query 获取结果
- 大文件用 query 过滤，不要 cat/grep 全文
- 搜索用 --index 建立索引，重复搜索更快
- 复杂分析用 SQL，Polars 零拷贝处理 GB 级数据

## Token 节约示例

# 传统方式（读取 50MB 日志）
cat log.json | grep "error"  # 返回数千行，消耗 15000+ Tokens

# XORE 方式（计算下推）
xore agent query log.json "SELECT * FROM this WHERE status='error' LIMIT 1"
# 返回精简 JSON，消耗 <50 Tokens"#
        }
        "claude" | "claude-3" | "claude-sonnet" | "anthropic" => {
            r#"# XORE Agent 初始化信息

你现在拥有 xore 工具，可用于高性能本地数据检索和分析。

## 核心命令

1. 获取数据结构（不读全文）：
   <bash>xore agent schema <file></bash>

2. 智能采样（获取代表性数据）：
   <bash>xore agent sample <file></bash>

3. 结构化查询（计算下推）：
   <bash>xore agent query <file> "SELECT * FROM this WHERE condition"</bash>

4. 全文/语义搜索：
   <bash>xore f "query" --index</bash>

## 使用原则

- 先 schema 了解结构，再 query 获取结果
- 大文件用 query 过滤，不要 cat/grep 全文
- 搜索用 --index 建立索引，重复搜索更快

## Token 节约示例

传统方式：
<bash>cat log.json | grep "error"</bash>  # 消耗 15000+ Tokens

XORE 方式：
<bash>xore agent query log.json "SELECT * FROM this WHERE status='error' LIMIT 1"</bash>
# 消耗 <50 Tokens"#
        }
        _ => {
            return Err(anyhow::anyhow!(
                "不支持的模型: {}。支持的模型: gpt-4, claude, ollama, deepseek",
                model
            ));
        }
    };

    Ok(template.to_string())
}

/// 执行 schema 命令
fn execute_schema(file: &str, histogram: bool, json_output: bool, minify: bool, with_context: bool) -> Result<()> {
    let path = Path::new(file);

    if !path.exists() {
        return Err(anyhow::anyhow!("文件不存在: {}", file));
    }

    // 注入上下文（P1.3）
    if with_context {
        if let Ok(ctx) = SessionContext::default_session() {
            let context = ctx.get_context_for_injection();
            if !context.is_empty() {
                println!("{}\n", context);
            }
        }
    }

    let parser = DataParser::new();
    let df = parser.read(path).with_context(|| format!("无法读取文件: {:?}", path))?;

    let profiler = DataProfiler::new();
    let report = profiler.profile(&df)?;

    if json_output {
        let schema_json = build_schema_json(&df, &report, histogram)?;
        let output = if minify {
            serde_json::to_string(&schema_json)?
        } else {
            serde_json::to_string_pretty(&schema_json)?
        };
        println!("{}", output);
    } else {
        print_schema_text(&df, &report, histogram)?;
    }

    // 记录操作到会话上下文
    if let Ok(ctx) = SessionContext::default_session() {
        let summary = format!("{}行 {}列", df.height(), df.width());
        let op = ContextOperation::new("schema", Some(file.to_string()), None, summary);
        let _ = ctx.add_operation(op);
    }

    Ok(())
}

/// 构建 Schema JSON
fn build_schema_json(
    df: &DataFrame,
    report: &xore_process::QualityReport,
    _histogram: bool,
) -> Result<Value> {
    let mut columns = Vec::new();

    for col_name in df.get_column_names() {
        let col_name_str = col_name.to_string();
        let column = df.column(&col_name_str)?;
        let series = column.as_materialized_series();
        let dtype = series.dtype();

        let col_info = json!({
            "name": col_name_str,
            "type": format!("{:?}", dtype),
            "nullable": series.null_count() > 0,
            "null_count": series.null_count(),
        });

        columns.push(col_info);
    }

    Ok(json!({
        "rows": df.height(),
        "columns": columns,
        "missing_values": report.missing_values.len(),
        "duplicate_rows": report.duplicate_rows,
    }))
}

/// 打印 Schema 文本格式
fn print_schema_text(
    df: &DataFrame,
    report: &xore_process::QualityReport,
    _histogram: bool,
) -> Result<()> {
    println!("{} {}\n", ICON_INFO, "数据结构信息".bold());

    println!("{}", "基本信息".cyan().bold());
    println!("  {} 总行数: {}", ICON_SUCCESS, df.height().to_string().yellow());
    println!("  {} 总列数: {}", ICON_SUCCESS, df.width().to_string().yellow());

    println!("\n{}", "列信息".cyan().bold());
    for col_name in df.get_column_names() {
        let col_name_str = col_name.to_string();
        let column = df.column(&col_name_str)?;
        let series = column.as_materialized_series();
        let dtype = series.dtype();
        let null_count = series.null_count();
        let nullable = if null_count > 0 {
            format!("({}% 缺失)", (null_count as f64 / df.height() as f64 * 100.0) as i32)
                .red()
                .to_string()
        } else {
            "".to_string()
        };

        println!("  - {}: {:?} {}", col_name_str.cyan(), dtype, nullable);
    }

    if report.duplicate_rows > 0 {
        println!("\n{} 检测到 {} 行重复数据", ICON_TIP, report.duplicate_rows.to_string().yellow());
    }

    Ok(())
}

/// 执行 sample 命令
fn execute_sample(file: &str, n: usize, strategy: SampleStrategy, json_output: bool, with_context: bool) -> Result<()> {
    let path = Path::new(file);

    if !path.exists() {
        return Err(anyhow::anyhow!("文件不存在: {}", file));
    }

    // 注入上下文（P1.3）
    if with_context {
        if let Ok(ctx) = SessionContext::default_session() {
            let context = ctx.get_context_for_injection();
            if !context.is_empty() {
                println!("{}\n", context);
            }
        }
    }

    let parser = DataParser::new();
    let df = parser.read(path).with_context(|| format!("无法读取文件: {:?}", path))?;

    let sampled_df = match strategy {
        SampleStrategy::Random => {
            // 简单随机采样
            let total_rows = df.height();
            if total_rows <= n {
                df.clone()
            } else {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                let mut indices: Vec<usize> = (0..total_rows).collect();
                indices.shuffle(&mut rng);
                indices.truncate(n);
                indices.sort_unstable();

                // 使用 slice 方法逐行提取
                let mut rows = Vec::new();
                for &idx in &indices {
                    rows.push(df.slice(idx as i64, 1));
                }

                // 合并所有行
                if rows.is_empty() {
                    df.head(Some(0))
                } else {
                    #[allow(unused_imports)]
                    use polars::prelude::*;
                    let mut result = rows[0].clone();
                    for row in rows.iter().skip(1) {
                        result.vstack_mut(row)?;
                    }
                    result
                }
            }
        }
        SampleStrategy::Head => df.head(Some(n)),
        SampleStrategy::Tail => df.tail(Some(n)),
        SampleStrategy::Smart => smart_sample(&df, n)?,
    };

    if json_output {
        let json_array = dataframe_to_json(&sampled_df)?;
        println!("{}", serde_json::to_string_pretty(&json_array)?);
    } else {
        print_dataframe_table(&sampled_df)?;
    }

    // 记录操作到会话上下文
    if let Ok(ctx) = SessionContext::default_session() {
        let summary = format!("采样 {} 行", sampled_df.height());
        let op = ContextOperation::new("sample", Some(file.to_string()), None, summary);
        let _ = ctx.add_operation(op);
    }

    Ok(())
}

/// 智能采样
fn smart_sample(df: &DataFrame, n: usize) -> Result<DataFrame> {
    let total_rows = df.height();
    if total_rows <= n {
        return Ok(df.clone());
    }

    // 均匀采样
    let step = (total_rows / n).max(1);
    let mut rows = Vec::new();

    for i in (0..total_rows).step_by(step).take(n) {
        rows.push(df.slice(i as i64, 1));
    }

    if rows.is_empty() {
        return Ok(df.head(Some(0)));
    }

    #[allow(unused_imports)]
    use polars::prelude::*;
    let mut result = rows[0].clone();
    for row in rows.iter().skip(1) {
        result.vstack_mut(row)?;
    }

    Ok(result)
}

/// DataFrame 转 JSON 数组
fn dataframe_to_json(df: &DataFrame) -> Result<Vec<Value>> {
    let mut result = Vec::new();
    let column_names: Vec<String> = df.get_column_names().iter().map(|s| s.to_string()).collect();

    for row_idx in 0..df.height() {
        let mut row_obj = serde_json::Map::new();

        for col_name in &column_names {
            let column = df.column(col_name)?;
            let value = column.get(row_idx)?;
            row_obj.insert(col_name.clone(), anyvalue_to_json(&value));
        }

        result.push(Value::Object(row_obj));
    }

    Ok(result)
}

/// AnyValue 转 JSON Value
fn anyvalue_to_json(val: &AnyValue) -> Value {
    match val {
        AnyValue::Null => Value::Null,
        AnyValue::Boolean(b) => Value::Bool(*b),
        AnyValue::Int8(i) => json!(*i),
        AnyValue::Int16(i) => json!(*i),
        AnyValue::Int32(i) => json!(*i),
        AnyValue::Int64(i) => json!(*i),
        AnyValue::UInt8(i) => json!(*i),
        AnyValue::UInt16(i) => json!(*i),
        AnyValue::UInt32(i) => json!(*i),
        AnyValue::UInt64(i) => json!(*i),
        AnyValue::Float32(f) => json!(*f),
        AnyValue::Float64(f) => json!(*f),
        AnyValue::String(s) => Value::String(s.to_string()),
        AnyValue::StringOwned(s) => Value::String(s.to_string()),
        _ => Value::String(format!("{:?}", val)),
    }
}

/// 打印 DataFrame 表格
fn print_dataframe_table(df: &DataFrame) -> Result<()> {
    let column_names: Vec<String> = df.get_column_names().iter().map(|s| s.to_string()).collect();

    let columns: Vec<Column> = column_names.iter().map(|name| Column::new(name)).collect();

    let mut table = Table::new(columns).with_style(TableStyle::Simple);

    for row_idx in 0..df.height() {
        let row_data: Vec<String> = column_names
            .iter()
            .map(|col_name| {
                df.column(col_name)
                    .ok()
                    .and_then(|col| col.get(row_idx).ok())
                    .map(|val| format_anyvalue(&val))
                    .unwrap_or_else(|| "null".to_string())
            })
            .collect();

        table.add_row(row_data);
    }

    print!("{}", table.render());
    Ok(())
}

/// 格式化 AnyValue
fn format_anyvalue(val: &AnyValue) -> String {
    match val {
        AnyValue::Null => "null".to_string(),
        AnyValue::Boolean(b) => b.to_string(),
        AnyValue::String(s) => s.to_string(),
        AnyValue::StringOwned(s) => s.to_string(),
        AnyValue::Int8(i) => i.to_string(),
        AnyValue::Int16(i) => i.to_string(),
        AnyValue::Int32(i) => i.to_string(),
        AnyValue::Int64(i) => i.to_string(),
        AnyValue::UInt8(i) => i.to_string(),
        AnyValue::UInt16(i) => i.to_string(),
        AnyValue::UInt32(i) => i.to_string(),
        AnyValue::UInt64(i) => i.to_string(),
        AnyValue::Float32(f) => format!("{:.2}", f),
        AnyValue::Float64(f) => format!("{:.2}", f),
        _ => format!("{:?}", val),
    }
}

/// 执行 query 命令
fn execute_query(
    file: &str,
    sql: &str,
    format: &str,
    minify: bool,
    limit: Option<usize>,
    with_context: bool,
) -> Result<()> {
    let path = Path::new(file);

    if !path.exists() {
        return Err(anyhow::anyhow!("文件不存在: {}", file));
    }

    // 注入上下文（P1.3）
    if with_context {
        if let Ok(ctx) = SessionContext::default_session() {
            let context = ctx.get_context_for_injection();
            if !context.is_empty() {
                println!("{}\n", context);
            }
        }
    }

    let mut engine = SqlEngine::new();

    // 始终注册为 "this"，与文档和 init 生成的提示词保持一致
    engine.register_table("this", path)?;

    // 执行 SQL
    let result_df = engine.execute(sql)?;

    // 应用 limit
    let limited_df = if let Some(limit) = limit {
        result_df.head(Some(limit))
    } else {
        result_df.head(Some(100)) // 默认限制 100 行
    };

    match format.to_lowercase().as_str() {
        "json" => {
            let json_array = dataframe_to_json(&limited_df)?;
            let output = if minify {
                serde_json::to_string(&json_array)?
            } else {
                serde_json::to_string_pretty(&json_array)?
            };
            println!("{}", output);
        }
        "csv" => {
            // 简单的 CSV 输出
            let column_names: Vec<String> =
                limited_df.get_column_names().iter().map(|s| s.to_string()).collect();
            println!("{}", column_names.join(","));

            for row_idx in 0..limited_df.height() {
                let row_data: Vec<String> = column_names
                    .iter()
                    .map(|col_name| {
                        limited_df
                            .column(col_name)
                            .ok()
                            .and_then(|col| col.get(row_idx).ok())
                            .map(|val| format_anyvalue(&val))
                            .unwrap_or_else(|| "".to_string())
                    })
                    .collect();
                println!("{}", row_data.join(","));
            }
        }
        _ => {
            print_dataframe_table(&limited_df)?;
        }
    }

    // 记录操作到会话上下文
    if let Ok(ctx) = SessionContext::default_session() {
        let summary = format!("返回 {} 行", limited_df.height());
        let op =
            ContextOperation::new("query", Some(file.to_string()), Some(sql.to_string()), summary);
        let _ = ctx.add_operation(op);
    }

    Ok(())
}

/// 执行 explain 命令
fn execute_explain(sql: &str) -> Result<()> {
    println!("{} {}\n", "❌".red(), "SQL 错误分析".bold());

    let suggestions = analyze_sql_error(sql);

    if suggestions.is_empty() {
        println!("原始 SQL: {}", sql.yellow());
        println!("\n{} 未检测到明显错误", ICON_INFO);
        println!("{} 请检查表名和列名是否正确", ICON_TIP);
    } else {
        for suggestion in suggestions {
            println!("{}", suggestion);
        }
    }

    Ok(())
}

/// 执行 context 命令（P1.2）
fn execute_context(subcommand: ContextSubcommand) -> Result<()> {
    match subcommand {
        ContextSubcommand::Get { level, session_id } => {
            let ctx = load_session(&session_id)?;
            println!("{}", ctx.get_summary(&level));
        }
        ContextSubcommand::Clear { session_id } => {
            let ctx = load_session(&session_id)?;
            let count = ctx.clear()?;
            println!("{} 已清空会话 {} 中的 {} 条操作记录", ICON_SUCCESS, session_id, count);
        }
        ContextSubcommand::Export { session_id } => {
            let ctx = load_session(&session_id)?;
            let data = ctx.export()?;
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        ContextSubcommand::Set { custom, session_id } => {
            let ctx = load_session(&session_id)?;
            ctx.set_custom(&custom)?;
            println!("{} 已设置会话 {} 的自定义上下文", ICON_SUCCESS, session_id);
        }
    }
    Ok(())
}

/// 加载指定会话（若为 "default" 则使用默认会话）
fn load_session(session_id: &str) -> Result<SessionContext> {
    use xore_core::get_default_sessions_dir;
    SessionContext::load_or_create(session_id, get_default_sessions_dir())
}

/// 分析 SQL 错误
fn analyze_sql_error(sql: &str) -> Vec<String> {
    let mut suggestions = Vec::new();

    let common_errors = vec![
        ("FORM", "FROM", "FORM 应该是 FROM"),
        ("form", "from", "form 应该是 from"),
        ("WHRER", "WHERE", "WHRER 应该是 WHERE"),
        ("whrer", "where", "whrer 应该是 where"),
        ("SELEC", "SELECT", "SELEC 应该是 SELECT"),
        ("selec", "select", "selec 应该是 select"),
        ("GROPU BY", "GROUP BY", "GROPU BY 应该是 GROUP BY"),
        ("ODER BY", "ORDER BY", "ODER BY 应该是 ORDER BY"),
    ];

    // 检测是否为独立拼写错误（typo 出现但其对应的正确关键字不存在）
    fn is_typo(sql: &str, wrong: &str, correct: &str) -> bool {
        let sql_upper = sql.to_uppercase();
        let wrong_upper = wrong.to_uppercase();
        let correct_upper = correct.to_uppercase();
        sql_upper.contains(&wrong_upper) && !sql_upper.contains(&correct_upper)
    }

    for (wrong, correct, hint) in common_errors {
        if is_typo(sql, wrong, correct) {
            suggestions.push(format!("原始 SQL: {}", sql.yellow()));
            suggestions.push(format!("         {}", "^".repeat(wrong.len()).red()));
            suggestions.push(format!("错误：{}\n", hint.red()));

            let fixed_sql = sql.replace(wrong, correct);
            suggestions.push("修正建议：".to_string());
            suggestions.push(format!("  {}\n", fixed_sql.green()));
            break;
        }
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_prompt_template() {
        let template = get_prompt_template("gpt-4").unwrap();
        assert!(template.contains("XORE Agent"));
        assert!(template.contains("xore agent schema"));
    }

    #[test]
    fn test_analyze_sql_error() {
        let sql = "SELECT * FORM users WHERE id = 1";
        let suggestions = analyze_sql_error(sql);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("FROM")));

        // 合法的 SELECT 不应被误报为 SELEC 拼写错误
        let valid_sql = "SELECT * FROM users";
        let no_suggestions = analyze_sql_error(valid_sql);
        assert!(no_suggestions.is_empty(), "合法 SELECT 不应触发 SELEC 拼写错误提示");
    }

    #[test]
    fn test_sample_strategy_from_str() {
        assert!(matches!("random".parse::<SampleStrategy>().unwrap(), SampleStrategy::Random));
        assert!(matches!("head".parse::<SampleStrategy>().unwrap(), SampleStrategy::Head));
    }

    // P1.1: 自描述能力增强测试
    #[test]
    fn test_mcp_tool_description_has_required_fields() {
        let desc = get_mcp_tool_description();

        assert!(desc.get("tool").is_some(), "应包含 tool 字段");
        assert!(desc.get("version").is_some(), "应包含 version 字段");
        assert!(desc.get("error_handling").is_some(), "应包含 error_handling 字段");
        assert!(desc.get("output_contract").is_some(), "应包含 output_contract 字段");
        assert!(desc.get("limits").is_some(), "应包含 limits 字段");
        assert!(desc.get("token_estimate").is_some(), "应包含 token_estimate 字段");
        assert!(desc.get("syntax_diff").is_some(), "应包含 syntax_diff 字段");
    }

    #[test]
    fn test_mcp_tool_description_error_handling() {
        let desc = get_mcp_tool_description();
        let error_handling = &desc["error_handling"];

        assert!(error_handling.get("retryable_errors").is_some());
        assert!(error_handling.get("max_retries").is_some());
        assert!(error_handling.get("fatal_errors").is_some());
        assert!(error_handling.get("exit_code_map").is_some());
        assert_eq!(error_handling["max_retries"], 2);
    }

    #[test]
    fn test_mcp_tool_description_output_contract() {
        let desc = get_mcp_tool_description();
        let contract = &desc["output_contract"];

        assert!(contract.get("json_common_fields").is_some());
        assert!(contract.get("schema_output_fields").is_some());
        assert!(contract.get("sample_output_fields").is_some());
        assert!(contract.get("query_output_fields").is_some());
    }

    #[test]
    fn test_mcp_tool_description_limits() {
        let desc = get_mcp_tool_description();
        let limits = &desc["limits"];

        assert_eq!(limits["max_file_size_mb"], 100);
        let formats = limits["supported_formats"].as_array().unwrap();
        assert!(formats.iter().any(|f| f == "csv"));
        assert!(formats.iter().any(|f| f == "parquet"));
    }

    #[test]
    fn test_mcp_tool_description_syntax_diff() {
        let desc = get_mcp_tool_description();
        let syntax = &desc["syntax_diff"];

        assert!(syntax.get("agent query").is_some());
        assert!(syntax.get("process query").is_some());
        // agent query 应使用 "this" 作为表名
        assert!(syntax["agent query"].as_str().unwrap().contains("this"));
    }

    #[test]
    fn test_mcp_tool_description_is_valid_json() {
        let desc = get_mcp_tool_description();
        // 确保可以序列化/反序列化
        let serialized = serde_json::to_string(&desc).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(desc, reparsed);
    }

    // P1.4: LangChain Tool 格式测试
    #[test]
    fn test_langchain_tool_code_contains_required_tools() {
        let code = get_langchain_tool_code("gpt-4");

        assert!(code.contains("xore_find_tool"), "应包含 find 工具");
        assert!(code.contains("xore_schema_tool"), "应包含 schema 工具");
        assert!(code.contains("xore_sample_tool"), "应包含 sample 工具");
        assert!(code.contains("xore_query_tool"), "应包含 query 工具");
        assert!(code.contains("XORE_TOOLS"), "应包含工具列表");
    }

    #[test]
    fn test_langchain_tool_code_contains_pydantic_models() {
        let code = get_langchain_tool_code("claude");

        assert!(code.contains("XoreFindInput"), "应包含 FindInput 模型");
        assert!(code.contains("XoreSchemaInput"), "应包含 SchemaInput 模型");
        assert!(code.contains("XoreSampleInput"), "应包含 SampleInput 模型");
        assert!(code.contains("XoreQueryInput"), "应包含 QueryInput 模型");
        assert!(code.contains("BaseModel"), "应使用 Pydantic BaseModel");
    }

    #[test]
    fn test_langchain_tool_code_includes_model_name() {
        let code = get_langchain_tool_code("gpt-4o");
        assert!(code.contains("gpt-4o"), "应包含目标模型名称");
    }

    #[test]
    fn test_openai_tools_schema_structure() {
        let schema = get_openai_tools_schema();
        assert_eq!(schema["type"], "function");
        assert!(schema.get("function").is_some());
        assert!(schema["function"].get("name").is_some());
        assert!(schema["function"].get("parameters").is_some());
    }

    #[test]
    fn test_execute_init_dispatch() {
        // MCP 格式应返回结构化 JSON（不会报错）
        assert!(execute_init("gpt-4", "mcp").is_ok());
        assert!(execute_init("claude", "openai").is_ok());
        assert!(execute_init("gpt-4", "langchain").is_ok());
        assert!(execute_init("gpt-4", "openapi").is_ok());
    }
}
