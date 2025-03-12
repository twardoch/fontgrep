// fontgrep - A tool for finding fonts with specific features
//
// this_file: fontgrep/src/main.rs

use clap::Parser;
use env_logger::{Builder, Env};
use fontgrep::{cli, Result, with_context};
use log::error;
use std::process;

/// Main entry point for the fontgrep application
fn main() {
    // Initialize logging
    Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_target(false)
        .init();
    
    // Run the application and handle errors
    if let Err(e) = run() {
        error!("Error: {}", e);
        process::exit(1);
    }
}

/// Run the application
fn run() -> Result<()> {
    // Parse command-line arguments
    let cli = cli::Cli::parse();
    
    // Enable verbose logging if requested
    if cli.verbose {
        log::set_max_level(log::LevelFilter::Debug);
    }
    
    // Execute the command
    with_context(cli::execute(cli), || "Failed to execute command".to_string())
} 