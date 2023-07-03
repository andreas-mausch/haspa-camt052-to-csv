use clap::Parser;

use haspa_camt052_to_csv::{Args, camt052};

fn main() {
    let args = Args::parse();
    camt052(args)
}
