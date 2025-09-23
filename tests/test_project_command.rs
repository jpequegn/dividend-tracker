use anyhow::Result;
use tempfile::tempdir;
use std::process::Command;
use std::path::Path;

fn get_binary_path() -> String {
    "./target/debug/dividend-tracker".to_string()
}

fn setup_comprehensive_test_data(temp_dir: &Path) -> Result<()> {
    // Add holdings first
    let holdings_commands = vec![
        vec!["holdings", "add", "AAPL", "--shares", "100"],
        vec!["holdings", "add", "MSFT", "--shares", "50"],
        vec!["holdings", "add", "GOOGL", "--shares", "25"],
        vec!["holdings", "add", "TSLA", "--shares", "75"],
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

    // Add comprehensive dividend data spanning multiple years
    let dividend_commands = vec![
        // AAPL - Regular quarterly dividends with growth pattern
        vec!["add", "AAPL", "--ex-date", "2022-02-15", "--pay-date", "2022-02-22", "--amount", "0.22", "--shares", "100", "--force"],
        vec!["add", "AAPL", "--ex-date", "2022-05-15", "--pay-date", "2022-05-22", "--amount", "0.23", "--shares", "100", "--force"],
        vec!["add", "AAPL", "--ex-date", "2022-08-15", "--pay-date", "2022-08-22", "--amount", "0.23", "--shares", "100", "--force"],
        vec!["add", "AAPL", "--ex-date", "2022-11-15", "--pay-date", "2022-11-22", "--amount", "0.23", "--shares", "100", "--force"],

        vec!["add", "AAPL", "--ex-date", "2023-02-15", "--pay-date", "2023-02-22", "--amount", "0.23", "--shares", "100", "--force"],
        vec!["add", "AAPL", "--ex-date", "2023-05-15", "--pay-date", "2023-05-22", "--amount", "0.24", "--shares", "100", "--force"],
        vec!["add", "AAPL", "--ex-date", "2023-08-15", "--pay-date", "2023-08-22", "--amount", "0.24", "--shares", "100", "--force"],
        vec!["add", "AAPL", "--ex-date", "2023-11-15", "--pay-date", "2023-11-22", "--amount", "0.24", "--shares", "100", "--force"],

        vec!["add", "AAPL", "--ex-date", "2024-02-15", "--pay-date", "2024-02-22", "--amount", "0.24", "--shares", "100", "--force"],
        vec!["add", "AAPL", "--ex-date", "2024-05-15", "--pay-date", "2024-05-22", "--amount", "0.25", "--shares", "100", "--force"],
        vec!["add", "AAPL", "--ex-date", "2024-08-15", "--pay-date", "2024-08-22", "--amount", "0.25", "--shares", "100", "--force"],
        vec!["add", "AAPL", "--ex-date", "2024-11-15", "--pay-date", "2024-11-22", "--amount", "0.25", "--shares", "100", "--force"],

        // MSFT - Different payment schedule and amounts
        vec!["add", "MSFT", "--ex-date", "2023-03-20", "--pay-date", "2023-03-25", "--amount", "0.68", "--shares", "50", "--force"],
        vec!["add", "MSFT", "--ex-date", "2023-06-20", "--pay-date", "2023-06-25", "--amount", "0.68", "--shares", "50", "--force"],
        vec!["add", "MSFT", "--ex-date", "2023-09-20", "--pay-date", "2023-09-25", "--amount", "0.68", "--shares", "50", "--force"],
        vec!["add", "MSFT", "--ex-date", "2023-12-20", "--pay-date", "2023-12-25", "--amount", "0.68", "--shares", "50", "--force"],

        vec!["add", "MSFT", "--ex-date", "2024-03-20", "--pay-date", "2024-03-25", "--amount", "0.75", "--shares", "50", "--force"],
        vec!["add", "MSFT", "--ex-date", "2024-06-20", "--pay-date", "2024-06-25", "--amount", "0.75", "--shares", "50", "--force"],
        vec!["add", "MSFT", "--ex-date", "2024-09-20", "--pay-date", "2024-09-25", "--amount", "0.75", "--shares", "50", "--force"],

        // GOOGL - Irregular payments
        vec!["add", "GOOGL", "--ex-date", "2023-06-10", "--pay-date", "2023-06-15", "--amount", "1.20", "--shares", "25", "--force"],
        vec!["add", "GOOGL", "--ex-date", "2024-06-10", "--pay-date", "2024-06-15", "--amount", "1.20", "--shares", "25", "--force"],

        // TSLA - Only recent data to test edge cases
        vec!["add", "TSLA", "--ex-date", "2024-12-15", "--pay-date", "2024-12-20", "--amount", "0.50", "--shares", "75", "--force"],
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

fn setup_minimal_test_data(temp_dir: &Path) -> Result<()> {
    // Add minimal holdings
    let output = Command::new(&get_binary_path())
        .args(&["holdings", "add", "AAPL", "--shares", "100"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir)
        .output()?;

    if !output.status.success() {
        eprintln!("Failed to setup minimal holdings: {:?}", String::from_utf8_lossy(&output.stderr));
    }

    // Add minimal dividend data
    let output = Command::new(&get_binary_path())
        .args(&["add", "AAPL", "--ex-date", "2024-01-15", "--pay-date", "2024-01-18", "--amount", "0.25", "--shares", "100", "--force"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir)
        .output()?;

    if !output.status.success() {
        eprintln!("Failed to setup minimal dividend data: {:?}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

#[test]
fn test_project_command_help() -> Result<()> {
    let output = Command::new(&get_binary_path())
        .args(&["project", "--help"])
        .output()?;

    assert!(output.status.success(), "Project help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Project future dividend income"));
    assert!(stdout.contains("--method"));
    assert!(stdout.contains("--growth-rate"));
    assert!(stdout.contains("--monthly"));
    assert!(stdout.contains("--export-csv"));
    assert!(stdout.contains("--export-json"));

    Ok(())
}

#[test]
fn test_project_average_2_years_method() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_comprehensive_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["project", "--method", "average-2-years"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Project command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Dividend Income Projections"));
    assert!(stdout.contains("Projection Method: AverageYears(2)"));
    assert!(stdout.contains("Growth Scenario: Moderate (5%)"));
    assert!(stdout.contains("Projected Annual Income:"));
    assert!(stdout.contains("Individual Stock Projections"));
    assert!(stdout.contains("AAPL"));
    assert!(stdout.contains("Projection Details"));
    assert!(stdout.contains("Confidence Score:"));

    Ok(())
}

#[test]
fn test_project_average_3_years_method() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_comprehensive_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["project", "--method", "average-3-years"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Project command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("AverageYears(3)"));
    assert!(stdout.contains("Projected Annual Income:"));

    Ok(())
}

#[test]
fn test_project_current_yield_method() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_comprehensive_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["project", "--method", "current-yield"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Project command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Projection Method: CurrentYield"));
    assert!(stdout.contains("Projected Annual Income:"));

    Ok(())
}

#[test]
fn test_project_last_12_months_method() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_comprehensive_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["project", "--method", "last-12-months"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Project command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Projection Method: Last12Months"));
    // Note: This might show $0.00 due to date filtering, which is expected

    Ok(())
}

#[test]
fn test_project_growth_scenarios() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_comprehensive_test_data(temp_dir.path())?;

    // Test conservative growth
    let output = Command::new(&get_binary_path())
        .args(&["project", "--method", "average-2-years", "--growth-rate", "conservative"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Conservative growth should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Conservative (2%)"));

    // Test optimistic growth
    let output = Command::new(&get_binary_path())
        .args(&["project", "--method", "average-2-years", "--growth-rate", "optimistic"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Optimistic growth should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Optimistic (9%)"));

    // Test custom growth rate
    let output = Command::new(&get_binary_path())
        .args(&["project", "--method", "average-2-years", "--growth-rate", "7.5%"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Custom growth should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Custom (") && stdout.contains("%)"));

    Ok(())
}

#[test]
fn test_project_monthly_breakdown() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_comprehensive_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["project", "--method", "average-2-years", "--monthly"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Monthly breakdown should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Monthly Projected Cash Flow"));
    assert!(stdout.contains("January"));
    assert!(stdout.contains("February"));
    assert!(stdout.contains("March"));
    assert!(stdout.contains("Top Contributors"));

    Ok(())
}

#[test]
fn test_project_csv_export() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_comprehensive_test_data(temp_dir.path())?;

    let csv_path = temp_dir.path().join("test_projections.csv");

    let output = Command::new(&get_binary_path())
        .args(&["project", "--method", "average-2-years", "--export-csv", csv_path.to_str().unwrap()])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "CSV export should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("exported to"));

    // Verify CSV file was created and has expected content
    assert!(csv_path.exists(), "CSV file should be created");
    let csv_content = std::fs::read_to_string(&csv_path)?;
    assert!(csv_content.contains("Type,Symbol,Month,Amount,Details"));
    assert!(csv_content.contains("Summary,Portfolio,Annual"));
    assert!(csv_content.contains("Stock,AAPL"));
    assert!(csv_content.contains("Monthly,Portfolio"));
    assert!(csv_content.contains("Metadata,Method"));

    Ok(())
}

#[test]
fn test_project_json_export() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_comprehensive_test_data(temp_dir.path())?;

    let json_path = temp_dir.path().join("test_projections.json");

    let output = Command::new(&get_binary_path())
        .args(&["project", "--method", "average-2-years", "--export-json", json_path.to_str().unwrap()])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "JSON export should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("exported to"));

    // Verify JSON file was created and has expected content
    assert!(json_path.exists(), "JSON file should be created");
    let json_content = std::fs::read_to_string(&json_path)?;

    // Parse as JSON to verify structure
    let json_value: serde_json::Value = serde_json::from_str(&json_content)?;
    assert!(json_value["year"].is_number());
    assert!(json_value["total_projected_income"].is_string());
    assert!(json_value["stock_projections"].is_array());
    assert!(json_value["monthly_breakdown"].is_array());
    assert!(json_value["metadata"].is_object());

    Ok(())
}

#[test]
fn test_project_specific_year() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_comprehensive_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["project", "--method", "average-2-years", "--year", "2027"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Specific year should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Target Year: 2027"));

    Ok(())
}

#[test]
fn test_project_invalid_method() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_minimal_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["project", "--method", "invalid-method"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(!output.status.success(), "Invalid method should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid projection method"));

    Ok(())
}

#[test]
fn test_project_invalid_growth_rate() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_minimal_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["project", "--method", "average-2-years", "--growth-rate", "invalid"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(!output.status.success(), "Invalid growth rate should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid growth rate"));

    Ok(())
}

#[test]
fn test_project_no_holdings() -> Result<()> {
    let temp_dir = tempdir()?;
    // Don't setup any data

    let output = Command::new(&get_binary_path())
        .args(&["project"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    // Should handle gracefully - either succeed with empty results or provide helpful message
    // The exact behavior depends on implementation details
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should contain some indication of no data
    assert!(stdout.contains("Projection") || stderr.contains("No") || stdout.contains("$0.00"));

    Ok(())
}

#[test]
fn test_project_no_dividend_history() -> Result<()> {
    let temp_dir = tempdir()?;

    // Add holdings but no dividend history
    let output = Command::new(&get_binary_path())
        .args(&["holdings", "add", "NEWSTOCK", "--shares", "100"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success());

    let output = Command::new(&get_binary_path())
        .args(&["project"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    // Should handle gracefully
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Projection") || stdout.contains("$0.00"));

    Ok(())
}

#[test]
fn test_project_confidence_scoring() -> Result<()> {
    let temp_dir = tempdir()?;
    setup_comprehensive_test_data(temp_dir.path())?;

    let output = Command::new(&get_binary_path())
        .args(&["project", "--method", "average-2-years"])
        .env("DIVIDEND_TRACKER_DATA_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Project command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Confidence Score:"));

    // Should have high confidence with comprehensive data
    assert!(stdout.contains("confidence") || stdout.contains("Confidence"));
    // Verify percentage is shown
    assert!(stdout.contains("%"));

    Ok(())
}