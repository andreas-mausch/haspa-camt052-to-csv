use std::error::Error;
use std::fs::File;
use std::io::{stdout, Write};
use std::process::ExitCode;

use clap::Parser;
use env_logger::{Builder, Env};
use log::error;

use haspa_camt052_to_csv::{Format, process};

/// Convert camt052 files into csv or ods
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input camt052 files
    #[arg(required = true)]
    files: Vec<String>,

    /// Format of the output file
    #[arg(value_enum, short, long, default_value_t = Format::Csv)]
    format: Format,

    /// Output filename. Use "-" to output to stdout
    #[arg(short, long, default_value = "-")]
    output: String,
}

fn main() -> ExitCode {
    Builder::from_env(Env::default().default_filter_or("debug")).init();

    let args = Args::parse();
    get_output_stream(&args.output).and_then(|mut output_stream|
        process(args.files, args.format, &mut output_stream))
        .map(|()| ExitCode::SUCCESS)
        .unwrap_or_else(|e| {
            error!("Could not parse files {:#?}", e);
            ExitCode::FAILURE
        })
}

fn get_output_stream(output: &str) -> Result<Box<dyn Write>, Box<dyn Error>> {
    match output {
        "-" => Ok(Box::new(stdout())),
        _ => File::create(output)
            .map(|file| -> Box<dyn Write> { Box::new(file) })
            .map_err(|e| e.into())
    }
}
