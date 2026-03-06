# XORE 语义搜索集成指南

本指南详细介绍如何下载 ONNX 模型并集成 XORE 的语义搜索功能。

## 目录

1. [模型选择](#1-模型选择)
2. [下载模型](#2-下载模型)
3. [模型转换（如需要）](#3-模型转换如需要)
4. [验证模型文件](#4-验证模型文件)
5. [CLI 集成](#5-cli-集成)
6. [测试语义搜索](#6-测试语义搜索)
7. [常见问题](#7-常见问题)

---

## 1. 模型选择

### 推荐模型：MiniLM-L6-v2

| 属性 | 值 |
|------|-----|
| 模型名称 | sentence-transformers/all-MiniLM-L6-v2 |
| 向量维度 | 384 |
| 模型大小 | ~80 MB |
| 语言 | 多语言（支持中文） |
| 速度 | 快（延迟 <100ms） |

### 其他可选模型

| 模型 | 向量维度 | 大小 | 特点 |
|------|----------|------|------|
| all-MiniLM-L6-v2 | 384 | 80MB | 速度最快，推荐首选 |
| all-mpnet-base-v2 | 768 | 420MB | 精度更高，速度较慢 |
| paraphrase-multilingual-MiniLM-L12-v2 | 384 | 420MB | 支持 50+ 语言 |

---

## 2. 下载模型

### 方法一：使用 HuggingFace CLI（推荐）

```bash
# 安装 huggingface-cli
pip install huggingface-hub

# 创建模型目录
mkdir -p assets/models

# 下载模型文件
cd assets/models

# 下载 ONNX 模型（如果已转换）
huggingface-cli download sentence-transformers/all-MiniLM-L6-v2 \
    --local-dir . \
    --include "*.onnx" "tokenizer.json" "config.json" "vocab.txt"

# 或者下载完整模型（PyTorch 格式）
huggingface-cli download sentence-transformers/all-MiniLM-L6-v2 \
    --local-dir .
```

### 方法二：直接下载

```bash
# 创建目录
mkdir -p assets/models

# 下载 tokenizer 文件
curl -L -o assets/models/tokenizer.json \
  "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json"

# 下载 vocab 文件
curl -L -o assets/models/vocab.txt \
  "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/vocab.txt"

# 下载 config 文件
curl -L -o assets/models/config.json \
  "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/config.json"
```

### 方法三：使用 Python 脚本

```python
# download_model.py
from huggingface_hub import snapshot_download

model_id = "sentence-transformers/all-MiniLM-L6-v2"
local_dir = "assets/models"

# 下载所有文件
snapshot_download(
    repo_id=model_id,
    local_dir=local_dir,
    local_dir_use_symlinks=False
)

print(f"模型已下载到: {local_dir}")
```

运行脚本：

```bash
pip install huggingface-hub
python download_model.py
```

---

## 3. 模型转换（如需要）

如果下载的是 PyTorch 模型，需要转换为 ONNX 格式。

### 使用 Optimum 转换

```python
# convert_to_onnx.py
from optimum.onnxruntime import ORTModelForFeatureExtraction
from transformers import AutoTokenizer

model_id = "sentence-transformers/all-MiniLM-L6-v2"
output_dir = "assets/models"

print("正在下载模型...")
tokenizer = AutoTokenizer.from_pretrained(model_id)

print("正在转换为 ONNX 格式...")
model = ORTModelForFeatureExtraction.from_pretrained(
    model_id,
    export=True
)

print(f"正在保存到 {output_dir}...")
model.save_pretrained(output_dir)
tokenizer.save_pretrained(output_dir)

print("转换完成！")
```

运行转换：

```bash
pip install optimum[onnxruntime] transformers
python convert_to_onnx.py
```

### 验证 ONNX 模型

```python
# verify_model.py
import onnx

model_path = "assets/models/model.onnx"
model = onnx.load(model_path)
onnx.checker.check_model(model)
print("ONNX 模型验证通过！")

# 查看模型输入输出
print("\n模型输入:")
for input_tensor in model.graph.input:
    print(f"  - {input_tensor.name}: {[d.dim_value for d in input_tensor.type.tensor_type.shape.dim]}")

print("\n模型输出:")
for output_tensor in model.graph.output:
    print(f"  - {output_tensor.name}: {[d.dim_value for d in output_tensor.type.tensor_type.shape.dim]}")
```

---

## 4. 验证模型文件

下载完成后，验证 `assets/models` 目录结构：

```bash
# 查看目录内容
ls -la assets/models/

# 应该包含以下文件：
# ├── config.json          # 模型配置
# ├── tokenizer.json       # 分词器配置
# ├── vocab.txt            # 词汇表
# └── model.onnx          # ONNX 模型（如果已转换）
```

### 快速测试模型加载

```rust
// test_model.rs
use xore_ai::EmbeddingModel;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let mut model = EmbeddingModel::load(
        Path::new("assets/models/model.onnx"),
        Path::new("assets/models/tokenizer.json")
    )?;
    
    let embedding = model.encode("Hello world")?;
    println!("向量维度: {}", embedding.len());
    
    Ok(())
}
```

---

## 5. CLI 集成

### 当前状态

XORE CLI 已支持 `--semantic` 参数（待优化），可以直接使用：

```bash
# 当前输出
xore f "搜索内容" --semantic
# 输出: "语义搜索功能.."
```

---

## 6. 测试语义搜索

### 单元测试

```bash
# 运行 xore-ai 测试
cargo test -p xore-ai

# 运行特定测试
cargo test -p xore-ai test_cosine_similarity
cargo test -p xore-ai test_embedding
```

### 集成测试

```rust
// 完整的语义搜索示例
use xore_ai::{EmbeddingModel, VectorSearcher, Document};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // 1. 加载模型
    let mut model = EmbeddingModel::load(
        Path::new("assets/models/model.onnx"),
        Path::new("assets/models/tokenizer.json")
    )?;
    
    // 2. 创建搜索引擎
    let mut searcher = VectorSearcher::new(model);
    
    // 3. 添加测试文档
    let docs = vec![
        Document {
            id: "1".to_string(),
            path: "rust_intro.txt".into(),
            content: "Rust 是一门系统编程语言，强调安全性和并发性".to_string(),
        },
        Document {
            id: "2".to_string(),
            path: "python_intro.txt".into(),
            content: "Python 是一门高级脚本语言，易于学习".to_string(),
        },
        Document {
            id: "3".to_string(),
            path: "js_intro.txt".into(),
            content: "JavaScript 是 Web 开发的主要语言".to_string(),
        },
    ];
    
    searcher.add_documents(docs)?;
    println!("已索引 {} 个文档", searcher.document_count());
    
    // 4. 执行语义搜索
    let results = searcher.search("系统编程", 3)?;
    
    println!("\n搜索结果:");
    for result in results {
        println!(
            "  - {} (相似度: {:.4})",
            result.document.path.display(),
            result.score
        );
    }
    
    Ok(())
}
```

---

## 7. 常见问题

### Q1: 模型下载失败

**问题**：网络连接 HuggingFace 失败

**解决方案**：

```bash
# 使用镜像
export HF_ENDPOINT=https://hf-mirror.com
huggingface-cli download sentence-transformers/all-MiniLM-L6-v2

# 或使用代理
export HTTPS_PROXY=http://127.0.0.1:7890
```

### Q2: ONNX 模型加载失败

**问题**：`ort` crate 无法加载模型

**解决方案**：

1. 确认模型是 ONNX 格式（.onnx 扩展名）
2. 检查模型输入输出名称是否正确
3. 尝试使用 `onnx.checker.check_model()` 验证模型

### Q3: 分词器加载失败

**问题**：`tokenizer.json` 格式错误

**解决方案**：

1. 确认 tokenizer.json 完整下载
2. 检查文件权限
3. 重新下载 tokenizer 文件

### Q4: 内存不足

**问题**：模型加载占用过多内存

**解决方案**：

1. 使用更小的模型（如 MiniLM-L6-v2）
2. 使用量化模型（INT8）
3. 增加系统内存

### Q5: 中文支持

**问题**：中文文本嵌入效果差

**解决方案**：

1. 使用多语言模型：`paraphrase-multilingual-MiniLM-L12-v2`
2. 或使用中文专用模型：`shibing624/text2vec-base-chinese`

---

## 快速开始脚本

一键下载并转换模型的完整脚本：

```bash
#!/bin/bash
# setup_semantic_search.sh

set -e

echo "=== XORE 语义搜索模型安装脚本 ==="

# 创建目录
mkdir -p assets/models
cd assets/models

# 检查 Python
if ! command -v python3 &> /dev/null; then
    echo "错误: 需要 Python 3.8+"
    exit 1
fi

# 安装依赖
echo "安装 Python 依赖..."
pip install -q huggingface-hub optimum[onnxruntime] transformers onnx

# 下载并转换模型
echo "下载 MiniLM-L6-v2 模型..."
python3 << 'EOF'
from huggingface_hub import snapshot_download
from optimum.onnxruntime import ORTModelForFeatureExtraction
from transformers import AutoTokenizer
import os

model_id = "sentence-transformers/all-MiniLM-L6-v2"
output_dir = "assets/models"

# 下载 tokenizer
print("下载 tokenizer...")
tokenizer = AutoTokenizer.from_pretrained(model_id)
tokenizer.save_pretrained(output_dir)

# 转换并保存 ONNX 模型
print("转换为 ONNX 格式...")
model = ORTModelForFeatureExtraction.from_pretrained(model_id, export=True)
model.save_pretrained(output_dir)

print("完成！")
print(f"模型文件保存在: {output_dir}")
EOF

# 验证文件
echo ""
echo "=== 验证安装 ==="
ls -lh assets/models/

echo ""
echo "=== 安装完成 ==="
echo "运行以下命令测试："
echo "  cargo test -p xore-ai"
```

运行脚本：

```bash
chmod +x setup_semantic_search.sh
./setup_semantic_search.sh
```

---

## 下一步

1. **运行测试**：验证模型正常工作
2. **CLI 集成**：实现 `xore f --semantic` 命令
3. **性能优化**：添加批量处理和缓存

如需更多帮助，请参考：

- [xore-ai README](../xore-ai/README.md)
- [ONNX Runtime 文档](https://onnxruntime.ai/)
- [HuggingFace Transformers](https://huggingface.co/docs/transformers/)
