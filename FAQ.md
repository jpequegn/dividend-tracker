# Frequently Asked Questions (FAQ)

## Table of Contents

1. [General Questions](#general-questions)
2. [Installation and Setup](#installation-and-setup)
3. [Data Entry and Management](#data-entry-and-management)
4. [Features and Functionality](#features-and-functionality)
5. [Tax and Legal Questions](#tax-and-legal-questions)
6. [Technical Issues](#technical-issues)
7. [Performance and Scaling](#performance-and-scaling)
8. [Integration and APIs](#integration-and-apis)

## General Questions

### Q: What is Dividend Tracker?
**A:** Dividend Tracker is a command-line application built in Rust that helps investors track dividend payments, manage portfolio holdings, analyze performance, and generate tax reports. It focuses on accuracy and ease of use for dividend-focused investors.

### Q: Who should use Dividend Tracker?
**A:** This tool is ideal for:
- Dividend-focused investors tracking multiple stocks
- Portfolio managers needing accurate financial calculations
- Individuals preparing tax documents
- Anyone wanting precise dividend income analysis
- Investors planning retirement income strategies

### Q: Is Dividend Tracker free?
**A:** Yes, Dividend Tracker is open-source software released under the MIT License. You can use, modify, and distribute it freely.

### Q: Does it work on all operating systems?
**A:** Yes, Dividend Tracker runs on:
- **macOS** (Intel and Apple Silicon)
- **Linux** (all major distributions)
- **Windows** (Windows 10/11)

## Installation and Setup

### Q: Do I need to know Rust to use this application?
**A:** No, you don't need Rust knowledge to use the application. However, you need Rust installed to build it from source. Alternatively, you can download pre-built binaries when available.

### Q: I'm getting "command not found" after building. What should I do?
**A:** After building with `cargo build --release`, you need to either:
1. Use the full path: `./target/release/dividend-tracker`
2. Add to PATH: `sudo cp target/release/dividend-tracker /usr/local/bin/`
3. Create an alias in your shell profile

### Q: Where is my data stored?
**A:** By default, data is stored in a `data/` directory in your current working directory:
```
data/
├── dividends.json      # Your dividend records
├── holdings.json       # Portfolio holdings
├── config.json         # Application settings
└── backups/            # Automatic backups
```

You can customize this location with the `DIVIDEND_TRACKER_DATA_DIR` environment variable.

### Q: How do I backup my data?
**A:** Multiple backup options are available:
```bash
# Automatic backup
dividend-tracker data backup

# Manual export
dividend-tracker data export --format json --data-type all --output my-backup

# Scheduled backups (add to cron)
0 2 * * 0 /usr/local/bin/dividend-tracker data backup
```

## Data Entry and Management

### Q: What's the difference between ex-date and pay-date?
**A:**
- **Ex-date**: The cutoff date to be eligible for the dividend. You must own the stock before this date.
- **Pay-date**: When the dividend is actually paid to shareholders.

For tracking purposes, the ex-date is more important as it determines eligibility.

### Q: Can I import data from my brokerage account?
**A:** Currently, you need to manually format your data into CSV files. Most brokerages allow CSV export, which you can then reformat to match our schema:

**Dividend CSV format:**
```csv
symbol,ex_date,pay_date,amount,shares
AAPL,2024-02-09,2024-02-15,0.24,100
```

**Holdings CSV format:**
```csv
symbol,shares,cost_basis,yield_pct
AAPL,100,175.50,0.52
```

### Q: How do I handle stock splits?
**A:** For stock splits, you'll need to:
1. Update your holdings with the new share count
2. Adjust the cost basis accordingly
3. Use `--force` flag when adding historical dividends that now appear different due to the split

Example:
```bash
# Before 2:1 split: 100 shares at $200
# After split: 200 shares at $100
dividend-tracker holdings add AAPL --shares 200 --cost-basis 100.00
```

### Q: Can I track REITs, ETFs, and mutual funds?
**A:** Yes! The application treats all dividend-paying securities equally. Just use their ticker symbols:
```bash
dividend-tracker add VNQ --ex-date 2024-03-20 --pay-date 2024-03-25 --amount 0.95 --shares 100  # REIT
dividend-tracker add VTI --ex-date 2024-03-21 --pay-date 2024-03-26 --amount 1.563 --shares 300 # ETF
```

### Q: How do I handle special dividends?
**A:** Special dividends can be added like regular dividends. Consider adding a note in your personal records about the special nature:
```bash
dividend-tracker add AAPL --ex-date 2024-08-15 --pay-date 2024-08-22 --amount 2.50 --shares 100
```

### Q: What if I made a mistake in data entry?
**A:** You can update records by:
1. Adding a corrected entry with `--force` flag
2. Manually editing the JSON files in the data directory (advanced users)
3. Exporting, correcting in a spreadsheet, and re-importing

## Features and Functionality

### Q: What analytics does the tool provide?
**A:** Comprehensive analytics include:
- **Growth analysis**: Year-over-year dividend growth
- **Frequency analysis**: Payment frequency patterns
- **Consistency analysis**: Reliability of dividend payments
- **Yield analysis**: Current and historical yields
- **Top payers**: Best performing stocks
- **Monthly/quarterly breakdowns**: Time-based analysis

### Q: How accurate are the financial calculations?
**A:** Very accurate! Dividend Tracker uses the `rust_decimal` library, which provides:
- Exact decimal arithmetic (no floating-point errors)
- Precision to 28 decimal places
- Consistent rounding behavior
- Financial-grade accuracy for all calculations

### Q: Can I project future dividend income?
**A:** Yes! The projection feature offers multiple methods:
- **Last 12 months**: Based on recent performance
- **Average 2 years**: Smoothed average approach
- **Last year**: Annual comparison
- **Growth scenarios**: Conservative (2%), Moderate (5%), Optimistic (8%), or custom rates

### Q: How does the tax reporting work?
**A:** Tax features include:
- **Annual summaries**: Total dividend income by year
- **1099-DIV style reports**: Similar to broker tax forms
- **Tax estimates**: Approximate tax liability calculations
- **Classification tracking**: Qualified vs. non-qualified dividends
- **Export capabilities**: CSV/JSON for tax software

### Q: Is tax advice provided?
**A:** **No**. The tool provides calculations and data organization but does not provide tax advice. Always consult with a qualified tax professional for tax planning and compliance.

## Tax and Legal Questions

### Q: Are the tax calculations legally binding?
**A:** **No**. All tax calculations are estimates for planning purposes only. The application:
- Provides mathematical calculations based on standard tax brackets
- Does not account for individual circumstances
- Cannot replace professional tax advice
- Should not be used as the sole basis for tax decisions

**Always consult a qualified tax professional.**

### Q: How do I classify dividends as qualified vs. non-qualified?
**A:** Use the tax classification feature:
```bash
# Mark dividends as qualified (most common stocks)
dividend-tracker tax classify AAPL --classification qualified --apply-future

# Mark as non-qualified (REITs, some bonds)
dividend-tracker tax classify VNQ --classification non-qualified --apply-future
```

**Note**: Classification rules are complex. Consult tax documentation or a professional.

### Q: Can this help with international tax reporting?
**A:** The tool supports basic foreign dividend tracking:
```bash
dividend-tracker tax classify FOREIGN-STOCK --classification foreign --apply-future
```

However, international tax rules are complex and vary by country. The tool is primarily designed for US tax scenarios.

## Technical Issues

### Q: The application is running slowly. What can I do?
**A:** Performance optimization tips:
1. **Regular cleanup**: Remove very old data if not needed
2. **Efficient queries**: Use specific filters instead of listing everything
3. **Index management**: The application automatically optimizes data access
4. **System resources**: Ensure adequate RAM and disk space

### Q: I'm getting CSV import errors. How do I fix them?
**A:** Common CSV issues and solutions:

**Format Problems:**
```bash
# Check your CSV headers match exactly
head -1 your-file.csv
# Should show: symbol,ex_date,pay_date,amount,shares

# Check for hidden characters
cat -A your-file.csv | head -3
```

**Data Issues:**
- Ensure dates are in YYYY-MM-DD format
- Remove extra spaces around values
- Use decimal points, not commas (0.24, not 0,24)
- Verify stock symbols are valid

**Encoding Issues:**
```bash
# Convert to UTF-8 if needed
iconv -f latin1 -t utf-8 your-file.csv > fixed-file.csv
```

### Q: Can I run multiple instances simultaneously?
**A:** **Not recommended**. The application uses file-based storage and concurrent access could cause data corruption. If you need to run multiple instances:
```bash
# Use different data directories
DIVIDEND_TRACKER_DATA_DIR="/tmp/test1" dividend-tracker list
DIVIDEND_TRACKER_DATA_DIR="/tmp/test2" dividend-tracker summary
```

### Q: How do I recover from data corruption?
**A:** Recovery options:
1. **Use automatic backups**:
   ```bash
   dividend-tracker data load backup-YYYY-MM-DD.json
   ```
2. **Restore from manual exports**:
   ```bash
   dividend-tracker import your-backup.csv
   ```
3. **Rebuild from scratch**: Re-import from original sources

**Prevention**: Regular backups are crucial!

## Performance and Scaling

### Q: How much data can the application handle?
**A:** The application can efficiently handle:
- **Dividends**: 100,000+ records
- **Holdings**: 1,000+ positions
- **Years**: Decades of historical data
- **Performance**: Sub-second response for most operations

For very large datasets (500,000+ records), consider periodic archiving of old data.

### Q: Why is the first run slower than subsequent runs?
**A:** The first run may be slower due to:
- Initial data file creation
- Index building
- Configuration setup

Subsequent runs use cached data and optimized file access.

### Q: Can I optimize for better performance?
**A:** Yes:
```bash
# Use filters to limit data processing
dividend-tracker list --year 2024 --symbol AAPL

# Export specific data ranges instead of everything
dividend-tracker data export --data-type dividends --output recent.csv

# Use specific date ranges for analysis
dividend-tracker summary --year 2024 --quarter Q4-2024
```

## Integration and APIs

### Q: What external APIs are supported?
**A:** Currently supported:
- **Alpha Vantage**: Free tier (5 calls/minute, 100 calls/day)
- **Plans for**: Yahoo Finance, IEX Cloud, Quandl

### Q: Why do API calls sometimes fail?
**A:** Common reasons:
1. **Rate limits**: Free tier limits (5 calls/minute)
2. **Invalid symbols**: Check ticker symbol accuracy
3. **Network issues**: Temporary connectivity problems
4. **API key issues**: Expired or invalid key

**Solutions:**
```bash
# Check configuration
dividend-tracker configure --show

# Test with demo key
dividend-tracker configure --api-key demo

# Respect rate limits (wait between calls)
sleep 15 && dividend-tracker fetch MSFT
```

### Q: Can I use this programmatically?
**A:** Yes! Several options:
1. **Command-line integration**: Call commands from scripts
2. **JSON export**: Process data in other tools
3. **Library usage**: See [API.md](API.md) for Rust library integration

Example script integration:
```bash
#!/bin/bash
# Monthly portfolio update script
dividend-tracker update --all
dividend-tracker summary --monthly --export-csv monthly-$(date +%Y-%m).csv
```

### Q: Is there a web interface planned?
**A:** A web dashboard is on the roadmap! Current priorities:
1. **CLI stability and features** ✅
2. **Comprehensive documentation** ✅
3. **API integrations** 🔄
4. **Web interface** 📋
5. **Mobile app** 📋

## Still Have Questions?

If your question isn't answered here:

1. **Check the documentation**:
   - [README.md](README.md) - Overview and basic usage
   - [TUTORIAL.md](TUTORIAL.md) - Comprehensive guide
   - [API.md](API.md) - Library usage
   - [CONTRIBUTING.md](CONTRIBUTING.md) - Development guidelines

2. **Use built-in help**:
   ```bash
   dividend-tracker --help
   dividend-tracker COMMAND --help
   ```

3. **Check GitHub issues**: [Open an issue](https://github.com/your-username/dividend-tracker/issues) if you find a bug or need a feature

4. **Community support**: Join discussions in GitHub Discussions section

## Contributing to the FAQ

Found an answer that helped? Consider contributing:
1. Fork the repository
2. Add your Q&A to this FAQ
3. Submit a pull request
4. Help other users with your experience!

---

*Last updated: December 2024*