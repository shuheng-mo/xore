//! Agent 命令实现
//!
//! 提供 Agent-Native 优化功能，包括初始化、Schema 获取、智能采样和查询。

use anyhow::{Context, Result};
use colored::*;
use serde_json::{json, Value};
use std::path::Path;
use xore_process::{AnyValue, DataFrame, DataParser, DataProfiler, SqlEngine};

use crate::ui::{Column, Table, TableStyle, ICON_INFO, ICON_SUCCESS, ICON_TIP};

/// Agent 子命令参数
pub struct AgentArgs {
    pub subcommand: AgentSubcommand,
}

/// Agent 子命令枚举
pub enum AgentSubcommand {
    Init { model: String },
    Schema { file: String, histogram: bool, json: bool, minify: bool },
    Sample { file: String, n: usize, strategy: SampleStrategy, json: bool },
    Query { file: String, sql: String, format: String, minify: bool, limit: Option<usize> },
    Explain { sql: String },
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
        AgentSubcommand::Init { model } => execute_init(&model),
        AgentSubcommand::Schema { file, histogram, json, minify } => {
            execute_schema(&file, histogram, json, minify)
        }
        AgentSubcommand::Sample { file, n, strategy, json } => {
            execute_sample(&file, n, strategy, json)
        }
        AgentSubcommand::Query { file, sql, format, minify, limit } => {
            execute_query(&file, &sql, &format, minify, limit)
        }
        AgentSubcommand::Explain { sql } => execute_explain(&sql),
    }
}

/// 执行 init 命令
fn execute_init(model: &str) -> Result<()> {
    let template = get_prompt_template(model)?;
    println!("{}", template);
    Ok(())
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
fn execute_schema(file: &str, histogram: bool, json_output: bool, minify: bool) -> Result<()> {
    let path = Path::new(file);

    if !path.exists() {
        return Err(anyhow::anyhow!("文件不存在: {}", file));
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
fn execute_sample(file: &str, n: usize, strategy: SampleStrategy, json_output: bool) -> Result<()> {
    let path = Path::new(file);

    if !path.exists() {
        return Err(anyhow::anyhow!("文件不存在: {}", file));
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
) -> Result<()> {
    let path = Path::new(file);

    if !path.exists() {
        return Err(anyhow::anyhow!("文件不存在: {}", file));
    }

    let mut engine = SqlEngine::new();
    let table_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("this");

    engine.register_table(table_name, path)?;

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

    for (wrong, correct, hint) in common_errors {
        if sql.contains(wrong) {
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
    }

    #[test]
    fn test_sample_strategy_from_str() {
        assert!(matches!("random".parse::<SampleStrategy>().unwrap(), SampleStrategy::Random));
        assert!(matches!("head".parse::<SampleStrategy>().unwrap(), SampleStrategy::Head));
    }
}
