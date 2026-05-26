use std::path::PathBuf;

use clap::Parser;
use rag_study::loader;
use rag_study::splitter::Splitter;

#[derive(Parser)]
#[command(name = "rag-study")]
struct Cli {
    /// PDF 文件路径
    #[arg(short, long)]
    file: PathBuf,

    /// 分块大小（字符数）
    #[arg(long, default_value = "500")]
    chunk_size: usize,

    /// 分块重叠（字符数）
    #[arg(long, default_value = "50")]
    chunk_overlap: usize,
}

fn main() {
    let cli = Cli::parse();

    let chunks = match loader::load_pdf(&cli.file) {
        Ok(c) => {
            println!("加载了 {} 页", c.len());
            c
        }
        Err(e) => {
            eprintln!("加载 PDF 失败: {}", e);
            std::process::exit(1);
        }
    };

    let splitter = Splitter::new(cli.chunk_size, cli.chunk_overlap);
    let result = splitter.split(&chunks);
    println!("切分为 {} 个 chunk", result.len());
}
