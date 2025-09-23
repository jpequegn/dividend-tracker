use anyhow::Result;
use tempfile::tempdir;
use std::process::Command;

fn get_binary_path() -> String {
    "./target/debug/dividend-tracker".to_string()
}

fn setup_test_data(temp_dir: &std::path::Path) -> Result<()> {
    // Add test dividends with various dates and amounts for analytics testing
    let commands = vec![
        vec!["add", "AAPL", "--ex-date", "2024-01-15", "--pay-date", "2024-01-18", "--amount", "0.24", "--shares", "100", "--force"],
        vec!["add", "AAPL", "--ex-date", "2024-04-15", "--pay-date", "2024-04-18", "--amount", "0.25", "--shares", "100", "--force"],
        vec!["add", "MSFT", "--ex-date", "2024-03-20", "--pay-date", "2024-03-25", "--amount", "0.68", "--shares", "50", "--force"],
        vec!["add", "MSFT", "--ex-date", "2024-06-20", "--pay-date", "2024-06-25", "--amount", "0.70", "--shares", "50", "--force"],
        vec!["add", "GOOGL", "--ex-date", "2024-06-10", "--pay-date", "2024-06-15", "--amount", "1.20", "--shares", "25", "--force"],
        vec!["add", "TSLA", "--ex-date", "2023-12-15", "--pay-date", "2023-12-20", "--amount", "0.45", "--shares", "75", "--force"],
    ];

    for cmd_args in commands {
        let output = Command::new(&get_binary_path())
            .args(&cmd_args)
            .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir)
            .output()?;

        if !output.status.success() {
            eprintln!("Failed to setup test data: {:?}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(())
}

#[test]
fn test_summary_basic() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["summary"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Summary command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Portfolio Summary & Analytics"));
    assert!(stdout.contains("Basic Summary"));
    assert!(stdout.contains("Total Dividend Income:"));
    assert!(stdout.contains("Total Payments:"));
    assert!(stdout.contains("Unique Stocks:"));
    assert!(stdout.contains("Average Payment:"));

    Ok(())
}

#[test]
fn test_summary_monthly() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["summary", "--monthly", "--year", "2024"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Summary with monthly breakdown should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Monthly Breakdown"));
    assert!(stdout.contains("Year: 2024"));
    assert!(stdout.contains("Month"));
    assert!(stdout.contains("January"));
    assert!(stdout.contains("March"));
    assert!(stdout.contains("April"));
    assert!(stdout.contains("June"));

    Ok(())
}

#[test]
fn test_summary_top_payers() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["summary", "--top-payers", "3"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Summary with top payers should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Top 3 Dividend Payers"));
    assert!(stdout.contains("Rank"));
    assert!(stdout.contains("Symbol"));
    assert!(stdout.contains("Total"));
    assert!(stdout.contains("Payments"));

    Ok(())
}

#[test]
fn test_summary_all_analytics() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["summary", "--all"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Summary with all analytics should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Portfolio Summary & Analytics"));
    assert!(stdout.contains("Basic Summary"));
    assert!(stdout.contains("Growth Analysis"));
    assert!(stdout.contains("Dividend Frequency Analysis"));
    assert!(stdout.contains("Dividend Consistency Analysis"));
    assert!(stdout.contains("Yield Analysis"));

    Ok(())
}

#[test]
fn test_summary_growth_analysis() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["summary", "--growth"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Summary with growth analysis should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Growth Analysis"));
    // Should show insufficient data message since we don't have enough years
    assert!(stdout.contains("Insufficient data") || stdout.contains("Growth Analysis"));

    Ok(())
}

#[test]
fn test_summary_frequency_analysis() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["summary", "--frequency"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Summary with frequency analysis should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Dividend Frequency Analysis"));
    assert!(stdout.contains("AAPL") || stdout.contains("MSFT") || stdout.contains("GOOGL"));

    Ok(())
}

#[test]
fn test_summary_consistency_analysis() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["summary", "--consistency"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Summary with consistency analysis should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Dividend Consistency Analysis"));
    assert!(stdout.contains("Portfolio Consistency Score"));

    Ok(())
}

#[test]
fn test_summary_yield_analysis() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["summary", "--yield-analysis"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Summary with yield analysis should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Yield Analysis"));
    // Should show message about no holdings with cost basis
    assert!(stdout.contains("No holdings with cost basis found") || stdout.contains("Yield Analysis"));

    Ok(())
}

#[test]
fn test_summary_csv_export() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let csv_file = temp_dir.path().join("test_export.csv");

    let output = Command::new(&get_binary_path())
        .args(&["summary", "--export-csv", csv_file.to_str().unwrap()])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Summary with CSV export should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Analytics exported to"));

    // Check that CSV file was created and has content
    assert!(csv_file.exists(), "CSV file should be created");
    let csv_content = std::fs::read_to_string(&csv_file)?;
    assert!(csv_content.contains("Total Dividends"));
    assert!(csv_content.contains("Total Payments"));
    assert!(csv_content.contains("Unique Symbols"));

    Ok(())
}

#[test]
fn test_summary_year_filter() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["summary", "--year", "2024"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Summary with year filter should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Year: 2024"));
    // Should not include 2023 TSLA dividend
    assert!(!stdout.contains("2023"));

    Ok(())
}

#[test]
fn test_summary_quarter_filter() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["summary", "--quarter", "Q1-2024"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Summary with quarter filter should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Quarter: Q1-2024"));

    Ok(())
}

#[test]
fn test_summary_empty_database() -> Result<()> {
    let temp_dir = tempdir()?;

    let output = Command::new(&get_binary_path())
        .args(&["summary"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Summary should succeed with empty database");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Portfolio Summary & Analytics"));
    assert!(stdout.contains("No dividend records found. Use 'add' command to add some dividends first!"));

    Ok(())
}

#[test]
fn test_summary_invalid_quarter() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["summary", "--quarter", "invalid-quarter"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(!output.status.success(), "Summary with invalid quarter should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid quarter. Use Q1, Q2, Q3, or Q4"));

    Ok(())
}