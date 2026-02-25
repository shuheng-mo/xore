# XORE 示例

本目录包含 XORE 的使用示例和基准测试脚本。

## 目录结构

```
examples/
├── README.md                    # 本文件
├── search-examples/             # 搜索功能示例
│   └── demo.sh                  # 综合演示脚本
├── scripts/                     # 实用脚本
│   └── run_benchmarks.sh        # 性能基准测试脚本
└── benchmark-data/              # 基准测试数据（.gitignore）
    └── large/                   # 10GB 测试数据
```

## 快速开始

### 1. 搜索功能演示

运行综合演示脚本，查看 XORE 的各种搜索功能：

```bash
# 确保已编译 release 版本
cargo build --release

# 运行演示
./examples/search-examples/demo.sh
```

**演示内容：**

- ✅ 标准全文搜索
- ✅ 前缀搜索（`term*`）
- ✅ 模糊搜索（`~term`）
- ✅ 文件类型过滤

### 2. 性能基准测试

运行完整的性能基准测试：

```bash
# 运行基准测试（会自动准备测试数据）
./examples/scripts/run_benchmarks.sh
```

**测试内容：**

- 文件扫描性能
- 搜索性能（标准/前缀/模糊）
- I/O 吞吐量
- 内存分配性能

**测试结果：**

- 实时输出到终端
- 保存到 `plans/benchmark-results-YYYYMMDD-HHMMSS.txt`
- 详细报告：[`plans/performance-report.md`](../plans/performance-report.md)

## 手动运行示例

### 搜索示例

```bash
# 标准搜索
xore f "error" --index

# 前缀搜索
xore f "config*" --index

# 模糊搜索
xore f "~databse" --index

# 中文搜索
xore f "错误" --index

# 增量监控
xore f "TODO" --index --watch

# 文件类型过滤
xore f "error" --index --type log
```

### 基准测试示例

```bash
# 搜索性能测试
xore benchmark --suite search \
  --data-path examples/benchmark-data/large -n 5

# 文件扫描测试
xore benchmark --suite scan \
  --data-path examples/benchmark-data/large -n 3

# 完整测试
xore benchmark --suite all \
  --data-path examples/benchmark-data/large -n 3
```

## 测试数据

### 自动准备

运行 `run_benchmarks.sh` 会自动准备测试数据。

### 手动准备

如果需要手动准备测试数据：

```bash
# 创建目录
mkdir -p examples/benchmark-data/large

# 复制现有数据（需要 data/huge_data.csv）
for i in {1..17}; do
  cp data/huge_data.csv examples/benchmark-data/large/data_$i.csv
done

# 验证数据大小
du -sh examples/benchmark-data/large
# 预期输出: ~9.5G
```

### 数据规模

| 规模 | 文件数 | 总大小 | 用途 |
|------|-------|--------|------|
| Small | 2 | ~100MB | 快速测试 |
| Medium | 2 | ~1.2GB | 中等规模验证 |
| Large | 17 | ~9.5GB | 完整性能测试 |

## 性能指标

基于 9.5GB 测试数据的实际结果：

| 操作 | 性能 | 状态 |
|------|------|------|
| 索引构建 | 92,678 MB/s | ✅ |
| 标准搜索 (p99) | 0.2 ms | ✅ |
| 前缀搜索 (p99) | 0.0 ms | ✅ |
| 模糊搜索 (p99) | 0.3 ms | ✅ |
| 增量索引延迟 | ~45 ms | ✅ |

详细报告：[Performance Report](../plans/performance-report.md)

## 故障排查

### 问题：找不到 xore 命令

**解决方案：**

```bash
# 编译 release 版本
cargo build --release

# 使用完整路径
./target/release/xore --version
```

### 问题：测试数据不存在

**解决方案：**

```bash
# 运行基准测试脚本会自动准备数据
./examples/scripts/run_benchmarks.sh

# 或手动准备（见上文"手动准备"部分）
```

### 问题：权限不足

**解决方案：**

```bash
# 添加执行权限
chmod +x examples/search-examples/demo.sh
chmod +x examples/scripts/run_benchmarks.sh
```

## 更多资源

- [命令文档](../docs/commands/)
- [性能报告](../plans/performance-report.md)
- [开发指南](../CONTRIBUTING.md)
- [项目 README](../README.md)

## 贡献

欢迎提交新的示例！请参考 [贡献指南](../CONTRIBUTING.md)。

---

**最后更新:** 2026-02-25  
**维护者:** XORE Development Team
