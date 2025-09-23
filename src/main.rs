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
            handle_data_command(command)?;
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
