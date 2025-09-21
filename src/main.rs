use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use colored::*;
use rust_decimal::Decimal;
use std::str::FromStr;

mod holdings;
mod models;

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
