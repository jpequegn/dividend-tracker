use anyhow::{anyhow, Result};
use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};
use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use rust_decimal::Decimal;
use std::str::FromStr;
use tabled::{builder::Builder, settings::Style};

mod analytics;
mod api;
mod config;
mod holdings;
mod models;
mod notifications;
mod persistence;
mod projections;
mod tax;

use persistence::PersistenceManager;

/// Global CLI configuration passed to all command handlers
#[derive(Clone)]
pub struct CliConfig {
    pub data_dir: Option<String>,
    pub verbose: bool,
    pub quiet: bool,
}

impl CliConfig {
    /// Create a PersistenceManager with the configured data directory
    pub fn create_persistence_manager(&self) -> Result<PersistenceManager> {
        if let Some(ref data_dir) = self.data_dir {
            Ok(PersistenceManager::with_custom_path(data_dir))
        } else {
            PersistenceManager::new()
        }
    }

    /// Print message respecting verbose/quiet flags
    pub fn print(&self, message: &str) {
        if !self.quiet {
            println!("{}", message);
        }
    }

    /// Print verbose message only in verbose mode
    pub fn print_verbose(&self, message: &str) {
        if self.verbose && !self.quiet {
            println!("🔧 {}", message);
        }
    }

    /// Print error message (always shown unless quiet)
    pub fn print_error(&self, message: &str) {
        if !self.quiet {
            eprintln!("❌ {}", message);
        }
    }

    /// Print success message (always shown unless quiet)
    pub fn print_success(&self, message: &str) {
        if !self.quiet {
            println!("✅ {}", message);
        }
    }
}

#[derive(Parser)]
#[command(name = "dividend-tracker")]
#[command(about = "A comprehensive CLI tool for tracking dividend payments, managing stock holdings, and analyzing portfolio performance")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(long_about = "
dividend-tracker helps you track dividend payments, manage stock holdings,
and analyze your portfolio performance. It supports importing data from CSV files,
fetching real-time data from APIs, and exporting reports in multiple formats.

Data is stored securely in JSON format with automatic backups.

EXAMPLES:
    # Add a dividend payment
    dividend-tracker add AAPL --amount 0.94 --shares 100 --date 2024-02-15

    # List all dividends for a specific year
    dividend-tracker list --year 2024

    # Add a stock holding
    dividend-tracker holdings add MSFT --shares 50 --cost-basis 150.00

    # Export data to CSV
    dividend-tracker data export --format csv --output my_portfolio

    # Show portfolio summary
    dividend-tracker summary --year 2024
")]
struct Cli {
    /// Custom data directory path (default: ~/.dividend-tracker)
    #[arg(long, global = true, help = "Specify custom data directory")]
    data_dir: Option<String>,

    /// Enable verbose output
    #[arg(short = 'v', long, global = true, help = "Show detailed output")]
    verbose: bool,

    /// Enable quiet mode (minimal output)
    #[arg(short = 'q', long, global = true, help = "Show minimal output")]
    quiet: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new dividend payment record
    Add {
        /// Stock symbol (e.g., AAPL, MSFT)
        symbol: String,
        /// Ex-dividend date (YYYY-MM-DD, 'tomorrow', 'next friday', etc.)
        #[arg(long)]
        ex_date: String,
        /// Payment date (YYYY-MM-DD, 'tomorrow', 'next friday', etc.)
        #[arg(long)]
        pay_date: String,
        /// Dividend amount per share
        #[arg(short, long)]
        amount: String,
        /// Number of shares owned
        #[arg(short, long)]
        shares: String,
        /// Force adding even if duplicate (same symbol + ex-date) exists
        #[arg(long)]
        force: bool,
    },
    /// List dividend payments
    List {
        /// Filter by stock symbol
        #[arg(short, long)]
        symbol: Option<String>,
        /// Show payments from specific year
        #[arg(short, long)]
        year: Option<i32>,
        /// Filter by specific month (1-12)
        #[arg(short, long)]
        month: Option<u32>,
        /// Filter by date range (start date YYYY-MM-DD)
        #[arg(long)]
        date_start: Option<String>,
        /// Filter by date range (end date YYYY-MM-DD)
        #[arg(long)]
        date_end: Option<String>,
        /// Minimum dividend amount per share
        #[arg(long)]
        amount_min: Option<String>,
        /// Show only upcoming pay dates (future)
        #[arg(long)]
        upcoming: bool,
        /// Sort by field (symbol, ex-date, pay-date, amount, total)
        #[arg(long, default_value = "ex-date")]
        sort_by: String,
        /// Sort in descending order
        #[arg(long)]
        reverse: bool,
    },
    /// Show portfolio summary and statistics
    Summary {
        /// Year to summarize (defaults to current year)
        #[arg(short, long)]
        year: Option<i32>,
        /// Quarter to summarize (format: Q1-2024, Q2-2024, etc.)
        #[arg(short, long)]
        quarter: Option<String>,
        /// Show top dividend paying stocks
        #[arg(long)]
        top_payers: Option<usize>,
        /// Show year-over-year growth analysis
        #[arg(long)]
        growth: bool,
        /// Show dividend frequency analysis
        #[arg(long)]
        frequency: bool,
        /// Show dividend consistency analysis
        #[arg(long)]
        consistency: bool,
        /// Show yield analysis (requires holdings with cost basis)
        #[arg(long)]
        yield_analysis: bool,
        /// Export summary to CSV file
        #[arg(long)]
        export_csv: Option<String>,
        /// Show monthly breakdown for the year
        #[arg(long)]
        monthly: bool,
        /// Show all analytics (equivalent to --growth --frequency --consistency --yield-analysis)
        #[arg(long)]
        all: bool,
    },
    /// Project future dividend income based on historical data
    Project {
        /// Projection method to use
        #[arg(long, default_value = "last-12-months")]
        method: String,
        /// Growth scenario (conservative, moderate, optimistic, or custom percentage)
        #[arg(long, default_value = "moderate")]
        growth_rate: String,
        /// Target year to project (defaults to next year)
        #[arg(long)]
        year: Option<i32>,
        /// Export projections to CSV file
        #[arg(long)]
        export_csv: Option<String>,
        /// Export projections to JSON file
        #[arg(long)]
        export_json: Option<String>,
        /// Show detailed monthly breakdown
        #[arg(long)]
        monthly: bool,
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
    /// Tax reporting and analysis commands
    Tax {
        #[command(subcommand)]
        command: TaxCommands,
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
enum TaxCommands {
    /// Generate annual tax summary for a specific year
    Summary {
        /// Tax year to analyze (defaults to current year)
        #[arg(short, long)]
        year: Option<i32>,
        /// Include estimated tax calculations
        #[arg(long)]
        estimate: bool,
        /// Filing status for tax estimates (single, married-jointly, married-separately, head-of-household)
        #[arg(long)]
        filing_status: Option<String>,
        /// Income bracket for tax estimates (low, medium, high, very-high)
        #[arg(long)]
        income_bracket: Option<String>,
        /// Export summary to CSV file
        #[arg(long)]
        export_csv: Option<String>,
    },
    /// Generate 1099-DIV style report
    Report {
        /// Tax year for the report (defaults to current year)
        #[arg(short, long)]
        year: Option<i32>,
        /// Export report to CSV file
        #[arg(long)]
        export_csv: Option<String>,
        /// Export report to JSON file
        #[arg(long)]
        export_json: Option<String>,
    },
    /// Calculate estimated taxes on dividend income
    Estimate {
        /// Tax year to analyze (defaults to current year)
        #[arg(short, long)]
        year: Option<i32>,
        /// Filing status (single, married-jointly, married-separately, head-of-household)
        #[arg(short, long, default_value = "single")]
        filing_status: String,
        /// Income bracket (low, medium, high, very-high)
        #[arg(short, long, default_value = "medium")]
        income_bracket: String,
    },
    /// Show tax lot breakdown (if cost basis tracking enabled)
    Lots {
        /// Tax year to analyze (defaults to current year)
        #[arg(short, long)]
        year: Option<i32>,
        /// Filter by stock symbol
        #[arg(short, long)]
        symbol: Option<String>,
        /// Export to CSV file
        #[arg(long)]
        export_csv: Option<String>,
    },
    /// Update tax classification for dividends
    Classify {
        /// Stock symbol to update
        symbol: String,
        /// Tax classification (qualified, non-qualified, return-of-capital, tax-free, foreign)
        #[arg(short, long)]
        classification: String,
        /// Year to update (optional, updates all if not specified)
        #[arg(short, long)]
        year: Option<i32>,
        /// Apply to all future dividends from this symbol
        #[arg(long)]
        apply_future: bool,
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

    // Create global CLI configuration
    let config = CliConfig {
        data_dir: cli.data_dir.clone(),
        verbose: cli.verbose,
        quiet: cli.quiet,
    };

    // Show verbose information about configuration
    if config.verbose {
        config.print_verbose("Starting dividend-tracker with configuration:");
        if let Some(ref data_dir) = config.data_dir {
            config.print_verbose(&format!("Data directory: {}", data_dir));
        } else {
            config.print_verbose("Data directory: ~/.dividend-tracker (default)");
        }
    }

    match cli.command {
        Some(Commands::Add {
            symbol,
            ex_date,
            pay_date,
            amount,
            shares,
            force,
        }) => {
            handle_add_command(symbol, ex_date, pay_date, amount, shares, force)?;
        }
        Some(Commands::List {
            symbol,
            year,
            month,
            date_start,
            date_end,
            amount_min,
            upcoming,
            sort_by,
            reverse
        }) => {
            handle_list_command(
                symbol,
                year,
                month,
                date_start,
                date_end,
                amount_min,
                upcoming,
                sort_by,
                reverse
            )?;
        }
        Some(Commands::Summary {
            year,
            quarter,
            top_payers,
            growth,
            frequency,
            consistency,
            yield_analysis,
            export_csv,
            monthly,
            all,
        }) => {
            handle_summary_command(
                year,
                quarter,
                top_payers,
                growth,
                frequency,
                consistency,
                yield_analysis,
                export_csv,
                monthly,
                all,
            )?;
        }
        Some(Commands::Project {
            method,
            growth_rate,
            year,
            export_csv,
            export_json,
            monthly,
        }) => {
            handle_project_command(method, growth_rate, year, export_csv, export_json, monthly)?;
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
        Some(Commands::Calendar {
            update,
            days,
            export,
        }) => {
            handle_calendar_command(update, days, export)?;
        }
        Some(Commands::Data { command }) => {
            handle_data_command(command, &config)?;
        }
        Some(Commands::Tax { command }) => {
            handle_tax_command(command)?;
        }
        None => {
            println!("{}", "Dividend Tracker CLI".green().bold());
            println!("Use --help to see available commands");
        }
    }

    Ok(())
}

/// Handle listing dividend payments with filtering and sorting
fn handle_list_command(
    symbol: Option<String>,
    year: Option<i32>,
    month: Option<u32>,
    date_start: Option<String>,
    date_end: Option<String>,
    amount_min: Option<String>,
    upcoming: bool,
    sort_by: String,
    reverse: bool,
) -> Result<()> {
    use crate::models::Dividend;

    println!("{}", "Listing dividend payments...".green().bold());

    // Load persistence manager and existing data
    let persistence = PersistenceManager::new()?;
    let tracker = persistence.load()?;

    if tracker.dividends.is_empty() {
        println!(
            "{}",
            "No dividend records found. Use 'add' command to add some!".yellow()
        );
        return Ok(());
    }

    // Parse filters
    let date_start_parsed = if let Some(ref ds) = date_start {
        Some(parse_dividend_date(ds)?)
    } else {
        None
    };

    let date_end_parsed = if let Some(ref de) = date_end {
        Some(parse_dividend_date(de)?)
    } else {
        None
    };

    let amount_min_parsed = if let Some(ref am) = amount_min {
        Some(Decimal::from_str(am).map_err(|_| {
            anyhow!("Invalid minimum amount format: {}. Use decimal format like 0.50", am)
        })?)
    } else {
        None
    };

    // Filter dividends
    let mut filtered_dividends: Vec<&Dividend> = tracker.dividends
        .iter()
        .filter(|div| {
            // Symbol filter
            if let Some(ref sym) = symbol {
                if !div.symbol.to_uppercase().contains(&sym.to_uppercase()) {
                    return false;
                }
            }

            // Year filter
            if let Some(y) = year {
                if div.ex_date.year() != y {
                    return false;
                }
            }

            // Month filter
            if let Some(m) = month {
                if div.ex_date.month() != m {
                    return false;
                }
            }

            // Date range filter
            if let Some(start) = date_start_parsed {
                if div.ex_date < start {
                    return false;
                }
            }

            if let Some(end) = date_end_parsed {
                if div.ex_date > end {
                    return false;
                }
            }

            // Amount minimum filter
            if let Some(min_amount) = amount_min_parsed {
                if div.amount_per_share < min_amount {
                    return false;
                }
            }

            // Upcoming filter (future pay dates only)
            if upcoming {
                let today = Local::now().naive_local().date();
                if div.pay_date <= today {
                    return false;
                }
            }

            true
        })
        .collect();

    if filtered_dividends.is_empty() {
        println!("{}", "No dividends match the specified filters.".yellow());
        return Ok(());
    }

    // Sort dividends
    filtered_dividends.sort_by(|a, b| {
        let comparison = match sort_by.as_str() {
            "symbol" => a.symbol.cmp(&b.symbol),
            "ex-date" => a.ex_date.cmp(&b.ex_date),
            "pay-date" => a.pay_date.cmp(&b.pay_date),
            "amount" => a.amount_per_share.cmp(&b.amount_per_share),
            "total" => a.total_amount.cmp(&b.total_amount),
            _ => a.ex_date.cmp(&b.ex_date), // Default to ex-date
        };

        if reverse {
            comparison.reverse()
        } else {
            comparison
        }
    });

    // Build table
    let mut builder = Builder::new();

    // Add header
    builder.push_record(vec![
        "Symbol".bold().to_string(),
        "Company".bold().to_string(),
        "Ex-Date".bold().to_string(),
        "Pay-Date".bold().to_string(),
        "$/Share".bold().to_string(),
        "Shares".bold().to_string(),
        "Total".bold().to_string(),
    ]);

    // Add dividend rows
    let today = Local::now().naive_local().date();
    let mut total_income = Decimal::ZERO;

    for dividend in &filtered_dividends {
        total_income += dividend.total_amount;

        // Color upcoming dividends green
        let is_upcoming = dividend.pay_date > today;

        let symbol = if is_upcoming {
            dividend.symbol.green().to_string()
        } else {
            dividend.symbol.to_string()
        };

        let company = dividend.company_name
            .as_ref()
            .map(|c| if is_upcoming { c.green().to_string() } else { c.to_string() })
            .unwrap_or_else(|| "-".to_string());

        let ex_date = if is_upcoming {
            dividend.ex_date.format("%Y-%m-%d").to_string().green().to_string()
        } else {
            dividend.ex_date.format("%Y-%m-%d").to_string()
        };

        let pay_date = if is_upcoming {
            dividend.pay_date.format("%Y-%m-%d").to_string().green().to_string()
        } else {
            dividend.pay_date.format("%Y-%m-%d").to_string()
        };

        let amount_str = format!("${:.4}", dividend.amount_per_share);
        let amount = if is_upcoming {
            amount_str.green().to_string()
        } else {
            amount_str
        };

        let shares_str = dividend.shares_owned.to_string();
        let shares = if is_upcoming {
            shares_str.green().to_string()
        } else {
            shares_str
        };

        let total_str = format!("${:.2}", dividend.total_amount);
        let total = if is_upcoming {
            total_str.green().to_string()
        } else {
            total_str
        };

        builder.push_record(vec![
            symbol,
            company,
            ex_date,
            pay_date,
            amount,
            shares,
            total,
        ]);
    }

    // Create and style the table
    let mut table = builder.build();
    table.with(Style::rounded());

    println!("{}", table);
    println!();

    // Show summary
    println!("{} {}",
        "Total Dividends:".bold(),
        format!("${:.2}", total_income).green().bold()
    );

    println!("{} {}",
        "Number of Payments:".bold(),
        filtered_dividends.len().to_string().cyan().bold()
    );

    // Show filter summary
    let has_filters = symbol.is_some() || year.is_some() || month.is_some() || date_start.is_some() ||
                     date_end.is_some() || amount_min.is_some() || upcoming;

    if has_filters || sort_by != "ex-date" || reverse {
        println!();

        if has_filters {
            println!("{}", "Applied Filters:".bold());

            if let Some(sym) = symbol {
                println!("  Symbol: {}", sym.cyan());
            }
            if let Some(y) = year {
                println!("  Year: {}", y.to_string().blue());
            }
            if let Some(m) = month {
                println!("  Month: {}", m.to_string().blue());
            }
            if let Some(ds) = date_start {
                println!("  Date Start: {}", ds.blue());
            }
            if let Some(de) = date_end {
                println!("  Date End: {}", de.blue());
            }
            if let Some(am) = amount_min {
                println!("  Min Amount: ${}", am.blue());
            }
            if upcoming {
                println!("  {} {}", "Upcoming Only:".blue(), "Yes".green());
            }
        }

        println!("  Sorted by: {} {}", sort_by.yellow(),
            if reverse { "(descending)".dimmed() } else { "(ascending)".dimmed() });
    }

    Ok(())
}

/// Handle summary command with comprehensive analytics
fn handle_summary_command(
    year: Option<i32>,
    quarter: Option<String>,
    top_payers: Option<usize>,
    growth: bool,
    frequency: bool,
    consistency: bool,
    yield_analysis: bool,
    export_csv: Option<String>,
    monthly: bool,
    all: bool,
) -> Result<()> {
    use crate::analytics::DividendAnalytics;

    println!("{}", "Portfolio Summary & Analytics".green().bold());
    println!();

    // Load persistence manager and existing data
    let persistence = PersistenceManager::new()?;
    let tracker = persistence.load()?;

    if tracker.dividends.is_empty() {
        println!(
            "{}",
            "No dividend records found. Use 'add' command to add some dividends first!".yellow()
        );
        return Ok(());
    }

    // Set flags based on 'all' option
    let show_growth = all || growth;
    let show_frequency = all || frequency;
    let show_consistency = all || consistency;
    let show_yield = all || yield_analysis;

    // Generate analytics
    let analytics = DividendAnalytics::generate(
        &tracker,
        year,
        quarter.as_deref(),
    )?;

    // Display basic summary
    display_basic_summary(&analytics, year, quarter.as_deref())?;

    // Display monthly breakdown if requested
    if monthly {
        display_monthly_breakdown(&analytics, year)?;
    }

    // Display quarterly breakdown if quarter filter is used
    if quarter.is_some() {
        display_quarterly_breakdown(&analytics)?;
    }

    // Display top payers
    if let Some(limit) = top_payers {
        display_top_payers(&analytics, limit)?;
    }

    // Display growth analysis
    if show_growth {
        display_growth_analysis(&analytics)?;
    }

    // Display frequency analysis
    if show_frequency {
        display_frequency_analysis(&analytics)?;
    }

    // Display consistency analysis
    if show_consistency {
        display_consistency_analysis(&analytics)?;
    }

    // Display yield analysis
    if show_yield {
        display_yield_analysis(&analytics)?;
    }

    // Export to CSV if requested
    if let Some(csv_path) = export_csv {
        analytics.export_to_csv(&csv_path)?;
        println!();
        println!("{} Analytics exported to {}",
                 "✓".green(),
                 csv_path.cyan());
    }

    Ok(())
}

fn display_basic_summary(
    analytics: &analytics::DividendAnalytics,
    year: Option<i32>,
    quarter: Option<&str>,
) -> Result<()> {
    println!("{}", "📊 Basic Summary".blue().bold());

    if let Some(year) = year {
        println!("  Year: {}", year.to_string().cyan());
    }
    if let Some(quarter) = quarter {
        println!("  Quarter: {}", quarter.cyan());
    }

    println!("  Total Dividend Income: {}",
             format!("${:.2}", analytics.total_dividends).green().bold());
    println!("  Total Payments: {}",
             analytics.total_payments.to_string().cyan());
    println!("  Unique Stocks: {}",
             analytics.unique_symbols.to_string().cyan());

    if analytics.total_payments > 0 {
        let avg_payment = analytics.total_dividends / rust_decimal::Decimal::from(analytics.total_payments);
        println!("  Average Payment: {}",
                 format!("${:.2}", avg_payment).yellow());
    }

    println!();
    Ok(())
}

fn display_monthly_breakdown(
    analytics: &analytics::DividendAnalytics,
    year: Option<i32>,
) -> Result<()> {
    if analytics.monthly_breakdown.is_empty() {
        return Ok(());
    }

    println!("{}", "📅 Monthly Breakdown".blue().bold());

    let current_year = chrono::Local::now().year();
    let display_year = year.unwrap_or(current_year);
    println!("  Year: {}", display_year.to_string().cyan());
    println!();

    let mut builder = Builder::new();
    builder.push_record(vec![
        "Month".bold().to_string(),
        "Total".bold().to_string(),
        "Payments".bold().to_string(),
        "Stocks".bold().to_string(),
        "Top Stock".bold().to_string(),
        "Top Amount".bold().to_string(),
    ]);

    let mut months: Vec<_> = analytics.monthly_breakdown.keys().collect();
    months.sort();

    for month in months {
        let summary = &analytics.monthly_breakdown[month];
        let month_name = match *month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => return Err(anyhow::anyhow!("Invalid month: {}", month)),
        }.to_string();

        builder.push_record(vec![
            month_name,
            format!("${:.2}", summary.total_amount),
            summary.payment_count.to_string(),
            summary.unique_symbols.to_string(),
            summary.top_symbol.as_deref().unwrap_or("-").to_string(),
            if summary.top_amount > rust_decimal::Decimal::ZERO {
                format!("${:.2}", summary.top_amount)
            } else {
                "-".to_string()
            },
        ]);
    }

    let mut table = builder.build();
    table.with(Style::rounded());
    println!("{}", table);
    println!();

    Ok(())
}

fn display_quarterly_breakdown(
    analytics: &analytics::DividendAnalytics,
) -> Result<()> {
    if analytics.quarterly_breakdown.is_empty() {
        return Ok(());
    }

    println!("{}", "📈 Quarterly Breakdown".blue().bold());
    println!();

    let mut builder = Builder::new();
    builder.push_record(vec![
        "Quarter".bold().to_string(),
        "Total".bold().to_string(),
        "Payments".bold().to_string(),
        "Stocks".bold().to_string(),
    ]);

    let mut quarters: Vec<_> = analytics.quarterly_breakdown.keys().collect();
    quarters.sort();

    for quarter in quarters {
        let summary = &analytics.quarterly_breakdown[quarter];
        builder.push_record(vec![
            quarter.clone(),
            format!("${:.2}", summary.total_amount),
            summary.payment_count.to_string(),
            summary.unique_symbols.to_string(),
        ]);
    }

    let mut table = builder.build();
    table.with(Style::rounded());
    println!("{}", table);
    println!();

    Ok(())
}

fn display_top_payers(
    analytics: &analytics::DividendAnalytics,
    limit: usize,
) -> Result<()> {
    if analytics.top_payers.is_empty() {
        return Ok(());
    }

    println!("{}", format!("🏆 Top {} Dividend Payers", limit).blue().bold());
    println!();

    let mut builder = Builder::new();
    builder.push_record(vec![
        "Rank".bold().to_string(),
        "Symbol".bold().to_string(),
        "Total".bold().to_string(),
        "Payments".bold().to_string(),
        "Avg/Payment".bold().to_string(),
        "First Payment".bold().to_string(),
        "Latest Payment".bold().to_string(),
    ]);

    for (i, payer) in analytics.top_payers.iter().take(limit).enumerate() {
        builder.push_record(vec![
            (i + 1).to_string(),
            payer.symbol.clone(),
            format!("${:.2}", payer.total_amount),
            payer.payment_count.to_string(),
            format!("${:.2}", payer.average_amount),
            payer.first_payment.format("%Y-%m-%d").to_string(),
            payer.last_payment.format("%Y-%m-%d").to_string(),
        ]);
    }

    let mut table = builder.build();
    table.with(Style::rounded());
    println!("{}", table);
    println!();

    Ok(())
}

fn display_growth_analysis(
    analytics: &analytics::DividendAnalytics,
) -> Result<()> {
    if let Some(growth) = &analytics.growth_analysis {
        println!("{}", "📈 Year-over-Year Growth Analysis".blue().bold());
        println!();

        let mut builder = Builder::new();
        builder.push_record(vec![
            "Year".bold().to_string(),
            "Total".bold().to_string(),
            "Payments".bold().to_string(),
            "Growth Rate".bold().to_string(),
        ]);

        for yearly in &growth.year_over_year {
            let growth_display = if let Some(rate) = yearly.growth_rate {
                if rate >= rust_decimal::Decimal::ZERO {
                    format!("+{:.1}%", rate).green().to_string()
                } else {
                    format!("{:.1}%", rate).red().to_string()
                }
            } else {
                "-".to_string()
            };

            builder.push_record(vec![
                yearly.year.to_string(),
                format!("${:.2}", yearly.total_dividends),
                yearly.payment_count.to_string(),
                growth_display,
            ]);
        }

        let mut table = builder.build();
        table.with(Style::rounded());
        println!("{}", table);

        println!("  Total Growth Rate: {}",
                 if growth.total_growth_rate >= rust_decimal::Decimal::ZERO {
                     format!("+{:.1}%", growth.total_growth_rate).green()
                 } else {
                     format!("{:.1}%", growth.total_growth_rate).red()
                 });

        println!("  Average Annual Growth: {}",
                 if growth.average_annual_growth >= rust_decimal::Decimal::ZERO {
                     format!("+{:.1}%", growth.average_annual_growth).green()
                 } else {
                     format!("{:.1}%", growth.average_annual_growth).red()
                 });

        if let Some(best) = &growth.best_year {
            println!("  Best Year: {} with {:.1}% growth",
                     best.year.to_string().cyan(),
                     best.growth_rate.unwrap_or_default());
        }

        if let Some(worst) = &growth.worst_year {
            println!("  Worst Year: {} with {:.1}% growth",
                     worst.year.to_string().cyan(),
                     worst.growth_rate.unwrap_or_default());
        }

        println!();
    } else {
        println!("{}", "📈 Growth Analysis: Insufficient data (need 2+ years)".yellow());
        println!();
    }

    Ok(())
}

fn display_frequency_analysis(
    analytics: &analytics::DividendAnalytics,
) -> Result<()> {
    let freq = &analytics.frequency_analysis;
    println!("{}", "⏰ Dividend Frequency Analysis".blue().bold());
    println!();

    if !freq.monthly_payers.is_empty() {
        println!("  {} {}: {}",
                 "Monthly Payers".green().bold(),
                 format!("({})", freq.monthly_payers.len()).dimmed(),
                 freq.monthly_payers.join(", "));
    }

    if !freq.quarterly_payers.is_empty() {
        println!("  {} {}: {}",
                 "Quarterly Payers".green().bold(),
                 format!("({})", freq.quarterly_payers.len()).dimmed(),
                 freq.quarterly_payers.join(", "));
    }

    if !freq.semi_annual_payers.is_empty() {
        println!("  {} {}: {}",
                 "Semi-Annual Payers".green().bold(),
                 format!("({})", freq.semi_annual_payers.len()).dimmed(),
                 freq.semi_annual_payers.join(", "));
    }

    if !freq.annual_payers.is_empty() {
        println!("  {} {}: {}",
                 "Annual Payers".green().bold(),
                 format!("({})", freq.annual_payers.len()).dimmed(),
                 freq.annual_payers.join(", "));
    }

    if !freq.irregular_payers.is_empty() {
        println!("  {} {}: {}",
                 "Irregular Payers".yellow().bold(),
                 format!("({})", freq.irregular_payers.len()).dimmed(),
                 freq.irregular_payers.join(", "));
    }

    println!();
    Ok(())
}

fn display_consistency_analysis(
    analytics: &analytics::DividendAnalytics,
) -> Result<()> {
    let consistency = &analytics.consistency_analysis;
    println!("{}", "🎯 Dividend Consistency Analysis".blue().bold());
    println!();

    println!("  Portfolio Consistency Score: {:.1}%",
             consistency.average_consistency_score.to_string().cyan());
    println!();

    if !consistency.consistent_payers.is_empty() {
        println!("  {} {} 🌟",
                 "Consistent Payers".green().bold(),
                 format!("({})", consistency.consistent_payers.len()).dimmed());

        let mut builder = Builder::new();
        builder.push_record(vec![
            "Symbol".bold().to_string(),
            "Score".bold().to_string(),
            "Frequency".bold().to_string(),
        ]);

        for payer in &consistency.consistent_payers {
            let score_color = if payer.consistency_score >= 90.0 {
                format!("{:.1}%", payer.consistency_score).green()
            } else if payer.consistency_score >= 80.0 {
                format!("{:.1}%", payer.consistency_score).yellow()
            } else {
                format!("{:.1}%", payer.consistency_score).normal()
            };

            builder.push_record(vec![
                payer.symbol.clone(),
                score_color.to_string(),
                payer.expected_frequency.clone(),
            ]);
        }

        let mut table = builder.build();
        table.with(Style::rounded());
        println!("{}", table);
    }

    if !consistency.inconsistent_payers.is_empty() {
        println!("  {} {}: {}",
                 "Inconsistent Payers".red().bold(),
                 format!("({})", consistency.inconsistent_payers.len()).dimmed(),
                 consistency.inconsistent_payers.join(", "));
    }

    println!();
    Ok(())
}

fn display_yield_analysis(
    analytics: &analytics::DividendAnalytics,
) -> Result<()> {
    if let Some(yields) = &analytics.yield_analysis {
        println!("{}", "💰 Dividend Yield Analysis".blue().bold());
        println!();

        println!("  Portfolio Average Yield: {:.2}%", yields.average_yield);
        println!();

        if !yields.stock_yields.is_empty() {
            let mut builder = Builder::new();
            builder.push_record(vec![
                "Symbol".bold().to_string(),
                "Annual Dividend".bold().to_string(),
                "Cost Basis".bold().to_string(),
                "Shares".bold().to_string(),
                "Yield %".bold().to_string(),
            ]);

            for stock_yield in &yields.stock_yields {
                let yield_color = if stock_yield.yield_percent >= rust_decimal::Decimal::from(5) {
                    format!("{:.2}%", stock_yield.yield_percent).green()
                } else if stock_yield.yield_percent >= rust_decimal::Decimal::from(3) {
                    format!("{:.2}%", stock_yield.yield_percent).yellow()
                } else {
                    format!("{:.2}%", stock_yield.yield_percent).normal()
                };

                builder.push_record(vec![
                    stock_yield.symbol.clone(),
                    format!("${:.2}", stock_yield.annual_dividend),
                    format!("${:.2}", stock_yield.cost_basis),
                    stock_yield.shares.to_string(),
                    yield_color.to_string(),
                ]);
            }

            let mut table = builder.build();
            table.with(Style::rounded());
            println!("{}", table);

            if let Some(highest) = &yields.highest_yielding {
                println!("  Highest Yielding: {} at {:.2}%",
                         highest.symbol.cyan(),
                         highest.yield_percent);
            }

            if let Some(lowest) = &yields.lowest_yielding {
                println!("  Lowest Yielding: {} at {:.2}%",
                         lowest.symbol.cyan(),
                         lowest.yield_percent);
            }
        }

        println!();
    } else {
        println!("{}", "💰 Yield Analysis: No holdings with cost basis found".yellow());
        println!("   Add holdings with cost basis using 'holdings add' command");
        println!();
    }

    Ok(())
}

/// Handle adding a new dividend record
fn handle_add_command(
    symbol: String,
    ex_date: String,
    pay_date: String,
    amount: String,
    shares: String,
    force: bool,
) -> Result<()> {
    use crate::models::{Dividend, DividendType};

    println!("{}", "Adding dividend record...".green().bold());

    // Parse and validate inputs
    let ex_date_parsed = parse_dividend_date(&ex_date)?;
    let pay_date_parsed = parse_dividend_date(&pay_date)?;

    let amount_decimal = Decimal::from_str(&amount).map_err(|_| {
        anyhow!(
            "Invalid amount format: {}. Use decimal format like 0.94",
            amount
        )
    })?;

    let shares_decimal = Decimal::from_str(&shares).map_err(|_| {
        anyhow!(
            "Invalid shares format: {}. Use decimal format like 100",
            shares
        )
    })?;

    // Load persistence manager and existing data
    let persistence = PersistenceManager::new()?;
    let mut tracker = persistence.load()?;

    // Check for duplicates unless force flag is used
    if !force && tracker.has_duplicate(&symbol, ex_date_parsed) {
        if let Some(existing) = tracker.find_duplicate(&symbol, ex_date_parsed) {
            println!("{} Duplicate dividend found!", "⚠".yellow());
            println!("  Symbol: {}", existing.symbol.cyan());
            println!(
                "  Ex-date: {}",
                existing.ex_date.format("%Y-%m-%d").to_string().blue()
            );
            println!("  Amount: ${:.4} per share", existing.amount_per_share);
            println!("  Total: ${:.2}", existing.total_amount);
            println!();
            println!(
                "Use {} to override duplicate protection.",
                "--force".yellow()
            );
            return Err(anyhow!(
                "Duplicate dividend exists for {} on {}",
                symbol,
                ex_date_parsed
            ));
        }
    }

    // Validate against holdings if available
    if let Some(holding) = tracker.holdings.get(&symbol.trim().to_uppercase()) {
        println!("📊 Validating against holdings for {}...", symbol.cyan());
        println!("  Holdings: {} shares", holding.shares);

        if shares_decimal > holding.shares {
            println!(
                "{} Warning: Dividend shares ({}) exceed current holdings ({})",
                "⚠".yellow(),
                shares_decimal,
                holding.shares
            );
            println!("  This may indicate a stock split or updated holdings needed.");
        }
    } else {
        println!(
            "{} No holdings found for {}. Consider adding holdings first with 'holdings add'",
            "ℹ".blue(),
            symbol.cyan()
        );
    }

    // Create dividend record
    let dividend = Dividend::new(
        symbol.clone(),
        None, // company_name
        ex_date_parsed,
        pay_date_parsed,
        amount_decimal,
        shares_decimal,
        DividendType::Regular,
    )?;

    // Display dividend details for confirmation
    println!();
    println!("{}", "💰 Dividend Details".green().bold());
    println!("  Symbol: {}", dividend.symbol.cyan());
    println!(
        "  Ex-date: {}",
        dividend.ex_date.format("%Y-%m-%d").to_string().blue()
    );
    println!(
        "  Pay-date: {}",
        dividend.pay_date.format("%Y-%m-%d").to_string().blue()
    );
    println!("  Amount per share: ${:.4}", dividend.amount_per_share);
    println!("  Shares owned: {}", dividend.shares_owned);
    println!(
        "  Total dividend: ${:.2}",
        dividend.total_amount.to_string().green()
    );

    // Add to tracker and save
    tracker.add_dividend(dividend);
    persistence.save(&tracker)?;

    println!();
    println!("{} Dividend record added successfully!", "✓".green());

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

/// Parse natural language date strings like "tomorrow", "next friday", or standard YYYY-MM-DD format
fn parse_dividend_date(date_str: &str) -> Result<NaiveDate> {
    let date_str = date_str.trim().to_lowercase();
    let today = Local::now().naive_local().date();

    match date_str.as_str() {
        "today" => Ok(today),
        "tomorrow" => Ok(today + Duration::days(1)),
        "yesterday" => Ok(today - Duration::days(1)),
        "next monday" => Ok(next_weekday(today, Weekday::Mon)),
        "next tuesday" => Ok(next_weekday(today, Weekday::Tue)),
        "next wednesday" => Ok(next_weekday(today, Weekday::Wed)),
        "next thursday" => Ok(next_weekday(today, Weekday::Thu)),
        "next friday" => Ok(next_weekday(today, Weekday::Fri)),
        "next saturday" => Ok(next_weekday(today, Weekday::Sat)),
        "next sunday" => Ok(next_weekday(today, Weekday::Sun)),
        _ => {
            // Try to parse as standard date format (YYYY-MM-DD)
            NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .map_err(|_| anyhow!("Invalid date format. Use YYYY-MM-DD or natural language like 'tomorrow', 'next friday'"))
        }
    }
}

/// Get the next occurrence of a specific weekday
fn next_weekday(from_date: NaiveDate, target_weekday: Weekday) -> NaiveDate {
    let current_weekday = from_date.weekday();
    let days_until_target = (target_weekday.num_days_from_monday() as i64 + 7
        - current_weekday.num_days_from_monday() as i64)
        % 7;
    let days_to_add = if days_until_target == 0 {
        7
    } else {
        days_until_target
    };
    from_date + Duration::days(days_to_add)
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
fn handle_data_command(command: DataCommands, config: &CliConfig) -> Result<()> {
    match command {
        DataCommands::Export {
            format,
            output,
            data_type,
        } => {
            config.print_verbose("Creating persistence manager for data export");
            let persistence = config.create_persistence_manager()?;

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
                        println!(
                            "  Dividends: {}",
                            dividends_path.display().to_string().cyan()
                        );
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
            config.print_verbose("Loading data statistics");
            let persistence = config.create_persistence_manager()?;
            let stats = persistence.get_stats()?;

            config.print(&format!("{}", "Data Statistics".green().bold()));
            if !config.quiet {
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
        }
        DataCommands::Backup => {
            config.print("Creating manual backup...");
            config.print_verbose("Initializing persistence manager for backup");
            let persistence = config.create_persistence_manager()?;

            // Load and save to force a backup
            config.print_verbose("Loading current data");
            let tracker = persistence.load()?;
            config.print_verbose("Saving data to create backup");
            persistence.save(&tracker)?;

            config.print_success("Manual backup created successfully!");
        }
        DataCommands::Load { file } => {
            config.print(&format!("{}", "Load functionality not yet implemented.".yellow()));
            config.print(&format!("Would load data from: {}", file.cyan()));
            config.print("This feature will be added in a future update.");
        }
    }

    Ok(())
}

/// Handle dividend projection command
fn handle_project_command(
    method: String,
    growth_rate: String,
    year: Option<i32>,
    export_csv: Option<String>,
    export_json: Option<String>,
    monthly: bool,
) -> Result<()> {
    use crate::projections::*;

    println!("{}", "Dividend Income Projections".green().bold());
    println!();

    // Load persistence manager and existing data
    let persistence = PersistenceManager::new()?;
    let tracker = persistence.load()?;

    if tracker.holdings.is_empty() {
        println!("{}", "No holdings found. Add holdings first to generate projections.".yellow());
        println!("Use the 'holdings add' command to add your stock positions.");
        return Ok(());
    }

    if tracker.dividends.is_empty() {
        println!("{}", "No dividend history found. Add dividend records first.".yellow());
        println!("Use the 'add' command to add historical dividend payments.");
        return Ok(());
    }

    // Parse projection method
    let projection_method = match method.as_str() {
        "last-12-months" => ProjectionMethod::Last12Months,
        "average-2-years" => ProjectionMethod::AverageYears(2),
        "average-3-years" => ProjectionMethod::AverageYears(3),
        "current-yield" => ProjectionMethod::CurrentYield,
        _ => {
            return Err(anyhow!("Invalid projection method: {}. Use: last-12-months, average-2-years, average-3-years, or current-yield", method));
        }
    };

    // Parse growth scenario
    let growth_scenario = match growth_rate.as_str() {
        "conservative" => GrowthScenario::Conservative,
        "moderate" => GrowthScenario::Moderate,
        "optimistic" => GrowthScenario::Optimistic,
        custom if custom.ends_with('%') => {
            let rate_str = custom.trim_end_matches('%');
            let rate = rate_str.parse::<f64>()
                .map_err(|_| anyhow!("Invalid custom growth rate: {}", custom))?;
            GrowthScenario::Custom(rust_decimal::Decimal::from_f64_retain(rate / 100.0)
                .ok_or_else(|| anyhow!("Invalid growth rate value"))?)
        }
        _ => {
            return Err(anyhow!("Invalid growth rate: {}. Use: conservative, moderate, optimistic, or a percentage like '7.5%'", growth_rate));
        }
    };

    // Generate projections
    let projection = ProjectionEngine::generate_projection(
        &tracker,
        projection_method,
        growth_scenario,
        year,
    )?;

    // Display basic projection summary
    display_projection_summary(&projection)?;

    // Display monthly breakdown if requested
    if monthly {
        display_monthly_projections(&projection)?;
    }

    // Display individual stock projections
    display_stock_projections(&projection)?;

    // Display metadata and confidence
    display_projection_metadata(&projection)?;

    // Export to CSV if requested
    if let Some(csv_path) = export_csv {
        ProjectionEngine::export_to_csv(&projection, &csv_path)?;
        println!();
        println!("{} Projections exported to {}",
                 "✓".green(),
                 csv_path.cyan());
    }

    // Export to JSON if requested
    if let Some(json_path) = export_json {
        ProjectionEngine::export_to_json(&projection, &json_path)?;
        println!();
        println!("{} Projections exported to {}",
                 "✓".green(),
                 json_path.cyan());
    }

    Ok(())
}

/// Display projection summary
fn display_projection_summary(projection: &projections::DividendProjection) -> Result<()> {
    println!("{}", "📊 Projection Summary".blue().bold());
    println!();

    println!("  Target Year: {}", projection.year.to_string().cyan());
    println!("  Projection Method: {}", format!("{:?}", projection.method).cyan());
    println!("  Growth Scenario: {}", projection.growth_scenario.name().cyan());
    println!();

    println!("  {} {}",
             "Projected Annual Income:".bright_blue(),
             format!("${:.2}", projection.total_projected_income).green().bold());

    // Calculate monthly average
    let monthly_average = projection.total_projected_income / rust_decimal::Decimal::from(12);
    println!("  {} {}",
             "Average Monthly Income:".bright_blue(),
             format!("${:.2}", monthly_average).yellow());

    println!();
    Ok(())
}

/// Display monthly projection breakdown
fn display_monthly_projections(projection: &projections::DividendProjection) -> Result<()> {
    println!("{}", "📅 Monthly Projected Cash Flow".blue().bold());
    println!();

    let mut builder = Builder::new();
    builder.push_record(vec![
        "Month".bold().to_string(),
        "Projected Income".bold().to_string(),
        "Payments".bold().to_string(),
        "Top Contributors".bold().to_string(),
    ]);

    for month in 1..=12 {
        if let Some(monthly) = projection.monthly_projections.get(&month) {
            let top_contributors = if monthly.top_payers.len() > 3 {
                format!("{}, +{} more",
                        monthly.top_payers[..3].join(", "),
                        monthly.top_payers.len() - 3)
            } else {
                monthly.top_payers.join(", ")
            };

            builder.push_record(vec![
                monthly.month_name.clone(),
                format!("${:.2}", monthly.projected_amount),
                monthly.payment_count.to_string(),
                top_contributors,
            ]);
        }
    }

    let mut table = builder.build();
    table.with(Style::rounded());
    println!("{}", table);
    println!();

    Ok(())
}

/// Display individual stock projections
fn display_stock_projections(projection: &projections::DividendProjection) -> Result<()> {
    if projection.stock_projections.is_empty() {
        return Ok(());
    }

    println!("{}", "📈 Individual Stock Projections".blue().bold());
    println!();

    let mut builder = Builder::new();
    builder.push_record(vec![
        "Symbol".bold().to_string(),
        "Shares".bold().to_string(),
        "Current $/Share".bold().to_string(),
        "Projected $/Share".bold().to_string(),
        "Annual Projection".bold().to_string(),
        "Frequency".bold().to_string(),
    ]);

    // Sort by projected annual dividend (highest first)
    let mut sorted_stocks = projection.stock_projections.clone();
    sorted_stocks.sort_by(|a, b| b.projected_annual_dividend.cmp(&a.projected_annual_dividend));

    for stock in &sorted_stocks {
        builder.push_record(vec![
            stock.symbol.clone(),
            stock.current_shares.to_string(),
            format!("${:.3}", stock.historical_dividend_per_share),
            format!("${:.3}", stock.projected_dividend_per_share),
            format!("${:.2}", stock.projected_annual_dividend),
            stock.payment_frequency.name().to_string(),
        ]);
    }

    let mut table = builder.build();
    table.with(Style::rounded());
    println!("{}", table);
    println!();

    Ok(())
}

/// Display projection metadata and confidence
fn display_projection_metadata(projection: &projections::DividendProjection) -> Result<()> {
    let metadata = &projection.metadata;

    println!("{}", "ℹ️ Projection Details".blue().bold());
    println!();

    println!("  {} {}",
             "Confidence Score:".bright_blue(),
             format!("{}%", metadata.confidence_score).cyan());

    println!("  {} {}",
             "Historical Data Points:".bright_blue(),
             metadata.data_points_used.to_string().cyan());

    println!("  {} {}",
             "Stocks Included:".bright_blue(),
             metadata.stocks_included.to_string().cyan());

    if !metadata.stocks_excluded.is_empty() {
        println!("  {} {} ({})",
                 "Stocks Excluded:".bright_blue(),
                 metadata.stocks_excluded.len().to_string().yellow(),
                 metadata.stocks_excluded.join(", "));
        println!("    {} {}",
                 "Reason:".dimmed(),
                 "No historical dividend data".dimmed());
    }

    if let (Some(start), Some(end)) = metadata.historical_range {
        println!("  {} {} to {}",
                 "Historical Range:".bright_blue(),
                 start.format("%Y-%m-%d").to_string().cyan(),
                 end.format("%Y-%m-%d").to_string().cyan());
    }

    println!();

    // Show confidence interpretation
    match metadata.confidence_score {
        90..=100 => println!("  {} High confidence based on comprehensive historical data",
                             "💚".green()),
        70..=89 => println!("  {} Moderate confidence - consider updating historical data",
                            "💛".yellow()),
        50..=69 => println!("  {} Low confidence - projections are estimates only",
                           "🧡".yellow()),
        _ => println!("  {} Very low confidence - add more historical data",
                     "❤️".red()),
    }

    println!();
    Ok(())
}

/// Handle tax-related commands
fn handle_tax_command(command: TaxCommands) -> Result<()> {
    

    match command {
        TaxCommands::Summary {
            year,
            estimate,
            filing_status,
            income_bracket,
            export_csv,
        } => {
            handle_tax_summary(year, estimate, filing_status, income_bracket, export_csv)?;
        }
        TaxCommands::Report {
            year,
            export_csv,
            export_json,
        } => {
            handle_tax_report(year, export_csv, export_json)?;
        }
        TaxCommands::Estimate {
            year,
            filing_status,
            income_bracket,
        } => {
            handle_tax_estimate(year, filing_status, income_bracket)?;
        }
        TaxCommands::Lots {
            year,
            symbol,
            export_csv,
        } => {
            handle_tax_lots(year, symbol, export_csv)?;
        }
        TaxCommands::Classify {
            symbol,
            classification,
            year,
            apply_future,
        } => {
            handle_tax_classify(symbol, classification, year, apply_future)?;
        }
    }
    Ok(())
}

/// Handle tax summary command
fn handle_tax_summary(
    year: Option<i32>,
    estimate: bool,
    filing_status: Option<String>,
    income_bracket: Option<String>,
    export_csv: Option<String>,
) -> Result<()> {
    use crate::tax::*;
    use chrono::Local;

    println!("{}", "Tax Summary Report".green().bold());
    println!();

    let persistence = PersistenceManager::new()?;
    let tracker = persistence.load()?;

    if tracker.dividends.is_empty() {
        println!("{}", "No dividend records found.".yellow());
        return Ok(());
    }

    let tax_year = year.unwrap_or_else(|| Local::now().year());

    // Parse tax assumptions if estimate is requested
    let tax_assumptions = if estimate {
        let filing = parse_filing_status(filing_status.as_deref())?;
        let bracket = parse_income_bracket(income_bracket.as_deref())?;
        Some(TaxAssumptions {
            filing_status: filing,
            income_bracket: bracket,
            tax_year,
        })
    } else {
        None
    };

    // Generate tax summary
    let summary = TaxAnalyzer::generate_tax_summary(&tracker, tax_year, tax_assumptions)?;

    // Display the summary
    display_tax_summary(&summary)?;

    // Export if requested
    if let Some(csv_path) = export_csv {
        TaxAnalyzer::export_tax_summary_csv(&summary, &csv_path)?;
        println!();
        println!("{} Tax summary exported to {}", "✓".green(), csv_path.cyan());
    }

    Ok(())
}

/// Handle tax report (1099-DIV style) command
fn handle_tax_report(
    year: Option<i32>,
    export_csv: Option<String>,
    export_json: Option<String>,
) -> Result<()> {
    use crate::tax::*;
    use chrono::Local;

    println!("{}", "1099-DIV Style Tax Report".green().bold());
    println!();

    let persistence = PersistenceManager::new()?;
    let tracker = persistence.load()?;

    if tracker.dividends.is_empty() {
        println!("{}", "No dividend records found.".yellow());
        return Ok(());
    }

    let tax_year = year.unwrap_or_else(|| Local::now().year());

    // Generate 1099-DIV report
    let report = TaxAnalyzer::generate_1099_div_report(&tracker, tax_year)?;

    // Display the report
    display_1099_div_report(&report)?;

    // Export if requested
    if let Some(csv_path) = export_csv {
        TaxAnalyzer::export_1099_div_csv(&report, &csv_path)?;
        println!();
        println!("{} 1099-DIV report exported to {}", "✓".green(), csv_path.cyan());
    }

    if let Some(json_path) = export_json {
        let json_str = serde_json::to_string_pretty(&report)?;
        std::fs::write(&json_path, json_str)?;
        println!();
        println!("{} 1099-DIV report exported to {}", "✓".green(), json_path.cyan());
    }

    Ok(())
}

/// Handle tax estimate command
fn handle_tax_estimate(
    year: Option<i32>,
    filing_status: String,
    income_bracket: String,
) -> Result<()> {
    use crate::tax::*;
    use chrono::Local;

    println!("{}", "Tax Estimate Calculator".green().bold());
    println!();

    let persistence = PersistenceManager::new()?;
    let tracker = persistence.load()?;

    if tracker.dividends.is_empty() {
        println!("{}", "No dividend records found.".yellow());
        return Ok(());
    }

    let tax_year = year.unwrap_or_else(|| Local::now().year());

    // Parse tax assumptions
    let filing = parse_filing_status(Some(&filing_status))?;
    let bracket = parse_income_bracket(Some(&income_bracket))?;

    let tax_assumptions = TaxAssumptions {
        filing_status: filing,
        income_bracket: bracket,
        tax_year,
    };

    // Generate tax summary with estimates
    let summary = TaxAnalyzer::generate_tax_summary(&tracker, tax_year, Some(tax_assumptions))?;

    // Display estimate-focused view
    display_tax_estimate(&summary)?;

    Ok(())
}

/// Handle tax lots command
fn handle_tax_lots(
    year: Option<i32>,
    symbol: Option<String>,
    export_csv: Option<String>,
) -> Result<()> {
    use chrono::Local;

    println!("{}", "Tax Lot Analysis".green().bold());
    println!();

    let persistence = PersistenceManager::new()?;
    let tracker = persistence.load()?;

    if tracker.dividends.is_empty() {
        println!("{}", "No dividend records found.".yellow());
        return Ok(());
    }

    let tax_year = year.unwrap_or_else(|| Local::now().year());

    // Generate tax summary to get tax lots
    let summary = crate::tax::TaxAnalyzer::generate_tax_summary(&tracker, tax_year, None)?;

    if summary.tax_lots.is_empty() {
        println!("{}", "No tax lot information found. Add tax lot IDs to dividends for detailed tracking.".yellow());
        return Ok(());
    }

    // Filter by symbol if requested
    let filtered_lots: Vec<_> = if let Some(ref sym) = symbol {
        summary.tax_lots.iter().filter(|lot| lot.symbol == *sym).collect()
    } else {
        summary.tax_lots.iter().collect()
    };

    // Display tax lots
    display_tax_lots(&filtered_lots, symbol.as_deref())?;

    // Export if requested
    if let Some(csv_path) = export_csv {
        export_tax_lots_csv(&filtered_lots, &csv_path)?;
        println!();
        println!("{} Tax lots exported to {}", "✓".green(), csv_path.cyan());
    }

    Ok(())
}

/// Handle tax classification command
fn handle_tax_classify(
    symbol: String,
    classification: String,
    year: Option<i32>,
    apply_future: bool,
) -> Result<()> {
    use crate::models::TaxClassification;

    println!("{}", "Update Tax Classification".green().bold());
    println!();

    // Parse classification
    let tax_class = match classification.to_lowercase().as_str() {
        "qualified" => TaxClassification::Qualified,
        "non-qualified" | "nonqualified" => TaxClassification::NonQualified,
        "return-of-capital" | "roc" => TaxClassification::ReturnOfCapital,
        "tax-free" | "taxfree" => TaxClassification::TaxFree,
        "foreign" => TaxClassification::Foreign,
        "unknown" => TaxClassification::Unknown,
        _ => return Err(anyhow!("Invalid classification. Use: qualified, non-qualified, return-of-capital, tax-free, foreign, unknown")),
    };

    let persistence = PersistenceManager::new()?;
    let mut tracker = persistence.load()?;

    let symbol_upper = symbol.to_uppercase();
    let mut updated_count = 0;

    // Update dividends
    for dividend in &mut tracker.dividends {
        if dividend.symbol == symbol_upper {
            let should_update = if let Some(target_year) = year {
                dividend.pay_date.year() == target_year
            } else {
                true
            };

            if should_update {
                dividend.tax_classification = tax_class.clone();
                updated_count += 1;
            }
        }
    }

    if updated_count == 0 {
        println!("{}", format!("No dividend records found for {} in the specified period.", symbol_upper).yellow());
        return Ok(());
    }

    // Save updated data
    persistence.save(&tracker)?;

    println!("{} Updated {} dividend records for {} to {:?}",
             "✓".green(),
             updated_count,
             symbol_upper.cyan(),
             tax_class);

    if apply_future {
        println!("{}", "Note: --apply-future flag noted. Future dividends will need to be manually classified.".yellow());
        println!("Consider updating your data import process to automatically classify {} dividends.", symbol_upper);
    }

    Ok(())
}

/// Display tax summary
fn display_tax_summary(summary: &crate::tax::TaxSummary) -> Result<()> {
    use tabled::{Table, Tabled};
    use colored::*;

    println!("{} {}", "📊 Tax Summary for".blue().bold(), summary.tax_year.to_string().cyan().bold());
    println!();

    // Summary totals
    println!("{}", "Total Income Breakdown".blue().bold());

    #[derive(Tabled)]
    struct IncomeSummary {
        #[tabled(rename = "Category")]
        category: String,
        #[tabled(rename = "Amount")]
        amount: String,
        #[tabled(rename = "Percentage")]
        percentage: String,
    }

    let mut income_data = vec![
        IncomeSummary {
            category: "Total Dividend Income".to_string(),
            amount: format!("${:.2}", summary.total_dividend_income),
            percentage: "100.0%".to_string(),
        },
        IncomeSummary {
            category: "  Qualified Dividends".to_string(),
            amount: format!("${:.2}", summary.qualified_dividends),
            percentage: if summary.total_dividend_income > rust_decimal::Decimal::ZERO {
                format!("{:.1}%", (summary.qualified_dividends / summary.total_dividend_income) * rust_decimal::Decimal::from(100))
            } else {
                "0.0%".to_string()
            },
        },
        IncomeSummary {
            category: "  Non-Qualified Dividends".to_string(),
            amount: format!("${:.2}", summary.non_qualified_dividends),
            percentage: if summary.total_dividend_income > rust_decimal::Decimal::ZERO {
                format!("{:.1}%", (summary.non_qualified_dividends / summary.total_dividend_income) * rust_decimal::Decimal::from(100))
            } else {
                "0.0%".to_string()
            },
        },
    ];

    if summary.return_of_capital > rust_decimal::Decimal::ZERO {
        income_data.push(IncomeSummary {
            category: "  Return of Capital".to_string(),
            amount: format!("${:.2}", summary.return_of_capital),
            percentage: if summary.total_dividend_income > rust_decimal::Decimal::ZERO {
                format!("{:.1}%", (summary.return_of_capital / summary.total_dividend_income) * rust_decimal::Decimal::from(100))
            } else {
                "0.0%".to_string()
            },
        });
    }

    if summary.tax_free_dividends > rust_decimal::Decimal::ZERO {
        income_data.push(IncomeSummary {
            category: "  Tax-Free Dividends".to_string(),
            amount: format!("${:.2}", summary.tax_free_dividends),
            percentage: if summary.total_dividend_income > rust_decimal::Decimal::ZERO {
                format!("{:.1}%", (summary.tax_free_dividends / summary.total_dividend_income) * rust_decimal::Decimal::from(100))
            } else {
                "0.0%".to_string()
            },
        });
    }

    if summary.foreign_dividends.total_foreign_income > rust_decimal::Decimal::ZERO {
        income_data.push(IncomeSummary {
            category: "  Foreign Dividends".to_string(),
            amount: format!("${:.2}", summary.foreign_dividends.total_foreign_income),
            percentage: if summary.total_dividend_income > rust_decimal::Decimal::ZERO {
                format!("{:.1}%", (summary.foreign_dividends.total_foreign_income / summary.total_dividend_income) * rust_decimal::Decimal::from(100))
            } else {
                "0.0%".to_string()
            },
        });
    }

    let table = Table::new(income_data).to_string();
    println!("{}", table);
    println!();

    // Display estimated tax if available
    if let Some(ref estimated_tax) = summary.estimated_tax {
        display_estimated_tax_section(estimated_tax)?;
    }

    // Display by-symbol breakdown if there are multiple symbols
    if summary.by_symbol.len() > 1 {
        display_symbol_breakdown(&summary.by_symbol)?;
    }

    Ok(())
}

/// Display estimated tax section
fn display_estimated_tax_section(estimated_tax: &crate::tax::EstimatedTax) -> Result<()> {
    use tabled::{Table, Tabled};
    use colored::*;

    println!("{}", "💰 Estimated Tax Liability".blue().bold());

    #[derive(Tabled)]
    struct TaxEstimate {
        #[tabled(rename = "Income Type")]
        income_type: String,
        #[tabled(rename = "Tax Rate")]
        tax_rate: String,
        #[tabled(rename = "Estimated Tax")]
        estimated_tax: String,
    }

    let tax_data = vec![
        TaxEstimate {
            income_type: "Qualified Dividends".to_string(),
            tax_rate: format!("{:.1}%", estimated_tax.capital_gains_rate * rust_decimal::Decimal::from(100)),
            estimated_tax: format!("${:.2}", estimated_tax.qualified_tax),
        },
        TaxEstimate {
            income_type: "Non-Qualified Dividends".to_string(),
            tax_rate: format!("{:.1}%", estimated_tax.ordinary_tax_bracket * rust_decimal::Decimal::from(100)),
            estimated_tax: format!("${:.2}", estimated_tax.non_qualified_tax),
        },
        TaxEstimate {
            income_type: "Total Estimated Tax".to_string(),
            tax_rate: "-".to_string(),
            estimated_tax: format!("${:.2}", estimated_tax.total_estimated_tax),
        },
    ];

    let table = Table::new(tax_data).to_string();
    println!("{}", table);

    println!();
    println!("{} Assumptions: {} filing, {} income bracket",
             "ℹ️".blue(),
             format!("{:?}", estimated_tax.tax_assumptions.filing_status).cyan(),
             format!("{:?}", estimated_tax.tax_assumptions.income_bracket).cyan());
    println!("{} Tax rates are estimates based on {} tax brackets",
             "⚠️".yellow(),
             estimated_tax.tax_assumptions.tax_year);
    println!();

    Ok(())
}

/// Display symbol breakdown
fn display_symbol_breakdown(by_symbol: &std::collections::HashMap<String, crate::tax::SymbolTaxSummary>) -> Result<()> {
    use tabled::{Table, Tabled};
    use colored::*;

    println!("{}", "📈 Breakdown by Stock Symbol".blue().bold());

    #[derive(Tabled)]
    struct SymbolRow {
        #[tabled(rename = "Symbol")]
        symbol: String,
        #[tabled(rename = "Total Income")]
        total_income: String,
        #[tabled(rename = "Qualified")]
        qualified: String,
        #[tabled(rename = "Non-Qualified")]
        non_qualified: String,
        #[tabled(rename = "Payments")]
        payments: String,
    }

    let mut symbol_data: Vec<SymbolRow> = by_symbol
        .iter()
        .map(|(symbol, summary)| SymbolRow {
            symbol: symbol.clone(),
            total_income: format!("${:.2}", summary.total_income),
            qualified: format!("${:.2}", summary.qualified_amount),
            non_qualified: format!("${:.2}", summary.non_qualified_amount),
            payments: summary.payment_count.to_string(),
        })
        .collect();

    // Sort by total income (highest first)
    symbol_data.sort_by(|a, b| {
        let a_amount: f64 = a.total_income[1..].parse().unwrap_or(0.0);
        let b_amount: f64 = b.total_income[1..].parse().unwrap_or(0.0);
        b_amount.partial_cmp(&a_amount).unwrap_or(std::cmp::Ordering::Equal)
    });

    let table = Table::new(symbol_data).to_string();
    println!("{}", table);
    println!();

    Ok(())
}

/// Display 1099-DIV report
fn display_1099_div_report(report: &crate::tax::Form1099DIV) -> Result<()> {
    use tabled::{Table, Tabled};
    use colored::*;

    println!("{} {}", "📋 1099-DIV Report for".blue().bold(), report.tax_year.to_string().cyan().bold());
    println!();

    // Summary section
    println!("{}", "Summary Totals".blue().bold());

    #[derive(Tabled)]
    struct SummaryRow {
        #[tabled(rename = "Box")]
        box_num: String,
        #[tabled(rename = "Description")]
        description: String,
        #[tabled(rename = "Amount")]
        amount: String,
    }

    let summary_data = vec![
        SummaryRow {
            box_num: "1a".to_string(),
            description: "Total Ordinary Dividends".to_string(),
            amount: format!("${:.2}", report.summary.total_ordinary_dividends),
        },
        SummaryRow {
            box_num: "1b".to_string(),
            description: "Qualified Dividends".to_string(),
            amount: format!("${:.2}", report.summary.total_qualified_dividends),
        },
        SummaryRow {
            box_num: "3".to_string(),
            description: "Non-dividend Distributions".to_string(),
            amount: format!("${:.2}", report.summary.total_non_dividend_distributions),
        },
    ];

    let table = Table::new(summary_data).to_string();
    println!("{}", table);
    println!();

    // Payer details
    if !report.payers.is_empty() {
        println!("{}", "Payer Details".blue().bold());

        #[derive(Tabled)]
        struct PayerRow {
            #[tabled(rename = "Payer")]
            payer: String,
            #[tabled(rename = "Symbol")]
            symbol: String,
            #[tabled(rename = "Box 1a")]
            box_1a: String,
            #[tabled(rename = "Box 1b")]
            box_1b: String,
            #[tabled(rename = "Box 3")]
            box_3: String,
        }

        let payer_data: Vec<PayerRow> = report.payers
            .iter()
            .map(|payer| PayerRow {
                payer: payer.payer_name.clone(),
                symbol: payer.symbols.join(", "),
                box_1a: format!("${:.2}", payer.total_ordinary_dividends),
                box_1b: format!("${:.2}", payer.qualified_dividends),
                box_3: format!("${:.2}", payer.non_dividend_distributions),
            })
            .collect();

        let table = Table::new(payer_data).to_string();
        println!("{}", table);
        println!();
    }

    println!("{} This report summarizes your dividend income in 1099-DIV format", "ℹ️".blue());
    println!("{} Use these amounts when filing your tax return", "📝".green());
    println!();

    Ok(())
}

/// Display tax estimate
fn display_tax_estimate(summary: &crate::tax::TaxSummary) -> Result<()> {
    use colored::*;

    println!("{} {}", "💰 Tax Estimate for".blue().bold(), summary.tax_year.to_string().cyan().bold());
    println!();

    if let Some(ref estimated_tax) = summary.estimated_tax {
        println!("  {} {}",
                 "Qualified Dividend Income:".bright_blue(),
                 format!("${:.2}", summary.qualified_dividends).green());
        println!("  {} {} ({})",
                 "  Estimated Tax:".dimmed(),
                 format!("${:.2}", estimated_tax.qualified_tax).yellow(),
                 format!("{:.1}% rate", estimated_tax.capital_gains_rate * rust_decimal::Decimal::from(100)).dimmed());

        println!();
        println!("  {} {}",
                 "Non-Qualified Dividend Income:".bright_blue(),
                 format!("${:.2}", summary.non_qualified_dividends).green());
        println!("  {} {} ({})",
                 "  Estimated Tax:".dimmed(),
                 format!("${:.2}", estimated_tax.non_qualified_tax).yellow(),
                 format!("{:.1}% rate", estimated_tax.ordinary_tax_bracket * rust_decimal::Decimal::from(100)).dimmed());

        println!();
        println!("  {} {}",
                 "Total Estimated Tax:".bright_blue().bold(),
                 format!("${:.2}", estimated_tax.total_estimated_tax).red().bold());

        println!();
        println!("{} Based on {} filing status, {} income bracket",
                 "ℹ️".blue(),
                 format!("{:?}", estimated_tax.tax_assumptions.filing_status).cyan(),
                 format!("{:?}", estimated_tax.tax_assumptions.income_bracket).cyan());
        println!("{} These are estimates based on {} tax rates. Consult a tax professional for accuracy.",
                 "⚠️".yellow(),
                 estimated_tax.tax_assumptions.tax_year);
    } else {
        println!("{}", "No tax estimates available. Use --estimate flag with filing status and income bracket.".yellow());
    }

    println!();
    Ok(())
}

/// Display tax lots
fn display_tax_lots(lots: &[&crate::tax::TaxLotSummary], symbol_filter: Option<&str>) -> Result<()> {
    use tabled::{Table, Tabled};
    use colored::*;

    if lots.is_empty() {
        if let Some(symbol) = symbol_filter {
            println!("{}", format!("No tax lot information found for {}.", symbol).yellow());
        } else {
            println!("{}", "No tax lot information found.".yellow());
        }
        return Ok(());
    }

    let title = if let Some(symbol) = symbol_filter {
        format!("📊 Tax Lots for {}", symbol)
    } else {
        "📊 Tax Lot Summary".to_string()
    };

    println!("{}", title.blue().bold());
    println!();

    #[derive(Tabled)]
    struct TaxLotRow {
        #[tabled(rename = "Tax Lot ID")]
        tax_lot_id: String,
        #[tabled(rename = "Symbol")]
        symbol: String,
        #[tabled(rename = "Dividend Income")]
        dividend_income: String,
        #[tabled(rename = "Shares")]
        shares: String,
        #[tabled(rename = "Purchase Date")]
        purchase_date: String,
        #[tabled(rename = "Cost Basis/Share")]
        cost_basis: String,
    }

    let lot_data: Vec<TaxLotRow> = lots
        .iter()
        .map(|lot| TaxLotRow {
            tax_lot_id: lot.tax_lot_id.clone(),
            symbol: lot.symbol.clone(),
            dividend_income: format!("${:.2}", lot.dividend_income),
            shares: lot.shares.map(|s| s.to_string()).unwrap_or_else(|| "N/A".to_string()),
            purchase_date: lot.purchase_date.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_else(|| "N/A".to_string()),
            cost_basis: lot.cost_basis_per_share.map(|c| format!("${:.2}", c)).unwrap_or_else(|| "N/A".to_string()),
        })
        .collect();

    let table = Table::new(lot_data).to_string();
    println!("{}", table);

    println!();
    println!("{} Tax lot tracking requires additional cost basis data", "ℹ️".blue());
    println!("{} Consider adding purchase dates and cost basis information for complete tracking", "💡".yellow());
    println!();

    Ok(())
}

/// Export tax lots to CSV
fn export_tax_lots_csv(lots: &[&crate::tax::TaxLotSummary], file_path: &str) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(file_path)?;

    // Write header
    writeln!(file, "Tax Lot ID,Symbol,Dividend Income,Shares,Purchase Date,Cost Basis Per Share")?;

    // Write data
    for lot in lots {
        writeln!(
            file,
            "{},{},{},{},{},{}",
            lot.tax_lot_id,
            lot.symbol,
            lot.dividend_income,
            lot.shares.map(|s| s.to_string()).unwrap_or_else(|| "".to_string()),
            lot.purchase_date.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_else(|| "".to_string()),
            lot.cost_basis_per_share.map(|c| c.to_string()).unwrap_or_else(|| "".to_string())
        )?;
    }

    Ok(())
}

/// Parse filing status from string
fn parse_filing_status(status: Option<&str>) -> Result<crate::tax::FilingStatus> {
    use crate::tax::FilingStatus;

    match status.unwrap_or("single").to_lowercase().as_str() {
        "single" => Ok(FilingStatus::Single),
        "married-jointly" | "marriedfilingjointly" | "mfj" => Ok(FilingStatus::MarriedFilingJointly),
        "married-separately" | "marriedfilingseparately" | "mfs" => Ok(FilingStatus::MarriedFilingSeparately),
        "head-of-household" | "headofhousehold" | "hoh" => Ok(FilingStatus::HeadOfHousehold),
        _ => Err(anyhow!("Invalid filing status. Use: single, married-jointly, married-separately, head-of-household")),
    }
}

/// Parse income bracket from string
fn parse_income_bracket(bracket: Option<&str>) -> Result<crate::tax::IncomeBracket> {
    use crate::tax::IncomeBracket;

    match bracket.unwrap_or("medium").to_lowercase().as_str() {
        "low" => Ok(IncomeBracket::Low),
        "medium" | "med" => Ok(IncomeBracket::Medium),
        "high" => Ok(IncomeBracket::High),
        "very-high" | "veryhigh" | "vh" => Ok(IncomeBracket::VeryHigh),
        _ => Err(anyhow!("Invalid income bracket. Use: low, medium, high, very-high")),
    }
}
