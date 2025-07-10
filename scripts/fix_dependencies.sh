#!/bin/bash

# DuckLake 依赖修复脚本
# 
# 本脚本用于修复 DuckLake 核心底座的编译依赖问题

set -e

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

# 检查项目根目录
if [ ! -f "Cargo.toml" ]; then
    log_error "请在项目根目录运行此脚本"
    exit 1
fi

log_info "开始修复 DuckLake 依赖问题..."

# 1. 添加缺失的依赖到工作空间 Cargo.toml
log_info "1. 更新工作空间依赖..."

# 检查是否已经有 duckdb 依赖
if ! grep -q "duckdb.*=" Cargo.toml; then
    log_info "添加 duckdb 依赖到工作空间..."
    # 这里我们需要手动编辑，因为 cargo add 不支持工作空间级别的依赖添加
    log_warning "请手动添加以下依赖到 Cargo.toml 的 [workspace.dependencies] 部分："
    echo ""
    echo "# DuckDB 数据库"
    echo "duckdb = { version = \"0.8\", features = [\"bundled\"] }"
    echo ""
    echo "# AWS SDK"
    echo "aws-config = \"0.55\""
    echo "aws-sdk-s3 = \"0.28\""
    echo ""
else
    log_success "duckdb 依赖已存在"
fi

# 2. 添加依赖到 database 包
log_info "2. 更新 database 包依赖..."

cd crates/core/database

# 添加 duckdb 依赖
if ! grep -q "duckdb.*=" Cargo.toml; then
    log_info "添加 duckdb 依赖到 database 包..."
    cat >> Cargo.toml << 'EOF'

# DuckDB 数据库
duckdb.workspace = true

# AWS SDK
aws-config.workspace = true
aws-sdk-s3.workspace = true
EOF
    log_success "依赖已添加到 database 包"
else
    log_success "database 包依赖已存在"
fi

cd ../../..

# 3. 检查并修复常见的编译问题
log_info "3. 检查常见编译问题..."

# 检查是否有 Mock 相关问题
if grep -r "MockStatement" crates/core/database/src/ > /dev/null 2>&1; then
    log_warning "发现 MockStatement 使用，可能需要添加 mockall 功能"
fi

# 检查是否有 prometheus 相关问题
if grep -r "prometheus::" crates/core/database/src/ > /dev/null 2>&1; then
    log_info "检查 prometheus 依赖..."
    if ! grep -q "prometheus.*=" crates/core/database/Cargo.toml; then
        log_warning "prometheus 依赖可能缺失"
    fi
fi

# 4. 尝试编译并检查问题
log_info "4. 尝试编译 common 包..."

if cargo build --package duckhub-common; then
    log_success "common 包编译成功"
else
    log_error "common 包编译失败，请检查错误信息"
fi

log_info "5. 尝试编译 database 包..."

if cargo build --package duckhub-database; then
    log_success "database 包编译成功"
else
    log_warning "database 包编译失败，这是预期的，需要手动修复依赖"
fi

# 5. 生成修复建议
log_info "6. 生成修复建议..."

echo ""
echo "=========================================="
echo "DuckLake 依赖修复建议"
echo "=========================================="
echo ""

echo "📋 需要手动执行的步骤："
echo ""

echo "1. 更新工作空间 Cargo.toml，在 [workspace.dependencies] 部分添加："
echo ""
cat << 'EOF'
# DuckDB 数据库
duckdb = { version = "0.8", features = ["bundled"] }

# AWS SDK
aws-config = "0.55"
aws-sdk-s3 = "0.28"
EOF
echo ""

echo "2. 更新 crates/core/database/Cargo.toml，在 [dependencies] 部分添加："
echo ""
cat << 'EOF'
# DuckDB 数据库
duckdb.workspace = true

# AWS SDK
aws-config.workspace = true
aws-sdk-s3.workspace = true
EOF
echo ""

echo "3. 修复类型不匹配问题："
echo "   - 将 u64 转换为 f64: value as f64"
echo "   - 修复 prometheus::Histogram::new() 调用"
echo "   - 添加缺失的泛型参数"
echo ""

echo "4. 修复 Mock 实现："
echo "   - 为 MockStatement 添加 query_map 方法"
echo "   - 或者使用条件编译 #[cfg(test)]"
echo ""

echo "5. 修复导入问题："
echo "   - 确保所有必要的 use 语句正确"
echo "   - 检查模块路径是否正确"
echo ""

echo "6. 运行测试："
echo "   cargo test --package duckhub-database"
echo ""

echo "📊 当前状态："
echo "   - ✅ 核心功能框架已完成"
echo "   - 🔄 编译依赖需要修复"
echo "   - 🔄 类型不匹配需要解决"
echo "   - 🔄 Mock 实现需要完善"
echo ""

echo "🎯 修复优先级："
echo "   1. 高优先级: 添加缺失的依赖"
echo "   2. 中优先级: 修复类型不匹配"
echo "   3. 低优先级: 完善 Mock 实现"
echo ""

log_success "依赖修复脚本执行完成！"
log_info "请按照上述建议手动修复剩余问题"

echo ""
echo "🔧 快速修复命令："
echo ""
echo "# 1. 编辑工作空间 Cargo.toml"
echo "vim Cargo.toml"
echo ""
echo "# 2. 编辑 database 包 Cargo.toml"
echo "vim crates/core/database/Cargo.toml"
echo ""
echo "# 3. 尝试编译"
echo "cargo build --package duckhub-database"
echo ""
echo "# 4. 运行测试"
echo "cargo test --package duckhub-database"
echo ""

log_info "修复完成后，可以运行 ./scripts/test_ducklake_core.sh 验证功能"
