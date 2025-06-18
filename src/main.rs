use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 合并级别: "hour" 或 "day"
    #[arg(short, long, value_enum, default_value = "hour")]
    level: MergeLevel,

    /// 输入文件夹路径
    #[arg(short, long)]
    input: String,

    /// 输出文件夹路径
    #[arg(short, long)]
    output: String,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum MergeLevel {
    Hour,
    Day,
}

impl std::fmt::Display for MergeLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MergeLevel::Hour => write!(f, "hour"),
            MergeLevel::Day => write!(f, "day"),
        }
    }
}

#[derive(Debug)]
struct VideoFile {
    path: PathBuf,
    timestamp: DateTime<Utc>,
}

struct VideoMerger {
    args: Args,
    video_regex: Regex,
}

impl VideoMerger {
    fn new(args: Args) -> Result<Self> {
        let video_regex =
            Regex::new(r"(\d{2})M(\d{2})S_(\d+)\.mp4$").context("创建视频文件名正则表达式失败")?;

        Ok(VideoMerger { args, video_regex })
    }

    fn run(&self) -> Result<()> {
        let input_path = Path::new(&self.args.input);
        if !input_path.exists() {
            anyhow::bail!("输入路径不存在: {}", self.args.input);
        }

        if !input_path.is_dir() {
            anyhow::bail!("输入路径不是目录: {}", self.args.input);
        }

        // 创建输出目录
        fs::create_dir_all(&self.args.output).context("创建输出目录失败")?;

        // 收集所有视频文件
        let video_files = self.collect_video_files(input_path)?;

        match self.args.level {
            MergeLevel::Hour => self.merge_by_hour(&video_files)?,
            MergeLevel::Day => self.merge_by_day(&video_files)?,
        }

        println!("视频合并完成！");
        Ok(())
    }

    fn collect_video_files(&self, input_path: &Path) -> Result<Vec<VideoFile>> {
        let mut video_files = Vec::new();

        for entry in fs::read_dir(input_path).context("读取输入目录失败")? {
            let entry = entry.context("读取目录项失败")?;
            let path = entry.path();

            if path.is_dir() {
                // 检查目录名是否符合 yyyyMMddHH 格式
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.len() == 10 && dir_name.chars().all(|c| c.is_ascii_digit()) {
                        // 递归处理子目录
                        let sub_files = self.collect_video_files(&path)?;
                        video_files.extend(sub_files);
                    }
                }
            } else if path.is_file() {
                // 检查是否是视频文件
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Some(captures) = self.video_regex.captures(file_name) {
                        if let (Ok(_minutes), Ok(_seconds), Ok(timestamp)) = (
                            captures[1].parse::<u32>(),
                            captures[2].parse::<u32>(),
                            captures[3].parse::<i64>(),
                        ) {
                            let video_file = VideoFile {
                                path,
                                timestamp: DateTime::from_timestamp(timestamp, 0)
                                    .unwrap_or_else(Utc::now),
                            };
                            video_files.push(video_file);
                        }
                    }
                }
            }
        }

        Ok(video_files)
    }

    fn is_valid_video_file(&self, file_path: &Path) -> bool {
        // 检查文件是否存在
        if !file_path.exists() || !file_path.is_file() {
            return false;
        }

        // 检查文件大小是否大于0
        if let Ok(metadata) = fs::metadata(file_path) {
            if metadata.len() == 0 {
                return false;
            }
        } else {
            return false;
        }

        // 使用 FFmpeg 检查视频文件是否有效
        let status = Command::new("ffmpeg")
            .arg("-v")
            .arg("quiet")
            .arg("-i")
            .arg(file_path)
            .arg("-f")
            .arg("null")
            .arg("-")
            .status();

        status.is_ok() && status.unwrap().success()
    }

    fn merge_by_hour(&self, video_files: &[VideoFile]) -> Result<()> {
        let mut hourly_groups: HashMap<String, Vec<&VideoFile>> = HashMap::new();

        // 按小时分组
        for video_file in video_files {
            let hour_key = video_file.timestamp.format("%Y%m%d%H").to_string();
            hourly_groups.entry(hour_key).or_default().push(video_file);
        }

        for (hour_key, files) in hourly_groups {
            if files.is_empty() {
                continue;
            }

            // 创建输出目录结构
            let date_str = &hour_key[..8]; // 取前8位作为日期
            let output_dir = Path::new(&self.args.output).join(date_str);
            let output_file = output_dir.join(format!("{}.mp4", hour_key));

            // 检查是否已经处理过
            if output_file.exists() && self.is_valid_video_file(&output_file) {
                println!("跳过已存在的有效小时视频: {}", output_file.display());
                continue;
            }

            // 按时间戳排序
            let mut sorted_files = files.to_vec();
            sorted_files.sort_by_key(|f| f.timestamp);

            // 创建输出目录
            fs::create_dir_all(&output_dir).context("创建小时输出目录失败")?;

            self.merge_video_files(&sorted_files, &output_file)?;
            println!("已合并小时视频: {}", output_file.display());
        }

        Ok(())
    }

    fn merge_by_day(&self, video_files: &[VideoFile]) -> Result<()> {
        let mut daily_groups: HashMap<String, Vec<&VideoFile>> = HashMap::new();

        // 按天分组
        for video_file in video_files {
            let day_key = video_file.timestamp.format("%Y%m%d").to_string();
            daily_groups.entry(day_key).or_default().push(video_file);
        }

        for (day_key, files) in daily_groups {
            if files.is_empty() {
                continue;
            }

            // 创建输出目录结构
            let output_dir = Path::new(&self.args.output).join(&day_key);
            let output_file = output_dir.join(format!("{}.mp4", day_key));

            // 检查是否已经处理过
            if output_file.exists() && self.is_valid_video_file(&output_file) {
                println!("跳过已存在的有效日视频: {}", output_file.display());
                continue;
            }

            // 按时间戳排序
            let mut sorted_files = files.to_vec();
            sorted_files.sort_by_key(|f| f.timestamp);

            // 创建输出目录
            fs::create_dir_all(&output_dir).context("创建日输出目录失败")?;

            self.merge_video_files(&sorted_files, &output_file)?;
            println!("已合并日视频: {}", output_file.display());
        }

        Ok(())
    }

    fn merge_video_files(&self, video_files: &[&VideoFile], output_path: &Path) -> Result<()> {
        if video_files.is_empty() {
            return Ok(());
        }

        if video_files.len() == 1 {
            // 只有一个文件，直接复制
            fs::copy(&video_files[0].path, output_path).context("复制单个视频文件失败")?;
            return Ok(());
        }

        // 创建文件列表
        let list_file = output_path.with_extension("txt");
        let mut list_content = String::new();

        for video_file in video_files {
            list_content.push_str(&format!("file '{}'\n", video_file.path.display()));
        }

        fs::write(&list_file, list_content).context("创建文件列表失败")?;

        // 使用 FFmpeg 合并视频
        let status = Command::new("ffmpeg")
            .arg("-f")
            .arg("concat")
            .arg("-safe")
            .arg("0")
            .arg("-i")
            .arg(&list_file)
            .arg("-c")
            .arg("copy")
            .arg("-y") // 覆盖输出文件
            .arg(output_path)
            .status()
            .context("执行 FFmpeg 命令失败")?;

        // 清理临时文件
        let _ = fs::remove_file(list_file);

        if !status.success() {
            anyhow::bail!("FFmpeg 合并失败，退出码: {}", status);
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 检查 FFmpeg 是否可用
    let ffmpeg_status = Command::new("ffmpeg").arg("-version").status();

    if ffmpeg_status.is_err() {
        anyhow::bail!("FFmpeg 未安装或不在 PATH 中。请先安装 FFmpeg。");
    }

    let merger = VideoMerger::new(args)?;
    merger.run()?;

    Ok(())
}
