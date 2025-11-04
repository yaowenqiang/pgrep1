use clap;
use clap::Parser;
use regex::Regex;
use std::path::Path;

#[derive(Debug)]
struct Record {
    line: usize,
    tx: String,
}

#[derive(Parser, Debug)]
#[command(version = "0.1.0", about = "一个简单的 grep 工具")]
struct Args {
    /// the file to test
    #[arg(short = 'f', long)]
    file: String,

    /// the file to test
    #[arg(short = 'p', long)]
    pattern: String,
}

fn process_file<P: AsRef<Path>>(p: P, re: Regex) -> Result<Vec<Record>, String> {
    let mut res = Vec::new();
    let bts = std::fs::read(p).unwrap();
    if let Ok(ss) = String::from_utf8(bts) {
        for (i, l) in ss.lines().enumerate() {
            if re.is_match(l) {
                res.push(Record {
                    line: i,
                    tx: l.to_string(),
                })
            }
        }
    }
    Ok(res)
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let re = Regex::new(&args.pattern).unwrap();
    let p = process_file(args.file, re);
    println!("{:?}", p);
    Ok(())
}
