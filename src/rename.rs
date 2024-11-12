use clap::Parser;
use std::fs;
use std::process;

/// Moves a file from one location to another.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source file path
    #[arg(short, long)]
    source_path: String,

    /// Destination file path
    #[arg(short, long)]
    dest_path: String,
}

fn main() {
    let args = Args::parse();

    match fs::rename(&args.source_path, &args.dest_path) {
        Ok(_) => {
            println!("File moved successfully from {} to {}", args.source_path, args.dest_path);
        }
        Err(e) => {
            eprintln!("Error: Failed to move file from {} to {}: {}", args.source_path, args.dest_path, e);
            process::exit(1);
        }
    }
}