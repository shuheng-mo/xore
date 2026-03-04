//! 语义嵌入模型
//!
//! 基于 ONNX Runtime 实现文本嵌入向量生成
//!
//! # 注意
//!
//! 此模块需要以下文件才能正常工作：
//! - ONNX 模型文件（如 MiniLM-L6-v2.onnx）
//! - tokenizer.json 文件
//!
//! 由于 ONNX Runtime 的 Session::run() 需要可变引用，
//! 所有使用模型的方法都需要 &mut self。

use anyhow::{Context, Result};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::TensorRef;
use std::path::Path;
use tokenizers::Tokenizer;
use tracing::{debug, info};

/// 嵌入模型
pub struct EmbeddingModel {
    /// ONNX 推理会话
    session: Session,
    /// 分词器
    tokenizer: Tokenizer,
    /// 向量维度
    dimension: usize,
    /// 最大序列长度
    max_length: usize,
}

impl EmbeddingModel {
    /// 从文件加载模型和分词器
    ///
    /// # 参数
    /// - `model_path`: ONNX 模型文件路径
    /// - `tokenizer_path`: tokenizer.json 文件路径
    ///
    /// # 返回
    /// 加载好的嵌入模型
    ///
    /// # 示例
    /// ```ignore
    /// let model = EmbeddingModel::load(
    ///     Path::new("assets/models/minilm-l6-v2.onnx"),
    ///     Path::new("assets/models/tokenizer.json")
    /// )?;
    /// ```
    pub fn load(model_path: &Path, tokenizer_path: &Path) -> Result<Self> {
        info!("加载嵌入模型: {:?}", model_path);
        info!("加载分词器: {:?}", tokenizer_path);

        // 1. 加载分词器
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("加载分词器失败: {}", e))?;

        // 2. 创建 ONNX Runtime 会话
        let session = Session::builder()
            .context("创建 Session Builder 失败")?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .context("设置优化级别失败")?
            .with_intra_threads(4)
            .context("设置线程数失败")?
            .commit_from_file(model_path)
            .context("加载 ONNX 模型失败")?;

        debug!("模型加载成功");

        Ok(Self {
            session,
            tokenizer,
            dimension: 384, // MiniLM-L6-v2 默认维度
            max_length: 512,
        })
    }

    /// 生成文本嵌入向量
    ///
    /// # 参数
    /// - `text`: 输入文本
    ///
    /// # 返回
    /// 归一化的嵌入向量（384维）
    ///
    /// # 注意
    /// 需要 &mut self 因为 ONNX Session::run() 需要可变引用
    pub fn encode(&mut self, text: &str) -> Result<Vec<f32>> {
        // 1. 分词
        let encoding =
            self.tokenizer.encode(text, true).map_err(|e| anyhow::anyhow!("分词失败: {}", e))?;

        let ids = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();

        // 2. 截断或填充到最大长度
        let seq_len = ids.len().min(self.max_length);
        let mut input_ids = vec![0i64; self.max_length];
        let mut input_mask = vec![0i64; self.max_length];
        let mut token_type_ids = vec![0i64; self.max_length]; // 添加 token_type_ids

        for i in 0..seq_len {
            input_ids[i] = ids[i] as i64;
            input_mask[i] = attention_mask[i] as i64;
            // token_type_ids 保持为 0（单句子输入）
        }

        // 3. 创建 TensorRef (使用元组格式: (shape, &data))
        let shape = vec![1_usize, self.max_length];
        let input_ids_tensor = TensorRef::from_array_view((shape.clone(), input_ids.as_slice()))
            .context("创建 input_ids tensor 失败")?;
        let attention_mask_tensor =
            TensorRef::from_array_view((shape.clone(), input_mask.as_slice()))
                .context("创建 attention_mask tensor 失败")?;
        let token_type_ids_tensor = TensorRef::from_array_view((shape, token_type_ids.as_slice()))
            .context("创建 token_type_ids tensor 失败")?;

        // 4. 运行推理
        let outputs = self
            .session
            .run(ort::inputs![
                "input_ids" => input_ids_tensor,
                "attention_mask" => attention_mask_tensor,
                "token_type_ids" => token_type_ids_tensor,
            ])
            .context("ONNX 推理失败")?;

        // 5. 提取输出 (last_hidden_state)
        // MiniLM 模型输出: [batch_size, seq_len, hidden_size]
        let output_tensor = &outputs[0];
        let output_array = output_tensor.try_extract_tensor::<f32>().context("提取输出张量失败")?;

        // 6. 平均池化 (取所有 token 的平均值)
        let embeddings =
            Self::mean_pooling(&output_array.1, &input_mask, self.max_length, self.dimension)?;

        // 7. L2 归一化
        let normalized = Self::normalize(&embeddings);

        Ok(normalized)
    }

    /// 批量编码优化
    ///
    /// # 参数
    /// - `texts`: 文本列表
    ///
    /// # 返回
    /// 嵌入向量列表
    pub fn encode_batch(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // 简单实现：逐个编码（后续可优化为真正的批量处理）
        texts.iter().map(|text| self.encode(text)).collect()
    }

    /// 平均池化
    ///
    /// 对所有非填充 token 的隐藏状态求平均
    fn mean_pooling(
        hidden_states: &[f32],
        attention_mask: &[i64],
        max_length: usize,
        dimension: usize,
    ) -> Result<Vec<f32>> {
        let seq_len = max_length;
        let hidden_size = dimension;

        let mut pooled = vec![0.0f32; hidden_size];
        let mut count = 0;

        for i in 0..seq_len {
            if attention_mask[i] == 1 {
                for j in 0..hidden_size {
                    let idx = i * hidden_size + j;
                    if idx < hidden_states.len() {
                        pooled[j] += hidden_states[idx];
                    }
                }
                count += 1;
            }
        }

        // 求平均
        if count > 0 {
            for val in pooled.iter_mut() {
                *val /= count as f32;
            }
        }

        Ok(pooled)
    }

    /// L2 归一化
    fn normalize(vec: &[f32]) -> Vec<f32> {
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            vec.iter().map(|x| x / norm).collect()
        } else {
            vec.to_vec()
        }
    }

    /// 计算余弦相似度
    ///
    /// # 参数
    /// - `a`: 向量 A
    /// - `b`: 向量 B
    ///
    /// # 返回
    /// 余弦相似度 [-1, 1]
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a > 0.0 && norm_b > 0.0 {
            dot_product / (norm_a * norm_b)
        } else {
            0.0
        }
    }

    /// 获取向量维度
    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        let vec = vec![3.0, 4.0];
        let normalized = EmbeddingModel::normalize(&vec);

        assert!((normalized[0] - 0.6).abs() < 1e-6);
        assert!((normalized[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_vectors() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];

        let similarity = EmbeddingModel::cosine_similarity(&a, &b);
        assert!((similarity - 1.0).abs() < 1e-6);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        let similarity2 = EmbeddingModel::cosine_similarity(&c, &d);
        assert!((similarity2 - 0.0).abs() < 1e-6);
    }
}
