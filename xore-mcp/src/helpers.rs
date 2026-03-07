//! 共用工具函数
//!
//! 移植自 xore-cli/src/commands/agent.rs，供各 MCP 工具共享。

use anyhow::Result;
use polars::prelude::{AnyValue, DataFrame};
use rmcp::{model::CallToolResult, ErrorData as McpError};
use serde_json::{json, Value};

use crate::error::into_mcp_error;

/// DataFrame 转 JSON 数组（每行一个对象）
pub fn dataframe_to_json(df: &DataFrame) -> Result<Vec<Value>> {
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

/// Polars AnyValue 转 serde_json::Value
pub fn anyvalue_to_json(val: &AnyValue) -> Value {
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

/// 均匀分布采样（smart 模式）
pub fn smart_sample(df: &DataFrame, n: usize) -> Result<DataFrame> {
    let total_rows = df.height();
    if total_rows <= n {
        return Ok(df.clone());
    }

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

/// 去除 Tantivy snippet 中注入的 ANSI 转义序列
pub fn strip_ansi(s: &str) -> String {
    // 移除加粗高亮 \x1b[1;33m 和重置 \x1b[0m
    s.replace("\x1b[1;33m", "").replace("\x1b[0m", "")
}

/// 将 JSON 值序列化为美化文本并包装成成功的 CallToolResult
pub fn text_result(value: Value) -> Result<CallToolResult, McpError> {
    let text = serde_json::to_string_pretty(&value)
        .map_err(|e| into_mcp_error(anyhow::anyhow!("JSON 序列化失败: {}", e)))?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(text)]))
}
