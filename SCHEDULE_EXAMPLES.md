# 定时功能使用示例

## 概述

小米摄像头视频合并工具现在支持定时执行功能，可以在每天指定的时间自动运行合并任务，无需外部调度器。

## 基本用法

### 1. 立即执行（原有功能）

```bash
# 按小时合并
./target/release/xiaomi-camera-merge \
  --input /path/to/input \
  --output /path/to/output \
  --level hour

# 按天合并
./target/release/xiaomi-camera-merge \
  --input /path/to/input \
  --output /path/to/output \
  --level day
```

### 2. 定时执行（新功能）

```bash
# 每天凌晨 2:30 执行小时合并
./target/release/xiaomi-camera-merge \
  --input /path/to/input \
  --output /path/to/output \
  --level hour \
  --schedule 02:30

# 每天下午 3:45 执行天合并
./target/release/xiaomi-camera-merge \
  --input /path/to/input \
  --output /path/to/output \
  --level day \
  --schedule 15:45
```

## Docker 使用示例

### 1. 使用 Docker 运行定时任务

```bash
# 每天凌晨 2:30 执行
docker run -d \
  --name xiaomi-camera-merge-scheduled \
  --restart unless-stopped \
  -v /host/input:/app/input:ro \
  -v /host/output:/app/output \
  -e TZ=Asia/Shanghai \
  xiaomi-camera-merge:latest \
  --level hour \
  --input /app/input \
  --output /app/output \
  --schedule 02:30
```

### 2. 使用 Docker Compose

创建 `docker-compose.scheduled.yml`：

```yaml
version: '3.8'

services:
  xiaomi-camera-merge-scheduled:
    image: xiaomi-camera-merge:latest
    container_name: xiaomi-camera-merge-scheduled
    restart: unless-stopped
    environment:
      - TZ=Asia/Shanghai
    volumes:
      - ./input:/app/input:ro
      - ./output:/app/output
    command: [
      "--level", "hour",
      "--input", "/app/input",
      "--output", "/app/output",
      "--schedule", "02:30"
    ]
```

启动服务：
```bash
docker-compose -f docker-compose.scheduled.yml up -d
```

## 时间格式说明

- 格式：`HH:MM`
- 24小时制
- 小时范围：00-23
- 分钟范围：00-59

### 有效的时间格式示例

```
02:30    # 凌晨 2:30
14:45    # 下午 2:45
00:00    # 午夜 12:00
23:59    # 晚上 11:59
```

### 无效的时间格式示例

```
25:30    # 小时超出范围
14:60    # 分钟超出范围
2:30     # 缺少前导零
14:5     # 分钟缺少前导零
```

## 定时执行行为

1. **启动时检查**：程序启动时会检查当前时间是否已经过了今天的指定时间
2. **等待计算**：如果时间已过，会等待到明天的指定时间
3. **自动执行**：到达指定时间后自动执行合并任务
4. **循环执行**：执行完成后，等待到第二天的指定时间再次执行
5. **持续运行**：程序会持续运行，直到手动停止

## 日志输出示例

```
定时模式已启动，将在每天 02:30 执行合并任务
下次执行时间: 2024-12-02 02:30:00，等待 86340 秒
开始执行定时合并任务...
已合并小时视频: /app/output/20241201/2024120110.mp4
已合并小时视频: /app/output/20241201/2024120111.mp4
定时合并任务完成
下次执行时间: 2024-12-03 02:30:00，等待 86400 秒
```

## 注意事项

1. **时区设置**：确保系统时区设置正确，特别是使用 Docker 时
2. **资源占用**：定时模式会持续运行，但占用资源很少
3. **错误处理**：如果合并任务执行失败，程序会记录错误但继续等待下次执行
4. **停止程序**：使用 `Ctrl+C` 或 `docker stop` 来停止定时任务
5. **重启恢复**：程序重启后会重新计算下次执行时间

## 故障排除

### 1. 时间格式错误

```
错误: 时间格式错误，应为 HH:MM 格式，例如: 02:30
```

解决方案：检查时间格式是否正确，确保使用24小时制。

### 2. 时区问题

如果执行时间不准确，检查时区设置：

```bash
# 检查系统时区
date

# Docker 中设置时区
-e TZ=Asia/Shanghai
```

### 3. 权限问题

确保程序有权限访问输入和输出目录：

```bash
# 检查目录权限
ls -la /path/to/input
ls -la /path/to/output
``` 