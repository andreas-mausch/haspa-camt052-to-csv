use std::path::Path;

use clap::Parser;
use env_logger::Env;
use log::{error, info};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[structopt(index = 1, required = true)]
    files: Vec<String>,
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    info!("Files {:?}!", args.files);

    let non_existing_files: Vec<_> = args.files.iter().filter(|file| {
        let path = Path::new(file);
        !path.exists() || !path.is_file()
    }).collect();

    if !non_existing_files.is_empty() {
        error!("File does not exist: {:?}", non_existing_files);
        return;
    }

    info!("All files exist: {:?}", args.files)
}
