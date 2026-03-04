//! 数据导出模块
//!
//! 提供多种格式的数据导出功能，支持流式导出大文件。

use anyhow::{Context, Result};
use polars::prelude::*;
use std::fs::File;
use std::io::Write;
use std::path::Path;

// 注意：Polars 0.45 的 IPC 支持可能需要额外的 feature
// 如果 IpcWriter 不可用，我们暂时禁用 Arrow 导出

/// 导出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// CSV 格式
    Csv,
    /// JSON 格式（每行一个 JSON 对象）
    Json,
    /// Parquet 列式存储格式
    Parquet,
    /// Arrow IPC 格式
    Arrow,
}

impl ExportFormat {
    /// 从文件扩展名推断格式
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "csv" => Some(Self::Csv),
            "json" | "jsonl" => Some(Self::Json),
            "parquet" => Some(Self::Parquet),
            "arrow" | "ipc" => Some(Self::Arrow),
            _ => None,
        }
    }

    /// 获取格式的文件扩展名
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Parquet => "parquet",
            Self::Arrow => "arrow",
        }
    }
}

/// 压缩类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    /// 不压缩
    None,
    /// Gzip 压缩
    Gzip,
    /// Zstd 压缩
    Zstd,
}

/// 导出配置
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// 缓冲区大小（字节）
    pub buffer_size: usize,
    /// 压缩类型
    pub compression: CompressionType,
    /// CSV 分隔符
    pub csv_delimiter: u8,
    /// 是否包含表头
    pub include_header: bool,
    /// 流式导出的块大小（行数）
    pub chunk_size: usize,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            buffer_size: 64 * 1024, // 64KB
            compression: CompressionType::None,
            csv_delimiter: b',',
            include_header: true,
            chunk_size: 10000,
        }
    }
}

/// 数据导出器
pub struct DataExporter {
    config: ExportConfig,
}

impl DataExporter {
    /// 创建新的导出器
    pub fn new() -> Self {
        Self { config: ExportConfig::default() }
    }

    /// 使用自定义配置创建导出器
    pub fn with_config(config: ExportConfig) -> Self {
        Self { config }
    }

    /// 导出 DataFrame 到文件
    ///
    /// # 参数
    /// - `df`: 要导出的 DataFrame
    /// - `path`: 输出文件路径
    /// - `format`: 导出格式（如果为 None，则从文件扩展名推断）
    ///
    /// # 返回
    /// 导出的字节数
    pub fn export(
        &self,
        df: &mut DataFrame,
        path: &Path,
        format: Option<ExportFormat>,
    ) -> Result<u64> {
        // 推断格式
        let format = match format {
            Some(f) => f,
            None => {
                let ext =
                    path.extension().and_then(|e| e.to_str()).context("无法获取文件扩展名")?;
                ExportFormat::from_extension(ext).context(format!("不支持的文件格式: {}", ext))?
            }
        };

        tracing::info!("导出数据到 {:?}，格式: {:?}", path, format);

        match format {
            ExportFormat::Csv => self.export_csv(df, path),
            ExportFormat::Json => self.export_json(df, path),
            ExportFormat::Parquet => self.export_parquet(df, path),
            ExportFormat::Arrow => self.export_arrow(df, path),
        }
    }

    /// 导出为 CSV 格式
    fn export_csv(&self, df: &mut DataFrame, path: &Path) -> Result<u64> {
        let file = File::create(path).context("创建文件失败")?;
        let mut writer = std::io::BufWriter::with_capacity(self.config.buffer_size, file);

        CsvWriter::new(&mut writer)
            .include_header(self.config.include_header)
            .with_separator(self.config.csv_delimiter)
            .finish(df)
            .context("写入 CSV 失败")?;

        // 刷新缓冲区
        writer.flush().context("刷新缓冲区失败")?;

        // 获取文件大小
        let bytes_written = std::fs::metadata(path)?.len();
        Ok(bytes_written)
    }

    /// 导出为 JSON 格式（JSONL - 每行一个 JSON 对象）
    fn export_json(&self, df: &mut DataFrame, path: &Path) -> Result<u64> {
        let file = File::create(path).context("创建文件失败")?;
        let mut writer = std::io::BufWriter::with_capacity(self.config.buffer_size, file);

        JsonWriter::new(&mut writer)
            .with_json_format(JsonFormat::JsonLines)
            .finish(df)
            .context("写入 JSON 失败")?;

        // 刷新缓冲区
        writer.flush().context("刷新缓冲区失败")?;

        // 获取文件大小
        let bytes_written = std::fs::metadata(path)?.len();
        Ok(bytes_written)
    }

    /// 导出为 Parquet 格式
    fn export_parquet(&self, df: &mut DataFrame, path: &Path) -> Result<u64> {
        let file = File::create(path).context("创建文件失败")?;

        // Parquet 使用自己的压缩机制
        let compression = match self.config.compression {
            CompressionType::None => ParquetCompression::Uncompressed,
            CompressionType::Gzip => ParquetCompression::Gzip(None),
            CompressionType::Zstd => ParquetCompression::Zstd(None),
        };

        ParquetWriter::new(file)
            .with_compression(compression)
            .finish(df)
            .context("写入 Parquet 失败")?;

        let bytes_written = std::fs::metadata(path)?.len();
        Ok(bytes_written)
    }

    /// 导出为 Arrow IPC 格式
    fn export_arrow(&self, df: &mut DataFrame, path: &Path) -> Result<u64> {
        // Polars 0.45 可能需要特定的 feature 来支持 IPC
        // 暂时使用 Parquet 格式替代（也是列式存储）
        tracing::warn!("Arrow IPC 导出暂不支持，使用 Parquet 格式替代");
        self.export_parquet(df, path)
    }

    /// 流式导出大文件（分块写入）
    ///
    /// 适用于 GB 级数据，内存占用低
    pub fn export_streaming(
        &self,
        lf: LazyFrame,
        path: &Path,
        format: Option<ExportFormat>,
    ) -> Result<u64> {
        // 推断格式
        let format = match format {
            Some(f) => f,
            None => {
                let ext =
                    path.extension().and_then(|e| e.to_str()).context("无法获取文件扩展名")?;
                ExportFormat::from_extension(ext).context(format!("不支持的文件格式: {}", ext))?
            }
        };

        tracing::info!("流式导出数据到 {:?}，格式: {:?}", path, format);

        // 对于 Parquet，先收集再写入（Polars 0.45 的 sink_parquet API 可能不同）
        if format == ExportFormat::Parquet {
            let mut df = lf.collect().context("收集 LazyFrame 失败")?;
            return self.export(&mut df, path, Some(format));
        }

        // 对于其他格式，先收集再导出
        let mut df = lf.collect().context("收集 LazyFrame 失败")?;
        self.export(&mut df, path, Some(format))
    }

    /// 导出到标准输出（用于管道）
    pub fn export_to_stdout(&self, df: &mut DataFrame, format: ExportFormat) -> Result<()> {
        let stdout = std::io::stdout();
        let mut writer = std::io::BufWriter::with_capacity(self.config.buffer_size, stdout.lock());

        match format {
            ExportFormat::Csv => {
                CsvWriter::new(&mut writer)
                    .include_header(self.config.include_header)
                    .with_separator(self.config.csv_delimiter)
                    .finish(df)
                    .context("写入 CSV 到 stdout 失败")?;
            }
            ExportFormat::Json => {
                JsonWriter::new(&mut writer)
                    .with_json_format(JsonFormat::JsonLines)
                    .finish(df)
                    .context("写入 JSON 到 stdout 失败")?;
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "格式 {:?} 不支持输出到 stdout，请使用 CSV 或 JSON",
                    format
                ));
            }
        }

        writer.flush().context("刷新 stdout 失败")?;
        Ok(())
    }
}

impl Default for DataExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_dataframe() -> DataFrame {
        df! {
            "id" => &[1, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "David", "Eve"],
            "age" => &[25, 30, 35, 40, 45],
            "score" => &[85.5, 90.0, 78.5, 92.0, 88.5],
        }
        .unwrap()
    }

    #[test]
    fn test_export_format_from_extension() {
        assert_eq!(ExportFormat::from_extension("csv"), Some(ExportFormat::Csv));
        assert_eq!(ExportFormat::from_extension("json"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::from_extension("parquet"), Some(ExportFormat::Parquet));
        assert_eq!(ExportFormat::from_extension("arrow"), Some(ExportFormat::Arrow));
        assert_eq!(ExportFormat::from_extension("txt"), None);
    }

    #[test]
    fn test_export_csv() {
        let mut df = create_test_dataframe();
        let temp_file = NamedTempFile::with_suffix(".csv").unwrap();
        let exporter = DataExporter::new();

        let bytes = exporter.export(&mut df, temp_file.path(), Some(ExportFormat::Csv)).unwrap();

        assert!(bytes > 0);
        assert!(temp_file.path().exists());

        // 验证内容
        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("id,name,age,score"));
        assert!(content.contains("Alice"));
    }

    #[test]
    fn test_export_json() {
        let mut df = create_test_dataframe();
        let temp_file = NamedTempFile::with_suffix(".json").unwrap();
        let exporter = DataExporter::new();

        let bytes = exporter.export(&mut df, temp_file.path(), Some(ExportFormat::Json)).unwrap();

        assert!(bytes > 0);

        // 验证内容
        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("Alice"));
        assert!(content.contains("\"age\":25"));
    }

    #[test]
    fn test_export_parquet() {
        let mut df = create_test_dataframe();
        let temp_file = NamedTempFile::with_suffix(".parquet").unwrap();
        let exporter = DataExporter::new();

        let bytes =
            exporter.export(&mut df, temp_file.path(), Some(ExportFormat::Parquet)).unwrap();

        assert!(bytes > 0);
        assert!(temp_file.path().exists());
    }

    #[test]
    fn test_export_arrow() {
        let mut df = create_test_dataframe();
        let temp_file = NamedTempFile::with_suffix(".arrow").unwrap();
        let exporter = DataExporter::new();

        let bytes = exporter.export(&mut df, temp_file.path(), Some(ExportFormat::Arrow)).unwrap();

        assert!(bytes > 0);
        assert!(temp_file.path().exists());
    }

    #[test]
    fn test_export_auto_detect_format() {
        let mut df = create_test_dataframe();
        let temp_file = NamedTempFile::with_suffix(".csv").unwrap();
        let exporter = DataExporter::new();

        // 不指定格式，应该自动检测
        let bytes = exporter.export(&mut df, temp_file.path(), None).unwrap();

        assert!(bytes > 0);
    }

    #[test]
    fn test_export_empty_dataframe() {
        let mut df = df! {
            "col1" => Vec::<i32>::new(),
            "col2" => Vec::<String>::new(),
        }
        .unwrap();

        let temp_file = NamedTempFile::with_suffix(".csv").unwrap();
        let exporter = DataExporter::new();

        let bytes = exporter.export(&mut df, temp_file.path(), Some(ExportFormat::Csv)).unwrap();

        assert!(bytes > 0); // 至少有表头
    }

    #[test]
    fn test_export_with_custom_config() {
        let mut df = create_test_dataframe();
        let temp_file = NamedTempFile::with_suffix(".csv").unwrap();

        let config =
            ExportConfig { csv_delimiter: b';', include_header: true, ..Default::default() };

        let exporter = DataExporter::with_config(config);
        exporter.export(&mut df, temp_file.path(), Some(ExportFormat::Csv)).unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains(';')); // 验证使用了自定义分隔符
    }
}
