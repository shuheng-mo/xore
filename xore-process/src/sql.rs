//! SQL 查询引擎
//!
//! 基于 Polars 内置 SQL 引擎执行查询。

use anyhow::{Context, Result};
use polars::prelude::*;
use polars::sql::SQLContext;
use std::collections::HashMap;
use std::path::Path;

use crate::parser::DataParser;

/// SQL 引擎
pub struct SqlEngine {
    /// 已注册的表（表名 -> LazyFrame）
    tables: HashMap<String, LazyFrame>,
    /// 数据解析器
    parser: DataParser,
}

impl SqlEngine {
    /// 创建新的 SQL 引擎
    pub fn new() -> Self {
        Self { tables: HashMap::new(), parser: DataParser::new() }
    }

    /// 注册表（从文件加载）
    pub fn register_table(&mut self, table_name: &str, path: &Path) -> Result<()> {
        let lf = self
            .parser
            .read_lazy(path)
            .with_context(|| format!("无法加载表 '{}' 从文件 {:?}", table_name, path))?;

        self.tables.insert(table_name.to_string(), lf);
        Ok(())
    }

    /// 注册表（从 LazyFrame）
    pub fn register_lazyframe(&mut self, table_name: &str, lf: LazyFrame) {
        self.tables.insert(table_name.to_string(), lf);
    }

    /// 执行 SQL 查询
    pub fn execute(&self, sql: &str) -> Result<DataFrame> {
        // 创建 SQL 上下文
        let mut ctx = SQLContext::new();

        // 注册所有表
        for (name, lf) in &self.tables {
            ctx.register(name, lf.clone());
        }

        // 执行查询
        let result_lf = ctx.execute(sql).with_context(|| format!("SQL 查询执行失败: {}", sql))?;

        // 收集结果
        result_lf.collect().with_context(|| "收集查询结果失败")
    }
}

impl Default for SqlEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::df;
    use tempfile::NamedTempFile;

    #[test]
    fn test_simple_select() {
        let df = df! {
            "id" => &[1, 2, 3],
            "name" => &["Alice", "Bob", "Charlie"],
            "age" => &[25, 30, 35],
        }
        .unwrap();

        let mut engine = SqlEngine::new();
        engine.register_lazyframe("users", df.lazy());

        let result = engine.execute("SELECT * FROM users").unwrap();
        assert_eq!(result.height(), 3);
        assert_eq!(result.width(), 3);
    }

    #[test]
    fn test_select_with_where() {
        let df = df! {
            "id" => &[1, 2, 3],
            "age" => &[25, 30, 35],
        }
        .unwrap();

        let mut engine = SqlEngine::new();
        engine.register_lazyframe("users", df.lazy());

        let result = engine.execute("SELECT * FROM users WHERE age > 28").unwrap();
        assert_eq!(result.height(), 2); // 30 和 35
    }

    #[test]
    fn test_select_columns() {
        let df = df! {
            "id" => &[1, 2, 3],
            "name" => &["Alice", "Bob", "Charlie"],
            "age" => &[25, 30, 35],
        }
        .unwrap();

        let mut engine = SqlEngine::new();
        engine.register_lazyframe("users", df.lazy());

        let result = engine.execute("SELECT name, age FROM users").unwrap();
        assert_eq!(result.width(), 2);
        assert!(result.column("name").is_ok());
        assert!(result.column("age").is_ok());
    }

    #[test]
    fn test_group_by_count() {
        let df = df! {
            "category" => &["A", "B", "A", "C", "B"],
            "value" => &[10, 20, 30, 40, 50],
        }
        .unwrap();

        let mut engine = SqlEngine::new();
        engine.register_lazyframe("data", df.lazy());

        let result = engine
            .execute("SELECT category, COUNT(*) as count FROM data GROUP BY category")
            .unwrap();
        assert_eq!(result.height(), 3); // A, B, C
        assert!(result.column("count").is_ok());
    }

    #[test]
    fn test_order_by() {
        let df = df! {
            "id" => &[3, 1, 2],
            "name" => &["Charlie", "Alice", "Bob"],
        }
        .unwrap();

        let mut engine = SqlEngine::new();
        engine.register_lazyframe("users", df.lazy());

        let result = engine.execute("SELECT * FROM users ORDER BY id").unwrap();
        let ids = result.column("id").unwrap();
        assert_eq!(ids.i32().unwrap().get(0), Some(1));
    }

    #[test]
    fn test_limit() {
        let df = df! {
            "id" => &[1, 2, 3, 4, 5],
        }
        .unwrap();

        let mut engine = SqlEngine::new();
        engine.register_lazyframe("data", df.lazy());

        let result = engine.execute("SELECT * FROM data LIMIT 3").unwrap();
        assert_eq!(result.height(), 3);
    }

    #[test]
    fn test_aggregate_functions() {
        let df = df! {
            "category" => &["A", "A", "B", "B"],
            "value" => &[10, 20, 30, 40],
        }
        .unwrap();

        let mut engine = SqlEngine::new();
        engine.register_lazyframe("data", df.lazy());

        let result =
            engine.execute("SELECT category, SUM(value) as total, AVG(value) as average FROM data GROUP BY category").unwrap();
        assert_eq!(result.height(), 2);
        assert!(result.column("total").is_ok());
        assert!(result.column("average").is_ok());
    }

    #[test]
    fn test_register_from_file() {
        // 创建带 .csv 扩展名的临时文件
        let temp_file = NamedTempFile::with_suffix(".csv").unwrap();
        let path = temp_file.path();

        // 写入 CSV 数据
        std::fs::write(path, "id,name,age\n1,Alice,25\n2,Bob,30\n").unwrap();

        let mut engine = SqlEngine::new();
        engine.register_table("users", path).unwrap();

        let result = engine.execute("SELECT * FROM users WHERE age > 26").unwrap();
        assert_eq!(result.height(), 1);
    }

    #[test]
    fn test_join() {
        let users = df! {
            "id" => &[1, 2, 3],
            "name" => &["Alice", "Bob", "Charlie"],
        }
        .unwrap();

        let orders = df! {
            "user_id" => &[1, 1, 2],
            "amount" => &[100, 200, 150],
        }
        .unwrap();

        let mut engine = SqlEngine::new();
        engine.register_lazyframe("users", users.lazy());
        engine.register_lazyframe("orders", orders.lazy());

        let result = engine
            .execute("SELECT users.name, SUM(orders.amount) as total FROM users INNER JOIN orders ON users.id = orders.user_id GROUP BY users.name")
            .unwrap();

        assert_eq!(result.height(), 2); // Alice 和 Bob
        assert!(result.column("total").is_ok());
    }
}
