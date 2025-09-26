# Dividend Tracker Tutorial

Welcome to the comprehensive tutorial for Dividend Tracker! This guide will walk you through all the features and help you master dividend tracking and portfolio management.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Basic Dividend Tracking](#basic-dividend-tracking)
3. [Portfolio Management](#portfolio-management)
4. [Analytics and Insights](#analytics-and-insights)
5. [Tax Planning](#tax-planning)
6. [Future Projections](#future-projections)
7. [Live Data Integration](#live-data-integration)
8. [Advanced Features](#advanced-features)
9. [Data Management](#data-management)
10. [Best Practices](#best-practices)

## Getting Started

### Installation

1. **Install Rust** (if you haven't already):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Clone and build the project**:
   ```bash
   git clone https://github.com/your-username/dividend-tracker.git
   cd dividend-tracker
   cargo build --release
   ```

3. **Add to your PATH** (optional but recommended):
   ```bash
   sudo cp target/release/dividend-tracker /usr/local/bin/
   ```

### First Time Setup

Let's start with a simple example to get you familiar with the basic commands:

```bash
# Check that everything is working
dividend-tracker --help

# Add your first dividend record
dividend-tracker add AAPL --ex-date 2024-02-09 --pay-date 2024-02-15 --amount 0.24 --shares 100

# List your dividends
dividend-tracker list

# Get a basic summary
dividend-tracker summary
```

**What we just did:**
- Added a dividend record for Apple (AAPL)
- Ex-date: February 9, 2024 (the date you needed to own the stock)
- Pay-date: February 15, 2024 (when the dividend was paid)
- Amount: $0.24 per share
- Shares: 100 shares owned

## Basic Dividend Tracking

### Adding Dividend Records

The `add` command is the foundation of dividend tracking. Let's explore different ways to add dividends:

#### Standard Format
```bash
# Basic dividend entry
dividend-tracker add MSFT --ex-date 2024-01-17 --pay-date 2024-02-08 --amount 0.75 --shares 50
```

#### Natural Language Dates
The system supports natural language date input:
```bash
# Using relative dates
dividend-tracker add GOOGL --ex-date "last friday" --pay-date "next tuesday" --amount 1.20 --shares 25

# Using specific descriptions
dividend-tracker add JNJ --ex-date "2024-02-26" --pay-date "march 12" --amount 1.19 --shares 75
```

#### Handling Duplicates
By default, the system prevents duplicate entries (same symbol + ex-date). Use `--force` to override:
```bash
# This will be rejected if it already exists
dividend-tracker add AAPL --ex-date 2024-02-09 --pay-date 2024-02-15 --amount 0.24 --shares 100

# Force adding duplicates (useful for stock splits or corrections)
dividend-tracker add AAPL --ex-date 2024-02-09 --pay-date 2024-02-15 --amount 0.24 --shares 100 --force
```

### Listing and Filtering Dividends

The `list` command offers powerful filtering capabilities:

#### Basic Listing
```bash
# List all dividends
dividend-tracker list

# Filter by specific symbol
dividend-tracker list --symbol AAPL
```

#### Time-based Filters
```bash
# Show dividends from specific year
dividend-tracker list --year 2024

# Show dividends from specific month
dividend-tracker list --year 2024 --month 3

# Date range filtering
dividend-tracker list --date-start 2024-01-01 --date-end 2024-03-31
```

#### Advanced Filters
```bash
# Show only high-value dividends
dividend-tracker list --amount-min 1.00

# Show upcoming payments only
dividend-tracker list --upcoming

# Combine filters and sort
dividend-tracker list --symbol AAPL --year 2024 --sort-by amount --reverse
```

#### Sorting Options
Available sort fields: `symbol`, `ex-date`, `pay-date`, `amount`, `total`
```bash
# Sort by total dividend amount (amount × shares)
dividend-tracker list --sort-by total --reverse

# Sort by ex-date (oldest first)
dividend-tracker list --sort-by ex-date
```

### Quick Exercise
Try adding several dividend records and exploring the listing options:

```bash
# Add multiple dividends
dividend-tracker add KO --ex-date 2024-03-14 --pay-date 2024-04-01 --amount 0.485 --shares 200
dividend-tracker add PG --ex-date 2024-01-18 --pay-date 2024-02-15 --amount 0.9133 --shares 60
dividend-tracker add VTI --ex-date 2024-03-21 --pay-date 2024-03-26 --amount 1.563 --shares 300

# Explore the data
dividend-tracker list --sort-by total --reverse
dividend-tracker list --upcoming
dividend-tracker list --year 2024 --month 3
```

## Portfolio Management

### Adding Holdings

Holdings represent your stock positions and enable advanced analytics:

```bash
# Add holdings with cost basis and yield information
dividend-tracker holdings add AAPL --shares 100 --cost-basis 175.50 --yield-pct 0.52
dividend-tracker holdings add MSFT --shares 50 --cost-basis 320.00 --yield-pct 0.72
dividend-tracker holdings add GOOGL --shares 25 --cost-basis 2650.00 --yield-pct 1.20
```

**Understanding the parameters:**
- `--shares`: Number of shares you own
- `--cost-basis`: Your average purchase price per share
- `--yield-pct`: Current dividend yield percentage

### Managing Holdings

```bash
# List all holdings
dividend-tracker holdings list

# Sort holdings by value
dividend-tracker holdings list --sort-by value

# Get portfolio summary
dividend-tracker holdings summary --include-yield

# Remove a holding
dividend-tracker holdings remove GOOGL

# Update an existing holding
dividend-tracker holdings add AAPL --shares 150 --cost-basis 180.00
```

### Importing Holdings from CSV

For larger portfolios, use CSV import:

```bash
# Import from the sample file
dividend-tracker holdings import examples/sample_holdings.csv
```

The CSV format should include:
```csv
symbol,shares,cost_basis,yield_pct
AAPL,100,175.50,0.52
MSFT,50,320.00,0.72
```

### Exercise: Setting Up Your Portfolio

1. **Import sample holdings**:
   ```bash
   dividend-tracker holdings import examples/sample_holdings.csv
   ```

2. **View your portfolio**:
   ```bash
   dividend-tracker holdings summary --include-yield
   ```

3. **Update a position**:
   ```bash
   dividend-tracker holdings add AAPL --shares 120 --cost-basis 170.00
   ```

## Analytics and Insights

### Basic Portfolio Summary

```bash
# Current year summary
dividend-tracker summary

# Specific year analysis
dividend-tracker summary --year 2023

# Monthly breakdown
dividend-tracker summary --monthly
```

### Advanced Analytics

Enable comprehensive analytics with the `--all` flag:

```bash
# Complete analytics package
dividend-tracker summary --all

# This is equivalent to:
dividend-tracker summary --growth --frequency --consistency --yield-analysis
```

**What each analysis shows:**
- `--growth`: Year-over-year dividend growth rates
- `--frequency`: How often each stock pays dividends
- `--consistency`: Reliability of dividend payments
- `--yield-analysis`: Yield calculations based on cost basis

### Specific Analytics

#### Growth Analysis
```bash
# See which stocks are increasing their dividends
dividend-tracker summary --growth --year 2024
```

#### Top Dividend Payers
```bash
# Show your best dividend performers
dividend-tracker summary --top-payers 10
```

#### Quarterly Analysis
```bash
# Analyze specific quarters
dividend-tracker summary --quarter Q1-2024
dividend-tracker summary --quarter Q4-2023
```

### Exporting Analytics

Save your analysis for further use:

```bash
# Export summary to CSV
dividend-tracker summary --all --export-csv annual-analysis-2024.csv

# Monthly breakdown export
dividend-tracker summary --monthly --export-csv monthly-breakdown-2024.csv
```

### Exercise: Portfolio Analysis

1. **Import sample data**:
   ```bash
   dividend-tracker import examples/sample_dividends.csv
   ```

2. **Run comprehensive analysis**:
   ```bash
   dividend-tracker summary --all --monthly
   ```

3. **Find your top performers**:
   ```bash
   dividend-tracker summary --top-payers 5
   ```

4. **Export for spreadsheet analysis**:
   ```bash
   dividend-tracker summary --all --export-csv my-portfolio-analysis.csv
   ```

## Tax Planning

Dividend Tracker includes comprehensive tax reporting features to help with tax preparation.

### Annual Tax Summary

```bash
# Generate tax summary for current year
dividend-tracker tax summary

# Specific year with estimates
dividend-tracker tax summary --year 2024 --estimate --filing-status married-jointly --income-bracket high
```

### 1099-DIV Style Reports

Generate reports similar to the 1099-DIV form you receive from brokers:

```bash
# Basic tax report
dividend-tracker tax report --year 2024

# Export to CSV for tax software
dividend-tracker tax report --year 2024 --export-csv 1099-div-2024.csv

# Export to JSON for further processing
dividend-tracker tax report --year 2024 --export-json tax-data-2024.json
```

### Tax Estimates

Calculate estimated taxes on your dividend income:

```bash
# Basic estimate
dividend-tracker tax estimate --year 2024

# Detailed estimate with specific parameters
dividend-tracker tax estimate --year 2024 --filing-status married-jointly --income-bracket high
```

**Filing Status Options:**
- `single`
- `married-jointly`
- `married-separately`
- `head-of-household`

**Income Bracket Options:**
- `low` (0-12% tax bracket)
- `medium` (22-24% tax bracket)
- `high` (32-35% tax bracket)
- `very-high` (37% tax bracket)

### Dividend Classification

Classify dividends for tax purposes:

```bash
# Mark dividends as qualified
dividend-tracker tax classify AAPL --classification qualified

# Mark as non-qualified
dividend-tracker tax classify REIT-STOCK --classification non-qualified

# Apply to specific year only
dividend-tracker tax classify AAPL --classification qualified --year 2024

# Apply to all future dividends from this stock
dividend-tracker tax classify AAPL --classification qualified --apply-future
```

**Classification Options:**
- `qualified`: Eligible for capital gains tax rates
- `non-qualified`: Taxed as ordinary income
- `return-of-capital`: Not immediately taxable
- `tax-free`: Municipal bond dividends
- `foreign`: Foreign dividends (may qualify for tax credit)

### Tax Lot Tracking

If you have cost basis tracking enabled:

```bash
# View tax lots
dividend-tracker tax lots --year 2024

# Filter by symbol
dividend-tracker tax lots --symbol AAPL

# Export tax lot information
dividend-tracker tax lots --export-csv tax-lots-2024.csv
```

### Exercise: Tax Preparation Workflow

1. **Generate annual summary**:
   ```bash
   dividend-tracker tax summary --year 2024 --export-csv tax-summary-2024.csv
   ```

2. **Create 1099-DIV report**:
   ```bash
   dividend-tracker tax report --year 2024 --export-csv 1099-div-2024.csv
   ```

3. **Estimate tax liability**:
   ```bash
   dividend-tracker tax estimate --year 2024 --filing-status single --income-bracket medium
   ```

4. **Classify your dividends**:
   ```bash
   dividend-tracker tax classify AAPL --classification qualified --apply-future
   dividend-tracker tax classify VTI --classification qualified --apply-future
   ```

## Future Projections

Plan your financial future with dividend income projections.

### Basic Projections

```bash
# Project next year's income
dividend-tracker project

# Project with monthly breakdown
dividend-tracker project --monthly
```

### Projection Methods

Choose different methods for calculating projections:

```bash
# Based on last 12 months (default)
dividend-tracker project --method last-12-months

# Based on average of last 2 years
dividend-tracker project --method average-2-years

# Based on last year's performance
dividend-tracker project --method last-year
```

### Growth Scenarios

Model different growth scenarios:

```bash
# Conservative growth (2% annual increase)
dividend-tracker project --growth-rate conservative

# Moderate growth (5% annual increase)
dividend-tracker project --growth-rate moderate

# Optimistic growth (8% annual increase)
dividend-tracker project --growth-rate optimistic

# Custom growth rate
dividend-tracker project --growth-rate 6.5%
```

### Specific Year Projections

```bash
# Project for specific year
dividend-tracker project --year 2025

# Project multiple years with custom growth
dividend-tracker project --year 2026 --growth-rate 7.2% --monthly
```

### Exporting Projections

Save projections for financial planning:

```bash
# Export to CSV
dividend-tracker project --monthly --export-csv projections-2025.csv

# Export to JSON for analysis tools
dividend-tracker project --export-json projections-2025.json
```

### Exercise: Financial Planning

1. **Create baseline projection**:
   ```bash
   dividend-tracker project --method last-12-months --monthly
   ```

2. **Compare growth scenarios**:
   ```bash
   dividend-tracker project --growth-rate conservative --export-csv conservative-2025.csv
   dividend-tracker project --growth-rate optimistic --export-csv optimistic-2025.csv
   ```

3. **Plan for retirement**:
   ```bash
   dividend-tracker project --year 2030 --growth-rate 6% --monthly --export-csv retirement-planning.csv
   ```

## Live Data Integration

Connect to financial APIs for automatic dividend data updates.

### API Setup

1. **Get Alpha Vantage API key**:
   - Visit [Alpha Vantage](https://www.alphavantage.co/support/#api-key)
   - Sign up for a free API key

2. **Configure the application**:
   ```bash
   dividend-tracker configure --api-key YOUR_API_KEY

   # Verify configuration
   dividend-tracker configure --show
   ```

### Fetching Dividend Data

```bash
# Fetch dividend history for specific stocks
dividend-tracker fetch AAPL,MSFT,GOOGL --year 2024

# Fetch with date range
dividend-tracker fetch AAPL --from 2024-01-01 --to 2024-12-31

# Fetch for all holdings
dividend-tracker fetch --portfolio examples/portfolio_symbols.csv
```

### Updating Existing Data

```bash
# Update all symbols in your database
dividend-tracker update --all

# Update specific symbol
dividend-tracker update --symbol AAPL

# Update with recent dividends only
dividend-tracker update --since-last-fetch
```

### Exercise: Setting Up Live Data

1. **Configure API** (use demo key for testing):
   ```bash
   dividend-tracker configure --api-key demo
   ```

2. **Fetch historical data**:
   ```bash
   dividend-tracker fetch AAPL,MSFT --year 2024
   ```

3. **Set up regular updates**:
   ```bash
   dividend-tracker update --all
   ```

## Advanced Features

### Calendar Management

Track upcoming dividend dates:

```bash
# View dividend calendar
dividend-tracker calendar

# Show next 90 days
dividend-tracker calendar --days 90

# Update calendar with latest data
dividend-tracker calendar --update

# Export to calendar application
dividend-tracker calendar --export dividend-calendar.ics
```

### Alerts and Notifications

Set up alerts for upcoming ex-dividend dates:

```bash
# Generate alerts for upcoming ex-dates
dividend-tracker alerts --generate

# Clear existing alerts
dividend-tracker alerts --clear
```

### Data Statistics

Monitor your data quality and storage:

```bash
# View comprehensive data statistics
dividend-tracker data stats
```

This will show:
- Number of dividend records
- Number of holdings
- Data quality metrics
- Storage information
- Backup status

### Exercise: Advanced Workflow

1. **Set up calendar tracking**:
   ```bash
   dividend-tracker calendar --update --days 120
   ```

2. **Generate alerts**:
   ```bash
   dividend-tracker alerts --generate
   ```

3. **Check data health**:
   ```bash
   dividend-tracker data stats
   ```

## Data Management

### Import/Export Operations

#### CSV Import
```bash
# Import dividends
dividend-tracker import examples/sample_dividends.csv

# Import holdings
dividend-tracker holdings import examples/sample_holdings.csv
```

#### Comprehensive Export
```bash
# Export all data to JSON
dividend-tracker data export --format json --data-type all --output complete-backup

# Export only dividends to CSV
dividend-tracker data export --format csv --data-type dividends --output dividends-export.csv

# Export only holdings
dividend-tracker data export --format csv --data-type holdings --output holdings-export.csv
```

### Backup and Recovery

```bash
# Create backup
dividend-tracker data backup

# Load from backup
dividend-tracker data load backup-2024-01-15.json
```

### Environment Variables

Control data storage location:

```bash
# Use custom data directory
export DIVIDEND_TRACKER_DATA_DIR="/path/to/your/data"

# Temporary directory for testing
DIVIDEND_TRACKER_DATA_DIR="/tmp/test_data" dividend-tracker list
```

### Exercise: Data Management

1. **Export your current data**:
   ```bash
   dividend-tracker data export --format json --data-type all --output my-backup-$(date +%Y-%m-%d)
   ```

2. **Create regular backup**:
   ```bash
   dividend-tracker data backup
   ```

3. **Test import/export cycle**:
   ```bash
   # Export to CSV
   dividend-tracker data export --format csv --data-type dividends --output test-export.csv

   # Clear data (be careful!)
   DIVIDEND_TRACKER_DATA_DIR="/tmp/test" dividend-tracker import test-export.csv
   DIVIDEND_TRACKER_DATA_DIR="/tmp/test" dividend-tracker list
   ```

## Best Practices

### Data Entry Best Practices

1. **Be Consistent with Symbols**:
   - Use standard ticker symbols (AAPL, not Apple)
   - Be consistent with format (BRK.B, not BRKB)

2. **Accurate Date Entry**:
   - Always use ex-dividend date, not payment date for tracking
   - Double-check dates from official sources

3. **Precise Amounts**:
   - Enter exact dividend amounts (0.24, not 0.2 or 0.25)
   - Include all digits for accuracy

### Portfolio Management Best Practices

1. **Regular Updates**:
   - Update holdings when you buy/sell stocks
   - Keep cost basis information current
   - Review and update yield information quarterly

2. **Categorization**:
   - Use consistent stock symbols
   - Classify dividends properly for tax purposes
   - Track special dividends separately if needed

### Workflow Recommendations

#### Monthly Review Process
```bash
# 1. Update with latest dividends
dividend-tracker update --all

# 2. Review last month's payments
dividend-tracker list --month $(date +%m) --year $(date +%Y)

# 3. Check upcoming payments
dividend-tracker list --upcoming

# 4. Generate monthly summary
dividend-tracker summary --monthly --export-csv monthly-$(date +%Y-%m).csv
```

#### Quarterly Analysis
```bash
# 1. Comprehensive quarterly review
dividend-tracker summary --all --quarter Q$(echo $(date +%m)/3+1 | bc)-$(date +%Y)

# 2. Tax planning check
dividend-tracker tax summary --year $(date +%Y)

# 3. Projection updates
dividend-tracker project --monthly --export-csv projections-$(date +%Y).csv
```

#### Annual Tax Preparation
```bash
# 1. Generate tax documents
dividend-tracker tax summary --year $(date +%Y) --export-csv tax-summary-$(date +%Y).csv
dividend-tracker tax report --year $(date +%Y) --export-csv 1099-div-$(date +%Y).csv

# 2. Estimate tax liability
dividend-tracker tax estimate --year $(date +%Y) --filing-status single --income-bracket medium

# 3. Full data backup
dividend-tracker data backup
dividend-tracker data export --format json --data-type all --output year-end-backup-$(date +%Y)
```

### Data Backup Strategy

1. **Regular Backups**:
   ```bash
   # Weekly backup (add to cron)
   dividend-tracker data backup
   ```

2. **Pre-operation Backups**:
   ```bash
   # Before major imports or changes
   dividend-tracker data export --format json --data-type all --output pre-import-backup-$(date +%Y-%m-%d)
   ```

3. **Annual Archives**:
   ```bash
   # End of year complete export
   dividend-tracker data export --format csv --data-type all --output annual-archive-$(date +%Y)
   ```

## Troubleshooting

### Common Issues and Solutions

1. **Command not found**:
   ```bash
   # Make sure the binary is in your PATH
   which dividend-tracker

   # If not found, copy to PATH
   sudo cp target/release/dividend-tracker /usr/local/bin/
   ```

2. **CSV Import Errors**:
   ```bash
   # Check CSV format matches expected headers
   head -1 examples/sample_dividends.csv

   # Ensure no extra spaces or special characters
   cat -A problematic.csv
   ```

3. **Date Format Issues**:
   ```bash
   # Use ISO format (YYYY-MM-DD) for reliability
   dividend-tracker add AAPL --ex-date 2024-02-09 --pay-date 2024-02-15 --amount 0.24 --shares 100
   ```

4. **API Connection Issues**:
   ```bash
   # Check API configuration
   dividend-tracker configure --show

   # Test with demo key
   dividend-tracker configure --api-key demo
   dividend-tracker fetch AAPL --year 2024
   ```

### Getting Help

- Use `--help` with any command for detailed usage information
- Check the [FAQ](FAQ.md) for common questions
- Review sample files in the `examples/` directory
- Refer to the [API documentation](API.md) for library usage

## Conclusion

You now have a comprehensive understanding of Dividend Tracker's capabilities! Start with basic dividend tracking, gradually add portfolio management features, and leverage the advanced analytics and tax reporting as your needs grow.

Remember to:
- Keep your data backed up
- Update your holdings regularly
- Review your portfolio performance monthly
- Use the tax features for annual preparation
- Take advantage of API integration for automated updates

Happy dividend tracking! 📈💰