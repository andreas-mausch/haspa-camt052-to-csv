use std::path::Path;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[structopt(index = 1, required = true)]
    files: Vec<String>,
}

fn main() {
    let args = Args::parse();
    println!("Files {:?}!", args.files);

    let non_existing_files: Vec<_> = args.files.iter().filter(|file| {
        let path = Path::new(file);
        !path.exists() || !path.is_file()
    }).collect();

    if !non_existing_files.is_empty() {
        println!("File does not exist: {:?}", non_existing_files);
        return;
    }

    println!("All files exist: {:?}", args.files)
}
