//! 向量存储模块
//! 为RAG系统提供文档索引和检索功能

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use thiserror::Error;
use async_trait::async_trait;

/// 向量存储错误类型
#[derive(Error, Debug)]
pub enum VectorStoreError {
    #[error("索引错误: {0}")]
    IndexError(String),
    #[error("检索错误: {0}")]
    RetrievalError(String),
    #[error("嵌入生成错误: {0}")]
    EmbeddingError(String),
    #[error("存储错误: {0}")]
    StorageError(String),
}

/// 文档片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    /// 文档ID
    pub id: String,
    /// 内容
    pub content: String,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 向量表示
    pub embedding: Option<Vec<f32>>,
    /// 相关性分数
    pub score: Option<f32>,
}

/// 检索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    /// 检索到的文档片段
    pub chunks: Vec<DocumentChunk>,
    /// 查询向量
    pub query_embedding: Vec<f32>,
    /// 检索时间（毫秒）
    pub retrieval_time_ms: u64,
}

/// 向量存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    /// 向量维度
    pub dimension: usize,
    /// 索引类型
    pub index_type: String,
    /// 最大检索数量
    pub max_results: usize,
    /// 相似度阈值
    pub similarity_threshold: f32,
    /// 嵌入模型名称
    pub embedding_model: String,
}

impl Default for VectorStoreConfig {
    fn default() -> Self {
        Self {
            dimension: 768,
            index_type: "IVFFlat".to_string(),
            max_results: 10,
            similarity_threshold: 0.7,
            embedding_model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        }
    }
}

/// 向量存储trait
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// 添加文档
    async fn add_documents(&mut self, documents: Vec<DocumentChunk>) -> std::result::Result<(), VectorStoreError>;
    
    /// 检索相似文档
    async fn retrieve(&self, query: &str, limit: usize) -> Result<RetrievalResult, VectorStoreError>;
    
    /// 删除文档
    async fn delete_document(&mut self, doc_id: &str) -> Result<(), VectorStoreError>;
    
    /// 更新文档
    async fn update_document(&mut self, document: DocumentChunk) -> Result<(), VectorStoreError>;
    
    /// 获取文档数量
    async fn count(&self) -> Result<usize, VectorStoreError>;
    
    /// 清空索引
    async fn clear(&mut self) -> Result<(), VectorStoreError>;
}

/// 内存向量存储实现
pub struct InMemoryVectorStore {
    /// 配置
    config: VectorStoreConfig,
    /// 文档存储
    documents: HashMap<String, DocumentChunk>,
    /// 嵌入生成器
    embedding_generator: Arc<dyn EmbeddingGenerator>,
}

impl InMemoryVectorStore {
    /// 创建新的内存向量存储
    pub fn new(config: VectorStoreConfig, embedding_generator: Arc<dyn EmbeddingGenerator>) -> Self {
        Self {
            config,
            documents: HashMap::new(),
            embedding_generator,
        }
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    #[instrument(skip(self, documents))]
    async fn add_documents(&mut self, mut documents: Vec<DocumentChunk>) -> Result<(), VectorStoreError> {
        for document in &mut documents {
            // 生成嵌入向量
            if document.embedding.is_none() {
                let embedding = self.embedding_generator
                    .generate_embedding(&document.content)
                    .await
                    .map_err(|e| VectorStoreError::EmbeddingError(e.to_string()))?;
                document.embedding = Some(embedding);
            }
            
            // 存储文档
            self.documents.insert(document.id.clone(), document.clone());
        }
        
        info!("添加了 {} 个文档到向量存储", documents.len());
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn retrieve(&self, query: &str, limit: usize) -> Result<RetrievalResult, VectorStoreError> {
        let start_time = std::time::Instant::now();
        
        // 生成查询向量
        let query_embedding = self.embedding_generator
            .generate_embedding(query)
            .await
            .map_err(|e| VectorStoreError::EmbeddingError(e.to_string()))?;
        
        // 计算相似度并排序
        let mut scored_docs: Vec<_> = self.documents
            .values()
            .filter_map(|doc| {
                if let Some(ref doc_embedding) = doc.embedding {
                    let similarity = cosine_similarity(&query_embedding, doc_embedding);
                    if similarity >= self.config.similarity_threshold {
                        let mut scored_doc = doc.clone();
                        scored_doc.score = Some(similarity);
                        Some(scored_doc)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        
        // 按相似度排序
        scored_docs.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // 限制结果数量
        scored_docs.truncate(limit.min(self.config.max_results));
        
        let retrieval_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(RetrievalResult {
            chunks: scored_docs,
            query_embedding,
            retrieval_time_ms,
        })
    }
    
    async fn delete_document(&mut self, doc_id: &str) -> Result<(), VectorStoreError> {
        self.documents.remove(doc_id);
        Ok(())
    }
    
    async fn update_document(&mut self, mut document: DocumentChunk) -> Result<(), VectorStoreError> {
        // 重新生成嵌入向量
        let embedding = self.embedding_generator
            .generate_embedding(&document.content)
            .await
            .map_err(|e| VectorStoreError::EmbeddingError(e.to_string()))?;
        document.embedding = Some(embedding);
        
        self.documents.insert(document.id.clone(), document);
        Ok(())
    }
    
    async fn count(&self) -> Result<usize, VectorStoreError> {
        Ok(self.documents.len())
    }
    
    async fn clear(&mut self) -> Result<(), VectorStoreError> {
        self.documents.clear();
        Ok(())
    }
}

/// 嵌入生成器trait
#[async_trait]
pub trait EmbeddingGenerator: Send + Sync {
    /// 生成文本嵌入向量
    async fn generate_embedding(&self, text: &str) -> std::result::Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>>;

    /// 批量生成嵌入向量
    async fn generate_embeddings(&self, texts: &[String]) -> std::result::Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>>;
}

/// 简单的嵌入生成器实现（用于测试）
pub struct SimpleEmbeddingGenerator {
    dimension: usize,
}

impl SimpleEmbeddingGenerator {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait]
impl EmbeddingGenerator for SimpleEmbeddingGenerator {
    async fn generate_embedding(&self, text: &str) -> std::result::Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        // 简单的哈希基础嵌入生成（仅用于测试）
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();
        
        let mut embedding = vec![0.0; self.dimension];
        for i in 0..self.dimension {
            embedding[i] = ((hash.wrapping_add(i as u64)) as f32 / u64::MAX as f32) * 2.0 - 1.0;
        }
        
        // 归一化
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }
        
        Ok(embedding)
    }
    
    async fn generate_embeddings(&self, texts: &[String]) -> std::result::Result<Vec<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut embeddings = Vec::new();
        for text in texts {
            embeddings.push(self.generate_embedding(text).await?);
        }
        Ok(embeddings)
    }
}

/// 计算余弦相似度
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// 金融知识库
pub struct FinancialKnowledgeBase {
    /// 向量存储
    vector_store: Box<dyn VectorStore>,
    /// 知识库配置
    config: VectorStoreConfig,
}

impl FinancialKnowledgeBase {
    /// 创建新的金融知识库
    pub fn new(vector_store: Box<dyn VectorStore>, config: VectorStoreConfig) -> Self {
        Self {
            vector_store,
            config,
        }
    }
    
    /// 初始化金融知识库
    #[instrument(skip(self))]
    pub async fn initialize(&mut self) -> Result<(), VectorStoreError> {
        info!("初始化金融知识库...");
        
        // 添加基础金融知识
        let financial_docs = self.create_financial_documents();
        self.vector_store.add_documents(financial_docs).await?;
        
        info!("金融知识库初始化完成");
        Ok(())
    }
    
    /// 检索相关金融知识
    #[instrument(skip(self))]
    pub async fn retrieve_knowledge(&self, query: &str) -> Result<RetrievalResult, VectorStoreError> {
        self.vector_store.retrieve(query, self.config.max_results).await
    }
    
    /// 创建基础金融文档
    fn create_financial_documents(&self) -> Vec<DocumentChunk> {
        vec![
            DocumentChunk {
                id: "fin_001".to_string(),
                content: "财务报表分析是评估公司财务健康状况的重要方法，包括资产负债表、利润表和现金流量表的分析。".to_string(),
                metadata: [("category".to_string(), "财务分析".to_string())].into_iter().collect(),
                embedding: None,
                score: None,
            },
            DocumentChunk {
                id: "fin_002".to_string(),
                content: "风险管理是金融机构的核心功能，包括信用风险、市场风险、操作风险和流动性风险的识别、测量和控制。".to_string(),
                metadata: [("category".to_string(), "风险管理".to_string())].into_iter().collect(),
                embedding: None,
                score: None,
            },
            DocumentChunk {
                id: "fin_003".to_string(),
                content: "投资组合理论强调通过分散投资来降低风险，同时追求最优的风险收益比。".to_string(),
                metadata: [("category".to_string(), "投资理论".to_string())].into_iter().collect(),
                embedding: None,
                score: None,
            },
            DocumentChunk {
                id: "fin_004".to_string(),
                content: "SQL查询优化包括索引使用、查询重写、执行计划分析等技术，可以显著提升数据库查询性能。".to_string(),
                metadata: [("category".to_string(), "数据库技术".to_string())].into_iter().collect(),
                embedding: None,
                score: None,
            },
            DocumentChunk {
                id: "fin_005".to_string(),
                content: "时间序列分析在金融数据分析中广泛应用，包括趋势分析、季节性分析和预测建模。".to_string(),
                metadata: [("category".to_string(), "数据分析".to_string())].into_iter().collect(),
                embedding: None,
                score: None,
            },
        ]
    }
}
