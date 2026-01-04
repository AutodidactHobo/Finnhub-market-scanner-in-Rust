use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;

mod config;
mod errors;
mod finnhub;
mod output;

use config::Config;
use errors::Result;
use finnhub::FinnhubClient;
use output::OutputFormat;

#[derive(Parser)]
#[command(name = "finnhub-scanner")]
#[command(author = "Your Name")]
#[command(version = "0.1.0")]
#[command(about = "Professional stock market scanner using Finnhub API", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan stocks and display results
    Scan {
        /// Stock symbols to scan (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        symbols: Option<Vec<String>>,

        /// Path to symbols file (one per line)
        #[arg(short = 'f', long)]
        symbols_file: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "table")]
        output: OutputFormat,

        /// Sort by absolute change
        #[arg(long)]
        sort_by_change: bool,

        /// Show only gainers
        #[arg(long)]
        gainers_only: bool,

        /// Show only losers
        #[arg(long)]
        losers_only: bool,

        /// Minimum absolute change threshold (%)
        #[arg(long)]
        min_change: Option<f64>,
    },

    /// Watch stocks with continuous updates
    Watch {
        /// Stock symbols to watch (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        symbols: Option<Vec<String>>,

        /// Path to symbols file
        #[arg(short = 'f', long)]
        symbols_file: Option<PathBuf>,

        /// Update interval in seconds
        #[arg(short, long, default_value = "60")]
        interval: u64,
    },

    /// Display configuration
    Config {
        /// Show current configuration
        #[arg(long)]
        show: bool,

        /// Initialize default config file
        #[arg(long)]
        init: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logger
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    // Load config
    let config = if let Some(config_path) = cli.config {
        Config::from_file(&config_path)?
    } else {
        Config::from_env_or_default()?
    };

    match cli.command {
        Commands::Scan {
            symbols,
            symbols_file,
            output,
            sort_by_change,
            gainers_only,
            losers_only,
            min_change,
        } => {
            let symbol_list = load_symbols(symbols, symbols_file, &config)?;
            
            let client = FinnhubClient::new(config.api_key.clone(), config.clone());
            let quotes = client.fetch_quotes(&symbol_list).await?;
            
            let filtered = output::filter_quotes(
                quotes,
                gainers_only,
                losers_only,
                min_change,
            );
            
            let sorted = if sort_by_change {
                output::sort_by_change(filtered)
            } else {
                filtered
            };
            
            output::display(&sorted, output)?;
        }

        Commands::Watch {
            symbols,
            symbols_file,
            interval,
        } => {
            let symbol_list = load_symbols(symbols, symbols_file, &config)?;
            let client = FinnhubClient::new(config.api_key.clone(), config.clone());
            
            log::info!("Starting watch mode. Press Ctrl+C to exit.");
            
            loop {
                match client.fetch_quotes(&symbol_list).await {
                    Ok(quotes) => {
                        output::clear_screen();
                        output::display(&quotes, OutputFormat::Table)?;
                        log::info!("Updated at: {}", chrono::Local::now().format("%H:%M:%S"));
                    }
                    Err(e) => {
                        log::error!("Failed to fetch quotes: {}", e);
                    }
                }
                
                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        }

        Commands::Config { show, init } => {
            if init {
                let default_config = Config::default();
                default_config.save_to_file("config.toml")?;
                println!("✓ Default config created at config.toml");
                println!("  Don't forget to add your Finnhub API key!");
            } else if show {
                println!("{:#?}", config);
            }
        }
    }

    Ok(())
}

fn load_symbols(
    symbols: Option<Vec<String>>,
    symbols_file: Option<PathBuf>,
    config: &Config,
) -> Result<Vec<String>> {
    // Priority: CLI args > file arg > config file > default
    if let Some(syms) = symbols {
        return Ok(syms.iter().map(|s| s.to_uppercase()).collect());
    }
    
    if let Some(path) = symbols_file {
        return config::load_symbols_from_file(&path);
    }
    
    if let Some(path) = &config.symbols_file {
        return config::load_symbols_from_file(path);
    }
    
    Err(errors::ScannerError::NoSymbols)
}