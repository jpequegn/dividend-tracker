use anyhow::{Result, anyhow};
use colored::*;
use csv::{Reader, Writer};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::str::FromStr;
use tabled::{Table, Tabled};

use crate::models::{DividendTracker, Holding};

/// CSV record for holdings import/export
#[derive(Debug, Serialize, Deserialize)]
struct HoldingRecord {
    symbol: String,
    shares: String,
    cost_basis: Option<String>,
    current_yield: Option<String>,
}

/// Data directory for storing holdings
const DATA_DIR: &str = "data";
const HOLDINGS_FILE: &str = "holdings.json";

/// Get the path to the holdings data file
fn get_holdings_file_path() -> Result<std::path::PathBuf> {
    let data_dir = Path::new(DATA_DIR);
    if !data_dir.exists() {
        fs::create_dir_all(data_dir)?;
    }
    Ok(data_dir.join(HOLDINGS_FILE))
}

/// Load existing holdings from the data file
fn load_holdings() -> Result<DividendTracker> {
    let holdings_path = get_holdings_file_path()?;

    if holdings_path.exists() {
        let contents = fs::read_to_string(&holdings_path)?;
        let tracker: DividendTracker = serde_json::from_str(&contents)?;
        Ok(tracker)
    } else {
        Ok(DividendTracker::new())
    }
}

/// Save holdings to the data file
fn save_holdings(tracker: &DividendTracker) -> Result<()> {
    let holdings_path = get_holdings_file_path()?;
    let contents = serde_json::to_string_pretty(tracker)?;
    fs::write(holdings_path, contents)?;
    Ok(())
}

/// Import holdings from a CSV file
pub fn import_holdings(file_path: &str) -> Result<()> {
    println!("{}", "Importing holdings from CSV...".green().bold());

    if !Path::new(file_path).exists() {
        return Err(anyhow!("File not found: {}", file_path));
    }

    let mut tracker = load_holdings()?;
    let mut reader = Reader::from_path(file_path)?;
    let mut imported_count = 0;
    let mut updated_count = 0;

    for result in reader.deserialize() {
        let record: HoldingRecord = result?;

        let shares = Decimal::from_str(&record.shares).map_err(|_| {
            anyhow!(
                "Invalid shares value for {}: {}",
                record.symbol,
                record.shares
            )
        })?;

        let cost_basis = if let Some(cb) = record.cost_basis {
            if cb.trim().is_empty() || cb.trim() == "0" {
                None
            } else {
                Some(
                    Decimal::from_str(&cb)
                        .map_err(|_| anyhow!("Invalid cost basis for {}: {}", record.symbol, cb))?,
                )
            }
        } else {
            None
        };

        let current_yield = if let Some(cy) = record.current_yield {
            if cy.trim().is_empty() || cy.trim() == "0" {
                None
            } else {
                Some(
                    Decimal::from_str(&cy)
                        .map_err(|_| anyhow!("Invalid yield for {}: {}", record.symbol, cy))?,
                )
            }
        } else {
            None
        };

        let holding = Holding::new(record.symbol.clone(), shares, cost_basis, current_yield)?;

        let symbol_upper = record.symbol.trim().to_uppercase();
        let is_update = tracker.holdings.contains_key(&symbol_upper);

        tracker.add_holding(holding);

        if is_update {
            updated_count += 1;
            println!("  {} {} shares", "Updated".yellow(), symbol_upper.cyan());
        } else {
            imported_count += 1;
            println!("  {} {} shares", "Imported".green(), symbol_upper.cyan());
        }
    }

    save_holdings(&tracker)?;

    println!();
    println!("{}", "Import completed successfully!".green().bold());
    println!(
        "  {} new holdings imported",
        imported_count.to_string().green()
    );
    println!(
        "  {} existing holdings updated",
        updated_count.to_string().yellow()
    );

    Ok(())
}

/// Add or update a holding
pub fn add_holding(
    symbol: &str,
    shares: Decimal,
    cost_basis: Option<Decimal>,
    current_yield: Option<Decimal>,
) -> Result<()> {
    let mut tracker = load_holdings()?;
    let holding = Holding::new(symbol.to_string(), shares, cost_basis, current_yield)?;

    let symbol_upper = symbol.trim().to_uppercase();
    let is_update = tracker.holdings.contains_key(&symbol_upper);

    tracker.add_holding(holding);
    save_holdings(&tracker)?;

    if is_update {
        println!(
            "{} Updated holding for {}",
            "✓".green(),
            symbol_upper.cyan()
        );
    } else {
        println!("{} Added holding for {}", "✓".green(), symbol_upper.cyan());
    }

    println!("  Shares: {}", shares.to_string().yellow());
    if let Some(cb) = cost_basis {
        println!("  Cost Basis: ${}", cb.to_string().yellow());
    }
    if let Some(cy) = current_yield {
        println!("  Current Yield: {}%", cy.to_string().yellow());
    }

    Ok(())
}

/// Remove a holding
pub fn remove_holding(symbol: &str) -> Result<()> {
    let mut tracker = load_holdings()?;
    let symbol_upper = symbol.trim().to_uppercase();

    if tracker.holdings.remove(&symbol_upper).is_some() {
        save_holdings(&tracker)?;
        println!(
            "{} Removed holding for {}",
            "✓".green(),
            symbol_upper.cyan()
        );
    } else {
        println!(
            "{} No holding found for {}",
            "⚠".yellow(),
            symbol_upper.cyan()
        );
    }

    Ok(())
}

/// Table display structure for holdings
#[derive(Tabled)]
struct HoldingDisplay {
    #[tabled(rename = "Symbol")]
    symbol: String,
    #[tabled(rename = "Shares")]
    shares: String,
    #[tabled(rename = "Cost Basis")]
    cost_basis: String,
    #[tabled(rename = "Current Yield")]
    current_yield: String,
    #[tabled(rename = "Total Value")]
    total_value: String,
}

/// List all holdings
pub fn list_holdings(sort_by: Option<&str>, desc: bool) -> Result<()> {
    let tracker = load_holdings()?;

    if tracker.holdings.is_empty() {
        println!(
            "{}",
            "No holdings found. Use 'holdings add' to add some!".yellow()
        );
        return Ok(());
    }

    println!("{}", "Portfolio Holdings".green().bold());
    println!();

    let mut holdings: Vec<_> = tracker.holdings.values().collect();

    // Sort holdings based on the specified field
    match sort_by {
        Some("symbol") => holdings.sort_by(|a, b| a.symbol.cmp(&b.symbol)),
        Some("shares") => holdings.sort_by(|a, b| a.shares.cmp(&b.shares)),
        Some("yield") => holdings.sort_by(|a, b| match (a.current_yield, b.current_yield) {
            (Some(a_yield), Some(b_yield)) => a_yield.cmp(&b_yield),
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (None, None) => std::cmp::Ordering::Equal,
        }),
        Some("value") => holdings.sort_by(|a, b| {
            let a_value = a.avg_cost_basis.map(|cb| cb * a.shares);
            let b_value = b.avg_cost_basis.map(|cb| cb * b.shares);
            match (a_value, b_value) {
                (Some(a_val), Some(b_val)) => a_val.cmp(&b_val),
                (Some(_), None) => std::cmp::Ordering::Greater,
                (None, Some(_)) => std::cmp::Ordering::Less,
                (None, None) => std::cmp::Ordering::Equal,
            }
        }),
        _ => holdings.sort_by(|a, b| a.symbol.cmp(&b.symbol)), // Default sort by symbol
    }

    if desc {
        holdings.reverse();
    }

    let display_holdings: Vec<HoldingDisplay> = holdings
        .iter()
        .map(|h| HoldingDisplay {
            symbol: h.symbol.clone(),
            shares: h.shares.to_string(),
            cost_basis: h
                .avg_cost_basis
                .map(|cb| format!("${:.2}", cb))
                .unwrap_or_else(|| "N/A".to_string()),
            current_yield: h
                .current_yield
                .map(|cy| format!("{:.2}%", cy))
                .unwrap_or_else(|| "N/A".to_string()),
            total_value: h
                .avg_cost_basis
                .map(|cb| format!("${:.2}", cb * h.shares))
                .unwrap_or_else(|| "N/A".to_string()),
        })
        .collect();

    let table = Table::new(display_holdings);
    println!("{}", table);

    Ok(())
}

/// Export holdings to CSV
pub fn export_holdings(output_path: &str) -> Result<()> {
    let tracker = load_holdings()?;

    if tracker.holdings.is_empty() {
        println!("{}", "No holdings to export.".yellow());
        return Ok(());
    }

    let mut writer = Writer::from_path(output_path)?;

    // Write header
    writer.write_record(&["symbol", "shares", "cost_basis", "current_yield"])?;

    for holding in tracker.holdings.values() {
        let record = HoldingRecord {
            symbol: holding.symbol.clone(),
            shares: holding.shares.to_string(),
            cost_basis: holding.avg_cost_basis.map(|cb| cb.to_string()),
            current_yield: holding.current_yield.map(|cy| cy.to_string()),
        };
        writer.serialize(&record)?;
    }

    writer.flush()?;

    println!(
        "{} Holdings exported to {}",
        "✓".green(),
        output_path.cyan()
    );
    println!(
        "  Exported {} holdings",
        tracker.holdings.len().to_string().yellow()
    );

    Ok(())
}

/// Show portfolio summary
pub fn show_summary(include_yield: bool) -> Result<()> {
    let tracker = load_holdings()?;

    if tracker.holdings.is_empty() {
        println!(
            "{}",
            "No holdings found. Use 'holdings add' to add some!".yellow()
        );
        return Ok(());
    }

    println!("{}", "Portfolio Holdings Summary".green().bold());
    println!();

    let total_positions = tracker.holdings.len();
    let mut total_shares = Decimal::ZERO;
    let mut total_value = Decimal::ZERO;
    let mut positions_with_cost_basis = 0;
    let mut positions_with_yield = 0;
    let mut weighted_yield = Decimal::ZERO;

    for holding in tracker.holdings.values() {
        total_shares += holding.shares;

        if let Some(cost_basis) = holding.avg_cost_basis {
            total_value += cost_basis * holding.shares;
            positions_with_cost_basis += 1;

            if include_yield {
                if let Some(yield_pct) = holding.current_yield {
                    weighted_yield += yield_pct * (cost_basis * holding.shares);
                    positions_with_yield += 1;
                }
            }
        }
    }

    println!(
        "📊 {} {}",
        "Total Positions:".bright_blue(),
        total_positions.to_string().cyan()
    );
    println!(
        "📈 {} {}",
        "Total Shares:".bright_blue(),
        total_shares.to_string().cyan()
    );

    if positions_with_cost_basis > 0 {
        println!(
            "💰 {} ${:.2}",
            "Total Portfolio Value:".bright_blue(),
            total_value.to_string().green()
        );
        println!(
            "💼 {} {} of {}",
            "Positions with Cost Basis:".bright_blue(),
            positions_with_cost_basis.to_string().cyan(),
            total_positions.to_string().cyan()
        );
    } else {
        println!(
            "💰 {} {}",
            "Total Portfolio Value:".bright_blue(),
            "N/A (no cost basis data)".yellow()
        );
    }

    if include_yield && positions_with_yield > 0 && total_value > Decimal::ZERO {
        let avg_yield = weighted_yield / total_value;
        println!(
            "📊 {} {:.2}%",
            "Weighted Average Yield:".bright_blue(),
            avg_yield.to_string().green()
        );
        println!(
            "🎯 {} {} of {}",
            "Positions with Yield Data:".bright_blue(),
            positions_with_yield.to_string().cyan(),
            total_positions.to_string().cyan()
        );
    } else if include_yield {
        println!(
            "📊 {} {}",
            "Weighted Average Yield:".bright_blue(),
            "N/A (insufficient data)".yellow()
        );
    }

    println!();

    // Show top 5 holdings by value
    if positions_with_cost_basis > 0 {
        let mut holdings_by_value: Vec<_> = tracker
            .holdings
            .values()
            .filter(|h| h.avg_cost_basis.is_some())
            .collect();

        holdings_by_value.sort_by(|a, b| {
            let a_value = a.avg_cost_basis.unwrap() * a.shares;
            let b_value = b.avg_cost_basis.unwrap() * b.shares;
            b_value.cmp(&a_value) // Descending order
        });

        println!("{}", "🏆 Top Holdings by Value:".bright_blue());
        for (i, holding) in holdings_by_value.iter().take(5).enumerate() {
            let value = holding.avg_cost_basis.unwrap() * holding.shares;
            let percentage = (value / total_value) * rust_decimal::Decimal::from(100);
            println!(
                "  {}. {} - ${:.2} ({:.1}%)",
                (i + 1).to_string().cyan(),
                holding.symbol.green(),
                value.to_string().yellow(),
                percentage.to_string().blue()
            );
        }
    }

    Ok(())
}

/// Validate if a dividend matches any holdings (used by dividend commands)
pub fn validate_dividend_against_holdings(symbol: &str, shares: Decimal) -> Result<bool> {
    let tracker = load_holdings()?;
    let symbol_upper = symbol.trim().to_uppercase();

    if let Some(holding) = tracker.holdings.get(&symbol_upper) {
        if shares > holding.shares {
            println!(
                "{} Warning: Dividend for {} shares of {} exceeds your holding of {} shares",
                "⚠".yellow(),
                shares.to_string().red(),
                symbol_upper.cyan(),
                holding.shares.to_string().green()
            );
            return Ok(false);
        }
        Ok(true)
    } else {
        println!(
            "{} Warning: No holding found for {}. Consider adding it with 'holdings add'",
            "⚠".yellow(),
            symbol_upper.cyan()
        );
        Ok(false)
    }
}
