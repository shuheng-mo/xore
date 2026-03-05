# 测试数据说明

本目录包含用于 XORE 性能测试和功能演示的测试数据。

## 目录结构

```
benchmark-data/
├── small/          # 小型测试数据 (~100KB)
├── medium/         # 中型测试数据 (~20MB)
├── large/          # 大型测试数据 (~10GB)
└── README.md       # 本文件
```

## 数据文件列表

### 小型数据 (small/)

| 文件名 | 格式 | 行数 | 大小 | 描述 |
|--------|------|------|------|------|
| sales_small.csv | CSV | 100 | 8.5KB | 销售订单数据 |
| sales_small.json | JSON | 100 | 29KB | 销售订单数据(JSON格式) |
| users_small.csv | CSV | 50 | 4.2KB | 用户信息数据 |
| server_log_small.log | LOG | 500 | 27KB | 服务器日志 |
| server_log_small.json | JSON | 100 | 12KB | JSON格式日志 |
| access_log.tsv | TSV | 1000 | 45KB | 访问日志(TSV格式) |
| error_log.yaml | YAML | 50 | 3.6KB | 错误日志(YAML格式) |
| test_small.db | SQLite | 100 | 16KB | SQLite数据库 |

### 中型数据 (medium/)

| 文件名 | 格式 | 行数 | 大小 | 描述 |
|--------|------|------|------|------|
| sales_medium.csv | CSV | 100K | 8.6MB | 销售订单数据 |
| users_medium.csv | CSV | 50K | 4.4MB | 用户信息数据 |
| server_log_medium.log | LOG | 200K | 11.3MB | 服务器日志 |
| test_medium.db | SQLite | 10K | 792KB | SQLite数据库 |

### 大型数据 (large/)

| 文件名 | 格式 | 大小 | 描述 |
|--------|------|------|------|
| data_1.csv ~ data_17.csv | CSV | 每文件 ~574MB | 大型销售数据(总计 ~10GB) |

## 数据格式说明

### 销售数据 (sales_*.csv)

```csv
order_id,product_name,category,quantity,unit_price,total_amount,customer_id,order_date,status,region
ORD-20240101-0001,Laptop,Electronics,2,1999.99,3999.98,CUST-1234,2024-01-01,completed,North
```

字段说明：
- `order_id`: 订单编号
- `product_name`: 产品名称
- `category`: 产品类别
- `quantity`: 数量
- `unit_price`: 单价
- `total_amount`: 总金额
- `customer_id`: 客户ID
- `order_date`: 订单日期
- `status`: 订单状态 (pending/completed/cancelled/refunded)
- `region`: 地区

### 用户数据 (users_*.csv)

```csv
user_id,username,email,age,gender,registration_date,last_login,country,city,membership_level
USER-000001,user1,user1@example.com,28,M,2024-01-01,2024-06-01,USA,New York,Gold
```

### 服务器日志 (server_log_*.log)

```
[2025-01-01 00:00:00] [INFO ] [server-01] Database connection established
[2025-01-01 00:00:10] [ERROR] [server-02] Request timeout after 30s
```

格式: `[时间戳] [级别] [来源] 消息`

## 使用方法

### 使用 XORE 处理数据

```bash
# 预览数据
xore p examples/benchmark-data/small/sales_small.csv

# SQL查询
xore p examples/benchmark-data/small/sales_small.csv "SELECT category, COUNT(*) FROM this GROUP BY category"

# 数据质量检查
xore p examples/benchmark-data/small/sales_small.csv --quality-check
```

### 生成更多测试数据

```bash
# 安装依赖
pip install pandas pyarrow pyyaml

# 运行生成脚本
python examples/benchmark-data/generate_test_data.py
```

## 数据生成脚本

`generate_test_data.py` 脚本可以生成各种格式的测试数据：

```bash
# 查看帮助
python examples/benchmark-data/generate_test_data.py --help
```

## 注意事项

1. 大型数据文件 (large/) 总计约 10GB，请确保磁盘空间充足
2. 生成大型数据可能需要较长时间
3. 部分格式 (如 Parquet) 需要额外安装依赖

## 更新历史

- 2026-03-05: 初始版本，生成小型和中型测试数据
