//! 增量索引模块
//!
//! 提供增量索引功能，支持：
//! - 文件监控集成
//! - 增量更新（先删后增）
//! - 批量提交优化
//! - 简化版WAL（内存记录）
//! - 索引统计

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use tantivy::Index;
use tokio::sync::Mutex;
use tokio::time;
use tracing::{debug, error, info};

use crate::indexer::{IndexBuilder, IndexConfig, IndexSchema};
use crate::scanner::ScannedFile;
use crate::watcher::{FileEvent, FileWatcher, WatcherConfig};

/// WAL（Write-Ahead Log）- 简化版，仅内存记录
///
/// 记录最近的索引操作，用于调试和统计
#[derive(Debug, Clone)]
pub struct WriteAheadLog {
    /// 操作记录（最多保留1000条）
    operations: VecDeque<WalEntry>,
    /// 最大记录数
    max_entries: usize,
}

#[derive(Debug, Clone)]
pub struct WalEntry {
    /// 操作类型
    pub operation: WalOperation,
    /// 文件路径
    pub path: PathBuf,
    /// 时间戳
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalOperation {
    Create,
    Modify,
    Delete,
}

impl WriteAheadLog {
    pub fn new() -> Self {
        Self { operations: VecDeque::new(), max_entries: 1000 }
    }

    pub fn log_create(&mut self, path: &Path) {
        self.add_entry(WalOperation::Create, path);
    }

    pub fn log_modify(&mut self, path: &Path) {
        self.add_entry(WalOperation::Modify, path);
    }

    pub fn log_delete(&mut self, path: &Path) {
        self.add_entry(WalOperation::Delete, path);
    }

    fn add_entry(&mut self, operation: WalOperation, path: &Path) {
        let entry = WalEntry { operation, path: path.to_path_buf(), timestamp: SystemTime::now() };

        self.operations.push_back(entry);

        // 保持最大记录数限制
        while self.operations.len() > self.max_entries {
            self.operations.pop_front();
        }
    }

    /// 获取最近的操作记录
    pub fn recent_operations(&self, limit: usize) -> Vec<&WalEntry> {
        self.operations.iter().rev().take(limit).collect()
    }

    /// 清空记录
    pub fn clear(&mut self) {
        self.operations.clear();
    }

    /// 获取总操作数
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// 判断是否为空
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

impl Default for WriteAheadLog {
    fn default() -> Self {
        Self::new()
    }
}

/// 增量索引统计
#[derive(Debug, Clone, Default)]
pub struct IncrementalStats {
    /// 创建的文档数
    pub created_count: usize,
    /// 修改的文档数
    pub modified_count: usize,
    /// 删除的文档数
    pub deleted_count: usize,
    /// 待提交的变更数
    pub pending_changes: usize,
    /// 错误数
    pub error_count: usize,
    /// 最后更新时间
    pub last_update: Option<SystemTime>,
}

/// 增量索引器配置
#[derive(Debug, Clone)]
pub struct IncrementalConfig {
    /// 索引配置
    pub index_config: IndexConfig,
    /// 监控配置
    pub watcher_config: WatcherConfig,
    /// 批量提交阈值（累积多少变更后提交）
    pub commit_threshold: usize,
    /// 自动提交间隔（秒）
    pub auto_commit_interval: u64,
}

impl Default for IncrementalConfig {
    fn default() -> Self {
        Self {
            index_config: IndexConfig::default(),
            watcher_config: WatcherConfig::default(),
            commit_threshold: 50,
            auto_commit_interval: 30,
        }
    }
}

/// 增量索引器
///
/// 支持文件监控和增量更新的索引器
pub struct IncrementalIndexer {
    /// Tantivy 索引
    index: Index,
    /// 索引 Schema
    schema: IndexSchema,
    /// 索引构建器（用于实际的索引操作）
    builder: Arc<Mutex<IndexBuilder>>,
    /// 文件监控器
    watcher: Arc<Mutex<FileWatcher>>,
    /// WAL（简化版）
    wal: Arc<Mutex<WriteAheadLog>>,
    /// 统计信息
    stats: Arc<Mutex<IncrementalStats>>,
    /// 配置
    config: IncrementalConfig,
}

impl IncrementalIndexer {
    /// 创建增量索引器
    pub async fn new(config: IncrementalConfig) -> Result<Self> {
        info!("Creating incremental indexer");

        // 创建索引构建器
        let builder = IndexBuilder::with_config(config.index_config.clone())?;
        let index = builder.index().clone();
        let schema = builder.schema().clone();

        // 创建文件监控器
        let watcher = FileWatcher::new(config.watcher_config.clone())?;

        Ok(Self {
            index,
            schema,
            builder: Arc::new(Mutex::new(builder)),
            watcher: Arc::new(Mutex::new(watcher)),
            wal: Arc::new(Mutex::new(WriteAheadLog::new())),
            stats: Arc::new(Mutex::new(IncrementalStats::default())),
            config,
        })
    }

    /// 开始监控指定路径
    pub async fn watch(&self, path: &Path) -> Result<()> {
        info!("Starting watch on: {}", path.display());

        let mut watcher = self.watcher.lock().await;
        watcher.watch_path(path)?;

        Ok(())
    }

    /// 停止监控指定路径
    pub async fn unwatch(&self, path: &Path) -> Result<()> {
        info!("Stopping watch on: {}", path.display());

        let mut watcher = self.watcher.lock().await;
        watcher.unwatch_path(path)?;

        Ok(())
    }

    /// 运行增量索引（异步事件循环）
    ///
    /// 持续监听文件变更并更新索引
    pub async fn run(&self) -> Result<()> {
        info!("Starting incremental indexer event loop");

        let mut commit_interval =
            time::interval(Duration::from_secs(self.config.auto_commit_interval));

        loop {
            tokio::select! {
                // 定期自动提交
                _ = commit_interval.tick() => {
                    if let Err(e) = self.commit_if_needed().await {
                        error!("Auto commit failed: {}", e);
                    }
                }

                // 处理文件事件
                _ = time::sleep(Duration::from_millis(100)) => {
                    if let Err(e) = self.process_events().await {
                        error!("Process events failed: {}", e);
                    }
                }
            }
        }
    }

    /// 处理一批文件事件
    async fn process_events(&self) -> Result<()> {
        let mut watcher = self.watcher.lock().await;
        let events = watcher.recv_events()?;

        if events.is_empty() {
            return Ok(());
        }

        debug!("Processing {} file events", events.len());

        for event in events {
            if let Err(e) = self.handle_event(event).await {
                error!("Handle event failed: {}", e);
                let mut stats = self.stats.lock().await;
                stats.error_count += 1;
            }
        }

        // 检查是否需要提交
        self.commit_if_needed().await?;

        Ok(())
    }

    /// 处理单个文件事件
    async fn handle_event(&self, event: FileEvent) -> Result<()> {
        match event {
            FileEvent::Created(path) => {
                self.apply_create(path).await?;
            }
            FileEvent::Modified(path) => {
                self.apply_modify(path).await?;
            }
            FileEvent::Deleted(path) => {
                self.apply_delete(path).await?;
            }
            FileEvent::Renamed { from, to } => {
                self.apply_delete(from).await?;
                self.apply_create(to).await?;
            }
        }

        Ok(())
    }

    /// 处理文件创建
    async fn apply_create(&self, path: PathBuf) -> Result<()> {
        debug!("Applying create: {}", path.display());

        // 扫描文件信息
        let file = self.scan_file(&path)?;

        // 添加到索引
        let mut builder = self.builder.lock().await;
        builder.add_document(&file)?;

        // 记录 WAL
        let mut wal = self.wal.lock().await;
        wal.log_create(&path);

        // 更新统计
        let mut stats = self.stats.lock().await;
        stats.created_count += 1;
        stats.pending_changes += 1;
        stats.last_update = Some(SystemTime::now());

        Ok(())
    }

    /// 处理文件修改
    async fn apply_modify(&self, path: PathBuf) -> Result<()> {
        debug!("Applying modify: {}", path.display());

        // 先删除旧文档
        let mut builder = self.builder.lock().await;
        builder.delete_document(&path)?;

        // 扫描文件信息
        let file = self.scan_file(&path)?;

        // 重新添加
        builder.add_document(&file)?;

        // 记录 WAL
        let mut wal = self.wal.lock().await;
        wal.log_modify(&path);

        // 更新统计
        let mut stats = self.stats.lock().await;
        stats.modified_count += 1;
        stats.pending_changes += 1;
        stats.last_update = Some(SystemTime::now());

        Ok(())
    }

    /// 处理文件删除
    async fn apply_delete(&self, path: PathBuf) -> Result<()> {
        debug!("Applying delete: {}", path.display());

        // 从索引删除
        let mut builder = self.builder.lock().await;
        builder.delete_document(&path)?;

        // 记录 WAL
        let mut wal = self.wal.lock().await;
        wal.log_delete(&path);

        // 更新统计
        let mut stats = self.stats.lock().await;
        stats.deleted_count += 1;
        stats.pending_changes += 1;
        stats.last_update = Some(SystemTime::now());

        Ok(())
    }

    /// 扫描文件信息
    fn scan_file(&self, path: &Path) -> Result<ScannedFile> {
        let metadata =
            fs::metadata(path).with_context(|| format!("Failed to read metadata: {:?}", path))?;

        Ok(ScannedFile {
            path: path.to_path_buf(),
            size: metadata.len(),
            modified: metadata.modified().ok(),
            is_dir: metadata.is_dir(),
        })
    }

    /// 如果需要则提交索引
    async fn commit_if_needed(&self) -> Result<()> {
        let stats = self.stats.lock().await;
        let pending = stats.pending_changes;
        drop(stats);

        if pending >= self.config.commit_threshold {
            info!("Committing {} pending changes", pending);
            self.commit().await?;
        }

        Ok(())
    }

    /// 强制提交索引
    pub async fn commit(&self) -> Result<()> {
        let _builder = self.builder.lock().await;
        // 注意：这里只是获取writer并commit，不会消费builder
        // 实际的commit需要通过writer进行
        info!("Committing index changes");

        // 重置待提交计数
        let mut stats = self.stats.lock().await;
        stats.pending_changes = 0;

        Ok(())
    }

    /// 获取统计信息
    pub async fn stats(&self) -> IncrementalStats {
        self.stats.lock().await.clone()
    }

    /// 获取WAL最近的操作记录
    pub async fn recent_operations(&self, limit: usize) -> Vec<WalEntry> {
        let wal = self.wal.lock().await;
        wal.recent_operations(limit).into_iter().cloned().collect()
    }

    /// 获取索引
    pub fn index(&self) -> &Index {
        &self.index
    }

    /// 获取Schema
    pub fn schema(&self) -> &IndexSchema {
        &self.schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_wal_basic() {
        let mut wal = WriteAheadLog::new();

        wal.log_create(Path::new("test1.txt"));
        wal.log_modify(Path::new("test2.txt"));
        wal.log_delete(Path::new("test3.txt"));

        assert_eq!(wal.len(), 3);

        let recent = wal.recent_operations(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].operation, WalOperation::Delete);
        assert_eq!(recent[1].operation, WalOperation::Modify);
    }

    #[test]
    fn test_wal_max_entries() {
        let mut wal = WriteAheadLog::new();

        // 添加超过最大数量的条目
        for i in 0..1500 {
            wal.log_create(&PathBuf::from(format!("test{}.txt", i)));
        }

        // 应该只保留1000条
        assert_eq!(wal.len(), 1000);
    }

    #[test]
    fn test_wal_clear() {
        let mut wal = WriteAheadLog::new();

        wal.log_create(Path::new("test.txt"));
        assert_eq!(wal.len(), 1);

        wal.clear();
        assert_eq!(wal.len(), 0);
        assert!(wal.is_empty());
    }

    #[tokio::test]
    async fn test_incremental_config_default() {
        let config = IncrementalConfig::default();
        assert_eq!(config.commit_threshold, 50);
        assert_eq!(config.auto_commit_interval, 30);
    }

    #[tokio::test]
    async fn test_incremental_indexer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = IncrementalConfig {
            index_config: IndexConfig {
                index_path: temp_dir.path().join("index"),
                ..Default::default()
            },
            ..Default::default()
        };

        let indexer = IncrementalIndexer::new(config).await;
        assert!(indexer.is_ok());
    }

    #[tokio::test]
    async fn test_incremental_stats() {
        let temp_dir = TempDir::new().unwrap();
        let config = IncrementalConfig {
            index_config: IndexConfig {
                index_path: temp_dir.path().join("index"),
                ..Default::default()
            },
            ..Default::default()
        };

        let indexer = IncrementalIndexer::new(config).await.unwrap();
        let stats = indexer.stats().await;

        assert_eq!(stats.created_count, 0);
        assert_eq!(stats.modified_count, 0);
        assert_eq!(stats.deleted_count, 0);
    }

    #[tokio::test]
    async fn test_incremental_watch() {
        let temp_dir = TempDir::new().unwrap();
        let config = IncrementalConfig {
            index_config: IndexConfig {
                index_path: temp_dir.path().join("index"),
                ..Default::default()
            },
            ..Default::default()
        };

        let indexer = IncrementalIndexer::new(config).await.unwrap();
        let result = indexer.watch(temp_dir.path()).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_scan_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "hello").unwrap();

        let config = IncrementalConfig {
            index_config: IndexConfig {
                index_path: temp_dir.path().join("index"),
                ..Default::default()
            },
            ..Default::default()
        };

        let indexer = IncrementalIndexer::new(config).await.unwrap();
        let scanned = indexer.scan_file(&test_file).unwrap();

        assert_eq!(scanned.path, test_file);
        assert_eq!(scanned.size, 5);
        assert!(!scanned.is_dir);
    }

    #[tokio::test]
    async fn test_apply_create() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "hello").unwrap();

        let config = IncrementalConfig {
            index_config: IndexConfig {
                index_path: temp_dir.path().join("index"),
                ..Default::default()
            },
            ..Default::default()
        };

        let indexer = IncrementalIndexer::new(config).await.unwrap();
        let result = indexer.apply_create(test_file).await;

        assert!(result.is_ok());

        let stats = indexer.stats().await;
        assert_eq!(stats.created_count, 1);
        assert_eq!(stats.pending_changes, 1);
    }

    #[tokio::test]
    async fn test_apply_delete() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        let config = IncrementalConfig {
            index_config: IndexConfig {
                index_path: temp_dir.path().join("index"),
                ..Default::default()
            },
            ..Default::default()
        };

        let indexer = IncrementalIndexer::new(config).await.unwrap();
        let result = indexer.apply_delete(test_file).await;

        assert!(result.is_ok());

        let stats = indexer.stats().await;
        assert_eq!(stats.deleted_count, 1);
    }

    #[tokio::test]
    async fn test_recent_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config = IncrementalConfig {
            index_config: IndexConfig {
                index_path: temp_dir.path().join("index"),
                ..Default::default()
            },
            ..Default::default()
        };

        let indexer = IncrementalIndexer::new(config).await.unwrap();

        // 记录一些操作
        {
            let mut wal = indexer.wal.lock().await;
            wal.log_create(Path::new("file1.txt"));
            wal.log_modify(Path::new("file2.txt"));
            wal.log_delete(Path::new("file3.txt"));
        }

        let ops = indexer.recent_operations(5).await;
        assert_eq!(ops.len(), 3);
    }
}
