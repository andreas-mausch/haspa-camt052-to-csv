use std::process::ExitCode;

use clap::Parser;
use env_logger::{Builder, Env};

use haspa_camt052_to_csv::{Args, camt052};

fn main() -> ExitCode {
    Builder::from_env(Env::default().default_filter_or("debug")).init();
    camt052(Args::parse())
}
