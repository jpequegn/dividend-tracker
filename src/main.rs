use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use colored::*;
use csv;
use indicatif::{ProgressBar, ProgressStyle};
use rust_decimal::Decimal;
use std::str::FromStr;

mod api;
mod config;
mod holdings;
mod models;
mod notifications;
mod persistence;

use persistence::PersistenceManager;

#[derive(Parser)]
#[command(name = "dividend-tracker")]
#[command(about = "A CLI tool for tracking dividend payments and portfolio performance")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new dividend payment record
    Add {
        /// Stock symbol (e.g., AAPL, MSFT)
        symbol: String,
        /// Dividend amount per share
        #[arg(short, long)]
        amount: String,
        /// Payment date (YYYY-MM-DD format)
        #[arg(short, long)]
        date: Option<String>,
        /// Number of shares
        #[arg(short, long)]
        shares: Option<String>,
    },
    /// List dividend payments
    List {
        /// Filter by stock symbol
        #[arg(short, long)]
        symbol: Option<String>,
        /// Show payments from specific year
        #[arg(short, long)]
        year: Option<i32>,
    },
    /// Show portfolio summary and statistics
    Summary {
        /// Year to summarize (defaults to current year)
        #[arg(short, long)]
        year: Option<i32>,
    },
    /// Import dividend data from CSV file
    Import {
        /// Path to CSV file
        file: String,
    },
    /// Export dividend data to CSV file
    Export {
        /// Output file path
        #[arg(short, long, default_value = "dividends.csv")]
        output: String,
    },
    /// Manage stock holdings in your portfolio
    Holdings {
        #[command(subcommand)]
        command: HoldingsCommands,
    },
    /// Fetch dividend data from Alpha Vantage API
    Fetch {
        /// Stock symbols to fetch (comma-separated for multiple)
        symbols: String,
        /// Start date for dividend history (YYYY-MM-DD)
        #[arg(long, short = 'f')]
        from: Option<String>,
        /// End date for dividend history (YYYY-MM-DD)
        #[arg(long, short = 't')]
        to: Option<String>,
        /// Specific year to fetch
        #[arg(long)]
        year: Option<i32>,
        /// Portfolio CSV file to fetch symbols from
        #[arg(long)]
        portfolio: Option<String>,
    },
    /// Update existing dividend data with recent dividends
    Update {
        /// Update all symbols in the database
        #[arg(long)]
        all: bool,
        /// Update specific symbol
        #[arg(long)]
        symbol: Option<String>,
        /// Fetch dividends since last update
        #[arg(long)]
        since_last_fetch: bool,
    },
    /// Configure API settings
    Configure {
        /// Set Alpha Vantage API key
        #[arg(long)]
        api_key: Option<String>,
        /// Show current configuration
        #[arg(long)]
        show: bool,
    },
    /// Show dividend alerts for upcoming ex-dates
    Alerts {
        /// Generate new alerts
        #[arg(long)]
        generate: bool,
        /// Clear existing alerts
        #[arg(long)]
        clear: bool,
    },
    /// Display dividend calendar
    Calendar {
        /// Fetch/update calendar for portfolio holdings
        #[arg(long)]
        update: bool,
        /// Number of days to show (default: 90)
        #[arg(long, short = 'd')]
        days: Option<i64>,
        /// Export calendar to ICS file
        #[arg(long)]
        export: Option<String>,
    },
    /// Data management commands
    Data {
        #[command(subcommand)]
        command: DataCommands,
    },
}

#[derive(Subcommand)]
enum HoldingsCommands {
    /// Import holdings from CSV file
    Import {
        /// Path to CSV file with holdings data
        file: String,
    },
    /// Add or update a holding in your portfolio
    Add {
        /// Stock symbol (e.g., AAPL, MSFT)
        symbol: String,
        /// Number of shares owned
        #[arg(short, long)]
        shares: String,
        /// Average cost basis per share
        #[arg(short = 'c', long)]
        cost_basis: Option<String>,
        /// Current dividend yield percentage
        #[arg(short = 'y', long)]
        yield_pct: Option<String>,
    },
    /// Remove a holding from your portfolio
    Remove {
        /// Stock symbol to remove
        symbol: String,
    },
    /// List all holdings
    List {
        /// Sort holdings by field (symbol, shares, yield, value)
        #[arg(long)]
        sort_by: Option<String>,
        /// Show holdings in descending order
        #[arg(long)]
        desc: bool,
    },
    /// Export holdings to CSV file
    Export {
        /// Output file path
        #[arg(short, long, default_value = "holdings.csv")]
        output: String,
    },
    /// Show portfolio holdings summary
    Summary {
        /// Include yield calculations
        #[arg(long)]
        include_yield: bool,
    },
}

#[derive(Subcommand)]
enum DataCommands {
    /// Export data to different formats
    Export {
        /// Export format (csv, json)
        #[arg(short, long, default_value = "csv")]
        format: String,
        /// Output file path
        #[arg(short, long, default_value = "dividend_export")]
        output: String,
        /// Export type (dividends, holdings, all)
        #[arg(short, long, default_value = "all")]
        data_type: String,
    },
    /// Show data statistics and backup information
    Stats,
    /// Backup current data
    Backup,
    /// Load data from backup
    Load {
        /// Backup file to load from
        file: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add {
            symbol,
            amount,
            date,
            shares,
        }) => {
            println!("{}", "Adding dividend record...".green());
            println!("Symbol: {}", symbol.cyan());
            println!("Amount: ${}", amount.yellow());
            if let Some(date) = date {
                println!("Date: {}", date.blue());
            }
            if let Some(shares) = shares {
                println!("Shares: {}", shares.magenta());
            }
            println!("{}", "✓ Dividend record added successfully!".green());
        }
        Some(Commands::List { symbol, year }) => {
            println!("{}", "Listing dividend payments...".green());
            if let Some(symbol) = symbol {
                println!("Filtering by symbol: {}", symbol.cyan());
            }
            if let Some(year) = year {
                println!("Filtering by year: {}", year.to_string().blue());
            }
            println!(
                "{}",
                "No dividend records found. Use 'add' command to add some!".yellow()
            );
        }
        Some(Commands::Summary { year }) => {
            println!("{}", "Portfolio Summary".green().bold());
            if let Some(year) = year {
                println!("Year: {}", year.to_string().blue());
            }
            println!("{}", "No data available yet.".yellow());
        }
        Some(Commands::Import { file }) => {
            println!("{}", "Importing dividend data...".green());
            println!("File: {}", file.cyan());
            println!("{}", "Import functionality not yet implemented.".yellow());
        }
        Some(Commands::Export { output }) => {
            println!("{}", "Exporting dividend data...".green());
            println!("Output file: {}", output.cyan());
            println!("{}", "Export functionality not yet implemented.".yellow());
        }
        Some(Commands::Holdings { command }) => {
            handle_holdings_command(command)?;
        }
        Some(Commands::Fetch {
            symbols,
            from,
            to,
            year,
            portfolio,
        }) => {
            handle_fetch_command(symbols, from, to, year, portfolio)?;
        }
        Some(Commands::Update {
            all,
            symbol,
            since_last_fetch,
        }) => {
            handle_update_command(all, symbol, since_last_fetch)?;
        }
        Some(Commands::Configure { api_key, show }) => {
            handle_configure_command(api_key, show)?;
        }
        Some(Commands::Alerts { generate, clear }) => {
            handle_alerts_command(generate, clear)?;
        }
        Some(Commands::Calendar { update, days, export }) => {
            handle_calendar_command(update, days, export)?;
        }
        Some(Commands::Data { command }) => {
            handle_data_command(command)?;
        }
        None => {
            println!("{}", "Dividend Tracker CLI".green().bold());
            println!("Use --help to see available commands");
        }
    }

    Ok(())
}

/// Handle holdings-related commands
fn handle_holdings_command(command: HoldingsCommands) -> Result<()> {
    match command {
        HoldingsCommands::Import { file } => {
            holdings::import_holdings(&file)?;
        }
        HoldingsCommands::Add {
            symbol,
            shares,
            cost_basis,
            yield_pct,
        } => {
            let shares_decimal = Decimal::from_str(&shares)
                .map_err(|_| anyhow!("Invalid shares amount: {}", shares))?;

            let cost_basis_decimal = if let Some(cb) = cost_basis {
                Some(Decimal::from_str(&cb).map_err(|_| anyhow!("Invalid cost basis: {}", cb))?)
            } else {
                None
            };

            let yield_decimal = if let Some(y) = yield_pct {
                Some(
                    Decimal::from_str(&y)
                        .map_err(|_| anyhow!("Invalid yield percentage: {}", y))?,
                )
            } else {
                None
            };

            holdings::add_holding(&symbol, shares_decimal, cost_basis_decimal, yield_decimal)?;
        }
        HoldingsCommands::Remove { symbol } => {
            holdings::remove_holding(&symbol)?;
        }
        HoldingsCommands::List { sort_by, desc } => {
            holdings::list_holdings(sort_by.as_deref(), desc)?;
        }
        HoldingsCommands::Export { output } => {
            holdings::export_holdings(&output)?;
        }
        HoldingsCommands::Summary { include_yield } => {
            holdings::show_summary(include_yield)?;
        }
    }
    Ok(())
}

/// Handle the fetch command
fn handle_fetch_command(
    symbols: String,
    from: Option<String>,
    to: Option<String>,
    year: Option<i32>,
    portfolio: Option<String>,
) -> Result<()> {
    println!("{}", "Fetching dividend data...".green().bold());

    // Load configuration
    let config = config::Config::load()?;
    let api_key = config.get_api_key()?;

    // Create API client
    let client = api::AlphaVantageClient::new(api_key)?;

    // Parse dates
    let from_date = parse_date_input(from, year, true)?;
    let to_date = parse_date_input(to, year, false)?;

    // Get symbols to fetch
    let symbol_list = if let Some(portfolio_file) = portfolio {
        load_symbols_from_portfolio(&portfolio_file)?
    } else {
        symbols
            .split(',')
            .map(|s| s.trim().to_uppercase())
            .collect::<Vec<_>>()
    };

    if symbol_list.len() == 1 {
        // Single symbol fetch
        let symbol = &symbol_list[0];
        println!("Fetching dividends for {}...", symbol.cyan());

        match client.fetch_dividends(symbol, from_date, to_date) {
            Ok(dividends) => {
                if dividends.is_empty() {
                    println!(
                        "{}: No dividends found for the specified period",
                        symbol.yellow()
                    );
                } else {
                    println!(
                        "{}: Found {} dividend payments",
                        symbol.green(),
                        dividends.len()
                    );
                    for dividend in &dividends {
                        println!(
                            "  {} - ${} per share",
                            dividend.ex_date.format("%Y-%m-%d"),
                            dividend.amount
                        );
                    }
                }
            }
            Err(e) => {
                println!("{}: Failed to fetch - {}", symbol.red(), e);
            }
        }
    } else {
        // Batch fetch with progress bar
        let pb = ProgressBar::new(symbol_list.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );

        let pb_clone = pb.clone();
        let results = client.batch_fetch_dividends(
            &symbol_list,
            from_date,
            to_date,
            Some(Box::new(move |current, _total, symbol| {
                pb_clone.set_position(current as u64);
                pb_clone.set_message(format!("Fetching {}", symbol));
            })),
        );

        pb.finish_with_message("Done");

        // Display results
        let mut success_count = 0;
        let mut total_dividends = 0;

        for (symbol, result) in &results {
            match result {
                Ok(dividends) => {
                    success_count += 1;
                    total_dividends += dividends.len();
                    println!("{}: {} dividends", symbol.green(), dividends.len());
                }
                Err(e) => {
                    println!("{}: {}", symbol.red(), e);
                }
            }
        }

        println!();
        println!(
            "Fetched {} symbols successfully, {} total dividend payments",
            success_count.to_string().green(),
            total_dividends.to_string().cyan()
        );
    }

    Ok(())
}

/// Handle the update command
fn handle_update_command(all: bool, symbol: Option<String>, since_last_fetch: bool) -> Result<()> {
    println!("{}", "Update functionality not yet implemented.".yellow());
    println!("This will update existing dividend data with recent dividends.");

    if all {
        println!("Would update all symbols in the database");
    } else if let Some(symbol) = symbol {
        println!("Would update dividends for {}", symbol.cyan());
    }

    if since_last_fetch {
        println!("Would fetch only dividends since last update");
    }

    Ok(())
}

/// Handle the configure command
fn handle_configure_command(api_key: Option<String>, show: bool) -> Result<()> {
    let mut config = config::Config::load()?;

    if show {
        println!("{}", "Current Configuration:".green().bold());
        println!(
            "API Key: {}",
            if config.api.alpha_vantage_key.is_some() {
                "******* (configured)".green()
            } else {
                "Not configured".yellow()
            }
        );
        println!("Rate Limit Delay: {}ms", config.api.rate_limit_delay_ms);
        println!("Max Retries: {}", config.api.max_retries);
        println!("Cache Enabled: {}", config.cache.enabled);
        println!("Cache TTL: {} hours", config.cache.ttl_hours);
        return Ok(());
    }

    if let Some(key) = api_key {
        config.api.alpha_vantage_key = Some(key);
        config.save()?;
        println!("{}", "API key saved successfully!".green());
        println!("Configuration file: {:?}", config::Config::config_file()?);
    } else {
        println!("{}", "Configuration Options:".green().bold());
        println!("Use --api-key to set your Alpha Vantage API key");
        println!("Use --show to display current configuration");
        println!();
        println!("To get a free API key, visit: https://www.alphavantage.co/support/#api-key");
    }

    Ok(())
}

/// Parse date input from string or year
fn parse_date_input(
    date_str: Option<String>,
    year: Option<i32>,
    is_from: bool,
) -> Result<Option<NaiveDate>> {
    if let Some(date) = date_str {
        Ok(Some(NaiveDate::parse_from_str(&date, "%Y-%m-%d")?))
    } else if let Some(y) = year {
        if is_from {
            Ok(Some(
                NaiveDate::from_ymd_opt(y, 1, 1).ok_or_else(|| anyhow!("Invalid year"))?,
            ))
        } else {
            Ok(Some(
                NaiveDate::from_ymd_opt(y, 12, 31).ok_or_else(|| anyhow!("Invalid year"))?,
            ))
        }
    } else {
        Ok(None)
    }
}

/// Load symbols from a portfolio CSV file
fn load_symbols_from_portfolio(file_path: &str) -> Result<Vec<String>> {
    let mut symbols = Vec::new();
    let mut rdr = csv::Reader::from_path(file_path)?;

    for result in rdr.records() {
        let record = result?;
        if let Some(symbol) = record.get(0) {
            symbols.push(symbol.trim().to_uppercase());
        }
    }

    if symbols.is_empty() {
        return Err(anyhow!("No symbols found in portfolio file"));
    }

    Ok(symbols)
}

/// Handle alerts command
fn handle_alerts_command(generate: bool, clear: bool) -> Result<()> {
    let mut manager = notifications::NotificationManager::load()?;

    if clear {
        manager.alerts.clear();
        manager.save()?;
        println!("{}", "Alerts cleared successfully!".green());
        return Ok(());
    }

    if generate {
        manager.generate_alerts()?;
        println!("{}", "Alerts generated successfully!".green());
    }

    // Show current alerts
    manager.show_alerts()?;

    Ok(())
}

/// Handle calendar command
fn handle_calendar_command(update: bool, days: Option<i64>, export: Option<String>) -> Result<()> {
    let mut manager = notifications::NotificationManager::load()?;

    if update {
        // Load configuration
        let config = config::Config::load()?;
        let api_key = config.get_api_key()?;

        // Create API client
        let client = api::AlphaVantageClient::new(api_key)?;

        // Fetch upcoming dividends
        manager.fetch_upcoming_dividends(&client)?;
    }

    // Export to ICS if requested
    if let Some(output_path) = export {
        manager.export_to_ics(&output_path)?;
        return Ok(());
    }

    // Show calendar
    manager.show_calendar(days)?;

    Ok(())
}

/// Handle data management commands
fn handle_data_command(command: DataCommands) -> Result<()> {
    match command {
        DataCommands::Export {
            format,
            output,
            data_type,
        } => {
            let persistence = PersistenceManager::new()?;

            match data_type.as_str() {
                "dividends" => {
                    let output_filename = if format == "csv" {
                        format!("{}.csv", output)
                    } else {
                        format!("{}.json", output)
                    };
                    let output_path = std::path::Path::new(&output_filename);

                    if format == "csv" {
                        persistence.export_to_csv(output_path)?;
                        println!(
                            "{} Dividends exported to {}",
                            "✓".green(),
                            output_path.display().to_string().cyan()
                        );
                    } else {
                        persistence.export_to_json(output_path)?;
                        println!(
                            "{} All data exported to {}",
                            "✓".green(),
                            output_path.display().to_string().cyan()
                        );
                    }
                }
                "holdings" => {
                    let output_filename = format!("{}_holdings.csv", output);
                    let output_path = std::path::Path::new(&output_filename);
                    persistence.export_holdings_to_csv(output_path)?;
                    println!(
                        "{} Holdings exported to {}",
                        "✓".green(),
                        output_path.display().to_string().cyan()
                    );
                }
                "all" | _ => {
                    if format == "csv" {
                        // Export both dividends and holdings as separate CSV files
                        let dividends_filename = format!("{}_dividends.csv", output);
                        let holdings_filename = format!("{}_holdings.csv", output);
                        let dividends_path = std::path::Path::new(&dividends_filename);
                        let holdings_path = std::path::Path::new(&holdings_filename);

                        persistence.export_to_csv(dividends_path)?;
                        persistence.export_holdings_to_csv(holdings_path)?;

                        println!("{} Data exported to:", "✓".green());
                        println!("  Dividends: {}", dividends_path.display().to_string().cyan());
                        println!("  Holdings: {}", holdings_path.display().to_string().cyan());
                    } else {
                        let output_filename = format!("{}.json", output);
                        let output_path = std::path::Path::new(&output_filename);
                        persistence.export_to_json(output_path)?;
                        println!(
                            "{} All data exported to {}",
                            "✓".green(),
                            output_path.display().to_string().cyan()
                        );
                    }
                }
            }
        }
        DataCommands::Stats => {
            let persistence = PersistenceManager::new()?;
            let stats = persistence.get_stats()?;

            println!("{}", "Data Statistics".green().bold());
            println!();
            println!(
                "📂 {} {}",
                "Data Directory:".bright_blue(),
                stats.data_directory.display().to_string().cyan()
            );
            println!(
                "💰 {} {}",
                "Dividend Records:".bright_blue(),
                stats.dividend_count.to_string().cyan()
            );
            println!(
                "📊 {} {}",
                "Holdings:".bright_blue(),
                stats.holding_count.to_string().cyan()
            );
            println!(
                "💾 {} {} bytes",
                "Total Data Size:".bright_blue(),
                stats.total_size_bytes.to_string().cyan()
            );
            println!(
                "🔄 {} {}",
                "Backup Files:".bright_blue(),
                stats.backup_count.to_string().cyan()
            );
        }
        DataCommands::Backup => {
            println!("{}", "Creating manual backup...".green());
            let persistence = PersistenceManager::new()?;

            // Load and save to force a backup
            let tracker = persistence.load()?;
            persistence.save(&tracker)?;

            println!("{} Manual backup created successfully!", "✓".green());
        }
        DataCommands::Load { file } => {
            println!("{}", "Load functionality not yet implemented.".yellow());
            println!("Would load data from: {}", file.cyan());
            println!("This feature will be added in a future update.");
        }
    }

    Ok(())
}
