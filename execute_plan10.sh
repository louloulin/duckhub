#!/bin/bash

# DuckHub Plan10 改造计划执行脚本
# 基于 plan10.md 的完整改造计划

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查前置条件
check_prerequisites() {
    log_info "检查前置条件..."
    
    # 检查 Rust 环境
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo 未安装，请先安装 Rust"
        exit 1
    fi
    
    # 检查 Node.js 环境
    if ! command -v npm &> /dev/null; then
        log_error "npm 未安装，请先安装 Node.js"
        exit 1
    fi
    
    # 检查 Git
    if ! command -v git &> /dev/null; then
        log_error "Git 未安装"
        exit 1
    fi
    
    log_success "前置条件检查通过"
}

# 创建备份
create_backup() {
    log_info "创建代码备份..."
    
    # 创建备份目录
    mkdir -p backup/$(date +%Y%m%d_%H%M%S)
    BACKUP_DIR="backup/$(date +%Y%m%d_%H%M%S)"
    
    # 备份关键文件
    log_info "备份模拟实现文件..."
    mkdir -p $BACKUP_DIR/mock_implementations
    
    if [ -f "crates/core/database/src/ducklake.rs" ]; then
        cp crates/core/database/src/ducklake.rs $BACKUP_DIR/mock_implementations/
        log_success "已备份 ducklake.rs"
    fi
    
    if [ -f "crates/core/database/src/lake.rs" ]; then
        cp crates/core/database/src/lake.rs $BACKUP_DIR/mock_implementations/
        log_success "已备份 lake.rs"
    fi
    
    if [ -f "scripts/fix_compilation.sh" ]; then
        cp scripts/fix_compilation.sh $BACKUP_DIR/mock_implementations/
        log_success "已备份 fix_compilation.sh"
    fi
    
    # 创建 Git 备份分支
    git checkout -b backup-before-plan10-$(date +%Y%m%d_%H%M%S)
    git add .
    git commit -m "Backup before Plan10 implementation" || true
    git checkout main
    
    log_success "备份完成: $BACKUP_DIR"
}

# Phase 1: Mock 代码清理
phase1_mock_cleanup() {
    log_info "=== Phase 1: Mock 代码清理 ==="
    
    # 1.1 后端 Mock 代码清理
    log_info "1.1 清理后端 Mock 代码..."
    
    # 更新导入引用
    log_info "更新导入引用..."
    find crates/ -name "*.rs" -type f -exec grep -l "use.*ducklake::" {} \; | while read file; do
        sed -i.bak 's/use crate::ducklake::/use crate::ducklake_real::/g' "$file"
        sed -i.bak 's/use.*ducklake::/use crate::ducklake_real::/g' "$file"
        rm "$file.bak"
        log_info "已更新: $file"
    done
    
    # 移除模拟实现文件
    if [ -f "crates/core/database/src/ducklake.rs" ]; then
        mv crates/core/database/src/ducklake.rs crates/core/database/src/ducklake_mock.rs.backup
        log_success "已移除 ducklake.rs 模拟实现"
    fi
    
    # 移除模拟构建脚本
    if [ -f "scripts/fix_compilation.sh" ]; then
        mv scripts/fix_compilation.sh scripts/fix_compilation.sh.backup
        log_success "已移除模拟构建脚本"
    fi
    
    # 1.2 清理 duckdb.rs 中的 fallback 代码
    log_info "1.2 清理 duckdb.rs 中的 fallback 代码..."
    
    DUCKDB_FILE="crates/core/database/src/duckdb.rs"
    if [ -f "$DUCKDB_FILE" ]; then
        # 备份原文件
        cp "$DUCKDB_FILE" "$DUCKDB_FILE.backup"
        
        # 移除 fallback 实现的注释标记
        sed -i.tmp '/\/\/ Fallback to mock implementation/,/^[[:space:]]*}$/d' "$DUCKDB_FILE"
        rm "$DUCKDB_FILE.tmp"
        
        log_success "已清理 duckdb.rs 中的 fallback 代码"
    fi
    
    # 1.3 清理前端 fallback 数据
    log_info "1.3 清理前端 fallback 数据..."
    
    # 清理 SnapshotBrowser.tsx
    SNAPSHOT_FILE="crates/web-frontend/src/components/ducklake/SnapshotBrowser.tsx"
    if [ -f "$SNAPSHOT_FILE" ]; then
        cp "$SNAPSHOT_FILE" "$SNAPSHOT_FILE.backup"
        
        # 移除 fallback 数据设置
        sed -i.tmp 's/setSnapshots(mockSnapshots)/setSnapshots([])/g' "$SNAPSHOT_FILE"
        sed -i.tmp '/\/\/ 不再使用fallback数据/d' "$SNAPSHOT_FILE"
        rm "$SNAPSHOT_FILE.tmp"
        
        log_success "已清理 SnapshotBrowser.tsx 中的 fallback 数据"
    fi
    
    # 清理其他前端组件
    find crates/web-frontend/src/components/ducklake/ -name "*.tsx" -type f | while read file; do
        if grep -q "fallback\|mock" "$file"; then
            cp "$file" "$file.backup"
            sed -i.tmp 's/\/\/ 不再使用fallback数据，直接设置为空数组/\/\/ 设置为空数组/g' "$file"
            sed -i.tmp '/fallback/d' "$file"
            rm "$file.tmp"
            log_info "已清理: $file"
        fi
    done
    
    log_success "Phase 1 完成: Mock 代码清理"
}

# Phase 2: 数据格式统一
phase2_data_format() {
    log_info "=== Phase 2: 数据格式统一 ==="
    
    # 2.1 后端响应格式标准化
    log_info "2.1 标准化后端响应格式..."
    
    # 创建统一响应结构
    cat > crates/services/web-api/src/models/response.rs << 'EOF'
//! 统一响应格式定义

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// 统一 API 响应格式
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: T,
    pub timestamp: String,
    pub request_id: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            message: "操作成功".to_string(),
            data,
            timestamp: Utc::now().to_rfc3339(),
            request_id: Some(Uuid::new_v4().to_string()),
        }
    }
    
    pub fn error(message: String) -> ApiResponse<serde_json::Value> {
        ApiResponse {
            success: false,
            message,
            data: serde_json::Value::Null,
            timestamp: Utc::now().to_rfc3339(),
            request_id: Some(Uuid::new_v4().to_string()),
        }
    }
}
EOF
    
    log_success "已创建统一响应格式"
    
    # 2.2 前端字段映射中间件
    log_info "2.2 添加前端字段映射中间件..."
    
    # 创建字段转换工具
    cat > crates/web-frontend/src/utils/dataTransform.ts << 'EOF'
// 字段名转换工具

export const convertSnakeCase = (obj: any): any => {
  if (Array.isArray(obj)) {
    return obj.map(convertSnakeCase)
  }
  
  if (obj !== null && typeof obj === 'object') {
    return Object.keys(obj).reduce((result, key) => {
      const camelKey = key.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase())
      result[camelKey] = convertSnakeCase(obj[key])
      return result
    }, {} as any)
  }
  
  return obj
}

export const convertCamelCase = (obj: any): any => {
  if (Array.isArray(obj)) {
    return obj.map(convertCamelCase)
  }
  
  if (obj !== null && typeof obj === 'object') {
    return Object.keys(obj).reduce((result, key) => {
      const snakeKey = key.replace(/[A-Z]/g, letter => `_${letter.toLowerCase()}`)
      result[snakeKey] = convertCamelCase(obj[key])
      return result
    }, {} as any)
  }
  
  return obj
}
EOF
    
    log_success "已创建字段转换工具"
    
    log_success "Phase 2 完成: 数据格式统一"
}

# Phase 3: API 端点完善
phase3_api_completion() {
    log_info "=== Phase 3: API 端点完善 ==="
    
    log_info "3.1 完善版本历史 API..."
    
    # 这里需要手动实现，脚本只能创建模板
    log_warning "版本历史 API 需要手动实现，请参考 plan10.md 中的代码示例"
    
    log_info "3.2 实现快照比较功能..."
    log_warning "快照比较功能需要手动实现，请参考 plan10.md 中的代码示例"
    
    log_success "Phase 3 完成: API 端点完善 (需要手动实现)"
}

# 编译和测试
compile_and_test() {
    log_info "=== 编译和测试 ==="
    
    # 编译后端
    log_info "编译后端代码..."
    if cargo build --workspace; then
        log_success "后端编译成功"
    else
        log_error "后端编译失败，请检查错误信息"
        return 1
    fi
    
    # 运行测试
    log_info "运行测试..."
    if cargo test --workspace; then
        log_success "测试通过"
    else
        log_warning "部分测试失败，请检查测试结果"
    fi
    
    # 编译前端
    log_info "编译前端代码..."
    cd crates/web-frontend
    if npm install && npm run build; then
        log_success "前端编译成功"
    else
        log_error "前端编译失败，请检查错误信息"
        cd ../..
        return 1
    fi
    cd ../..
    
    log_success "编译和测试完成"
}

# 主执行函数
main() {
    echo "🦆 DuckHub Plan10 改造计划执行开始"
    echo "========================================"
    
    check_prerequisites
    create_backup
    
    # 执行各个阶段
    phase1_mock_cleanup
    phase2_data_format
    phase3_api_completion
    
    # 编译和测试
    compile_and_test
    
    echo "========================================"
    echo "🎉 Plan10 改造计划执行完成！"
    echo ""
    echo "📋 后续手动任务："
    echo "1. 实现版本历史 API (参考 plan10.md)"
    echo "2. 实现快照比较功能"
    echo "3. 完善错误处理和用户体验"
    echo "4. 添加监控和告警配置"
    echo ""
    echo "📖 详细信息请查看 plan10.md"
}

# 执行主函数
main "$@"
