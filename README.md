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

### Adding Dividend Records

To add a new dividend payment, use the `add` command.

```bash
# Add a dividend payment with all details
dividend-tracker add AAPL --amount 0.94 --date 2024-02-15 --shares 100

# Quick add (uses the current date)
dividend-tracker add MSFT --amount 0.75 --shares 50
```

### Listing Dividends

The `list` command displays all recorded dividend payments.

```bash
# List all dividend payments
dividend-tracker list

# Filter by stock symbol
dividend-tracker list --symbol AAPL

# Filter by year
dividend-tracker list --year 2024
```

### Portfolio Summary

The `summary` command provides a performance overview of your portfolio.

```bash
# Summary for the current year
dividend-tracker summary

# Summary for a specific year
dividend-tracker summary --year 2023
```

### Data Management

You can import and export dividend data in CSV format.

```bash
# Import from a CSV file
dividend-tracker import dividends.csv

# Export to a CSV file
dividend-tracker export --output my-dividends.csv
```

## CSV Format

The expected CSV format for imports is as follows:

```csv
symbol,amount,date,shares
AAPL,0.94,2024-02-15,100
MSFT,0.75,2024-01-18,50
```

## Configuration

The application stores data in a `holdings.json` file in the `data/` directory. This file is created automatically when you add the first dividend record.

## Troubleshooting

-   **Error: "command not found: dividend-tracker"**: Make sure you have installed the binary in a directory included in your system's `PATH`.
-   **CSV import errors**: Ensure your CSV file follows the format specified in the "CSV Format" section.

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