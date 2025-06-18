# Xiaomi Camera Video Merge Tool

[中文文档](README_zh.md) | English

A Rust-based tool for merging MP4 video files recorded by Xiaomi cameras. It can merge video files by hour or by day based on timestamps, with support for Docker containerization and Synology NAS deployment.

## Features

- Recursive directory traversal
- Support for hourly or daily video merging
- Automatic video file sorting by timestamp
- Video merging using FFmpeg (no re-encoding)
- Docker containerization support
- Synology NAS deployment ready
- Command-line interface for easy use
- Comprehensive error handling
- File validation and integrity checks

## System Requirements

- Rust 1.70+ (uses Rust 2024 edition)
- FFmpeg (must be pre-installed)

### Installing FFmpeg

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

## Installation

### Method 1: Build from Source

1. Clone the project:
```bash
git clone <repository-url>
cd xiaomi-camera-merge
```

2. Build the project:
```bash
cargo build --release
```

3. Run the program:
```bash
./target/release/xiaomi-camera-merge --help
```

### Method 2: Docker Deployment

1. Build the Docker image:
```bash
docker build -t xiaomi-camera-merge:latest .
```

2. Run with Docker:
```bash
docker run -v /path/to/input:/app/input -v /path/to/output:/app/output xiaomi-camera-merge:latest --level hour --input /app/input --output /app/output
```

## Usage

### Command Line Arguments

- `--level` or `-l`: Merge level, either "hour" (by hour) or "day" (by day), defaults to "hour"
- `--input` or `-i`: Input folder path (required)
- `--output` or `-o`: Output folder path (required)

### Input Directory Structure

The program expects the input directory to follow this structure:

```
input_folder/
├── 2024120110/          # yyyyMMddHH format (year, month, day, hour)
│   ├── 30M15S_1701432000.mp4  # mmMssS_unix_timestamp.mp4 format
│   ├── 30M15S_1701432060.mp4
│   └── ...
├── 2024120111/
│   ├── 00M00S_1701435600.mp4
│   └── ...
└── ...
```

### Output Structure

#### Hourly Merging
```
output_folder/
├── 20241201/           # yyyyMMdd folder
│   ├── 2024120110.mp4  # yyyyMMddHH.mp4
│   ├── 2024120111.mp4
│   └── ...
└── ...
```

#### Daily Merging
```
output_folder/
├── 20241201/           # yyyyMMdd folder
│   └── 20241201.mp4    # yyyyMMdd.mp4
├── 20241202/           # yyyyMMdd folder
│   └── 20241202.mp4    # yyyyMMdd.mp4
└── ...
```

### Usage Examples

1. **Merge videos by hour:**
```bash
./target/release/xiaomi-camera-merge \
  --input /path/to/input/folder \
  --output /path/to/output/folder \
  --level hour
```

2. **Merge videos by day:**
```bash
./target/release/xiaomi-camera-merge \
  --input /path/to/input/folder \
  --output /path/to/output/folder \
  --level day
```

3. **Using short parameters:**
```bash
./target/release/xiaomi-camera-merge \
  -i /path/to/input/folder \
  -o /path/to/output/folder \
  -l day
```

4. **Using Docker:**
```bash
docker run -v /host/input:/app/input -v /host/output:/app/output \
  xiaomi-camera-merge:latest \
  --level hour --input /app/input --output /app/output
```

## How It Works

1. **File Collection**: The program recursively traverses the input directory to find files matching the naming convention
2. **Timestamp Parsing**: Extracts timestamp information from filenames using regex pattern matching
3. **File Validation**: Validates video files using FFmpeg to ensure integrity
4. **Grouping and Sorting**: Groups files according to merge level and sorts by timestamp
5. **Video Merging**: Uses FFmpeg's concat functionality to merge video files without re-encoding
6. **Output Saving**: Saves merged videos to the specified output directory with proper folder structure

## Docker Support

The project includes comprehensive Docker support:

- Multi-stage build process for optimized image size
- Alpine Linux base with FFmpeg support
- ARM64 architecture support for Synology NAS
- Non-root user for security
- Health check support
- Environment variable configuration
- Volume mounting for input and output directories

### Docker Features

- **Multi-stage build**: Reduces final image size
- **Alpine Linux**: Minimal footprint (~50MB base)
- **FFmpeg included**: No need to install separately
- **Security**: Runs as non-root user
- **Monitoring**: Built-in health checks
- **Flexibility**: Environment variable configuration

## Synology NAS Deployment

For detailed Synology NAS deployment instructions, see [README_Synology.md](README_Synology.md).

Key features for Synology deployment:
- ARM64 architecture support
- Docker Compose configuration
- Automated task scheduling
- Resource monitoring
- Backup and restore procedures

## Important Notes

- Ensure video files in the input directory follow the naming convention
- The program automatically creates output directory structures
- Existing output files will be overwritten
- The merging process preserves original video quality without re-encoding
- Single video files will be copied directly without merging
- Invalid or corrupted video files are automatically skipped
- The program includes comprehensive error handling and validation

## Error Handling

The program includes comprehensive error handling:

- Validates input path existence and permissions
- Verifies FFmpeg installation and availability
- Handles file read and write errors
- Validates video file integrity using FFmpeg
- Provides clear error messages in Chinese
- Skips invalid files and continues processing
- Creates output directories as needed

## Project Structure

```
xiaomi-camera-merge/
├── Cargo.toml          # Rust project configuration
├── src/main.rs         # Main application entry point
├── Dockerfile          # Docker container configuration
├── docker-compose.yml  # Docker Compose configuration
├── README.md           # English documentation
├── README_zh.md        # Chinese documentation
└── README_Synology.md  # Synology NAS deployment guide
```

## License

This project is licensed under the MIT License. 