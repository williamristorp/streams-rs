use clap::Parser;
use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
};
use streams::MultiWriter;

#[derive(Parser, Debug)]
#[command(version, about = "Copy standard input to each FILE and standard output", long_about = None)]
struct Args {
    #[arg(short, long)]
    append: bool,

    #[arg(value_name = "FILE")]
    paths: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let mut files = if args.append {
        args.paths
            .iter()
            .map(|p| {
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(p)
                    .unwrap()
            })
            .collect::<Vec<_>>()
    } else {
        args.paths
            .iter()
            .map(|p| File::create(p).unwrap())
            .collect::<Vec<_>>()
    };

    let mut writers: Vec<_> = files.iter_mut().map(|f| f as &mut dyn Write).collect();
    let mut stdout = io::stdout().lock();
    writers.push(&mut stdout as &mut dyn Write);

    let mut multi_writer = MultiWriter::new(writers);
    io::copy(&mut io::stdin().lock(), &mut multi_writer).unwrap();
}
