#!/bin/bash

# Xiaomi Camera Video Merge 构建脚本
# 优化后的 Docker 构建

set -e

echo "🚀 开始构建 Xiaomi Camera Video Merge Docker 镜像..."

# 构建参数
IMAGE_NAME="xiaomi-camera-merge"
TAG="latest"
BUILD_ARGS=""

# 检查是否有自定义标签
if [ $# -eq 1 ]; then
    TAG=$1
fi

echo "📦 构建镜像: ${IMAGE_NAME}:${TAG}"

# 使用 BuildKit 加速构建
export DOCKER_BUILDKIT=1

# 构建镜像
docker build \
    --build-arg BUILDKIT_INLINE_CACHE=1 \
    -t "${IMAGE_NAME}:${TAG}" \
    .

echo "✅ 构建完成！"
echo "📋 镜像信息:"
docker images "${IMAGE_NAME}:${TAG}"

echo ""
echo "🔧 使用方法:"
echo "docker run -v /path/to/input:/app/input -v /path/to/output:/app/output ${IMAGE_NAME}:${TAG} --help" 