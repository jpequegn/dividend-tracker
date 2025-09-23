use anyhow::Result;
use chrono::{Duration, Local};
use tempfile::tempdir;

#[test]
fn test_add_dividend_basic() -> Result<()> {
    // Setup test environment
    let temp_dir = tempdir()?;
    let _data_file = temp_dir.path().join("dividends.json");

    // Run add command
    let output = std::process::Command::new("./target/debug/dividend-tracker")
        .args(&[
            "add",
            "AAPL",
            "--ex-date",
            "2024-01-15",
            "--pay-date",
            "2024-01-18",
            "--amount",
            "0.24",
            "--shares",
            "100",
        ])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Adding dividend record"));
    assert!(stdout.contains("Dividend Details"));
    assert!(stdout.contains("Symbol: AAPL"));
    assert!(stdout.contains("Ex-date: 2024-01-15"));
    assert!(stdout.contains("Pay-date: 2024-01-18"));
    assert!(stdout.contains("Amount per share: $0.2400"));
    assert!(stdout.contains("Shares owned: 100"));
    assert!(stdout.contains("Total dividend: $24"));
    assert!(stdout.contains("Dividend record added successfully"));

    Ok(())
}

#[test]
fn test_add_dividend_with_natural_language_dates() -> Result<()> {
    let temp_dir = tempdir()?;

    // Test "tomorrow" date parsing
    let output = std::process::Command::new("./target/debug/dividend-tracker")
        .args(&[
            "add",
            "MSFT",
            "--ex-date",
            "tomorrow",
            "--pay-date",
            "next friday",
            "--amount",
            "0.68",
            "--shares",
            "50",
        ])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Dividend record added successfully"));

    // Verify the dates were parsed correctly by checking output
    let tomorrow = (Local::now() + Duration::days(1)).naive_local().date();
    assert!(stdout.contains(&tomorrow.format("%Y-%m-%d").to_string()));

    Ok(())
}

#[test]
fn test_add_dividend_duplicate_detection() -> Result<()> {
    let temp_dir = tempdir()?;

    // Add first dividend
    let output1 = std::process::Command::new("./target/debug/dividend-tracker")
        .args(&[
            "add",
            "AAPL",
            "--ex-date",
            "2024-01-15",
            "--pay-date",
            "2024-01-18",
            "--amount",
            "0.24",
            "--shares",
            "100",
        ])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output1.status.success());

    // Try to add duplicate (same symbol and ex-date)
    let output2 = std::process::Command::new("./target/debug/dividend-tracker")
        .args(&[
            "add",
            "AAPL",
            "--ex-date",
            "2024-01-15",
            "--pay-date",
            "2024-01-18",
            "--amount",
            "0.24",
            "--shares",
            "100",
        ])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(!output2.status.success(), "Duplicate should be rejected");

    let stderr = String::from_utf8_lossy(&output2.stderr);
    assert!(stderr.contains("Duplicate dividend exists"));

    let stdout = String::from_utf8_lossy(&output2.stdout);
    assert!(stdout.contains("Duplicate dividend found"));
    assert!(stdout.contains("Use --force to override"));

    Ok(())
}

#[test]
fn test_add_dividend_force_flag() -> Result<()> {
    let temp_dir = tempdir()?;

    // Add first dividend
    let output1 = std::process::Command::new("./target/debug/dividend-tracker")
        .args(&[
            "add",
            "AAPL",
            "--ex-date",
            "2024-01-15",
            "--pay-date",
            "2024-01-18",
            "--amount",
            "0.24",
            "--shares",
            "100",
        ])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output1.status.success());

    // Add duplicate with --force flag
    let output2 = std::process::Command::new("./target/debug/dividend-tracker")
        .args(&[
            "add",
            "AAPL",
            "--ex-date",
            "2024-01-15",
            "--pay-date",
            "2024-01-20",
            "--amount",
            "0.25",
            "--shares",
            "100",
            "--force",
        ])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output2.status.success(), "Force flag should allow duplicate");

    let stdout = String::from_utf8_lossy(&output2.stdout);
    assert!(stdout.contains("Dividend record added successfully"));

    Ok(())
}

#[test]
fn test_add_dividend_invalid_amount() -> Result<()> {
    let temp_dir = tempdir()?;

    let output = std::process::Command::new("./target/debug/dividend-tracker")
        .args(&[
            "add",
            "AAPL",
            "--ex-date",
            "2024-01-15",
            "--pay-date",
            "2024-01-18",
            "--amount",
            "invalid",
            "--shares",
            "100",
        ])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(!output.status.success(), "Invalid amount should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid amount format"));

    Ok(())
}

#[test]
fn test_add_dividend_invalid_shares() -> Result<()> {
    let temp_dir = tempdir()?;

    let output = std::process::Command::new("./target/debug/dividend-tracker")
        .args(&[
            "add",
            "AAPL",
            "--ex-date",
            "2024-01-15",
            "--pay-date",
            "2024-01-18",
            "--amount",
            "0.24",
            "--shares",
            "invalid",
        ])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(!output.status.success(), "Invalid shares should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid shares format"));

    Ok(())
}

#[test]
fn test_add_dividend_invalid_date_format() -> Result<()> {
    let temp_dir = tempdir()?;

    let output = std::process::Command::new("./target/debug/dividend-tracker")
        .args(&[
            "add",
            "AAPL",
            "--ex-date",
            "invalid-date",
            "--pay-date",
            "2024-01-18",
            "--amount",
            "0.24",
            "--shares",
            "100",
        ])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(!output.status.success(), "Invalid date should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid date format"));

    Ok(())
}

#[test]
fn test_add_dividend_validation_against_holdings() -> Result<()> {
    let temp_dir = tempdir()?;

    // First add a holding for AAPL
    let output1 = std::process::Command::new("./target/debug/dividend-tracker")
        .args(&[
            "holdings",
            "add",
            "AAPL",
            "--shares",
            "50",
        ])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output1.status.success());

    // Now add a dividend with more shares than holdings
    let output2 = std::process::Command::new("./target/debug/dividend-tracker")
        .args(&[
            "add",
            "AAPL",
            "--ex-date",
            "2024-01-15",
            "--pay-date",
            "2024-01-18",
            "--amount",
            "0.24",
            "--shares",
            "100", // More than the 50 shares in holdings
        ])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    // Should still succeed but show a warning
    assert!(output2.status.success());

    let stdout = String::from_utf8_lossy(&output2.stdout);
    assert!(stdout.contains("Validating against holdings"));
    assert!(stdout.contains("Warning: Dividend shares (100) exceed current holdings (50)"));
    assert!(stdout.contains("Dividend record added successfully"));

    Ok(())
}

#[test]
fn test_add_dividend_with_decimal_shares() -> Result<()> {
    let temp_dir = tempdir()?;

    let output = std::process::Command::new("./target/debug/dividend-tracker")
        .args(&[
            "add",
            "AAPL",
            "--ex-date",
            "2024-01-15",
            "--pay-date",
            "2024-01-18",
            "--amount",
            "0.24",
            "--shares",
            "100.5", // Fractional shares
        ])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Decimal shares should be accepted");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Shares owned: 100.5"));
    assert!(stdout.contains("Total dividend: $24.12")); // 0.24 * 100.5 = 24.12
    assert!(stdout.contains("Dividend record added successfully"));

    Ok(())
}

#[test]
fn test_add_dividend_date_validation() -> Result<()> {
    let temp_dir = tempdir()?;

    // Try to add dividend with pay date before ex-date
    let output = std::process::Command::new("./target/debug/dividend-tracker")
        .args(&[
            "add",
            "AAPL",
            "--ex-date",
            "2024-01-18",
            "--pay-date",
            "2024-01-15", // Before ex-date
            "--amount",
            "0.24",
            "--shares",
            "100",
        ])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(!output.status.success(), "Pay date before ex-date should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Pay date cannot be before ex-dividend date"));

    Ok(())
}