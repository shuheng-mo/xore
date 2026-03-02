//! 数据质量分析器

use anyhow::Result;
use polars::prelude::*;
use std::collections::HashMap;

/// 数据质量报告
#[derive(Debug, Clone)]
pub struct QualityReport {
    /// 总行数
    pub total_rows: usize,
    /// 总列数
    pub total_columns: usize,
    /// 列名列表
    pub column_names: Vec<String>,
    /// 每列的缺失值统计
    pub missing_values: HashMap<String, MissingStats>,
    /// 重复行数
    pub duplicate_rows: usize,
    /// 数据类型信息
    pub column_types: HashMap<String, String>,
}

/// 缺失值统计
#[derive(Debug, Clone)]
pub struct MissingStats {
    /// 缺失值数量
    pub count: usize,
    /// 缺失值百分比
    pub percentage: f64,
}

/// 列统计信息
#[derive(Debug, Clone)]
pub struct ColumnStats {
    /// 列名
    pub name: String,
    /// 数据类型
    pub dtype: String,
    /// 唯一值数量
    pub unique_count: usize,
    /// 缺失值数量
    pub null_count: usize,
    /// 缺失值百分比
    pub null_percentage: f64,
}

/// 数据分析器
pub struct DataProfiler;

impl DataProfiler {
    /// 创建新的分析器
    pub fn new() -> Self {
        Self
    }

    /// 生成数据质量报告
    pub fn profile(&self, df: &DataFrame) -> Result<QualityReport> {
        let total_rows = df.height();
        let total_columns = df.width();
        let column_names: Vec<String> =
            df.get_column_names().iter().map(|s| s.to_string()).collect();

        // 统计缺失值
        let mut missing_values = HashMap::new();
        let mut column_types = HashMap::new();

        for col_name in &column_names {
            let series = df.column(col_name).map_err(|e| anyhow::anyhow!("获取列失败: {}", e))?;

            // 缺失值统计
            let null_count = series.null_count();
            let percentage =
                if total_rows > 0 { (null_count as f64 / total_rows as f64) * 100.0 } else { 0.0 };

            if null_count > 0 {
                missing_values
                    .insert(col_name.clone(), MissingStats { count: null_count, percentage });
            }

            // 数据类型
            column_types.insert(col_name.clone(), format!("{:?}", series.dtype()));
        }

        // 统计重复行（使用哈希）
        let duplicate_rows = self.count_duplicates(df)?;

        Ok(QualityReport {
            total_rows,
            total_columns,
            column_names,
            missing_values,
            duplicate_rows,
            column_types,
        })
    }

    /// 统计重复行数
    fn count_duplicates(&self, df: &DataFrame) -> Result<usize> {
        // 使用 Polars 的 is_duplicated 功能
        let mask = df.is_duplicated().map_err(|e| anyhow::anyhow!("检测重复行失败: {}", e))?;

        let duplicate_count = mask.sum().ok_or_else(|| anyhow::anyhow!("计算重复行数失败"))?;

        Ok(duplicate_count as usize)
    }

    /// 获取列的详细统计信息
    pub fn column_stats(&self, df: &DataFrame, column_name: &str) -> Result<ColumnStats> {
        let series = df.column(column_name).map_err(|e| anyhow::anyhow!("获取列失败: {}", e))?;

        let total_rows = df.height();
        let null_count = series.null_count();
        let null_percentage =
            if total_rows > 0 { (null_count as f64 / total_rows as f64) * 100.0 } else { 0.0 };

        let unique_count =
            series.n_unique().map_err(|e| anyhow::anyhow!("计算唯一值失败: {}", e))?;

        Ok(ColumnStats {
            name: column_name.to_string(),
            dtype: format!("{:?}", series.dtype()),
            unique_count,
            null_count,
            null_percentage,
        })
    }

    /// 检测数值列的离群值（使用 IQR 方法）
    /// 注意：此功能需要将 Column 转换为 Series
    pub fn detect_outliers(&self, df: &DataFrame, column_name: &str) -> Result<Vec<usize>> {
        let column = df.column(column_name).map_err(|e| anyhow::anyhow!("获取列失败: {}", e))?;

        // 转换为 Series
        let series = column.as_materialized_series();

        // 只处理数值类型
        if !series.dtype().is_numeric() {
            return Err(anyhow::anyhow!("列 {} 不是数值类型", column_name));
        }

        // 计算 Q1, Q3 使用 median 和排序
        let sorted =
            series.sort(Default::default()).map_err(|e| anyhow::anyhow!("排序失败: {}", e))?;

        let len = sorted.len();
        let q1_idx = len / 4;
        let q3_idx = (3 * len) / 4;

        let q1_val = sorted
            .get(q1_idx)
            .map_err(|e| anyhow::anyhow!("获取 Q1 失败: {}", e))?
            .try_extract::<f64>()
            .map_err(|e| anyhow::anyhow!("提取 Q1 值失败: {}", e))?;

        let q3_val = sorted
            .get(q3_idx)
            .map_err(|e| anyhow::anyhow!("获取 Q3 失败: {}", e))?
            .try_extract::<f64>()
            .map_err(|e| anyhow::anyhow!("提取 Q3 值失败: {}", e))?;

        let iqr = q3_val - q1_val;
        let lower_bound = q1_val - 1.5 * iqr;
        let upper_bound = q3_val + 1.5 * iqr;

        // 找出离群值的索引
        let mut outlier_indices = Vec::new();
        for idx in 0..series.len() {
            if let Ok(val) = series.get(idx) {
                if let Ok(v) = val.try_extract::<f64>() {
                    if v < lower_bound || v > upper_bound {
                        outlier_indices.push(idx);
                    }
                }
            }
        }

        Ok(outlier_indices)
    }
}

impl Default for DataProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::df;

    #[test]
    fn test_profile_basic() {
        let df = df! {
            "id" => &[1, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "David", "Eve"],
            "age" => &[Some(25), Some(30), None, Some(35), Some(40)],
        }
        .unwrap();

        let profiler = DataProfiler::new();
        let report = profiler.profile(&df).unwrap();

        assert_eq!(report.total_rows, 5);
        assert_eq!(report.total_columns, 3);
        assert_eq!(report.column_names.len(), 3);

        // 检查缺失值统计
        assert!(report.missing_values.contains_key("age"));
        let age_missing = &report.missing_values["age"];
        assert_eq!(age_missing.count, 1);
        assert_eq!(age_missing.percentage, 20.0);
    }

    #[test]
    fn test_column_stats() {
        let df = df! {
            "id" => &[1, 2, 3, 4, 5],
            "category" => &["A", "B", "A", "C", "B"],
        }
        .unwrap();

        let profiler = DataProfiler::new();
        let stats = profiler.column_stats(&df, "category").unwrap();

        assert_eq!(stats.name, "category");
        assert_eq!(stats.unique_count, 3); // A, B, C
        assert_eq!(stats.null_count, 0);
    }

    #[test]
    fn test_count_duplicates() {
        let df = df! {
            "id" => &[1, 2, 3, 2, 1],
            "value" => &[10, 20, 30, 20, 10],
        }
        .unwrap();

        let profiler = DataProfiler::new();
        let duplicate_count = profiler.count_duplicates(&df).unwrap();

        // is_duplicated() 标记所有重复的行（包括原始行）
        // 所以 [1,10], [2,20], [2,20], [1,10] 都被标记，共4行
        assert_eq!(duplicate_count, 4);
    }

    #[test]
    fn test_detect_outliers() {
        let df = df! {
            "values" => &[1.0, 2.0, 3.0, 4.0, 5.0, 100.0], // 100.0 是离群值
        }
        .unwrap();

        let profiler = DataProfiler::new();
        let outliers = profiler.detect_outliers(&df, "values").unwrap();

        assert!(!outliers.is_empty());
        assert!(outliers.contains(&5)); // 索引 5 是离群值
    }

    #[test]
    fn test_no_missing_values() {
        let df = df! {
            "id" => &[1, 2, 3],
            "name" => &["A", "B", "C"],
        }
        .unwrap();

        let profiler = DataProfiler::new();
        let report = profiler.profile(&df).unwrap();

        assert!(report.missing_values.is_empty());
    }
}
