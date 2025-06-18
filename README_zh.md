# 小米摄像头视频合并工具

[English](README.md) | 中文文档

这是一个用 Rust 编写的工具，用于合并小米摄像头录制的 MP4 视频文件。它可以根据时间戳将视频文件按小时或按天进行合并，支持 Docker 容器化和群晖 NAS 部署。

支持小米智能摄像机2 (云台版)

## 功能特性

- 递归遍历目录结构
- 支持按小时或按天合并视频
- 自动按时间戳排序视频文件
- 使用 FFmpeg 进行视频合并（不重新编码）
- **内置定时执行模式** - 可在每天指定时间自动运行
- Docker 容器化支持
- 群晖 NAS 部署就绪
- 命令行界面，易于使用
- 完善的错误处理机制
- 文件验证和完整性检查

## 系统要求

- Rust 1.70+（使用 Rust 2024 edition）
- FFmpeg（需要预先安装）

### 安装 FFmpeg

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install ffmpeg
```

**Arch Linux:**
```bash
sudo pacman -S ffmpeg
```

**macOS:**
```bash
brew install ffmpeg
```

## 安装

### 方法一：从源码构建

1. 克隆项目：
```bash
git clone <repository-url>
cd xiaomi-camera-merge
```

2. 编译项目：
```bash
cargo build --release
```

3. 运行程序：
```bash
./target/release/xiaomi-camera-merge --help
```

### 方法二：Docker 部署

1. 构建 Docker 镜像：
```bash
docker build -t xiaomi-camera-merge:latest .
```

2. 使用 Docker 运行：
```bash
docker run -v /path/to/input:/app/input -v /path/to/output:/app/output xiaomi-camera-merge:latest --level hour --input /app/input --output /app/output
```

## 使用方法

### 命令行参数

- `--level` 或 `-l`: 合并级别，可选 "hour"（按小时）或 "day"（按天），默认为 "hour"
- `--input` 或 `-i`: 输入文件夹路径（必需）
- `--output` 或 `-o`: 输出文件夹路径（必需）
- `--schedule` 或 `-s`: 每日定时执行时间（格式：HH:MM，24小时制，例如："02:30"）

### 输入目录结构

程序期望输入目录遵循以下结构：

```
input_folder/
├── 2024120110/          # yyyyMMddHH 格式（年、月、日、小时）
│   ├── 30M15S_1701432000.mp4  # mmMssS_unix_timestamp.mp4 格式
│   ├── 30M15S_1701432060.mp4
│   └── ...
├── 2024120111/
│   ├── 00M00S_1701435600.mp4
│   └── ...
└── ...
```

### 输出结构

#### 按小时合并
```
output_folder/
├── 20241201/           # yyyyMMdd 文件夹
│   ├── 2024120110.mp4  # yyyyMMddHH.mp4
│   ├── 2024120111.mp4
│   └── ...
└── ...
```

#### 按天合并
```
output_folder/
├── 20241201/           # yyyyMMdd 文件夹
│   └── 20241201.mp4    # yyyyMMdd.mp4
├── 20241202/           # yyyyMMdd 文件夹
│   └── 20241202.mp4    # yyyyMMdd.mp4
└── ...
```

### 使用示例

1. **按小时合并视频：**
```bash
./target/release/xiaomi-camera-merge \
  --input /path/to/input/folder \
  --output /path/to/output/folder \
  --level hour
```

2. **按天合并视频：**
```bash
./target/release/xiaomi-camera-merge \
  --input /path/to/input/folder \
  --output /path/to/output/folder \
  --level day
```

3. **使用短参数：**
```bash
./target/release/xiaomi-camera-merge \
  -i /path/to/input/folder \
  -o /path/to/output/folder \
  -l day
```

4. **使用 Docker：**
```bash
docker run -v /host/input:/app/input -v /host/output:/app/output \
  xiaomi-camera-merge:latest \
  --level hour --input /app/input --output /app/output
```

### 定时执行

工具支持定时执行模式，可以在每天指定的时间自动运行：

```bash
# 每天凌晨 2:30 定时执行
./target/release/xiaomi-camera-merge \
  --input /path/to/input/folder \
  --output /path/to/output/folder \
  --level hour \
  --schedule 02:30
```

使用定时模式时：
- 程序会等待到每天指定的时间
- 自动执行合并操作
- 程序会持续运行，直到手动停止
- 如果当天指定的时间已经过了，会等待到第二天
- 其他命令行参数与立即执行模式相同

## 工作原理

1. **文件收集**: 程序递归遍历输入目录，查找符合命名规则的文件
2. **时间戳解析**: 使用正则表达式从文件名中提取时间戳信息
3. **文件验证**: 使用 FFmpeg 验证视频文件以确保完整性
4. **分组排序**: 根据合并级别将文件分组，并按时间戳排序
5. **视频合并**: 使用 FFmpeg 的 concat 功能合并视频文件，不重新编码
6. **输出保存**: 将合并后的视频保存到指定的输出目录，保持正确的文件夹结构

## Docker 支持

项目包含全面的 Docker 支持：

- 多阶段构建过程，优化镜像大小
- 基于 Alpine Linux，支持 FFmpeg
- 支持 ARM64 架构（群晖 NAS）
- 非 root 用户运行，提高安全性
- 健康检查支持
- 环境变量配置
- 输入和输出目录的卷挂载

### Docker 特性

- **多阶段构建**: 减少最终镜像大小
- **Alpine Linux**: 最小化占用空间（约 50MB 基础镜像）
- **内置 FFmpeg**: 无需单独安装
- **安全性**: 以非 root 用户身份运行
- **监控**: 内置健康检查
- **灵活性**: 环境变量配置

## 群晖 NAS 部署

详细的群晖 NAS 部署说明，请参阅 [README_Synology.md](README_Synology.md)。

群晖部署的关键特性：
- ARM64 架构支持
- Docker Compose 配置
- 自动化任务调度
- 资源监控
- 备份和恢复程序

## 注意事项

- 确保输入目录中的视频文件命名符合规范
- 程序会自动创建输出目录结构
- 如果输出文件已存在，会被覆盖
- 合并过程会保持原始视频质量，不会重新编码
- 单个视频文件会直接复制，不会进行合并操作
- 无效或损坏的视频文件会自动跳过
- 程序包含完善的错误处理和验证机制

## 错误处理

程序包含完善的错误处理机制：

- 检查输入路径是否存在和权限
- 验证 FFmpeg 是否已安装和可用
- 处理文件读取和写入错误
- 使用 FFmpeg 验证视频文件完整性
- 提供中文错误信息
- 跳过无效文件并继续处理
- 根据需要创建输出目录

## 项目结构

```
xiaomi-camera-merge/
├── Cargo.toml          # Rust 项目配置
├── src/main.rs         # 主应用程序入口点
├── Dockerfile          # Docker 容器配置
├── docker-compose.yml  # Docker Compose 配置
├── README.md           # 英文文档
├── README_zh.md        # 中文文档
└── README_Synology.md  # 群晖 NAS 部署指南
```

## 许可证

本项目采用 MIT 许可证。 