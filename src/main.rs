use log::{error, info};
use std::{env, fs, process};
use std::error::Error;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use clap::Parser;
use std::process::Command;

/// 嵌入 `rename.exe` 的二进制数据
const RENAME_EXE_DATA: &[u8] = include_bytes!("rename.exe");

/// 程序的命令行参数
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// 是否只进行重命名操作
    #[arg(short, long)]
    rename: bool,

    /// 源文件路径，仅在 --rename 模式下使用
    #[arg(short = 's', long, default_value = "")]
    source_path: String,

    /// 目标文件路径，仅在 --rename 模式下使用
    #[arg(short = 'd', long, default_value = "")]
    dest_path: String,
}

fn main() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let args = Args::parse();

    // 释放 rename.exe 到临时目录
    let temp_dir = std::env::temp_dir();
    let rename_exe_path = temp_dir.join("rename.exe");
    if let Err(err) = write_temp_rename_exe(&rename_exe_path) {
        error!("释放 rename.exe 失败: {}", err);
        return;
    }

    // 如果指定了 --rename 参数，则执行重命名功能
    if args.rename {
        if args.source_path.is_empty() || args.dest_path.is_empty() {
            error!("请提供源文件路径 (-s) 和目标文件路径 (-d)");
            return;
        }

        if let Err(err) = run_rename_exe(&rename_exe_path, &args.source_path, &args.dest_path) {
            error!("重命名文件失败: {}", err);
        } else {
            info!("文件成功重命名: {} -> {}", args.source_path, args.dest_path);
        }

        // 删除临时释放的 rename.exe
        let _ = fs::remove_file(&rename_exe_path);
        return;
    }

    // 否则执行默认的扫描和复制重命名流程
    let exe_path = std::env::current_exe().expect("无法获取可执行文件路径");
    let self_name = exe_path.file_name().unwrap().to_str().unwrap();
    let current_dir = std::env::current_dir().expect("无法获取当前工作目录");
    let all_files = get_all_files_include_sub_folder(&current_dir);

    for path in all_files {
        // 跳过当前执行程序
        if path.ends_with(self_name) {
            continue;
        }

        let dst_file_path = format!("{}.temp", path);
        if let Err(err) = copy_file(&path, &dst_file_path) {
            error!("复制文件 {} 到 {} 失败: {}", path, dst_file_path, err);
            continue;
        }

        info!("正在删除原文件");
        if let Err(err) = fs::remove_file(&path) {
            error!("删除文件 {} 失败: {}", path, err);
            continue;
        }
        info!("删除完成");

        // 重命名为原名称
        let new_file_path = path.clone();
        if let Err(err) = run_rename_exe(&rename_exe_path, &dst_file_path, &new_file_path) {
            error!("重命名文件 {} 到 {} 失败: {}", dst_file_path, new_file_path, err);
            continue;
        }
        info!("文件成功重命名: {} -> {}", dst_file_path, new_file_path);
    }

    // 删除临时释放的 rename.exe
    let _ = fs::remove_file(&rename_exe_path);

    info!("操作完成，按回车键退出");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("读取输入失败");
}

/// 释放 `rename.exe` 到指定路径
fn write_temp_rename_exe(rename_exe_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut file = fs::File::create(rename_exe_path)?;
    file.write_all(RENAME_EXE_DATA)?;
    Ok(())
}

/// 调用 `rename.exe` 进行重命名
fn run_rename_exe(rename_exe_path: &Path, source_path: &str, dest_path: &str) -> Result<(), Box<dyn Error>> {
    let output = Command::new(rename_exe_path)
        .arg("-s")
        .arg(source_path)
        .arg("-d")
        .arg(dest_path)
        .output()?;

    if !output.status.success() {
        error!("rename.exe 执行失败: {:?}", String::from_utf8_lossy(&output.stderr));
        return Err("重命名失败".into());
    }

    if !output.stdout.is_empty() {
        info!("输出信息: {:?}", String::from_utf8_lossy(&output.stdout));
    }

    Ok(())
}

/// 复制文件
fn copy_file(source_path: &str, dst_file_path: &str) -> Result<(), Box<dyn Error>> {
    let mut source = fs::File::open(source_path)?;
    let mut destination = fs::File::create(dst_file_path)?;
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = source.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        destination.write_all(&buffer[..bytes_read])?;
    }
    Ok(())
}

/// 获取目录下所有文件（包含子目录）
fn get_all_files_include_sub_folder(folder: &PathBuf) -> Vec<String> {
    let mut result = Vec::new();
    for entry in WalkDir::new(folder) {
        let entry = entry.expect("无法读取目录条目");
        if entry.file_type().is_file() {
            result.push(entry.path().to_string_lossy().into_owned());
        }
    }
    result
}
