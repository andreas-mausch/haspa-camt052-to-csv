use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[structopt(index = 1, required = true)]
    files: Vec<String>
}

fn main() {
    let args = Args::parse();
    println!("Hello {:?}!", args.files)
}
