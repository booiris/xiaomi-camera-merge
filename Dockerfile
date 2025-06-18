# 多阶段构建 Dockerfile for Xiaomi Camera Video Merge
# 支持 ARM64 架构 (群晖 NAS)

# 第一阶段：构建阶段
FROM rust:1.87-alpine AS builder

# 安装构建依赖
RUN apk add --no-cache \
    musl-dev

# 设置工作目录
WORKDIR /app

# 复制 Cargo 文件
COPY Cargo.toml Cargo.lock ./

# 创建虚拟项目以缓存依赖
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制源代码
COPY src/ ./src/

# 重新构建应用
RUN cargo build --release

# 第二阶段：运行阶段
FROM alpine:3.19

RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories

# 安装运行时依赖和时区数据
RUN apk add --no-cache \
    ffmpeg \
    tzdata \
    && rm -rf /var/cache/apk/*

# 创建非 root 用户
RUN addgroup -g 1000 appuser && \
    adduser -D -s /bin/sh -u 1000 -G appuser appuser

# 设置工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/xiaomi-camera-merge /app/xiaomi-camera-merge

# 创建必要的目录并设置权限
RUN mkdir -p /app/input /app/output && \
    chown -R appuser:appuser /app

# 切换到非 root 用户
USER appuser

# 设置环境变量
ENV TZ=Asia/Shanghai
ENV RUST_LOG=info

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ps aux | grep xiaomi-camera-merge || exit 1

# 设置入口点
ENTRYPOINT ["/app/xiaomi-camera-merge"]

# 默认命令
CMD ["--help"] 