// fontgrep - A tool for finding fonts with specific features
//
// this_file: fontgrep/src/main.rs

use clap::Parser;
use env_logger::{Builder, Env};
use fontgrep::cli;
use log::{error, info};
use std::process;

/// Main entry point for the fontgrep application
fn main() {
    // Initialize logging
    Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_target(false)
        .init();

    // Parse command-line arguments
    let cli = cli::Cli::parse();

    // Enable verbose logging if requested
    if cli.verbose {
        log::set_max_level(log::LevelFilter::Debug);
    }

    // Log startup information
    info!("fontgrep v{}", env!("CARGO_PKG_VERSION"));

    // Run the application and handle errors
    if let Err(e) = cli::execute(cli) {
        error!("Error: {}", e);
        process::exit(1);
    }
}
