# 小米摄像头视频合并工具 - 群晖 NAS 部署指南

## 概述

本指南将帮助您在群晖 NAS 上部署小米摄像头视频合并工具，实现自动化视频文件合并功能。

## 系统要求

- 群晖 DSM 7.0 或更高版本
- 支持 Docker 的群晖 NAS 型号
- 至少 2GB 可用内存
- 足够的存储空间用于视频处理

## 前置准备

### 1. 安装 Docker

1. 打开 **套件中心**
2. 搜索并安装 **Docker**
3. 安装完成后启动 Docker 服务

### 2. 准备目录结构

在群晖 File Station 中创建以下目录结构：

```
/xiaomi-camera-merge/
├── input/          # 输入视频文件目录
│   ├── 2024120110/
│   ├── 2024120111/
│   └── ...
├── output/         # 输出合并文件目录
└── logs/           # 日志文件目录
```

## 部署步骤

### 方法一：使用 Docker Compose（推荐）

1. **创建 docker-compose.yml 文件**

在群晖 File Station 中创建 `/xiaomi-camera-merge/docker-compose.yml` 文件：

```yaml
version: '3.8'

services:
  xiaomi-camera-merge:
    image: xiaomi-camera-merge:latest
    container_name: xiaomi-camera-merge
    restart: unless-stopped
    environment:
      - TZ=Asia/Shanghai
      - RUST_LOG=info
    volumes:
      - /volume1/xiaomi-camera-merge/input:/app/input:ro
      - /volume1/xiaomi-camera-merge/output:/app/output
      - /volume1/xiaomi-camera-merge/logs:/app/logs
    user: "1000:1000"
    healthcheck:
      test: ["CMD", "ps", "aux", "|", "grep", "xiaomi-camera-merge"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 5s
```

2. **构建 Docker 镜像**

通过 SSH 连接到群晖 NAS，执行以下命令：

```bash
# 进入项目目录
cd /volume1/xiaomi-camera-merge

# 构建镜像
docker build -t xiaomi-camera-merge:latest .

# 启动服务
docker-compose up -d
```

### 方法二：使用群晖 Docker 界面

1. **打开 Docker 应用**
2. **点击 "映像" → "新增" → "从 Dockerfile 新增"**
3. **上传项目文件并构建镜像**
4. **创建容器配置**

容器配置参数：
- **映像**: `xiaomi-camera-merge:latest`
- **容器名称**: `xiaomi-camera-merge`
- **高级设置** → **环境变量**:
  - `TZ=Asia/Shanghai`
  - `RUST_LOG=info`
- **卷**:
  - `/volume1/xiaomi-camera-merge/input` → `/app/input` (只读)
  - `/volume1/xiaomi-camera-merge/output` → `/app/output`
  - `/volume1/xiaomi-camera-merge/logs` → `/app/logs`

## 使用方法

### 1. 手动运行

通过 SSH 连接到群晖 NAS：

```bash
# 进入容器
docker exec -it xiaomi-camera-merge /bin/sh

# 按小时合并
xiaomi-camera-merge --level hour --input /app/input --output /app/output

# 按天合并
xiaomi-camera-merge --level day --input /app/input --output /app/output
```

### 2. 定时执行模式（推荐）

#### 使用内置定时功能

程序支持内置定时执行模式，无需外部调度器：

```bash
# 每天凌晨 2:30 自动执行小时合并
docker run -d \
  --name xiaomi-camera-merge-scheduled \
  --restart unless-stopped \
  -v /volume1/xiaomi-camera-merge/input:/app/input:ro \
  -v /volume1/xiaomi-camera-merge/output:/app/output \
  -v /volume1/xiaomi-camera-merge/logs:/app/logs \
  -e TZ=Asia/Shanghai \
  xiaomi-camera-merge:latest \
  --level hour \
  --input /app/input \
  --output /app/output \
  --schedule 02:30
```

#### 使用 Docker Compose 定时模式

创建 `docker-compose.scheduled.yml` 文件：

```yaml
version: '3.8'

services:
  xiaomi-camera-merge-scheduled:
    image: xiaomi-camera-merge:latest
    container_name: xiaomi-camera-merge-scheduled
    restart: unless-stopped
    environment:
      - TZ=Asia/Shanghai
      - RUST_LOG=info
    volumes:
      - /volume1/xiaomi-camera-merge/input:/app/input:ro
      - /volume1/xiaomi-camera-merge/output:/app/output
      - /volume1/xiaomi-camera-merge/logs:/app/logs
    user: "1000:1000"
    command: [
      "--level", "hour",
      "--input", "/app/input",
      "--output", "/app/output",
      "--schedule", "02:30"
    ]
```

启动定时服务：
```bash
docker-compose -f docker-compose.scheduled.yml up -d
```

### 3. 传统自动化运行

#### 使用群晖任务计划

1. 打开 **控制面板** → **任务计划**
2. 点击 **新增** → **计划的任务** → **用户定义的脚本**
3. 配置任务：

```bash
#!/bin/bash
# 按小时合并视频
docker exec xiaomi-camera-merge xiaomi-camera-merge \
  --level hour \
  --input /app/input \
  --output /app/output

# 按天合并视频（每天凌晨2点执行）
docker exec xiaomi-camera-merge xiaomi-camera-merge \
  --level day \
  --input /app/input \
  --output /app/output
```

#### 使用 Cron 任务

在容器内设置定时任务：

```bash
# 编辑 crontab
docker exec -it xiaomi-camera-merge crontab -e

# 添加定时任务（每小时执行一次）
0 * * * * xiaomi-camera-merge --level hour --input /app/input --output /app/output

# 每天凌晨2点执行天合并
0 2 * * * xiaomi-camera-merge --level day --input /app/input --output /app/output
```

## 监控和日志

### 1. 查看容器状态

```bash
# 查看容器运行状态
docker ps -a | grep xiaomi-camera-merge

# 查看容器日志
docker logs xiaomi-camera-merge

# 实时查看日志
docker logs -f xiaomi-camera-merge
```

### 2. 资源监控

在群晖 **资源监控** 中查看：
- CPU 使用率
- 内存使用情况
- 磁盘 I/O
- 网络流量

### 3. 日志文件

日志文件位置：`/volume1/xiaomi-camera-merge/logs/`

## 故障排除

### 常见问题

1. **容器启动失败**
   ```bash
   # 检查容器日志
   docker logs xiaomi-camera-merge
   
   # 检查目录权限
   ls -la /volume1/xiaomi-camera-merge/
   ```

2. **权限问题**
   ```bash
   # 修改目录权限
   chmod -R 755 /volume1/xiaomi-camera-merge/
   chown -R 1000:1000 /volume1/xiaomi-camera-merge/
   ```

3. **FFmpeg 错误**
   ```bash
   # 检查 FFmpeg 是否安装
   docker exec xiaomi-camera-merge ffmpeg -version
   ```

4. **内存不足**
   - 增加群晖 NAS 内存
   - 调整 Docker 内存限制
   - 分批处理视频文件

### 性能优化

1. **SSD 缓存**
   - 将输入/输出目录放在 SSD 上
   - 启用群晖 SSD 缓存

2. **网络优化**
   - 使用有线网络连接
   - 配置 Jumbo Frame

3. **存储优化**
   - 使用 RAID 配置提高 I/O 性能
   - 定期清理临时文件

## 备份和恢复

### 1. 备份配置

```bash
# 备份 docker-compose.yml
cp /volume1/xiaomi-camera-merge/docker-compose.yml /volume1/backup/

# 备份环境变量
docker inspect xiaomi-camera-merge > /volume1/backup/container-config.json
```

### 2. 恢复配置

```bash
# 恢复容器配置
docker-compose down
docker-compose up -d
```

## 更新和维护

### 1. 更新镜像

```bash
# 停止容器
docker-compose down

# 重新构建镜像
docker build -t xiaomi-camera-merge:latest .

# 启动容器
docker-compose up -d
```

### 2. 清理资源

```bash
# 清理未使用的镜像
docker image prune -f

# 清理未使用的容器
docker container prune -f

# 清理未使用的卷
docker volume prune -f
```

## 安全建议

1. **网络安全**
   - 使用防火墙限制访问
   - 定期更新 DSM 和 Docker

2. **数据安全**
   - 定期备份重要数据
   - 使用 RAID 保护数据

3. **访问控制**
   - 限制 SSH 访问
   - 使用强密码
   - 启用双因素认证

## 技术支持

如遇到问题，请检查：
1. 群晖 DSM 版本兼容性
2. Docker 版本
3. 硬件资源是否充足
4. 网络连接状态
5. 存储空间是否足够

---

**注意**: 本工具仅用于个人用途，请遵守相关法律法规和隐私政策。 