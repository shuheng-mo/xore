//! 错误转换：anyhow::Error → rmcp::ErrorData

use rmcp::ErrorData;

/// 将任意 anyhow 错误转换为 MCP 内部错误。
/// 错误链（通过 `.context()` 添加的信息）会序列化进 MCP 错误消息。
pub fn into_mcp_error(err: anyhow::Error) -> ErrorData {
    ErrorData::internal_error(err.to_string(), None)
}
