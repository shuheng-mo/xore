//! 会话上下文模块
//!
//! 提供 Agent 多轮操作时的上下文压缩和会话管理功能。
//! 解决智能体多轮操作时上下文膨胀问题。

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

/// 单次操作记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextOperation {
    /// 执行的命令（如 "schema", "sample", "query", "find"）
    pub command: String,
    /// 操作的文件路径（可选）
    pub file: Option<String>,
    /// 执行的 SQL（可选）
    pub sql: Option<String>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 结果摘要（如行数、匹配数等）
    pub result_summary: String,
}

impl ContextOperation {
    /// 创建新的操作记录
    pub fn new(
        command: impl Into<String>,
        file: Option<String>,
        sql: Option<String>,
        result_summary: impl Into<String>,
    ) -> Self {
        Self {
            command: command.into(),
            file,
            sql,
            timestamp: Utc::now(),
            result_summary: result_summary.into(),
        }
    }
}

/// 会话上下文数据（可序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextData {
    /// 会话 ID
    pub session_id: String,
    /// 操作历史列表
    pub operations: Vec<ContextOperation>,
    /// 自定义注入的上下文文本
    pub custom_context: Option<String>,
    /// 会话创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

impl ContextData {
    fn new(session_id: &str) -> Self {
        let now = Utc::now();
        Self {
            session_id: session_id.to_string(),
            operations: Vec::new(),
            custom_context: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 会话上下文管理器
pub struct SessionContext {
    session_id: String,
    storage_path: PathBuf,
    data: Mutex<ContextData>,
}

impl SessionContext {
    /// 从磁盘加载会话，若不存在则创建新会话
    ///
    /// # Arguments
    /// * `session_id` - 会话 ID（默认 "default"）
    /// * `sessions_dir` - 会话存储目录（默认 `~/.xore/sessions`）
    pub fn load_or_create(session_id: &str, sessions_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&sessions_dir)
            .with_context(|| format!("无法创建会话目录: {:?}", sessions_dir))?;

        let storage_path = sessions_dir.join(format!("{}.json", session_id));

        let data = if storage_path.exists() {
            let content = std::fs::read_to_string(&storage_path)
                .with_context(|| format!("无法读取会话文件: {:?}", storage_path))?;
            serde_json::from_str::<ContextData>(&content)
                .unwrap_or_else(|_| ContextData::new(session_id))
        } else {
            ContextData::new(session_id)
        };

        Ok(Self { session_id: session_id.to_string(), storage_path, data: Mutex::new(data) })
    }

    /// 获取默认会话（使用当前目录哈希作为会话 ID）
    pub fn default_session() -> Result<Self> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let sessions_dir = home.join(".xore").join("sessions");
        Self::load_or_create("default", sessions_dir)
    }

    /// 记录一次操作
    pub fn add_operation(&self, op: ContextOperation) -> Result<()> {
        let mut data = self.data.lock().unwrap();
        data.operations.push(op);
        data.updated_at = Utc::now();

        // 限制最大记录数
        const MAX_OPERATIONS: usize = 1000;
        if data.operations.len() > MAX_OPERATIONS {
            let remove = data.operations.len() - MAX_OPERATIONS;
            data.operations.drain(0..remove);
        }

        self.persist(&data)
    }

    /// 获取会话摘要
    ///
    /// # Arguments
    /// * `level` - "short" 仅显示统计，"detailed" 显示完整操作列表
    pub fn get_summary(&self, level: &str) -> String {
        let data = self.data.lock().unwrap();

        if data.operations.is_empty() && data.custom_context.is_none() {
            return format!("[会话 {}] 暂无操作记录", self.session_id);
        }

        let mut parts = Vec::new();
        parts.push(format!("会话 ID: {}", self.session_id));
        parts.push(format!("操作数: {}", data.operations.len()));

        if let Some(ref custom) = data.custom_context {
            parts.push(format!("自定义上下文: {}", custom));
        }

        if level == "detailed" && !data.operations.is_empty() {
            parts.push("操作历史:".to_string());
            for op in &data.operations {
                let file_info = op.file.as_deref().map(|f| format!(" [{}]", f)).unwrap_or_default();
                let sql_info = op
                    .sql
                    .as_deref()
                    .map(|s| {
                        let truncated = if s.len() > 60 { &s[..60] } else { s };
                        format!(" SQL: {}", truncated)
                    })
                    .unwrap_or_default();
                parts.push(format!(
                    "  - [{}] {}{}{} → {}",
                    op.timestamp.format("%H:%M:%S"),
                    op.command,
                    file_info,
                    sql_info,
                    op.result_summary
                ));
            }
        } else if !data.operations.is_empty() {
            // 短模式：仅显示最近的操作
            let recent = data.operations.iter().rev().take(3).collect::<Vec<_>>();
            parts.push("最近操作:".to_string());
            for op in recent.iter().rev() {
                let file_info = op.file.as_deref().map(|f| format!(" [{}]", f)).unwrap_or_default();
                parts.push(format!("  - {} {}: {}", op.command, file_info, op.result_summary));
            }
        }

        parts.join("\n")
    }

    /// 清空会话
    pub fn clear(&self) -> Result<usize> {
        let mut data = self.data.lock().unwrap();
        let count = data.operations.len();
        data.operations.clear();
        data.custom_context = None;
        data.updated_at = Utc::now();

        // 删除会话文件
        if self.storage_path.exists() {
            std::fs::remove_file(&self.storage_path)
                .with_context(|| format!("无法删除会话文件: {:?}", self.storage_path))?;
        }

        Ok(count)
    }

    /// 导出会话为结构化 JSON
    pub fn export(&self) -> Result<serde_json::Value> {
        let data = self.data.lock().unwrap();
        let json = serde_json::to_value(&*data).with_context(|| "无法序列化会话数据")?;
        Ok(json)
    }

    /// 设置自定义上下文文本
    pub fn set_custom(&self, text: &str) -> Result<()> {
        let mut data = self.data.lock().unwrap();
        data.custom_context = if text.is_empty() { None } else { Some(text.to_string()) };
        data.updated_at = Utc::now();
        self.persist(&data)
    }

    /// 获取用于注入的上下文字符串（用于 --with-context）
    pub fn get_context_for_injection(&self) -> String {
        let data = self.data.lock().unwrap();

        if data.operations.is_empty() && data.custom_context.is_none() {
            return String::new();
        }

        let mut lines = vec!["# 当前会话上下文".to_string()];

        if let Some(ref custom) = data.custom_context {
            lines.push(format!("## 自定义上下文\n{}", custom));
        }

        if !data.operations.is_empty() {
            lines.push("## 已执行操作".to_string());
            for op in data.operations.iter().rev().take(5) {
                let file_info =
                    op.file.as_deref().map(|f| format!(" 文件={}", f)).unwrap_or_default();
                lines.push(format!("- {}{}: {}", op.command, file_info, op.result_summary));
            }
        }

        lines.join("\n")
    }

    /// 获取当前会话 ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// 获取操作数量
    pub fn operation_count(&self) -> usize {
        self.data.lock().unwrap().operations.len()
    }

    /// 将数据持久化到磁盘
    fn persist(&self, data: &ContextData) -> Result<()> {
        let content = serde_json::to_string_pretty(data).with_context(|| "无法序列化会话数据")?;
        std::fs::write(&self.storage_path, content)
            .with_context(|| format!("无法写入会话文件: {:?}", self.storage_path))?;
        Ok(())
    }
}

/// 获取默认会话目录路径
pub fn get_default_sessions_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".xore").join("sessions")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_context(temp_dir: &TempDir) -> SessionContext {
        SessionContext::load_or_create("test", temp_dir.path().to_path_buf()).unwrap()
    }

    #[test]
    fn test_create_new_session() {
        let temp = TempDir::new().unwrap();
        let ctx = create_test_context(&temp);
        assert_eq!(ctx.session_id(), "test");
        assert_eq!(ctx.operation_count(), 0);
    }

    #[test]
    fn test_add_operation() {
        let temp = TempDir::new().unwrap();
        let ctx = create_test_context(&temp);

        let op = ContextOperation::new("schema", Some("data.csv".to_string()), None, "10行 3列");
        ctx.add_operation(op).unwrap();

        assert_eq!(ctx.operation_count(), 1);
    }

    #[test]
    fn test_get_summary_short() {
        let temp = TempDir::new().unwrap();
        let ctx = create_test_context(&temp);

        let op = ContextOperation::new("schema", Some("test.csv".to_string()), None, "5行");
        ctx.add_operation(op).unwrap();

        let summary = ctx.get_summary("short");
        assert!(summary.contains("test"));
        assert!(summary.contains("1")); // 1 个操作
    }

    #[test]
    fn test_get_summary_detailed() {
        let temp = TempDir::new().unwrap();
        let ctx = create_test_context(&temp);

        let op1 = ContextOperation::new("schema", Some("data.csv".to_string()), None, "10行 3列");
        let op2 = ContextOperation::new(
            "query",
            Some("data.csv".to_string()),
            Some("SELECT * FROM this LIMIT 5".to_string()),
            "5行结果",
        );
        ctx.add_operation(op1).unwrap();
        ctx.add_operation(op2).unwrap();

        let summary = ctx.get_summary("detailed");
        assert!(summary.contains("schema"));
        assert!(summary.contains("query"));
        assert!(summary.contains("操作历史"));
    }

    #[test]
    fn test_clear_session() {
        let temp = TempDir::new().unwrap();
        let ctx = create_test_context(&temp);

        let op = ContextOperation::new("schema", Some("data.csv".to_string()), None, "5行");
        ctx.add_operation(op).unwrap();
        assert_eq!(ctx.operation_count(), 1);

        let cleared = ctx.clear().unwrap();
        assert_eq!(cleared, 1);
        assert_eq!(ctx.operation_count(), 0);
    }

    #[test]
    fn test_export_session() {
        let temp = TempDir::new().unwrap();
        let ctx = create_test_context(&temp);

        let op = ContextOperation::new("query", Some("data.csv".to_string()), None, "3行");
        ctx.add_operation(op).unwrap();

        let exported = ctx.export().unwrap();
        assert_eq!(exported["session_id"], "test");
        assert!(exported["operations"].is_array());
        assert_eq!(exported["operations"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_set_custom_context() {
        let temp = TempDir::new().unwrap();
        let ctx = create_test_context(&temp);

        ctx.set_custom("这是一个销售数据分析任务").unwrap();

        let summary = ctx.get_summary("short");
        assert!(summary.contains("销售数据分析"));
    }

    #[test]
    fn test_get_context_for_injection() {
        let temp = TempDir::new().unwrap();
        let ctx = create_test_context(&temp);

        // 空会话时返回空字符串
        assert!(ctx.get_context_for_injection().is_empty());

        // 添加操作后应有内容
        let op = ContextOperation::new("schema", Some("data.csv".to_string()), None, "10行");
        ctx.add_operation(op).unwrap();

        let injection = ctx.get_context_for_injection();
        assert!(injection.contains("当前会话上下文"));
        assert!(injection.contains("schema"));
    }

    #[test]
    fn test_session_persistence() {
        let temp = TempDir::new().unwrap();

        // 创建并写入
        {
            let ctx = create_test_context(&temp);
            let op = ContextOperation::new("schema", Some("test.csv".to_string()), None, "5行 2列");
            ctx.add_operation(op).unwrap();
        }

        // 重新加载
        let ctx2 = create_test_context(&temp);
        assert_eq!(ctx2.operation_count(), 1);
    }

    #[test]
    fn test_max_operations_limit() {
        let temp = TempDir::new().unwrap();
        let ctx = create_test_context(&temp);

        // 添加超过限制的操作数
        for i in 0..1010usize {
            let op = ContextOperation::new(
                "query",
                Some("data.csv".to_string()),
                None,
                format!("操作{}", i),
            );
            ctx.add_operation(op).unwrap();
        }

        // 应该被截断到最大数量
        assert!(ctx.operation_count() <= 1000);
    }
}
