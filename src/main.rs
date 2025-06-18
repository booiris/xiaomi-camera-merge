use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Local, Utc};
use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 合并级别: "hour" 或 "day"
    #[arg(short, long, value_enum, default_value = "hour")]
    level: MergeLevel,

    /// 输入文件夹路径
    #[arg(short, long, default_value = "/app/input")]
    input: String,

    /// 输出文件夹路径
    #[arg(short, long, default_value = "/app/output")]
    output: String,

    /// 定时执行时间 (格式: HH:MM，24小时制，例如: 02:30)
    #[arg(short, long)]
    schedule: Option<String>,
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
    parent_hour: String,
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
                        // 递归处理子目录，传递父目录小时信息
                        let sub_files =
                            self.collect_video_files_with_parent_hour(&path, dir_name)?;
                        video_files.extend(sub_files);
                    }
                }
            } else if path.is_file() {
                // 检查是否是视频文件
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Some(captures) = self.video_regex.captures(file_name) {
                        if let (Ok(_minutes), Ok(_seconds), Ok(_timestamp)) = (
                            captures[1].parse::<u32>(),
                            captures[2].parse::<u32>(),
                            captures[3].parse::<i64>(),
                        ) {
                            // 从父目录名解析时间，而不是使用Unix时间戳
                            let parent_hour = self.extract_parent_hour_from_path(&path);
                            let timestamp = self.parse_timestamp_from_parent_hour(
                                &parent_hour,
                                &captures[1],
                                &captures[2],
                            )?;

                            let video_file = VideoFile {
                                path,
                                timestamp,
                                parent_hour,
                            };
                            video_files.push(video_file);
                        }
                    }
                }
            }
        }

        Ok(video_files)
    }

    fn collect_video_files_with_parent_hour(
        &self,
        input_path: &Path,
        parent_hour: &str,
    ) -> Result<Vec<VideoFile>> {
        let mut video_files = Vec::new();

        for entry in fs::read_dir(input_path).context("读取输入目录失败")? {
            let entry = entry.context("读取目录项失败")?;
            let path = entry.path();

            if path.is_dir() {
                // 检查目录名是否符合 yyyyMMddHH 格式
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.len() == 10 && dir_name.chars().all(|c| c.is_ascii_digit()) {
                        // 递归处理子目录
                        let sub_files =
                            self.collect_video_files_with_parent_hour(&path, dir_name)?;
                        video_files.extend(sub_files);
                    }
                }
            } else if path.is_file() {
                // 检查是否是视频文件
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Some(captures) = self.video_regex.captures(file_name) {
                        if let (Ok(_minutes), Ok(_seconds), Ok(_timestamp)) = (
                            captures[1].parse::<u32>(),
                            captures[2].parse::<u32>(),
                            captures[3].parse::<i64>(),
                        ) {
                            // 使用父目录小时信息解析时间
                            let timestamp = self.parse_timestamp_from_parent_hour(
                                parent_hour,
                                &captures[1],
                                &captures[2],
                            )?;

                            let video_file = VideoFile {
                                path,
                                timestamp,
                                parent_hour: parent_hour.to_string(),
                            };
                            video_files.push(video_file);
                        }
                    }
                }
            }
        }

        Ok(video_files)
    }

    fn extract_parent_hour_from_path(&self, file_path: &Path) -> String {
        // 从文件路径中提取父目录的小时信息
        if let Some(parent) = file_path.parent() {
            if let Some(parent_name) = parent.file_name().and_then(|n| n.to_str()) {
                if parent_name.len() == 10 && parent_name.chars().all(|c| c.is_ascii_digit()) {
                    return parent_name.to_string();
                }
            }
        }
        // 如果无法提取，返回当前时间的小时信息
        Utc::now().format("%Y%m%d%H").to_string()
    }

    fn parse_timestamp_from_parent_hour(
        &self,
        parent_hour: &str,
        minutes: &str,
        seconds: &str,
    ) -> Result<DateTime<Utc>> {
        // 从父目录小时信息解析时间
        if parent_hour.len() != 10 {
            anyhow::bail!("父目录名格式错误，应为10位数字: {}", parent_hour);
        }

        let year = parent_hour[0..4].parse::<i32>().context("解析年份失败")?;
        let month = parent_hour[4..6].parse::<u32>().context("解析月份失败")?;
        let day = parent_hour[6..8].parse::<u32>().context("解析日期失败")?;
        let hour = parent_hour[8..10].parse::<u32>().context("解析小时失败")?;
        let minutes = minutes.parse::<u32>().context("解析分钟失败")?;
        let seconds = seconds.parse::<u32>().context("解析秒数失败")?;

        // 使用chrono创建DateTime
        let naive_datetime = chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(year, month, day).context("创建日期失败")?,
            chrono::NaiveTime::from_hms_opt(hour, minutes, seconds).context("创建时间失败")?,
        );

        Ok(DateTime::from_naive_utc_and_offset(naive_datetime, Utc))
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

        true

        // // 使用 FFmpeg 检查视频文件是否有效
        // let status = Command::new("ffmpeg")
        //     .arg("-v")
        //     .arg("quiet")
        //     .arg("-i")
        //     .arg(file_path)
        //     .arg("-f")
        //     .arg("null")
        //     .arg("-")
        //     .status();

        // status.is_ok() && status.unwrap().success()
    }

    /// 检查是否为当天的视频
    fn is_today_video(&self, date_str: &str) -> bool {
        let today = Local::now().format("%Y%m%d").to_string();
        date_str == today
    }

    fn merge_by_hour(&self, video_files: &[VideoFile]) -> Result<()> {
        let mut hourly_groups: HashMap<String, Vec<&VideoFile>> = HashMap::new();

        // 按父目录小时分组，确保时区一致性
        for video_file in video_files {
            let hour_key = video_file.parent_hour.clone();
            hourly_groups.entry(hour_key).or_default().push(video_file);
        }

        for (hour_key, files) in hourly_groups {
            if files.is_empty() {
                continue;
            }

            // 创建输出目录结构
            let date_str = &hour_key[..8]; // 取前8位作为日期

            // 跳过当天的视频，因为还没有录制完成
            if self.is_today_video(date_str) {
                println!("跳过当天的视频: {} (日期: {})", hour_key, date_str);
                continue;
            }

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

        // 按天分组，使用父目录小时信息的前8位作为日期
        for video_file in video_files {
            let day_key = video_file.parent_hour[..8].to_string();
            daily_groups.entry(day_key).or_default().push(video_file);
        }

        for (day_key, files) in daily_groups {
            if files.is_empty() {
                continue;
            }

            // 跳过当天的视频，因为还没有录制完成
            if self.is_today_video(&day_key) {
                println!("跳过当天的视频: {} (日期: {})", day_key, day_key);
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

    // 如果指定了定时参数，启动定时模式
    if let Some(schedule_time) = &args.schedule {
        run_scheduled_mode(&args, schedule_time)?;
    } else {
        // 立即执行一次
        let merger = VideoMerger::new(args)?;
        merger.run()?;
    }

    Ok(())
}

fn run_scheduled_mode(args: &Args, schedule_time: &str) -> Result<()> {
    // 解析时间格式 HH:MM
    let time_parts: Vec<&str> = schedule_time.split(':').collect();
    if time_parts.len() != 2 {
        anyhow::bail!("时间格式错误，应为 HH:MM 格式，例如: 02:30");
    }

    let hour: u32 = time_parts[0].parse().context("解析小时失败")?;
    let minute: u32 = time_parts[1].parse().context("解析分钟失败")?;

    if hour > 23 || minute > 59 {
        anyhow::bail!("时间格式错误，小时应在 0-23 之间，分钟应在 0-59 之间");
    }

    println!(
        "定时模式已启动，将在每天 {}:{} 执行合并任务",
        format_args!("{:02}", hour),
        format_args!("{:02}", minute)
    );

    loop {
        let now = Local::now();

        // 创建今天的目标时间（使用Local时区）
        let today_target = now
            .date_naive()
            .and_hms_opt(hour, minute, 0)
            .expect("创建目标时间失败")
            .and_local_timezone(Local)
            .unwrap();

        // 创建明天的目标时间
        let tomorrow_target = today_target + Duration::days(1);

        // 确定下一个目标时间
        let target_datetime = if now >= today_target {
            // 如果今天的目标时间已经过了，使用明天的时间
            tomorrow_target
        } else {
            // 否则使用今天的时间
            today_target
        };

        let sleep_duration = target_datetime.signed_duration_since(now);
        let sleep_seconds = sleep_duration.num_seconds() as u64;

        println!(
            "下次执行时间: {}，等待 {} 秒",
            target_datetime.format("%Y-%m-%d %H:%M:%S"),
            sleep_seconds
        );

        // 等待到目标时间
        std::thread::sleep(std::time::Duration::from_secs(sleep_seconds));

        // 执行合并任务
        println!("开始执行定时合并任务...");
        let merger = VideoMerger::new(args.clone())?;
        if let Err(e) = merger.run() {
            eprintln!("合并任务执行失败: {}", e);
        } else {
            println!("定时合并任务完成");
        }
    }
}
