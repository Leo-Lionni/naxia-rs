use log::{error, info};
use std::{env, fs};
use std::error::Error;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::process::{Command, Stdio};

fn main() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let exe_path = std::env::current_exe().expect("无法获取可执行文件路径");
    let self_name = exe_path.file_name().unwrap().to_str().unwrap();

    let current_dir = std::env::current_dir().expect("无法获取当前工作目录");
    let all_files = get_all_files_include_sub_folder(&current_dir);

    for path in all_files {
        // 跳过 rename.exe 和当前执行程序
        if path.ends_with("rename.exe") || path.ends_with(self_name) {
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

        // 调用外部进程命令来进行解锁
        let unlock_exe_path = get_unlock_exe_path();
        if let Err(err) = rename_file(&dst_file_path, &new_file_path) {
            error!("使用 rename.exe 重命名文件 {} 到 {} 失败: {}", dst_file_path, new_file_path, err);
            continue;
        }
    }

    info!("解密完成，按回车键退出");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("读取输入失败");
}

// 获取解锁程序路径
fn get_unlock_exe_path() -> PathBuf {
    let current_dir = std::env::current_dir().expect("无法获取当前工作目录");
    current_dir.join("rename.exe")
}
fn rename_file(source_path: &str, new_file_path: &str) -> Result<(), Box<dyn Error>> {
    // 获取当前工作目录
    let current_dir = env::current_dir()?;
    let rename_exe_path = current_dir.join("rename.exe");

    // 设置命令行参数，调用 rename.exe 并传递参数
    let args = format!(r#" -s "{}" -d "{}""#, source_path, new_file_path);

    // 打印构建的命令行
    let cmd_str = format!(
        r#" {} {} "#, // 此处的""包裹
        rename_exe_path.display(),
        args
    );
    // info!("即将执行的命令: cmd /C {}", cmd_str);

    // 创建新的 cmd.exe 进程
    let mut cmd = Command::new("cmd");
    cmd.arg("/C")
        .arg(rename_exe_path.clone())
        .arg("-s")
        .arg(source_path)
        .arg("-d")
        .arg(new_file_path);
    info!(
        "即将执行的命令: cmd /C {} -s \"{}\" -d \"{}\"",
        rename_exe_path.display(),
        source_path,
        new_file_path
    );

    // 执行命令并获取输出
    let output = cmd.output()?;


    // 如果命令执行失败，输出错误信息
    if !output.status.success() {
        eprintln!("rename.exe 执行失败: {:?}", output.stderr);
        return Err("重命名失败".into());
    }

    // 如果有标准输出，打印出来
    if !output.stdout.is_empty() {
        println!("输出信息: {:?}", String::from_utf8_lossy(&output.stdout));
    }

    Ok(())
}




// 执行 rename.exe
// fn rename_file(source_path: &str, new_file_path: &str, unlock_exe_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
//     let args = format!(r#" -sourcePath="{}" -destPath="{}"#, source_path, new_file_path);
//     let mut cmd = Command::new(unlock_exe_path);
//     cmd.arg(args)
//         .stdout(Stdio::piped())
//         .stderr(Stdio::piped());
//
//     let output = cmd.output()?;
//     if !output.status.success() {
//         error!("rename.exe 执行失败: {:?}", output.stderr);
//         return Err("重命名失败".into());
//     }
//
//     if !output.stdout.is_empty() {
//         info!("输出信息: {:?}", String::from_utf8_lossy(&output.stdout));
//     }
//
//     Ok(())
// }

// 复制文件
fn copy_file(source_path: &str, dst_file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
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

// 获取目录下所有文件（包含子目录）
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
