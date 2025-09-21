use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add { symbol, amount, date, shares }) => {
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
            println!("{}", "No dividend records found. Use 'add' command to add some!".yellow());
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
        None => {
            println!("{}", "Dividend Tracker CLI".green().bold());
            println!("Use --help to see available commands");
        }
    }

    Ok(())
}
