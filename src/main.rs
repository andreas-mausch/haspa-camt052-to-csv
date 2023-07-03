use clap::Parser;
use env_logger::{Builder, Env};

use haspa_camt052_to_csv::{Args, camt052};

fn main() {
    Builder::from_env(Env::default().default_filter_or("debug")).init();

    let args = Args::parse();
    camt052(args)
}
