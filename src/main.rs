use clap;
use clap::Parser;
use failure::{Error, Fail};
use regex::Regex;
use std::fmt;
use std::path::Path;

// https://boats.gitlab.io/failure/
//
#[derive(Debug)]
struct Record {
    line: usize,
    tx: String,
}

#[derive(Debug, Fail)]
#[fail(display = "Argument not provided {}", arg)]
struct ArgErr {
    arg: &'static str,
}

//impl Fail for ArgErr {}

/*
impl std::fmt::Display for ArgErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Argument Not provided: {}", self.arg)
    }
}
*/

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

fn process_file<P: AsRef<Path>>(p: P, re: &Regex) -> Result<Vec<Record>, Error> {
    let mut res = Vec::new();
    let bts = std::fs::read(p)?;
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

fn process_path<P, FF, EF>(p: P, re: &Regex, ff:&FF, ef: &EF) -> Result<(), Error> 
where 
    P: AsRef<Path>, 
    FF: Fn(&Path, Vec<Record>),
    EF: Fn(Error),
{
    let p = p.as_ref();
    let md = p.metadata()?;
    let ft = md.file_type();

    if ft.is_file() {
        let dt = process_file(p, re)?;
        ff(p, dt);
    }

    if ft.is_dir() {
        let dd = std::fs::read_dir(p)?;
        for d in dd {
            if let Err(e) = process_path(d?.path(), re, ff, ef) {
                ef(e);
            }
        }
    }

    Ok(())
}
fn run() -> Result<(), Error> {
    let args = Args::parse();
    let re = Regex::new(&args.pattern)?;
    //let p = process_file(args.file, &re);
    let p = process_path(args.file, &re, &|pt, v| {
        println!("{:?}",pt);
        println!("{:?}", v);
    },
    &|e| {
        println!("Error: {}",e);
    }
    );
    println!("{:?}", p);
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("There was an error: {}", e);
    }
}
