//! 数据解析器 - 基于 Polars 的高性能数据加载

use anyhow::Result;
use polars::prelude::*;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;

/// 数据解析器配置
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// 是否使用内存映射（适用于大文件）
    pub use_mmap: bool,
    /// 内存映射阈值（字节），超过此大小使用 mmap
    pub mmap_threshold: u64,
    /// CSV 分隔符
    pub csv_delimiter: u8,
    /// 是否自动推断 Schema
    pub infer_schema: bool,
    /// Schema 推断时扫描的行数
    pub infer_schema_length: Option<usize>,
    /// 是否跳过空行
    pub skip_rows: usize,
    /// 是否有表头
    pub has_header: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            use_mmap: true,
            mmap_threshold: 1024 * 1024, // 1MB
            csv_delimiter: b',',
            infer_schema: true,
            infer_schema_length: Some(1000),
            skip_rows: 0,
            has_header: true,
        }
    }
}

/// 数据解析器
pub struct DataParser {
    config: ParserConfig,
}

impl DataParser {
    /// 创建新的解析器
    pub fn new() -> Self {
        Self { config: ParserConfig::default() }
    }

    /// 使用自定义配置创建解析器
    pub fn with_config(config: ParserConfig) -> Self {
        Self { config }
    }

    /// 读取 CSV 文件并返回 LazyFrame
    pub fn read_csv_lazy(&self, path: &Path) -> Result<LazyFrame> {
        tracing::debug!("读取 CSV 文件: {:?}", path);

        // 检查文件是否存在
        if !path.exists() {
            return Err(anyhow::anyhow!("文件不存在: {:?}", path));
        }

        // 获取文件大小
        let file_size = std::fs::metadata(path)?.len();
        tracing::debug!("文件大小: {} 字节", file_size);

        // 根据文件大小决定是否使用 mmap
        let use_mmap = self.config.use_mmap && file_size > self.config.mmap_threshold;

        if use_mmap {
            tracing::debug!("使用内存映射读取大文件");
            self.read_csv_with_mmap(path)
        } else {
            tracing::debug!("使用标准文件读取");
            self.read_csv_standard(path)
        }
    }

    /// 使用标准方式读取 CSV
    fn read_csv_standard(&self, path: &Path) -> Result<LazyFrame> {
        let df = LazyCsvReader::new(path)
            .with_has_header(self.config.has_header)
            .with_separator(self.config.csv_delimiter)
            .with_skip_rows(self.config.skip_rows)
            .with_infer_schema_length(self.config.infer_schema_length)
            .finish()
            .map_err(|e| anyhow::anyhow!("读取 CSV 失败: {}", e))?;

        Ok(df)
    }

    /// 使用内存映射读取 CSV
    fn read_csv_with_mmap(&self, path: &Path) -> Result<LazyFrame> {
        use memmap2::Mmap;

        // 打开文件
        let file = File::open(path).map_err(|e| anyhow::anyhow!("无法打开文件: {}", e))?;

        // 创建内存映射
        let mmap =
            unsafe { Mmap::map(&file).map_err(|e| anyhow::anyhow!("内存映射失败: {}", e))? };

        // 使用 Cursor 包装 mmap 数据
        let cursor = Cursor::new(&mmap[..]);

        // 读取 CSV
        let df = CsvReadOptions::default()
            .with_has_header(self.config.has_header)
            .with_infer_schema_length(self.config.infer_schema_length)
            .into_reader_with_file_handle(cursor)
            .finish()
            .map_err(|e| anyhow::anyhow!("读取 CSV 失败: {}", e))?
            .lazy();

        Ok(df)
    }

    /// 读取 Parquet 文件并返回 LazyFrame
    pub fn read_parquet_lazy(&self, path: &Path) -> Result<LazyFrame> {
        tracing::debug!("读取 Parquet 文件: {:?}", path);

        // 检查文件是否存在
        if !path.exists() {
            return Err(anyhow::anyhow!("文件不存在: {:?}", path));
        }

        let args = ScanArgsParquet::default();
        let df = LazyFrame::scan_parquet(path, args)
            .map_err(|e| anyhow::anyhow!("读取 Parquet 失败: {}", e))?;

        Ok(df)
    }

    /// 自动识别格式并读取
    pub fn read_lazy(&self, path: &Path) -> Result<LazyFrame> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        match extension.as_str() {
            "csv" => self.read_csv_lazy(path),
            "parquet" => self.read_parquet_lazy(path),
            _ => {
                Err(anyhow::anyhow!("不支持的文件格式: {}。支持的格式: csv, parquet", extension)
                    .into())
            }
        }
    }

    /// 读取并收集为 DataFrame（用于小数据集或需要立即执行的场景）
    pub fn read_csv(&self, path: &Path) -> Result<DataFrame> {
        let lf = self.read_csv_lazy(path)?;
        lf.collect().map_err(|e| anyhow::anyhow!("收集 DataFrame 失败: {}", e).into())
    }

    /// 读取 Parquet 并收集为 DataFrame
    pub fn read_parquet(&self, path: &Path) -> Result<DataFrame> {
        let lf = self.read_parquet_lazy(path)?;
        lf.collect().map_err(|e| anyhow::anyhow!("收集 DataFrame 失败: {}", e).into())
    }

    /// 自动识别格式并读取为 DataFrame
    pub fn read(&self, path: &Path) -> Result<DataFrame> {
        let lf = self.read_lazy(path)?;
        lf.collect().map_err(|e| anyhow::anyhow!("收集 DataFrame 失败: {}", e).into())
    }
}

impl Default for DataParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,age,city").unwrap();
        writeln!(file, "1,Alice,28,Beijing").unwrap();
        writeln!(file, "2,Bob,32,Shanghai").unwrap();
        writeln!(file, "3,Charlie,25,Guangzhou").unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_read_csv_lazy() {
        let file = create_test_csv();
        let parser = DataParser::new();

        let lf = parser.read_csv_lazy(file.path()).unwrap();
        let df = lf.collect().unwrap();

        assert_eq!(df.height(), 3);
        assert_eq!(df.width(), 4);
        assert_eq!(df.get_column_names(), vec!["id", "name", "age", "city"]);
    }

    #[test]
    fn test_read_csv() {
        let file = create_test_csv();
        let parser = DataParser::new();

        let df = parser.read_csv(file.path()).unwrap();

        assert_eq!(df.height(), 3);
        assert_eq!(df.width(), 4);
    }

    #[test]
    fn test_read_auto_detect() {
        // 创建带 .csv 扩展名的临时文件
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.csv");

        std::fs::write(
            &file_path,
            "id,name,age,city\n1,Alice,28,Beijing\n2,Bob,32,Shanghai\n3,Charlie,25,Guangzhou\n",
        )
        .unwrap();

        let parser = DataParser::new();
        let df = parser.read(&file_path).unwrap();

        assert_eq!(df.height(), 3);
        assert_eq!(df.width(), 4);
    }

    #[test]
    fn test_file_not_found() {
        let parser = DataParser::new();
        let result = parser.read_csv_lazy(Path::new("/nonexistent/file.csv"));

        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_format() {
        let parser = DataParser::new();
        let result = parser.read_lazy(Path::new("test.txt"));

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("不支持的文件格式"));
        }
    }

    #[test]
    fn test_custom_config() {
        let config = ParserConfig { csv_delimiter: b';', has_header: false, ..Default::default() };

        let parser = DataParser::with_config(config);
        assert_eq!(parser.config.csv_delimiter, b';');
        assert!(!parser.config.has_header);
    }

    #[test]
    fn test_mmap_threshold() {
        let file = create_test_csv();
        let file_size = std::fs::metadata(file.path()).unwrap().len();

        // 设置阈值低于文件大小，应该使用 mmap
        let config =
            ParserConfig { use_mmap: true, mmap_threshold: file_size - 1, ..Default::default() };

        let parser = DataParser::with_config(config);
        let df = parser.read_csv(file.path()).unwrap();

        assert_eq!(df.height(), 3);
    }
}
