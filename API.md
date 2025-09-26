# API Documentation

This document provides comprehensive API documentation for using Dividend Tracker as a Rust library in your own applications.

## Table of Contents

1. [Overview](#overview)
2. [Getting Started](#getting-started)
3. [Core Data Structures](#core-data-structures)
4. [Main API](#main-api)
5. [Analytics](#analytics)
6. [Tax Reporting](#tax-reporting)
7. [Data Management](#data-management)
8. [External API Integration](#external-api-integration)
9. [Examples](#examples)
10. [Error Handling](#error-handling)

## Overview

Dividend Tracker can be used as a Rust library (`dividend-tracker-lib`) in addition to the CLI application. This allows integration into other Rust applications, web services, or custom tools.

### Features Available as Library

- **Core dividend tracking**: Add, retrieve, and manage dividend records
- **Portfolio management**: Track holdings with cost basis calculations
- **Analytics engine**: Growth analysis, yield calculations, performance metrics
- **Tax reporting**: Generate tax summaries and classifications
- **Data persistence**: JSON-based storage with backup capabilities
- **External API integration**: Alpha Vantage and other financial data providers
- **Projections**: Future income modeling with multiple scenarios

## Getting Started

### Adding to Your Project

Add this to your `Cargo.toml`:

```toml
[dependencies]
dividend-tracker = { git = "https://github.com/your-username/dividend-tracker", default-features = false }
# Or if published to crates.io:
# dividend-tracker = "0.1.0"

# Required dependencies
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
rust_decimal = { version = "1.0", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
```

### Basic Usage

```rust
use dividend_tracker::models::{DividendTracker, Dividend, Holding};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;

fn main() -> anyhow::Result<()> {
    // Create a new tracker instance
    let mut tracker = DividendTracker::new();

    // Add a dividend record
    let dividend = Dividend::new(
        "AAPL".to_string(),
        NaiveDate::from_ymd_opt(2024, 2, 9).unwrap(),
        NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
        Decimal::from_str("0.24")?,
        Decimal::from_str("100")?,
    )?;

    tracker.add_dividend(dividend)?;

    // Get total income for the year
    let total_2024 = tracker.get_total_income_for_year(2024);
    println!("Total 2024 dividend income: ${}", total_2024);

    Ok(())
}
```

## Core Data Structures

### Dividend

The core structure representing a dividend payment:

```rust
use dividend_tracker::models::{Dividend, DividendType, TaxClassification};
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dividend {
    pub symbol: String,
    pub company_name: Option<String>,
    pub ex_date: NaiveDate,
    pub pay_date: NaiveDate,
    pub amount_per_share: Decimal,
    pub shares_owned: Decimal,
    pub total_amount: Decimal,
    pub dividend_type: DividendType,
    pub tax_classification: TaxClassification,
    pub tax_lot_id: Option<String>,
    pub withholding_tax: Option<Decimal>,
}

impl Dividend {
    /// Create a new dividend record
    pub fn new(
        symbol: String,
        ex_date: NaiveDate,
        pay_date: NaiveDate,
        amount_per_share: Decimal,
        shares_owned: Decimal,
    ) -> Result<Self>;

    /// Create dividend with tax information
    pub fn new_with_tax(
        symbol: String,
        ex_date: NaiveDate,
        pay_date: NaiveDate,
        amount_per_share: Decimal,
        shares_owned: Decimal,
        tax_classification: TaxClassification,
        withholding_tax: Option<Decimal>,
    ) -> Result<Self>;

    /// Calculate total dividend amount
    pub fn calculate_total(&self) -> Decimal;

    /// Validate dividend data integrity
    pub fn validate(&self) -> Result<()>;
}
```

### Holding

Represents a stock position in your portfolio:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Holding {
    pub symbol: String,
    pub shares: Decimal,
    pub avg_cost_basis: Option<Decimal>,
    pub current_yield: Option<Decimal>,
}

impl Holding {
    /// Create a new holding
    pub fn new(symbol: String, shares: Decimal) -> Self;

    /// Create holding with cost basis
    pub fn with_cost_basis(
        symbol: String,
        shares: Decimal,
        cost_basis: Decimal
    ) -> Self;

    /// Calculate current market value (requires current price)
    pub fn market_value(&self, current_price: Decimal) -> Option<Decimal>;

    /// Calculate unrealized gain/loss
    pub fn unrealized_gain_loss(&self, current_price: Decimal) -> Option<Decimal>;
}
```

### DividendType

Enumeration for different types of dividend payments:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DividendType {
    Regular,          // Quarterly or annual dividend
    Special,          // One-time special dividend
    ReturnOfCapital,  // Return of capital distribution
    Stock,            // Stock dividend (shares)
    SpinOff,          // Spin-off distribution
}
```

### TaxClassification

Tax classification for US tax purposes:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaxClassification {
    Qualified,        // Eligible for capital gains rates
    NonQualified,     // Taxed as ordinary income
    ReturnOfCapital,  // Not taxable as income
    TaxFree,          // Tax-free (municipal bonds)
    Foreign,          // Foreign dividends
    Unknown,          // Default for existing data
}
```

## Main API

### DividendTracker

The main class for managing all dividend and portfolio data:

```rust
use dividend_tracker::models::DividendTracker;

impl DividendTracker {
    /// Create a new empty tracker
    pub fn new() -> Self;

    /// Add a dividend record
    pub fn add_dividend(&mut self, dividend: Dividend) -> Result<()>;

    /// Get all dividends for a specific symbol
    pub fn get_dividends_for_symbol(&self, symbol: &str) -> Vec<&Dividend>;

    /// Get dividends for a specific year
    pub fn get_dividends_for_year(&self, year: i32) -> Vec<&Dividend>;

    /// Get total income for a year
    pub fn get_total_income_for_year(&self, year: i32) -> Decimal;

    /// Add or update a holding
    pub fn add_holding(&mut self, holding: Holding) -> Result<()>;

    /// Remove a holding
    pub fn remove_holding(&mut self, symbol: &str) -> Result<()>;

    /// Get holding by symbol
    pub fn get_holding(&self, symbol: &str) -> Option<&Holding>;

    /// Get all holdings
    pub fn get_holdings(&self) -> &HashMap<String, Holding>;
}
```

### Filtering and Queries

Advanced querying capabilities:

```rust
use dividend_tracker::models::{DividendFilter, SortBy, SortOrder};

#[derive(Debug, Default)]
pub struct DividendFilter {
    pub symbol: Option<String>,
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub date_start: Option<NaiveDate>,
    pub date_end: Option<NaiveDate>,
    pub amount_min: Option<Decimal>,
    pub upcoming: bool,
    pub sort_by: SortBy,
    pub reverse: bool,
}

impl DividendTracker {
    /// Get filtered dividends
    pub fn get_filtered_dividends(&self, filter: &DividendFilter) -> Vec<&Dividend>;

    /// Count dividends matching filter
    pub fn count_dividends(&self, filter: &DividendFilter) -> usize;
}
```

## Analytics

### Portfolio Analytics

```rust
use dividend_tracker::analytics::{
    PortfolioAnalytics, GrowthAnalysis, YieldAnalysis, ConsistencyAnalysis
};

impl PortfolioAnalytics {
    /// Create analytics engine for a tracker
    pub fn new(tracker: &DividendTracker) -> Self;

    /// Calculate year-over-year growth
    pub fn growth_analysis(&self, years: Vec<i32>) -> Result<GrowthAnalysis>;

    /// Calculate yield metrics
    pub fn yield_analysis(&self) -> Result<YieldAnalysis>;

    /// Analyze dividend consistency
    pub fn consistency_analysis(&self) -> Result<ConsistencyAnalysis>;

    /// Get top dividend paying stocks
    pub fn top_dividend_payers(&self, limit: usize) -> Vec<TopPayer>;

    /// Calculate monthly breakdown
    pub fn monthly_breakdown(&self, year: i32) -> Result<Vec<MonthlyDividendSummary>>;

    /// Calculate quarterly breakdown
    pub fn quarterly_breakdown(&self, year: i32) -> Result<Vec<QuarterlyDividendSummary>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthAnalysis {
    pub overall_growth_rate: Decimal,
    pub year_over_year: Vec<YearOverYearGrowth>,
    pub stock_growth_rates: HashMap<String, Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldAnalysis {
    pub portfolio_yield: Decimal,
    pub stock_yields: HashMap<String, Decimal>,
    pub yield_weighted_average: Decimal,
}
```

### Performance Metrics

```rust
use dividend_tracker::analytics::PerformanceMetrics;

impl PerformanceMetrics {
    /// Calculate total return including dividends
    pub fn total_return(
        &self,
        initial_value: Decimal,
        final_value: Decimal,
        total_dividends: Decimal
    ) -> Decimal;

    /// Calculate dividend yield
    pub fn dividend_yield(
        &self,
        annual_dividends: Decimal,
        current_price: Decimal
    ) -> Decimal;

    /// Calculate compound annual growth rate (CAGR)
    pub fn cagr(
        &self,
        initial_value: Decimal,
        final_value: Decimal,
        years: Decimal
    ) -> Decimal;
}
```

## Tax Reporting

### Tax Summary Generation

```rust
use dividend_tracker::tax::{TaxReporter, TaxSummary, TaxSettings};

impl TaxReporter {
    /// Create tax reporter
    pub fn new(tracker: &DividendTracker) -> Self;

    /// Generate annual tax summary
    pub fn annual_summary(&self, year: i32) -> Result<TaxSummary>;

    /// Generate 1099-DIV style report
    pub fn generate_1099_div(&self, year: i32) -> Result<Tax1099Div>;

    /// Estimate tax liability
    pub fn estimate_taxes(
        &self,
        year: i32,
        settings: &TaxSettings
    ) -> Result<TaxEstimate>;

    /// Classify dividends by tax treatment
    pub fn classify_dividends(
        &mut self,
        symbol: &str,
        classification: TaxClassification,
        year: Option<i32>,
        apply_future: bool
    ) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxSummary {
    pub year: i32,
    pub total_dividends: Decimal,
    pub qualified_dividends: Decimal,
    pub non_qualified_dividends: Decimal,
    pub foreign_dividends: Decimal,
    pub tax_free_dividends: Decimal,
    pub total_withholding: Decimal,
    pub estimated_tax_owed: Option<Decimal>,
}

#[derive(Debug, Clone)]
pub struct TaxSettings {
    pub filing_status: FilingStatus,
    pub income_bracket: IncomeBracket,
    pub state_tax_rate: Option<Decimal>,
}
```

## Data Management

### Persistence

```rust
use dividend_tracker::persistence::PersistenceManager;

impl PersistenceManager {
    /// Create with default data directory
    pub fn new() -> Self;

    /// Create with custom data directory
    pub fn with_custom_path<P: AsRef<Path>>(path: P) -> Self;

    /// Save tracker data
    pub fn save_tracker(&self, tracker: &DividendTracker) -> Result<()>;

    /// Load tracker data
    pub fn load_tracker(&self) -> Result<DividendTracker>;

    /// Create backup
    pub fn create_backup(&self) -> Result<PathBuf>;

    /// Load from backup
    pub fn load_from_backup(&self, backup_path: &Path) -> Result<DividendTracker>;

    /// Export to CSV
    pub fn export_csv(
        &self,
        tracker: &DividendTracker,
        output_path: &Path,
        data_type: ExportType
    ) -> Result<()>;

    /// Import from CSV
    pub fn import_csv(&self, file_path: &Path) -> Result<DividendTracker>;
}

#[derive(Debug, Clone)]
pub enum ExportType {
    Dividends,
    Holdings,
    All,
}
```

### Configuration Management

```rust
use dividend_tracker::config::{Config, ApiSettings};

impl Config {
    /// Load configuration
    pub fn load() -> Result<Self>;

    /// Save configuration
    pub fn save(&self) -> Result<()>;

    /// Get API key for external services
    pub fn get_api_key(&self) -> Result<String>;

    /// Set API key
    pub fn set_api_key(&mut self, api_key: String);
}
```

## External API Integration

### Alpha Vantage Integration

```rust
use dividend_tracker::api::{AlphaVantageClient, DividendData};

impl AlphaVantageClient {
    /// Create client with API key
    pub fn new(api_key: String) -> Result<Self>;

    /// Fetch dividend data for a symbol
    pub fn fetch_dividends(
        &self,
        symbol: &str,
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>
    ) -> Result<Vec<DividendData>>;

    /// Batch fetch multiple symbols
    pub fn batch_fetch_dividends(
        &self,
        symbols: Vec<String>,
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>
    ) -> Result<HashMap<String, Vec<DividendData>>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DividendData {
    pub symbol: String,
    pub ex_date: NaiveDate,
    pub amount: Decimal,
}
```

## Examples

### Basic Portfolio Tracking

```rust
use dividend_tracker::models::{DividendTracker, Dividend, Holding};
use dividend_tracker::persistence::PersistenceManager;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;

fn main() -> anyhow::Result<()> {
    // Create tracker and persistence manager
    let mut tracker = DividendTracker::new();
    let persistence = PersistenceManager::new();

    // Add holdings
    let aapl_holding = Holding::with_cost_basis(
        "AAPL".to_string(),
        Decimal::from_str("100")?,
        Decimal::from_str("175.50")?
    );
    tracker.add_holding(aapl_holding)?;

    // Add dividend record
    let dividend = Dividend::new(
        "AAPL".to_string(),
        NaiveDate::from_ymd_opt(2024, 2, 9).unwrap(),
        NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
        Decimal::from_str("0.24")?,
        Decimal::from_str("100")?,
    )?;
    tracker.add_dividend(dividend)?;

    // Save data
    persistence.save_tracker(&tracker)?;

    println!("Portfolio saved successfully!");
    Ok(())
}
```

### Analytics and Reporting

```rust
use dividend_tracker::analytics::PortfolioAnalytics;
use dividend_tracker::tax::TaxReporter;

fn analyze_portfolio(tracker: &DividendTracker) -> anyhow::Result<()> {
    // Create analytics engine
    let analytics = PortfolioAnalytics::new(tracker);

    // Calculate growth analysis
    let growth = analytics.growth_analysis(vec![2022, 2023, 2024])?;
    println!("Overall growth rate: {}%", growth.overall_growth_rate);

    // Get top dividend payers
    let top_payers = analytics.top_dividend_payers(5);
    for payer in top_payers {
        println!("{}: ${}", payer.symbol, payer.total_amount);
    }

    // Generate tax summary
    let tax_reporter = TaxReporter::new(tracker);
    let tax_summary = tax_reporter.annual_summary(2024)?;
    println!("Total 2024 dividends: ${}", tax_summary.total_dividends);
    println!("Qualified dividends: ${}", tax_summary.qualified_dividends);

    Ok(())
}
```

### API Data Integration

```rust
use dividend_tracker::api::AlphaVantageClient;
use dividend_tracker::config::Config;

async fn fetch_live_data() -> anyhow::Result<()> {
    // Load API configuration
    let config = Config::load()?;
    let api_key = config.get_api_key()?;

    // Create API client
    let client = AlphaVantageClient::new(api_key)?;

    // Fetch dividend data
    let dividend_data = client.fetch_dividends(
        "AAPL",
        Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        None
    )?;

    // Process data
    for data in dividend_data {
        println!("{}: ${} on {}", data.symbol, data.amount, data.ex_date);
    }

    Ok(())
}
```

### Custom Filters and Queries

```rust
use dividend_tracker::models::{DividendFilter, SortBy};
use chrono::NaiveDate;
use rust_decimal::Decimal;

fn advanced_filtering(tracker: &DividendTracker) -> anyhow::Result<()> {
    // Create custom filter
    let filter = DividendFilter {
        year: Some(2024),
        amount_min: Some(Decimal::from_str("1.00")?),
        sort_by: SortBy::Total,
        reverse: true,
        ..Default::default()
    };

    // Get filtered results
    let dividends = tracker.get_filtered_dividends(&filter);

    for dividend in dividends {
        println!("{}: ${} total", dividend.symbol, dividend.total_amount);
    }

    Ok(())
}
```

## Error Handling

The library uses `anyhow::Result<T>` for error handling. Common error types:

### ValidationError
```rust
use dividend_tracker::error::ValidationError;

// Errors during data validation
pub enum ValidationError {
    InvalidSymbol(String),
    InvalidDate(String),
    InvalidAmount(String),
    FutureDateError,
    DuplicateEntry,
}
```

### PersistenceError
```rust
// File I/O and data persistence errors
pub enum PersistenceError {
    FileNotFound(PathBuf),
    PermissionDenied(PathBuf),
    CorruptedData(String),
    BackupFailed(String),
}
```

### ApiError
```rust
// External API integration errors
pub enum ApiError {
    InvalidApiKey,
    RateLimitExceeded,
    NetworkError(String),
    InvalidResponse(String),
    SymbolNotFound(String),
}
```

### Example Error Handling

```rust
use dividend_tracker::models::DividendTracker;
use anyhow::Context;

fn safe_operations() -> anyhow::Result<()> {
    let mut tracker = DividendTracker::new();

    // Handle validation errors
    let dividend = Dividend::new(
        "INVALID".to_string(),
        NaiveDate::from_ymd_opt(2024, 2, 9).unwrap(),
        NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
        Decimal::from_str("0.24")?,
        Decimal::from_str("100")?,
    ).with_context(|| "Failed to create dividend record")?;

    tracker.add_dividend(dividend)
        .with_context(|| "Failed to add dividend to tracker")?;

    Ok(())
}
```

---

This API documentation provides a comprehensive guide for integrating Dividend Tracker into your Rust applications. For more examples and advanced usage, see the [examples directory](examples/) and the [tutorial](TUTORIAL.md).