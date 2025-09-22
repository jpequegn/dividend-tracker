use anyhow::{anyhow, Result};
use chrono::{Duration, Local, NaiveDate};
use colored::*;
use rust_decimal::Decimal;
use std::fs;
use std::path::Path;
use uuid::Uuid;

use crate::api::AlphaVantageClient;
use crate::holdings;
use crate::models::{
    AlertType, DividendAlert, DividendCalendarEntry, DividendFrequency, DividendTracker, Holding,
};

/// Data directory for storing notifications
const DATA_DIR: &str = "data";
const CALENDAR_FILE: &str = "dividend_calendar.json";
const ALERTS_FILE: &str = "dividend_alerts.json";

/// Notifications manager for dividend alerts and calendar
pub struct NotificationManager {
    /// Dividend calendar entries
    pub calendar: Vec<DividendCalendarEntry>,
    /// Active alerts
    pub alerts: Vec<DividendAlert>,
}

impl NotificationManager {
    /// Create a new notification manager
    pub fn new() -> Self {
        NotificationManager {
            calendar: Vec::new(),
            alerts: Vec::new(),
        }
    }

    /// Load notifications from disk
    pub fn load() -> Result<Self> {
        let data_dir = Path::new(DATA_DIR);
        if !data_dir.exists() {
            fs::create_dir_all(data_dir)?;
        }

        let calendar_path = data_dir.join(CALENDAR_FILE);
        let alerts_path = data_dir.join(ALERTS_FILE);

        let calendar = if calendar_path.exists() {
            let contents = fs::read_to_string(&calendar_path)?;
            serde_json::from_str(&contents)?
        } else {
            Vec::new()
        };

        let alerts = if alerts_path.exists() {
            let contents = fs::read_to_string(&alerts_path)?;
            serde_json::from_str(&contents)?
        } else {
            Vec::new()
        };

        Ok(NotificationManager { calendar, alerts })
    }

    /// Save notifications to disk
    pub fn save(&self) -> Result<()> {
        let data_dir = Path::new(DATA_DIR);
        if !data_dir.exists() {
            fs::create_dir_all(data_dir)?;
        }

        let calendar_path = data_dir.join(CALENDAR_FILE);
        let alerts_path = data_dir.join(ALERTS_FILE);

        fs::write(calendar_path, serde_json::to_string_pretty(&self.calendar)?)?;
        fs::write(alerts_path, serde_json::to_string_pretty(&self.alerts)?)?;

        Ok(())
    }

    /// Fetch upcoming dividends for portfolio holdings
    pub fn fetch_upcoming_dividends(&mut self, client: &AlphaVantageClient) -> Result<()> {
        println!(
            "{}",
            "Fetching upcoming dividend calendar...".green().bold()
        );

        // Load current holdings
        let tracker = holdings::load_holdings()?;
        if tracker.holdings.is_empty() {
            return Err(anyhow!("No holdings found. Please add holdings first."));
        }

        // Get current date and date range (next 90 days)
        let today = Local::now().naive_local().date();
        let end_date = today + Duration::days(90);

        // Clear old calendar entries
        self.calendar.clear();

        let total_symbols = tracker.holdings.len();
        let mut fetched_count = 0;

        // Fetch calendar for each holding
        for (symbol, holding) in &tracker.holdings {
            println!("Fetching calendar for {}...", symbol.cyan());

            // Fetch historical dividends to estimate upcoming ones
            match client.fetch_dividends(symbol, Some(today - Duration::days(365)), Some(today)) {
                Ok(historical) => {
                    if !historical.is_empty() {
                        // Estimate next dividend based on historical pattern
                        if let Some(estimated_entry) =
                            estimate_next_dividend(symbol, &historical, today, end_date, holding)
                        {
                            self.calendar.push(estimated_entry);
                            fetched_count += 1;
                        }
                    } else {
                        println!("  {} No historical dividend data available", "⚠".yellow());
                    }
                }
                Err(e) => {
                    println!("  {} Failed to fetch: {}", "✗".red(), e);
                }
            }

            // Rate limiting delay
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }

        // Sort calendar by ex-date
        self.calendar.sort_by(|a, b| a.ex_date.cmp(&b.ex_date));

        println!();
        println!(
            "{}",
            format!(
                "Fetched calendar for {} of {} holdings",
                fetched_count, total_symbols
            )
            .green()
        );

        self.save()?;
        Ok(())
    }

    /// Generate alerts for upcoming ex-dates
    pub fn generate_alerts(&mut self) -> Result<()> {
        // Load current holdings
        let tracker = holdings::load_holdings()?;

        // Clear old alerts
        self.alerts.clear();

        let _today = Local::now().naive_local().date();

        for entry in &self.calendar {
            if let Some(alert_type) = entry.get_alert_type() {
                // Get holding information
                let holding = tracker.holdings.get(&entry.symbol);
                let shares = holding.map(|h| h.shares);
                let estimated_income = match (entry.estimated_amount, shares) {
                    (Some(amount), Some(shares)) => Some(amount * shares),
                    _ => None,
                };

                let message = format_alert_message(&alert_type, entry, estimated_income);

                let alert = DividendAlert {
                    symbol: entry.symbol.clone(),
                    alert_type,
                    ex_date: entry.ex_date,
                    estimated_amount: entry.estimated_amount,
                    shares_owned: shares,
                    estimated_income,
                    message,
                };

                self.alerts.push(alert);
            }
        }

        self.save()?;
        Ok(())
    }

    /// Display current alerts
    pub fn show_alerts(&self) -> Result<()> {
        if self.alerts.is_empty() {
            println!("{}", "No upcoming dividend alerts.".yellow());
            return Ok(());
        }

        println!("{}", "📢 Dividend Alerts".green().bold());
        println!();

        for alert in &self.alerts {
            let icon = match alert.alert_type {
                AlertType::ExDateTomorrow => "🚨",
                AlertType::ExDateThisWeek => "⚠️",
                AlertType::ExDateThisMonth => "ℹ️",
                _ => "📌",
            };

            println!("{} {}", icon, alert.message.bright_white());

            if let Some(income) = alert.estimated_income {
                println!("   Estimated income: ${:.2}", income.to_string().green());
            }
            println!();
        }

        // Show summary
        let total_estimated_income: Decimal =
            self.alerts.iter().filter_map(|a| a.estimated_income).sum();

        if total_estimated_income > Decimal::ZERO {
            println!(
                "💰 {} ${:.2}",
                "Total estimated upcoming income:".bright_blue(),
                total_estimated_income.to_string().green()
            );
        }

        Ok(())
    }

    /// Display dividend calendar
    pub fn show_calendar(&self, days: Option<i64>) -> Result<()> {
        if self.calendar.is_empty() {
            println!("{}", "No upcoming dividends in calendar.".yellow());
            return Ok(());
        }

        let filter_days = days.unwrap_or(90);
        let _today = Local::now().naive_local().date();

        println!("{}", "📅 Dividend Calendar".green().bold());
        println!();

        let mut displayed_count = 0;

        for entry in &self.calendar {
            if entry.is_upcoming(filter_days) {
                let days_text = match entry.days_until_ex {
                    0 => "TODAY".red().bold().to_string(),
                    1 => "Tomorrow".yellow().to_string(),
                    d => format!("In {} days", d).cyan().to_string(),
                };

                println!(
                    "{} - {} - {}",
                    entry.ex_date.format("%Y-%m-%d").to_string().blue(),
                    entry.symbol.green().bold(),
                    days_text
                );

                if let Some(amount) = entry.estimated_amount {
                    let estimated_text = if entry.is_estimated {
                        " (estimated)".dimmed().to_string()
                    } else {
                        String::new()
                    };
                    println!("  Amount: ${:.4} per share{}", amount, estimated_text);
                }

                if let Some(pay_date) = entry.pay_date {
                    println!(
                        "  Pay date: {}",
                        pay_date.format("%Y-%m-%d").to_string().dimmed()
                    );
                }

                println!();
                displayed_count += 1;
            }
        }

        if displayed_count == 0 {
            println!("No dividends in the next {} days.", filter_days);
        } else {
            println!(
                "Showing {} upcoming dividend{} in the next {} days",
                displayed_count.to_string().cyan(),
                if displayed_count == 1 { "" } else { "s" },
                filter_days
            );
        }

        Ok(())
    }

    /// Export calendar to ICS format
    pub fn export_to_ics(&self, output_path: &str) -> Result<()> {
        let mut ics_content = String::new();

        // ICS header
        ics_content.push_str("BEGIN:VCALENDAR\r\n");
        ics_content.push_str("VERSION:2.0\r\n");
        ics_content.push_str("PRODID:-//Dividend Tracker//EN\r\n");
        ics_content.push_str("CALSCALE:GREGORIAN\r\n");

        // Add each calendar entry as an event
        for entry in &self.calendar {
            if entry.is_upcoming(90) {
                // Generate unique ID
                let uid = format!("{}@dividend-tracker", Uuid::new_v4());

                ics_content.push_str("BEGIN:VEVENT\r\n");
                ics_content.push_str(&format!("UID:{}\r\n", uid));

                // Set date (all-day event for ex-date)
                let ex_date_str = entry.ex_date.format("%Y%m%d").to_string();
                ics_content.push_str(&format!("DTSTART;VALUE=DATE:{}\r\n", ex_date_str));
                ics_content.push_str(&format!("DTEND;VALUE=DATE:{}\r\n", ex_date_str));

                // Event summary
                let summary = format!(
                    "{} Ex-Dividend{}",
                    entry.symbol,
                    if let Some(amt) = entry.estimated_amount {
                        format!(" (${:.4}/share)", amt)
                    } else {
                        String::new()
                    }
                );
                ics_content.push_str(&format!("SUMMARY:{}\r\n", summary));

                // Event description
                let mut description = format!("Stock: {}", entry.symbol);
                if let Some(name) = &entry.company_name {
                    description.push_str(&format!("\\nCompany: {}", name));
                }
                if let Some(amount) = entry.estimated_amount {
                    description.push_str(&format!("\\nDividend: ${:.4} per share", amount));
                    if entry.is_estimated {
                        description.push_str(" (estimated)");
                    }
                }
                if let Some(pay_date) = entry.pay_date {
                    description.push_str(&format!("\\nPay Date: {}", pay_date.format("%Y-%m-%d")));
                }
                ics_content.push_str(&format!("DESCRIPTION:{}\r\n", description));

                // Set alarm for day before ex-date
                ics_content.push_str("BEGIN:VALARM\r\n");
                ics_content.push_str("ACTION:DISPLAY\r\n");
                ics_content.push_str("TRIGGER:-P1D\r\n");
                ics_content.push_str(&format!(
                    "DESCRIPTION:Tomorrow is ex-dividend date for {}\r\n",
                    entry.symbol
                ));
                ics_content.push_str("END:VALARM\r\n");

                ics_content.push_str("END:VEVENT\r\n");
            }
        }

        ics_content.push_str("END:VCALENDAR\r\n");

        // Write to file
        fs::write(output_path, ics_content)?;

        println!(
            "{} Calendar exported to {}",
            "✓".green(),
            output_path.cyan()
        );

        Ok(())
    }
}

/// Estimate next dividend based on historical patterns
fn estimate_next_dividend(
    symbol: &str,
    historical: &[crate::api::DividendData],
    today: NaiveDate,
    end_date: NaiveDate,
    _holding: &Holding,
) -> Option<DividendCalendarEntry> {
    if historical.is_empty() {
        return None;
    }

    // Get the most recent dividend
    let most_recent = historical.iter().max_by_key(|d| d.ex_date)?;

    // Calculate average dividend amount
    let avg_amount: Decimal =
        historical.iter().map(|d| d.amount).sum::<Decimal>() / Decimal::from(historical.len());

    // Detect frequency (simplified - assumes quarterly if 3-5 dividends per year)
    let frequency = match historical.len() {
        1..=2 => DividendFrequency::SemiAnnual,
        3..=5 => DividendFrequency::Quarterly,
        11..=13 => DividendFrequency::Monthly,
        _ => DividendFrequency::Irregular,
    };

    // Estimate next ex-date based on frequency
    let days_to_add = match frequency {
        DividendFrequency::Monthly => 30,
        DividendFrequency::Quarterly => 90,
        DividendFrequency::SemiAnnual => 180,
        DividendFrequency::Annual => 365,
        DividendFrequency::Irregular => 90, // Default to quarterly
    };

    let estimated_ex_date = most_recent.ex_date + Duration::days(days_to_add);

    // Only include if within our date range
    if estimated_ex_date > today && estimated_ex_date <= end_date {
        let mut entry = DividendCalendarEntry::new(
            symbol.to_string(),
            None,
            estimated_ex_date,
            Some(estimated_ex_date + Duration::days(7)), // Estimate pay date as 7 days after ex
            Some(avg_amount),
            true, // This is an estimate
        );
        entry.frequency = Some(frequency);
        Some(entry)
    } else {
        None
    }
}

/// Format alert message based on type
fn format_alert_message(
    alert_type: &AlertType,
    entry: &DividendCalendarEntry,
    estimated_income: Option<Decimal>,
) -> String {
    let base_msg = match alert_type {
        AlertType::ExDateTomorrow => format!(
            "{} goes ex-dividend TOMORROW ({})",
            entry.symbol,
            entry.ex_date.format("%Y-%m-%d")
        ),
        AlertType::ExDateThisWeek => format!(
            "{} goes ex-dividend in {} days ({})",
            entry.symbol,
            entry.days_until_ex,
            entry.ex_date.format("%Y-%m-%d")
        ),
        AlertType::ExDateThisMonth => format!(
            "{} has an ex-dividend date on {}",
            entry.symbol,
            entry.ex_date.format("%Y-%m-%d")
        ),
        _ => format!("{} dividend alert", entry.symbol),
    };

    if let Some(income) = estimated_income {
        format!("{} - Estimated income: ${:.2}", base_msg, income)
    } else if let Some(amount) = entry.estimated_amount {
        format!("{} - ${:.4} per share", base_msg, amount)
    } else {
        base_msg
    }
}

/// Load holdings helper function
pub fn load_holdings() -> Result<DividendTracker> {
    holdings::load_holdings()
}
