# XORE AI Module

基于 ONNX Runtime 的语义嵌入和向量搜索模块。

## 功能特性

- ✅ ONNX 模型加载和推理
- ✅ 文本分词（基于 HuggingFace tokenizers）
- ✅ 文本嵌入向量生成（384维）
- ✅ 向量相似度搜索
- ✅ 余弦相似度计算

## 使用前准备

### 1. 下载模型文件

需要以下文件：

```bash
# 创建模型目录
mkdir -p assets/models

# 下载 MiniLM-L6-v2 模型（示例）
# 从 Hugging Face 下载：
# https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2

# 需要的文件：
# - model.onnx (或 minilm-l6-v2.onnx)
# - tokenizer.json
```

### 2. 模型转换（如果需要）

如果只有 PyTorch 模型，需要转换为 ONNX 格式：

```python
from optimum.onnxruntime import ORTModelForFeatureExtraction
from transformers import AutoTokenizer

model_id = "sentence-transformers/all-MiniLM-L6-v2"
tokenizer = AutoTokenizer.from_pretrained(model_id)
model = ORTModelForFeatureExtraction.from_pretrained(model_id, export=True)

# 保存
model.save_pretrained("assets/models/")
tokenizer.save_pretrained("assets/models/")
```

## 使用示例

### 基本用法

```rust
use xore_ai::{EmbeddingModel, VectorSearcher, Document};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 加载模型
    let mut model = EmbeddingModel::load(
        Path::new("assets/models/model.onnx"),
        Path::new("assets/models/tokenizer.json")
    )?;

    // 2. 生成嵌入向量
    let text = "这是一个测试文本";
    let embedding = model.encode(text)?;
    println!("向量维度: {}", embedding.len()); // 384

    // 3. 计算相似度
    let text2 = "这是另一个测试";
    let embedding2 = model.encode(text2)?;
    let similarity = EmbeddingModel::cosine_similarity(&embedding, &embedding2);
    println!("相似度: {:.4}", similarity);

    Ok(())
}
```

### 向量搜索

```rust
use xore_ai::{EmbeddingModel, VectorSearcher, Document};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 加载模型
    let model = EmbeddingModel::load(
        Path::new("assets/models/model.onnx"),
        Path::new("assets/models/tokenizer.json")
    )?;

    // 2. 创建搜索引擎
    let mut searcher = VectorSearcher::new(model);

    // 3. 添加文档
    searcher.add_document(Document {
        id: "1".to_string(),
        path: PathBuf::from("doc1.txt"),
        content: "Rust 是一门系统编程语言".to_string(),
    })?;

    searcher.add_document(Document {
        id: "2".to_string(),
        path: PathBuf::from("doc2.txt"),
        content: "Python 是一门脚本语言".to_string(),
    })?;

    // 4. 语义搜索
    let results = searcher.search("编程语言", 5)?;
    
    for result in results {
        println!("文档: {:?}, 相似度: {:.4}", 
            result.document.path, 
            result.score
        );
    }

    Ok(())
}
```

### 批量编码

```rust
let texts = vec![
    "文本1".to_string(),
    "文本2".to_string(),
    "文本3".to_string(),
];

let embeddings = model.encode_batch(&texts)?;
println!("生成了 {} 个嵌入向量", embeddings.len());
```

## API 文档

### EmbeddingModel

#### 方法

- `load(model_path, tokenizer_path) -> Result<Self>`
  - 加载 ONNX 模型和分词器

- `encode(&mut self, text: &str) -> Result<Vec<f32>>`
  - 生成文本嵌入向量（需要 &mut self）

- `encode_batch(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>>`
  - 批量生成嵌入向量

- `cosine_similarity(a: &[f32], b: &[f32]) -> f32`
  - 计算余弦相似度（静态方法）

- `dimension(&self) -> usize`
  - 获取向量维度

### VectorSearcher

#### 方法

- `new(model: EmbeddingModel) -> Self`
  - 创建搜索引擎

- `add_document(&mut self, doc: Document) -> Result<()>`
  - 添加单个文档到索引

- `add_documents(&mut self, docs: Vec<Document>) -> Result<usize>`
  - 批量添加文档

- `search(&mut self, query: &str, top_k: usize) -> Result<Vec<SearchResult>>`
  - 语义搜索

- `document_count(&self) -> usize`
  - 获取索引中的文档数量

- `clear(&mut self)`
  - 清空索引

## 性能指标

| 操作 | 目标 | 说明 |
|------|------|------|
| 单文本嵌入 | <100ms | CPU 推理 |
| 批量编码 | >100 texts/s | 批量处理 |
| 相似度搜索 | <50ms | 10K 文档 |
| 模型加载 | <2s | 首次加载 |

## 注意事项

1. **可变引用要求**：由于 ONNX Runtime 的 `Session::run()` 需要可变引用，所有使用模型的方法都需要 `&mut self`。

2. **模型文件**：确保模型文件和 tokenizer.json 在正确的路径。

3. **内存占用**：模型加载后会占用约 80-100MB 内存。

4. **线程安全**：当前实现不是线程安全的，如需多线程使用，请为每个线程创建独立的模型实例。

## 测试

```bash
# 运行单元测试
cargo test -p xore-ai

# 运行特定测试
cargo test -p xore-ai test_cosine_similarity
```

## 依赖

- `ort`: ONNX Runtime Rust 绑定
- `tokenizers`: HuggingFace tokenizers
- `ndarray`: 数组操作
- `anyhow`: 错误处理

## 许可证

MIT
