use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use clap::Parser;
use env_logger::Env;
use log::{debug, error, info};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[structopt(index = 1, required = true)]
    files: Vec<String>,
}

fn read_zip(path: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for index in 0..archive.len() {
        let mut file_in_archive = archive.by_index(index)?;
        debug!("File in archive: {:?}", file_in_archive.name());
        let mut content = String::new();
        file_in_archive.read_to_string(&mut content)?;
        debug!("Content: {:?}", content);
    }
    Ok(())
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let args = Args::parse();
    info!("Files {:?}!", args.files);
    let paths: Vec<_> = args.files.iter().map(|file| Path::new(file)).collect();

    let non_existing_files: Vec<_> = paths.iter().filter(|path| { !path.exists() || !path.is_file() }).collect();

    if !non_existing_files.is_empty() {
        error!("File does not exist: {:?}", non_existing_files);
        return;
    }

    info!("All files exist: {:?}", args.files);

    paths.iter().for_each(|path| {
        match tree_magic_mini::from_filepath(path) {
            Some("application/zip") => {
                info!("Zip file: {:?}", path);
                read_zip(path);
            }
            _ => {
                info!("No zip file: {:?}", path);
            }
        }
    });
}
