# Dividend Tracker

A command-line tool for tracking dividend payments and portfolio performance, built with Rust.

## Overview

Dividend Tracker is a CLI application that helps investors track their dividend income, analyze portfolio performance, and manage dividend-paying securities. The tool focuses on accuracy and ease of use, with precise decimal calculations to handle financial data properly.

## Features

- **Comprehensive Dividend Tracking**: Add, list, and filter dividend records with advanced options
- **Portfolio Management**: Track holdings with cost basis, yield calculations, and performance metrics
- **Advanced Analytics**: Growth analysis, consistency tracking, and yield analysis
- **Tax Reporting**: Generate tax summaries, 1099-DIV reports, and estimated tax calculations
- **Future Projections**: Project dividend income based on historical data with multiple scenarios
- **Data Integration**: Fetch live dividend data from Alpha Vantage API
- **Multiple Export Formats**: CSV, JSON export with comprehensive data management
- **Precise Financial Calculations**: Uses rust_decimal to avoid floating-point errors
- **Calendar Integration**: Track upcoming ex-dates and payment dates
- **Colorized Terminal Output**: Enhanced readability with intuitive formatting

## Getting Started

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))

### Installation and Setup

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/your-username/dividend-tracker.git
    cd dividend-tracker
    ```

2.  **Build the project:**

    ```bash
    cargo build --release
    ```

    The binary will be available at `target/release/dividend-tracker`.

3.  **(Optional) Install the binary:**

    You can copy the binary to a directory in your `PATH` for easier access.

    ```bash
    sudo cp target/release/dividend-tracker /usr/local/bin/
    ```

## Usage

### Quick Start

Get up and running with dividend tracking in minutes:

```bash
# Add your first dividend record
dividend-tracker add AAPL --ex-date 2024-02-09 --pay-date 2024-02-15 --amount 0.24 --shares 100

# List all dividends
dividend-tracker list

# Get a summary of your dividend income
dividend-tracker summary
```

### Core Commands

#### Adding Dividend Records

The `add` command supports various date formats and validation:

```bash
# Standard dividend entry
dividend-tracker add AAPL --ex-date 2024-02-09 --pay-date 2024-02-15 --amount 0.24 --shares 100

# Using natural language dates
dividend-tracker add MSFT --ex-date "next friday" --pay-date "2024-03-15" --amount 0.75 --shares 50

# Force add duplicates (same symbol + ex-date)
dividend-tracker add GOOGL --ex-date 2024-01-15 --pay-date 2024-01-22 --amount 1.20 --shares 25 --force
```

#### Listing and Filtering Dividends

The `list` command offers powerful filtering options:

```bash
# List all dividend payments
dividend-tracker list

# Filter by stock symbol
dividend-tracker list --symbol AAPL

# Filter by year and month
dividend-tracker list --year 2024 --month 3

# Filter by date range
dividend-tracker list --date-start 2024-01-01 --date-end 2024-03-31

# Show only upcoming payments
dividend-tracker list --upcoming

# Filter by minimum amount and sort
dividend-tracker list --amount-min 1.00 --sort-by amount --reverse
```

#### Portfolio Analytics

Generate comprehensive portfolio insights:

```bash
# Basic summary for current year
dividend-tracker summary

# Detailed analytics for specific year
dividend-tracker summary --year 2023 --all

# Monthly breakdown with growth analysis
dividend-tracker summary --monthly --growth --frequency

# Top dividend payers
dividend-tracker summary --top-payers 10

# Export summary to CSV
dividend-tracker summary --export-csv annual-summary-2024.csv
```

### Advanced Features

#### Portfolio Holdings Management

Track your stock positions for enhanced analytics:

```bash
# Add holdings to your portfolio
dividend-tracker holdings add AAPL --shares 150 --cost-basis 175.50 --yield-pct 0.5

# List all holdings
dividend-tracker holdings list --sort-by value

# Portfolio summary with yield calculations
dividend-tracker holdings summary --include-yield

# Export holdings
dividend-tracker holdings export --output holdings.csv
```

#### Future Income Projections

Project dividend income using historical data:

```bash
# Basic projection for next year
dividend-tracker project

# Conservative growth scenario
dividend-tracker project --method last-12-months --growth-rate conservative

# Custom growth rate with monthly breakdown
dividend-tracker project --growth-rate 5.5% --monthly --export-csv projections-2025.csv

# Project using average of last 2 years
dividend-tracker project --method average-2-years --growth-rate optimistic
```

#### Tax Reporting

Generate tax documents and estimates:

```bash
# Annual tax summary
dividend-tracker tax summary --year 2024

# 1099-DIV style report
dividend-tracker tax report --year 2024 --export-csv tax-report-2024.csv

# Estimate taxes on dividend income
dividend-tracker tax estimate --filing-status married-jointly --income-bracket high

# Classify dividends for tax purposes
dividend-tracker tax classify AAPL --classification qualified
```

#### Live Data Integration

Fetch current dividend data from financial APIs:

```bash
# Configure API key (Alpha Vantage)
dividend-tracker configure --api-key YOUR_API_KEY

# Fetch dividend history for specific symbols
dividend-tracker fetch AAPL,MSFT,GOOGL --year 2024

# Update all holdings with recent dividends
dividend-tracker update --all

# Fetch from portfolio file
dividend-tracker fetch --portfolio holdings.csv
```

#### Calendar and Alerts

Track upcoming dividend dates:

```bash
# Display dividend calendar
dividend-tracker calendar --days 90

# Export calendar to ICS file
dividend-tracker calendar --export dividend-calendar.ics

# Generate alerts for upcoming ex-dates
dividend-tracker alerts --generate
```

### Data Management

#### Import/Export Operations

```bash
# Import dividend data from CSV
dividend-tracker import dividends.csv

# Export all data to JSON
dividend-tracker data export --format json --data-type all --output backup-2024

# Export only dividends to CSV
dividend-tracker data export --format csv --data-type dividends --output dividends-2024.csv

# Create data backup
dividend-tracker data backup

# Load from backup
dividend-tracker data load backup-2024-01-15.json
```

#### Data Statistics

```bash
# View data statistics and backup info
dividend-tracker data stats
```

## Data Formats

### Dividend CSV Format

For importing dividend records, use this format:

```csv
symbol,ex_date,pay_date,amount,shares
AAPL,2024-02-09,2024-02-15,0.24,100
MSFT,2024-01-17,2024-02-08,0.75,50
GOOGL,2024-03-11,2024-03-25,1.20,25
```

### Holdings CSV Format

For importing portfolio holdings:

```csv
symbol,shares,cost_basis,yield_pct
AAPL,150,175.50,0.5
MSFT,100,320.00,0.8
GOOGL,50,2650.00,1.2
```

### Portfolio Import Format

For bulk operations from portfolio files:

```csv
symbol,shares
AAPL,150
MSFT,100
GOOGL,50
VTI,500
```

## Configuration

### Data Storage

The application uses a structured data directory:

```
data/
├── dividends.json      # Dividend payment records
├── holdings.json       # Portfolio holdings
├── config.json         # Application configuration
└── backups/            # Automatic backups
    ├── dividends_backup_YYYY-MM-DD.json
    └── holdings_backup_YYYY-MM-DD.json
```

### Environment Variables

You can customize data storage location:

```bash
# Use custom data directory
export DIVIDEND_TRACKER_DATA_DIR="/path/to/your/data"

# Temporary data directory for testing
DIVIDEND_TRACKER_DATA_DIR="/tmp/test_data" dividend-tracker list
```

### API Configuration

Set up Alpha Vantage API for live data:

```bash
# Set API key
dividend-tracker configure --api-key YOUR_API_KEY

# View current configuration
dividend-tracker configure --show
```

## Common Workflows

### Getting Started Workflow

```bash
# 1. Add your first holdings
dividend-tracker holdings add AAPL --shares 100 --cost-basis 150.00
dividend-tracker holdings add MSFT --shares 50 --cost-basis 300.00

# 2. Add dividend records
dividend-tracker add AAPL --ex-date 2024-02-09 --pay-date 2024-02-15 --amount 0.24 --shares 100
dividend-tracker add MSFT --ex-date 2024-01-17 --pay-date 2024-02-08 --amount 0.75 --shares 50

# 3. View your portfolio
dividend-tracker holdings summary --include-yield
dividend-tracker summary --all

# 4. Project future income
dividend-tracker project --monthly
```

### Annual Tax Preparation

```bash
# 1. Generate tax summary
dividend-tracker tax summary --year 2024 --export-csv tax-summary-2024.csv

# 2. Create 1099-DIV style report
dividend-tracker tax report --year 2024 --export-csv 1099-div-2024.csv

# 3. Estimate tax liability
dividend-tracker tax estimate --year 2024 --filing-status married-jointly --income-bracket high

# 4. Export all data for records
dividend-tracker data export --format csv --data-type all --output tax-records-2024
```

### Portfolio Review Workflow

```bash
# 1. Monthly performance review
dividend-tracker summary --monthly --growth --top-payers 10

# 2. Update with latest dividends
dividend-tracker update --all

# 3. Review upcoming payments
dividend-tracker list --upcoming
dividend-tracker calendar --days 60

# 4. Annual projections
dividend-tracker project --method average-2-years --growth-rate moderate --export-csv projections.csv
```

## Development

### Running Tests

```bash
cargo test
```

### Formatting Code

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Dependencies

-   **clap**: Command-line argument parsing
-   **serde**: Data serialization
-   **chrono**: Date and time handling
-   **anyhow**: Error handling
-   **csv**: CSV file parsing
-   **rust_decimal**: Precise decimal calculations
-   **colored**: Terminal colors
-   **tabled**: Table formatting

## Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for more details.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Roadmap

-   [ ] Data persistence with a local database
-   [ ] Portfolio performance metrics
-   [ ] Dividend yield calculations
-   [ ] Tax reporting features
-   [ ] Web dashboard interface
-   [ ] Stock price integration