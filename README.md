# Dividend Tracker

A command-line tool for tracking dividend payments and portfolio performance, built with Rust.

## Overview

Dividend Tracker is a CLI application that helps investors track their dividend income, analyze portfolio performance, and manage dividend-paying securities. The tool focuses on accuracy and ease of use, with precise decimal calculations to handle financial data properly.

## Features

- **Add dividend records** with stock symbol, amount, date, and share count
- **List and filter** dividend payments by symbol or year
- **Portfolio summaries** with performance statistics
- **CSV import/export** for data management and backup
- **Precise financial calculations** using rust_decimal to avoid floating-point errors
- **Colorized terminal output** for better readability

## Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))

### Build from source

```bash
git clone https://github.com/your-username/dividend-tracker.git
cd dividend-tracker
cargo build --release
```

The binary will be available at `target/release/dividend-tracker`.

## Usage

### Adding dividend records

```bash
# Add a dividend payment
dividend-tracker add AAPL --amount 0.94 --date 2024-02-15 --shares 100

# Quick add (current date will be used)
dividend-tracker add MSFT --amount 0.75 --shares 50
```

### Listing dividends

```bash
# List all dividend payments
dividend-tracker list

# Filter by stock symbol
dividend-tracker list --symbol AAPL

# Filter by year
dividend-tracker list --year 2024
```

### Portfolio summary

```bash
# Current year summary
dividend-tracker summary

# Specific year summary
dividend-tracker summary --year 2023
```

### Data management

```bash
# Import from CSV
dividend-tracker import dividends.csv

# Export to CSV
dividend-tracker export --output my-dividends.csv
```

## CSV Format

The expected CSV format for imports:

```csv
symbol,amount,date,shares
AAPL,0.94,2024-02-15,100
MSFT,0.75,2024-01-18,50
```

## Project Structure

```
dividend-tracker/
├── src/           # Source code
├── tests/         # Integration tests
├── data/          # Sample data files
├── docs/          # Documentation
├── examples/      # Usage examples
└── Cargo.toml     # Project configuration
```

## Development

### Running tests

```bash
cargo test
```

### Formatting code

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Dependencies

- **clap**: Command-line argument parsing
- **serde**: Data serialization
- **chrono**: Date and time handling
- **anyhow**: Error handling
- **csv**: CSV file parsing
- **rust_decimal**: Precise decimal calculations
- **colored**: Terminal colors
- **tabled**: Table formatting

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Roadmap

- [ ] Data persistence with local database
- [ ] Portfolio performance metrics
- [ ] Dividend yield calculations
- [ ] Tax reporting features
- [ ] Web dashboard interface
- [ ] Stock price integration