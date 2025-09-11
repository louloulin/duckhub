# DuckHub 生产级多阶段构建 Dockerfile
# 基于 Alpine Linux 的轻量级容器镜像

# ============================================================================
# 阶段1: Rust 构建环境
# ============================================================================
FROM rust:1.75-alpine AS rust-builder

# 安装构建依赖
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    sqlite-dev \
    postgresql-dev \
    mysql-dev \
    git \
    build-base

# 设置工作目录
WORKDIR /app

# 复制 Cargo 配置文件
COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo

# 复制源代码
COPY crates ./crates

# 设置环境变量
ENV RUSTFLAGS="-C target-feature=-crt-static"
ENV PKG_CONFIG_ALL_STATIC=1
ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr

# 构建发布版本
RUN cargo build --release --workspace

# ============================================================================
# 阶段2: Node.js 构建环境
# ============================================================================
FROM node:18-alpine AS frontend-builder

# 设置工作目录
WORKDIR /app/frontend

# 复制 package.json 和 package-lock.json
COPY frontend/package*.json ./

# 安装依赖
RUN npm ci --only=production

# 复制前端源代码
COPY frontend ./

# 构建前端
RUN npm run build

# ============================================================================
# 阶段3: 生产运行时环境
# ============================================================================
FROM alpine:3.18 AS runtime

# 安装运行时依赖
RUN apk add --no-cache \
    ca-certificates \
    tzdata \
    libgcc \
    libssl3 \
    libcrypto3 \
    postgresql-client \
    mysql-client \
    redis \
    curl \
    jq

# 创建应用用户
RUN addgroup -g 1001 -S duckhub && \
    adduser -u 1001 -S duckhub -G duckhub

# 设置工作目录
WORKDIR /app

# 创建必要的目录
RUN mkdir -p /app/bin /app/config /app/data /app/logs /app/static && \
    chown -R duckhub:duckhub /app

# 复制构建的二进制文件
COPY --from=rust-builder /app/target/release/duckhub-api-gateway /app/bin/
COPY --from=rust-builder /app/target/release/duckhub-auth-service /app/bin/
COPY --from=rust-builder /app/target/release/duckhub-query-service /app/bin/
COPY --from=rust-builder /app/target/release/duckhub-data-ingestion /app/bin/
COPY --from=rust-builder /app/target/release/duckhub-ai-agent /app/bin/
COPY --from=rust-builder /app/target/release/duckhub-notification /app/bin/
COPY --from=rust-builder /app/target/release/duckhub-monitoring /app/bin/
COPY --from=rust-builder /app/target/release/duckhub-file-storage /app/bin/

# 复制前端构建文件
COPY --from=frontend-builder /app/frontend/dist /app/static/

# 复制配置文件
COPY config/production.toml /app/config/
COPY config/docker.toml /app/config/
COPY scripts/docker-entrypoint.sh /app/
COPY scripts/health-check.sh /app/

# 设置权限
RUN chmod +x /app/docker-entrypoint.sh /app/health-check.sh && \
    chown -R duckhub:duckhub /app

# 切换到应用用户
USER duckhub

# 暴露端口
EXPOSE 8080 8081 8082 8083 8084 8085 8086 8087

# 设置环境变量
ENV RUST_LOG=info
ENV DUCKHUB_ENV=production
ENV DUCKHUB_CONFIG_PATH=/app/config/production.toml

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /app/health-check.sh

# 设置入口点
ENTRYPOINT ["/app/docker-entrypoint.sh"]

# 默认命令
CMD ["api-gateway"]

# ============================================================================
# 元数据标签
# ============================================================================
LABEL maintainer="DuckHub Team <team@duckhub.dev>"
LABEL version="1.0.0"
LABEL description="DuckHub - AI-Powered Financial Data Platform"
LABEL org.opencontainers.image.title="DuckHub"
LABEL org.opencontainers.image.description="AI-Powered Financial Data Platform with DuckDB and DuckLake"
LABEL org.opencontainers.image.version="1.0.0"
LABEL org.opencontainers.image.vendor="DuckHub"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.source="https://github.com/duckhub/duckhub"
LABEL org.opencontainers.image.documentation="https://docs.duckhub.dev"

# ============================================================================
# 开发环境 Dockerfile (可选)
# ============================================================================
FROM runtime AS development

# 切换回 root 用户安装开发工具
USER root

# 安装开发依赖
RUN apk add --no-cache \
    rust \
    cargo \
    nodejs \
    npm \
    git \
    vim \
    htop \
    strace

# 安装 Rust 开发工具
RUN cargo install cargo-watch cargo-edit

# 切换回应用用户
USER duckhub

# 设置开发环境变量
ENV RUST_LOG=debug
ENV DUCKHUB_ENV=development
ENV DUCKHUB_CONFIG_PATH=/app/config/docker.toml

# 开发模式入口点
CMD ["cargo", "watch", "-x", "run"]

# ============================================================================
# 测试环境 Dockerfile (可选)
# ============================================================================
FROM rust-builder AS test

# 运行测试
RUN cargo test --workspace --release

# 运行基准测试
RUN cargo bench --workspace

# 生成测试报告
RUN cargo test --workspace --release -- --test-threads=1 --nocapture > /tmp/test-results.txt

# ============================================================================
# 安全扫描阶段 (可选)
# ============================================================================
FROM runtime AS security-scan

# 切换到 root 用户进行安全扫描
USER root

# 安装安全扫描工具
RUN apk add --no-cache \
    trivy \
    clamav \
    rkhunter

# 运行安全扫描
RUN trivy filesystem --exit-code 0 --no-progress /app
RUN freshclam && clamscan -r /app

# 切换回应用用户
USER duckhub

# ============================================================================
# 多架构构建支持
# ============================================================================
# 支持 linux/amd64, linux/arm64, linux/arm/v7

# 使用 buildx 构建多架构镜像:
# docker buildx build --platform linux/amd64,linux/arm64,linux/arm/v7 -t duckhub:latest .

# ============================================================================
# 构建优化说明
# ============================================================================
# 1. 多阶段构建减少最终镜像大小
# 2. 使用 Alpine Linux 作为基础镜像
# 3. 静态链接减少运行时依赖
# 4. 分层缓存优化构建速度
# 5. 安全用户运行应用
# 6. 健康检查确保服务可用性
# 7. 适当的标签和元数据

# ============================================================================
# 使用示例
# ============================================================================
# 构建镜像:
# docker build -t duckhub:latest .
#
# 运行容器:
# docker run -d \
#   --name duckhub \
#   -p 8080:8080 \
#   -e DUCKHUB_DATABASE_URL=postgresql://user:pass@db:5432/duckhub \
#   -e DUCKHUB_REDIS_URL=redis://redis:6379 \
#   -v duckhub-data:/app/data \
#   duckhub:latest
#
# 开发模式:
# docker build --target development -t duckhub:dev .
# docker run -it --rm -v $(pwd):/app duckhub:dev
#
# 运行测试:
# docker build --target test -t duckhub:test .
# docker run --rm duckhub:test
