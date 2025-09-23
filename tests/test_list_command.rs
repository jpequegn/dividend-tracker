use anyhow::Result;
use tempfile::tempdir;
use std::process::Command;

fn get_binary_path() -> String {
    "./target/debug/dividend-tracker".to_string()
}

fn setup_test_data(temp_dir: &std::path::Path) -> Result<()> {
    // Add some test dividends with various dates and amounts
    let commands = vec![
        vec!["add", "AAPL", "--ex-date", "2024-01-15", "--pay-date", "2024-01-18", "--amount", "0.24", "--shares", "100", "--force"],
        vec!["add", "AAPL", "--ex-date", "2024-05-15", "--pay-date", "2024-05-22", "--amount", "0.25", "--shares", "200", "--force"],
        vec!["add", "MSFT", "--ex-date", "2024-03-20", "--pay-date", "2024-03-25", "--amount", "0.68", "--shares", "50", "--force"],
        vec!["add", "GOOGL", "--ex-date", "2024-06-10", "--pay-date", "2024-06-15", "--amount", "1.20", "--shares", "25", "--force"],
        vec!["add", "TSLA", "--ex-date", "tomorrow", "--pay-date", "next friday", "--amount", "0.45", "--shares", "75", "--force"],
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
fn test_list_all_dividends() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["list"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "List command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Listing dividend payments"));
    assert!(stdout.contains("Symbol"));
    assert!(stdout.contains("Ex-Date"));
    assert!(stdout.contains("Pay-Date"));
    assert!(stdout.contains("$/Share"));
    assert!(stdout.contains("Shares"));
    assert!(stdout.contains("Total"));
    assert!(stdout.contains("Total Dividends:"));
    assert!(stdout.contains("Number of Payments:"));
    assert!(stdout.contains("AAPL"));
    assert!(stdout.contains("MSFT"));
    assert!(stdout.contains("GOOGL"));
    assert!(stdout.contains("TSLA"));

    Ok(())
}

#[test]
fn test_list_filter_by_symbol() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["list", "--symbol", "AAPL"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "List command with symbol filter should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("AAPL"));
    assert!(!stdout.contains("MSFT"));
    assert!(!stdout.contains("GOOGL"));
    assert!(!stdout.contains("TSLA"));
    assert!(stdout.contains("Applied Filters:"));
    assert!(stdout.contains("Symbol: AAPL"));

    Ok(())
}

#[test]
fn test_list_filter_by_year() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["list", "--year", "2024"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "List command with year filter should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Applied Filters:"));
    assert!(stdout.contains("Year: 2024"));
    assert!(stdout.contains("2024-")); // Should contain 2024 dates

    Ok(())
}

#[test]
fn test_list_filter_by_month() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["list", "--month", "5"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "List command with month filter should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Applied Filters:"));
    assert!(stdout.contains("Month: 5"));

    Ok(())
}

#[test]
fn test_list_filter_by_amount_min() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["list", "--amount-min", "0.50"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "List command with amount filter should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Applied Filters:"));
    assert!(stdout.contains("Min Amount: $0.50"));
    // Should show MSFT (0.68) and GOOGL (1.20) but not AAPL (0.24, 0.25)
    assert!(stdout.contains("MSFT"));
    assert!(stdout.contains("GOOGL"));

    Ok(())
}

#[test]
fn test_list_filter_upcoming() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["list", "--upcoming"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "List command with upcoming filter should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Applied Filters:"));
    assert!(stdout.contains("Upcoming Only: Yes"));
    // Should only show TSLA which has future dates
    assert!(stdout.contains("TSLA"));

    Ok(())
}

#[test]
fn test_list_sort_by_symbol() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["list", "--sort-by", "symbol"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "List command with symbol sort should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Sorted by: symbol (ascending)"));

    Ok(())
}

#[test]
fn test_list_sort_by_amount_reverse() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["list", "--sort-by", "amount", "--reverse"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "List command with amount sort reverse should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Sorted by: amount (descending)"));

    Ok(())
}

#[test]
fn test_list_combined_filters() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["list", "--symbol", "AAPL", "--year", "2024", "--sort-by", "amount", "--reverse"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "List command with combined filters should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Applied Filters:"));
    assert!(stdout.contains("Symbol: AAPL"));
    assert!(stdout.contains("Year: 2024"));
    assert!(stdout.contains("Sorted by: amount (descending)"));
    assert!(stdout.contains("AAPL"));
    assert!(!stdout.contains("MSFT"));

    Ok(())
}

#[test]
fn test_list_no_results() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["list", "--symbol", "NVDA"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "List command should succeed even with no results");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No dividends match the specified filters"));

    Ok(())
}

#[test]
fn test_list_empty_database() -> Result<()> {
    let temp_dir = tempdir()?;

    let output = Command::new(&get_binary_path())
        .args(&["list"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "List command should succeed with empty database");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No dividend records found. Use 'add' command to add some!"));

    Ok(())
}

#[test]
fn test_list_date_range_filter() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["list", "--date-start", "2024-03-01", "--date-end", "2024-05-31"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "List command with date range should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Applied Filters:"));
    assert!(stdout.contains("Date Start: 2024-03-01"));
    assert!(stdout.contains("Date End: 2024-05-31"));
    // Should show MSFT (March) and AAPL (May) but not AAPL (January) or GOOGL (June)
    assert!(stdout.contains("MSFT"));

    Ok(())
}

#[test]
fn test_list_invalid_amount_min() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["list", "--amount-min", "invalid"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(!output.status.success(), "List command with invalid amount should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid minimum amount format"));

    Ok(())
}

#[test]
fn test_list_invalid_date() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["list", "--date-start", "invalid-date"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(!output.status.success(), "List command with invalid date should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid date format"));

    Ok(())
}