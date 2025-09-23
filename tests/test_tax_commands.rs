use anyhow::Result;
use tempfile::tempdir;
use std::process::Command;
use std::path::Path;

fn get_binary_path() -> String {
    "./target/debug/dividend-tracker".to_string()
}

fn setup_tax_test_data(temp_dir: &Path) -> Result<()> {
    // Add holdings first
    let holdings_commands = vec![
        vec!["holdings", "add", "AAPL", "--shares", "100"],
        vec!["holdings", "add", "MSFT", "--shares", "50"],
        vec!["holdings", "add", "REIT", "--shares", "200"],
        vec!["holdings", "add", "FOREIGN", "--shares", "75"],
    ];

    for cmd_args in holdings_commands {
        let output = Command::new(&get_binary_path())
            .args(&cmd_args)
            .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir)
            .output()?;

        if !output.status.success() {
            eprintln!("Failed to setup holdings: {:?}", String::from_utf8_lossy(&output.stderr));
        }
    }

    // Add comprehensive dividend data with different tax classifications for 2023 and 2024
    let dividend_commands = vec![
        // AAPL - Qualified dividends for 2023
        vec!["add", "AAPL", "--ex-date", "2023-02-15", "--pay-date", "2023-02-22", "--amount", "0.23", "--shares", "100", "--force"],
        vec!["add", "AAPL", "--ex-date", "2023-05-15", "--pay-date", "2023-05-22", "--amount", "0.24", "--shares", "100", "--force"],
        vec!["add", "AAPL", "--ex-date", "2023-08-15", "--pay-date", "2023-08-22", "--amount", "0.24", "--shares", "100", "--force"],
        vec!["add", "AAPL", "--ex-date", "2023-11-15", "--pay-date", "2023-11-22", "--amount", "0.24", "--shares", "100", "--force"],

        // AAPL - Qualified dividends for 2024
        vec!["add", "AAPL", "--ex-date", "2024-02-15", "--pay-date", "2024-02-22", "--amount", "0.24", "--shares", "100", "--force"],
        vec!["add", "AAPL", "--ex-date", "2024-05-15", "--pay-date", "2024-05-22", "--amount", "0.25", "--shares", "100", "--force"],
        vec!["add", "AAPL", "--ex-date", "2024-08-15", "--pay-date", "2024-08-22", "--amount", "0.25", "--shares", "100", "--force"],

        // MSFT - Qualified dividends for 2023
        vec!["add", "MSFT", "--ex-date", "2023-03-20", "--pay-date", "2023-03-25", "--amount", "0.68", "--shares", "50", "--force"],
        vec!["add", "MSFT", "--ex-date", "2023-06-20", "--pay-date", "2023-06-25", "--amount", "0.68", "--shares", "50", "--force"],
        vec!["add", "MSFT", "--ex-date", "2023-09-20", "--pay-date", "2023-09-25", "--amount", "0.68", "--shares", "50", "--force"],
        vec!["add", "MSFT", "--ex-date", "2023-12-20", "--pay-date", "2023-12-25", "--amount", "0.68", "--shares", "50", "--force"],

        // MSFT - Qualified dividends for 2024
        vec!["add", "MSFT", "--ex-date", "2024-03-20", "--pay-date", "2024-03-25", "--amount", "0.75", "--shares", "50", "--force"],
        vec!["add", "MSFT", "--ex-date", "2024-06-20", "--pay-date", "2024-06-25", "--amount", "0.75", "--shares", "50", "--force"],

        // REIT - Non-qualified dividends (REITs typically pay non-qualified)
        vec!["add", "REIT", "--ex-date", "2023-01-31", "--pay-date", "2023-02-08", "--amount", "0.12", "--shares", "200", "--force"],
        vec!["add", "REIT", "--ex-date", "2023-04-30", "--pay-date", "2023-05-08", "--amount", "0.12", "--shares", "200", "--force"],
        vec!["add", "REIT", "--ex-date", "2023-07-31", "--pay-date", "2023-08-08", "--amount", "0.13", "--shares", "200", "--force"],
        vec!["add", "REIT", "--ex-date", "2023-10-31", "--pay-date", "2023-11-08", "--amount", "0.13", "--shares", "200", "--force"],

        vec!["add", "REIT", "--ex-date", "2024-01-31", "--pay-date", "2024-02-08", "--amount", "0.13", "--shares", "200", "--force"],
        vec!["add", "REIT", "--ex-date", "2024-04-30", "--pay-date", "2024-05-08", "--amount", "0.14", "--shares", "200", "--force"],

        // FOREIGN - Foreign dividends with withholding
        vec!["add", "FOREIGN", "--ex-date", "2023-06-15", "--pay-date", "2023-06-30", "--amount", "0.80", "--shares", "75", "--force"],
        vec!["add", "FOREIGN", "--ex-date", "2023-12-15", "--pay-date", "2023-12-30", "--amount", "0.85", "--shares", "75", "--force"],

        vec!["add", "FOREIGN", "--ex-date", "2024-06-15", "--pay-date", "2024-06-30", "--amount", "0.85", "--shares", "75", "--force"],
    ];

    for cmd_args in dividend_commands {
        let output = Command::new(&get_binary_path())
            .args(&cmd_args)
            .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir)
            .output()?;

        if !output.status.success() {
            eprintln!("Failed to setup dividend data: {:?}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(())
}

#[test]
fn test_tax_command_help() -> Result<()> {
    let output = Command::new(&get_binary_path())
        .args(&["tax", "--help"])
        .output()?;

    assert!(output.status.success(), "Tax help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Tax reporting and analysis"));
    assert!(stdout.contains("summary"));
    assert!(stdout.contains("report"));
    assert!(stdout.contains("estimate"));
    assert!(stdout.contains("lots"));
    assert!(stdout.contains("classify"));

    Ok(())
}

#[test]
fn test_tax_summary_help() -> Result<()> {
    let output = Command::new(&get_binary_path())
        .args(&["tax", "summary", "--help"])
        .output()?;

    assert!(output.status.success(), "Tax summary help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Generate annual tax summary for a specific year"));
    assert!(stdout.contains("--year"));
    assert!(stdout.contains("--export-csv"));

    Ok(())
}

#[test]
fn test_tax_summary_default_year() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_tax_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["tax", "summary"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Tax summary should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Tax Summary Report"));
    assert!(stdout.contains("Total Dividend Income"));
    assert!(stdout.contains("Qualified Dividends"));
    assert!(stdout.contains("Non-Qualified Dividends"));

    Ok(())
}

#[test]
fn test_tax_summary_specific_year() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_tax_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["tax", "summary", "--year", "2023"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Tax summary for 2023 should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Tax Summary for 2023"));
    assert!(stdout.contains("Total Dividend Income"));
    assert!(stdout.contains("AAPL"));
    assert!(stdout.contains("MSFT"));
    assert!(stdout.contains("REIT"));

    Ok(())
}

#[test]
fn test_tax_summary_csv_export() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_tax_test_data(temp_dir.path())?;

    let csv_path = temp_dir.path().join("tax_summary.csv");

    let output = Command::new(&get_binary_path())
        .args(&["tax", "summary", "--year", "2023", "--export-csv", csv_path.to_str().unwrap()])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Tax summary CSV export should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("exported to"));

    // Verify CSV file was created and has expected content
    assert!(csv_path.exists(), "CSV file should be created");
    let csv_content = std::fs::read_to_string(&csv_path)?;
    assert!(csv_content.contains("Tax Year,2023"));
    assert!(csv_content.contains("Summary"));
    assert!(csv_content.contains("By Symbol"));
    assert!(csv_content.contains("AAPL"));
    assert!(csv_content.contains("MSFT"));

    Ok(())
}


#[test]
fn test_tax_report_help() -> Result<()> {
    let output = Command::new(&get_binary_path())
        .args(&["tax", "report", "--help"])
        .output()?;

    assert!(output.status.success(), "Tax report help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Generate 1099-DIV style report"));
    assert!(stdout.contains("--year"));

    Ok(())
}

#[test]
fn test_tax_report_all_symbols() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_tax_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["tax", "report", "--year", "2023"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Tax report should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1099-DIV Style Tax Report"));
    assert!(stdout.contains("1099-DIV Report for 2023"));
    assert!(stdout.contains("Payer Details"));
    assert!(stdout.contains("AAPL"));
    assert!(stdout.contains("MSFT"));
    assert!(stdout.contains("Total Ordinary Dividends"));
    assert!(stdout.contains("Qualified Dividends"));

    Ok(())
}

#[test]
fn test_tax_report_json_export() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_tax_test_data(temp_dir.path())?;

    let json_path = temp_dir.path().join("tax_report.json");

    let output = Command::new(&get_binary_path())
        .args(&["tax", "report", "--year", "2023", "--export-json", json_path.to_str().unwrap()])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Tax report JSON export should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("exported to"));

    // Verify JSON file was created
    assert!(json_path.exists(), "JSON file should be created");

    Ok(())
}

#[test]
fn test_tax_estimate_help() -> Result<()> {
    let output = Command::new(&get_binary_path())
        .args(&["tax", "estimate", "--help"])
        .output()?;

    assert!(output.status.success(), "Tax estimate help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Calculate estimated taxes on dividend income"));
    assert!(stdout.contains("--filing-status"));
    assert!(stdout.contains("--income-bracket"));
    assert!(stdout.contains("--year"));

    Ok(())
}

#[test]
fn test_tax_estimate_single() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_tax_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["tax", "estimate", "--year", "2023", "--filing-status", "single", "--income-bracket", "medium"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Tax estimate should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Tax Estimate Calculator"));
    assert!(stdout.contains("Tax Estimate for"));
    assert!(stdout.contains("Single filing status"));
    assert!(stdout.contains("Medium income bracket"));
    assert!(stdout.contains("Qualified Dividend Income"));
    assert!(stdout.contains("Total Estimated Tax"));

    Ok(())
}

#[test]
fn test_tax_estimate_married_joint() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_tax_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["tax", "estimate", "--year", "2023", "--filing-status", "married-jointly", "--income-bracket", "high"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Tax estimate married filing jointly should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MarriedFilingJointly filing status"));
    assert!(stdout.contains("High income bracket"));

    Ok(())
}

#[test]
fn test_tax_lots_help() -> Result<()> {
    let output = Command::new(&get_binary_path())
        .args(&["tax", "lots", "--help"])
        .output()?;

    assert!(output.status.success(), "Tax lots help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Show tax lot breakdown"));
    assert!(stdout.contains("--year"));
    assert!(stdout.contains("--symbol"));

    Ok(())
}

#[test]
fn test_tax_lots_no_data() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_tax_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["tax", "lots", "--year", "2023"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Tax lots should succeed even with no tax lot data");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Tax Lot Analysis"));
    // Should handle gracefully when no tax lot information is available

    Ok(())
}

#[test]
fn test_tax_classify_help() -> Result<()> {
    let output = Command::new(&get_binary_path())
        .args(&["tax", "classify", "--help"])
        .output()?;

    assert!(output.status.success(), "Tax classify help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Update tax classification for dividends"));
    assert!(stdout.contains("<SYMBOL>"));
    assert!(stdout.contains("--classification"));
    assert!(stdout.contains("--year"));

    Ok(())
}

#[test]
fn test_tax_classify_specific_dividend() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_tax_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["tax", "classify", "REIT", "--year", "2023", "--classification", "non-qualified"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Tax classify should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Updated") && stdout.contains("dividend records"));
    assert!(stdout.contains("REIT"));
    assert!(stdout.contains("NonQualified"));

    Ok(())
}

#[test]
fn test_tax_classify_all_symbol() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_tax_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["tax", "classify", "REIT", "--classification", "non-qualified"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Tax classify all should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Updated") && stdout.contains("dividend records"));
    assert!(stdout.contains("REIT"));

    Ok(())
}

#[test]
fn test_tax_invalid_year() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_tax_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["tax", "summary", "--year", "invalid"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(!output.status.success(), "Invalid year should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid") || stderr.contains("Invalid"));

    Ok(())
}

#[test]
fn test_tax_invalid_filing_status() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_tax_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["tax", "estimate", "--filing-status", "invalid", "--income-bracket", "medium"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(!output.status.success(), "Invalid filing status should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid") || stderr.contains("Invalid"));

    Ok(())
}

#[test]
fn test_tax_invalid_classification() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_tax_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["tax", "classify", "AAPL", "--classification", "invalid"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(!output.status.success(), "Invalid classification should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid") || stderr.contains("Invalid"));

    Ok(())
}

#[test]
fn test_tax_no_data() -> Result<()> {
    let temp_dir = tempdir()?;
    // Don't setup any data

    let output = Command::new(&get_binary_path())
        .args(&["tax", "summary"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    // Should handle gracefully - either succeed with empty results or provide helpful message
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should contain some indication of no data or show empty summary
    assert!(stdout.contains("Tax Summary") || stderr.contains("No") || stdout.contains("$0.00"));

    Ok(())
}

#[test]
fn test_tax_year_with_no_dividends() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_tax_test_data(temp_dir.path())?;

    // Test a year with no dividends (like 2020)
    let output = Command::new(&get_binary_path())
        .args(&["tax", "summary", "--year", "2020"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Tax summary for year with no data should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Tax Summary for 2020"));
    assert!(stdout.contains("$0.00"));

    Ok(())
}

#[test]
fn test_tax_future_year() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_tax_test_data(temp_dir.path())?;

    // Test a future year
    let output = Command::new(&get_binary_path())
        .args(&["tax", "summary", "--year", "2030"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Tax summary for future year should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Tax Summary for 2030"));
    assert!(stdout.contains("$0.00"));

    Ok(())
}

#[test]
fn test_tax_comprehensive_flow() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_tax_test_data(temp_dir.path())?;

    // 1. Get tax summary
    let output = Command::new(&get_binary_path())
        .args(&["tax", "summary", "--year", "2023"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;
    assert!(output.status.success(), "Tax summary should succeed");

    // 2. Classify REIT dividends as non-qualified
    let output = Command::new(&get_binary_path())
        .args(&["tax", "classify", "REIT", "--classification", "non-qualified"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;
    assert!(output.status.success(), "Tax classify should succeed");

    // 3. Generate 1099-DIV report
    let output = Command::new(&get_binary_path())
        .args(&["tax", "report", "--year", "2023"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;
    assert!(output.status.success(), "Tax report should succeed");

    // 4. Calculate tax estimate
    let output = Command::new(&get_binary_path())
        .args(&["tax", "estimate", "--year", "2023", "--filing-status", "single", "--income-bracket", "medium"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;
    assert!(output.status.success(), "Tax estimate should succeed");

    // 5. Export to CSV
    let csv_path = temp_dir.path().join("comprehensive_tax.csv");
    let output = Command::new(&get_binary_path())
        .args(&["tax", "summary", "--year", "2023", "--export-csv", csv_path.to_str().unwrap()])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;
    assert!(output.status.success(), "Tax CSV export should succeed");
    assert!(csv_path.exists(), "CSV file should be created");

    Ok(())
}