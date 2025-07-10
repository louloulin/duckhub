//! RAG增强的AI Agent实现
//! 集成向量检索和知识库，提供更准确的AI响应

use duckhub_common::prelude::*;
use crate::vector_store::{VectorStore, FinancialKnowledgeBase, RetrievalResult, VectorStoreError};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug, instrument, error};
use anyhow::{Result as AnyhowResult, Context};
use thiserror::Error;
use async_trait::async_trait;

// Rig框架导入
use rig::{
    providers::deepseek,
    agent::Agent,
    completion::Prompt,
};

/// RAG Agent错误类型
#[derive(Error, Debug)]
pub enum RAGAgentError {
    #[error("知识检索错误: {0}")]
    RetrievalError(String),
    #[error("上下文生成错误: {0}")]
    ContextError(String),
    #[error("Agent调用错误: {0}")]
    AgentError(String),
    #[error("向量存储错误: {0}")]
    VectorStoreError(#[from] VectorStoreError),
}

/// RAG增强的查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGQueryRequest {
    /// 用户查询
    pub query: String,
    /// 是否启用RAG
    pub enable_rag: bool,
    /// 最大检索文档数
    pub max_retrieval_docs: Option<usize>,
    /// 上下文窗口大小
    pub context_window_size: Option<usize>,
}

/// RAG增强的查询响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGQueryResponse {
    /// AI生成的回答
    pub answer: String,
    /// 使用的上下文
    pub context: Vec<String>,
    /// 检索到的相关文档
    pub retrieved_docs: Vec<String>,
    /// 检索时间（毫秒）
    pub retrieval_time_ms: u64,
    /// 生成时间（毫秒）
    pub generation_time_ms: u64,
    /// 是否使用了RAG
    pub used_rag: bool,
}

/// RAG Agent配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGAgentConfig {
    /// 最大检索文档数
    pub max_retrieval_docs: usize,
    /// 上下文窗口大小
    pub context_window_size: usize,
    /// 相关性阈值
    pub relevance_threshold: f32,
    /// 是否默认启用RAG
    pub enable_rag_by_default: bool,
}

impl Default for RAGAgentConfig {
    fn default() -> Self {
        Self {
            max_retrieval_docs: 5,
            context_window_size: 4000,
            relevance_threshold: 0.7,
            enable_rag_by_default: true,
        }
    }
}

/// RAG增强的SQL Agent
pub struct RAGSQLAgent {
    /// 基础Agent
    base_agent: Agent<deepseek::DeepSeekCompletionModel>,
    /// 金融知识库
    knowledge_base: Arc<FinancialKnowledgeBase>,
    /// 配置
    config: RAGAgentConfig,
}

impl RAGSQLAgent {
    /// 创建新的RAG SQL Agent
    pub fn new(
        base_agent: Agent<deepseek::DeepSeekCompletionModel>,
        knowledge_base: Arc<FinancialKnowledgeBase>,
        config: RAGAgentConfig,
    ) -> Self {
        Self {
            base_agent,
            knowledge_base,
            config,
        }
    }
    
    /// 处理RAG增强的查询
    #[instrument(skip(self, request))]
    pub async fn process_query(&self, request: RAGQueryRequest) -> Result<RAGQueryResponse> {
        let start_time = std::time::Instant::now();
        
        let mut context = Vec::new();
        let mut retrieved_docs = Vec::new();
        let mut retrieval_time_ms = 0;
        let used_rag = request.enable_rag;
        
        // 如果启用RAG，先检索相关知识
        if request.enable_rag {
            let retrieval_start = std::time::Instant::now();
            
            match self.retrieve_relevant_knowledge(&request.query).await {
                Ok(retrieval_result) => {
                    retrieval_time_ms = retrieval_result.retrieval_time_ms;
                    
                    // 提取检索到的文档内容
                    for chunk in retrieval_result.chunks {
                        if chunk.score.unwrap_or(0.0) >= self.config.relevance_threshold {
                            context.push(chunk.content.clone());
                            retrieved_docs.push(format!("文档ID: {} (相关性: {:.2})", 
                                chunk.id, chunk.score.unwrap_or(0.0)));
                        }
                    }
                    
                    info!("检索到 {} 个相关文档", context.len());
                }
                Err(e) => {
                    warn!("知识检索失败: {}, 继续使用基础Agent", e);
                }
            }
        }
        
        // 构建增强的提示词
        let enhanced_prompt = self.build_enhanced_prompt(&request.query, &context);
        
        // 调用基础Agent生成回答
        let generation_start = std::time::Instant::now();
        let answer = self.base_agent
            .prompt(&enhanced_prompt)
            .await
            .map_err(|e| DuckHubError::internal(e.to_string()))?;
        
        let generation_time_ms = generation_start.elapsed().as_millis() as u64;
        
        Ok(RAGQueryResponse {
            answer,
            context,
            retrieved_docs,
            retrieval_time_ms,
            generation_time_ms,
            used_rag,
        })
    }
    
    /// 检索相关知识
    async fn retrieve_relevant_knowledge(&self, query: &str) -> Result<RetrievalResult> {
        let max_docs = self.config.max_retrieval_docs;
        self.knowledge_base
            .retrieve_knowledge(query)
            .await
            .map_err(|e| DuckHubError::internal(e.to_string()))
    }
    
    /// 构建增强的提示词
    fn build_enhanced_prompt(&self, query: &str, context: &[String]) -> String {
        if context.is_empty() {
            // 没有检索到相关知识，使用基础提示词
            format!(
                "你是一个专业的金融数据分析师和SQL专家。请根据用户的问题生成准确的SQL查询或提供专业的分析建议。\n\n用户问题: {}\n\n请提供详细的回答:",
                query
            )
        } else {
            // 有检索到的知识，构建增强提示词
            let context_str = context.join("\n\n");
            format!(
                "你是一个专业的金融数据分析师和SQL专家。请根据以下相关知识和用户的问题，生成准确的SQL查询或提供专业的分析建议。\n\n相关知识:\n{}\n\n用户问题: {}\n\n请基于上述知识提供详细的回答:",
                context_str,
                query
            )
        }
    }
}

/// RAG增强的分析Agent
pub struct RAGAnalysisAgent {
    /// 基础Agent
    base_agent: Agent<deepseek::DeepSeekCompletionModel>,
    /// 金融知识库
    knowledge_base: Arc<FinancialKnowledgeBase>,
    /// 配置
    config: RAGAgentConfig,
}

impl RAGAnalysisAgent {
    /// 创建新的RAG分析Agent
    pub fn new(
        base_agent: Agent<deepseek::DeepSeekCompletionModel>,
        knowledge_base: Arc<FinancialKnowledgeBase>,
        config: RAGAgentConfig,
    ) -> Self {
        Self {
            base_agent,
            knowledge_base,
            config,
        }
    }
    
    /// 处理分析请求
    #[instrument(skip(self, request))]
    pub async fn analyze(&self, request: RAGQueryRequest) -> Result<RAGQueryResponse> {
        let start_time = std::time::Instant::now();
        
        let mut context = Vec::new();
        let mut retrieved_docs = Vec::new();
        let mut retrieval_time_ms = 0;
        let used_rag = request.enable_rag;
        
        // RAG检索
        if request.enable_rag {
            match self.knowledge_base.retrieve_knowledge(&request.query).await {
                Ok(retrieval_result) => {
                    retrieval_time_ms = retrieval_result.retrieval_time_ms;
                    
                    for chunk in retrieval_result.chunks {
                        if chunk.score.unwrap_or(0.0) >= self.config.relevance_threshold {
                            context.push(chunk.content.clone());
                            retrieved_docs.push(format!("文档ID: {} (相关性: {:.2})", 
                                chunk.id, chunk.score.unwrap_or(0.0)));
                        }
                    }
                }
                Err(e) => {
                    warn!("知识检索失败: {}", e);
                }
            }
        }
        
        // 构建分析提示词
        let analysis_prompt = self.build_analysis_prompt(&request.query, &context);
        
        // 生成分析结果
        let generation_start = std::time::Instant::now();
        let answer = self.base_agent
            .prompt(&analysis_prompt)
            .await
            .map_err(|e| DuckHubError::internal(e.to_string()))?;
        
        let generation_time_ms = generation_start.elapsed().as_millis() as u64;
        
        Ok(RAGQueryResponse {
            answer,
            context,
            retrieved_docs,
            retrieval_time_ms,
            generation_time_ms,
            used_rag,
        })
    }
    
    /// 构建分析提示词
    fn build_analysis_prompt(&self, query: &str, context: &[String]) -> String {
        if context.is_empty() {
            format!(
                "你是一个专业的金融数据分析师。请对以下数据或问题进行深入分析，提供专业的洞察和建议。\n\n分析请求: {}\n\n请提供详细的分析报告:",
                query
            )
        } else {
            let context_str = context.join("\n\n");
            format!(
                "你是一个专业的金融数据分析师。请基于以下相关知识对数据或问题进行深入分析，提供专业的洞察和建议。\n\n相关知识:\n{}\n\n分析请求: {}\n\n请基于上述知识提供详细的分析报告:",
                context_str,
                query
            )
        }
    }
}

/// RAG增强的聊天Agent
pub struct RAGChatAgent {
    /// 基础Agent
    base_agent: Agent<deepseek::DeepSeekCompletionModel>,
    /// 金融知识库
    knowledge_base: Arc<FinancialKnowledgeBase>,
    /// 配置
    config: RAGAgentConfig,
    /// 对话历史
    conversation_history: Vec<String>,
}

impl RAGChatAgent {
    /// 创建新的RAG聊天Agent
    pub fn new(
        base_agent: Agent<deepseek::DeepSeekCompletionModel>,
        knowledge_base: Arc<FinancialKnowledgeBase>,
        config: RAGAgentConfig,
    ) -> Self {
        Self {
            base_agent,
            knowledge_base,
            config,
            conversation_history: Vec::new(),
        }
    }
    
    /// 处理聊天消息
    #[instrument(skip(self, request))]
    pub async fn chat(&mut self, request: RAGQueryRequest) -> Result<RAGQueryResponse> {
        let mut context = Vec::new();
        let mut retrieved_docs = Vec::new();
        let mut retrieval_time_ms = 0;
        let used_rag = request.enable_rag;
        
        // RAG检索
        if request.enable_rag {
            match self.knowledge_base.retrieve_knowledge(&request.query).await {
                Ok(retrieval_result) => {
                    retrieval_time_ms = retrieval_result.retrieval_time_ms;
                    
                    for chunk in retrieval_result.chunks {
                        if chunk.score.unwrap_or(0.0) >= self.config.relevance_threshold {
                            context.push(chunk.content.clone());
                            retrieved_docs.push(format!("文档ID: {} (相关性: {:.2})", 
                                chunk.id, chunk.score.unwrap_or(0.0)));
                        }
                    }
                }
                Err(e) => {
                    warn!("知识检索失败: {}", e);
                }
            }
        }
        
        // 构建聊天提示词
        let chat_prompt = self.build_chat_prompt(&request.query, &context);
        
        // 生成回答
        let generation_start = std::time::Instant::now();
        let answer = self.base_agent
            .prompt(&chat_prompt)
            .await
            .map_err(|e| DuckHubError::internal(e.to_string()))?;
        
        let generation_time_ms = generation_start.elapsed().as_millis() as u64;
        
        // 更新对话历史
        self.conversation_history.push(format!("用户: {}", request.query));
        self.conversation_history.push(format!("助手: {}", answer));
        
        // 保持对话历史在合理长度内
        if self.conversation_history.len() > 20 {
            self.conversation_history.drain(0..2);
        }
        
        Ok(RAGQueryResponse {
            answer,
            context,
            retrieved_docs,
            retrieval_time_ms,
            generation_time_ms,
            used_rag,
        })
    }
    
    /// 构建聊天提示词
    fn build_chat_prompt(&self, query: &str, context: &[String]) -> String {
        let mut prompt = String::new();
        
        // 添加系统提示
        prompt.push_str("你是一个专业的金融AI助手，能够回答各种金融相关问题并提供专业建议。");
        
        // 添加相关知识
        if !context.is_empty() {
            prompt.push_str("\n\n相关知识:\n");
            prompt.push_str(&context.join("\n\n"));
        }
        
        // 添加对话历史
        if !self.conversation_history.is_empty() {
            prompt.push_str("\n\n对话历史:\n");
            for history in &self.conversation_history {
                prompt.push_str(history);
                prompt.push('\n');
            }
        }
        
        // 添加当前问题
        prompt.push_str(&format!("\n\n用户: {}\n助手:", query));
        
        prompt
    }
    
    /// 清除对话历史
    pub fn clear_history(&mut self) {
        self.conversation_history.clear();
    }
    
    /// 获取对话历史
    pub fn get_history(&self) -> &[String] {
        &self.conversation_history
    }
}
