use anyhow::{Context, Result};
use chrono::Local;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

use crate::models::{DividendTracker, Dividend, Holding};

/// Schema version for data migration
const SCHEMA_VERSION: u32 = 1;

/// Data structure for versioned persistence
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PersistedData {
    /// Schema version for migration support
    schema_version: u32,
    /// The actual dividend tracker data
    #[serde(flatten)]
    data: DividendTracker,
    /// Metadata about the saved data
    metadata: DataMetadata,
}

/// Metadata about persisted data
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DataMetadata {
    /// Last save timestamp
    last_saved: String,
    /// Number of saves
    save_count: u32,
    /// Application version that saved the data
    app_version: String,
}

/// Manages data persistence for the dividend tracker
pub struct PersistenceManager {
    /// Base directory for all data files
    data_dir: PathBuf,
    /// Directory for backup files
    backup_dir: PathBuf,
}

impl PersistenceManager {
    /// Create a new persistence manager with default paths
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        let data_dir = home_dir.join(".dividend-tracker");
        let backup_dir = data_dir.join("backups");

        Ok(PersistenceManager {
            data_dir,
            backup_dir,
        })
    }

    /// Create a persistence manager with custom paths (mainly for testing)
    pub fn with_custom_path<P: AsRef<Path>>(path: P) -> Self {
        let data_dir = path.as_ref().to_path_buf();
        let backup_dir = data_dir.join("backups");

        PersistenceManager {
            data_dir,
            backup_dir,
        }
    }

    /// Ensure all required directories exist
    pub fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.data_dir)
            .with_context(|| format!("Failed to create data directory: {:?}", self.data_dir))?;

        fs::create_dir_all(&self.backup_dir)
            .with_context(|| format!("Failed to create backup directory: {:?}", self.backup_dir))?;

        Ok(())
    }

    /// Get the path to the dividends JSON file
    fn dividends_file(&self) -> PathBuf {
        self.data_dir.join("dividends.json")
    }

    /// Get the path to the holdings JSON file
    fn holdings_file(&self) -> PathBuf {
        self.data_dir.join("holdings.json")
    }

    /// Get the path to the config JSON file
    fn config_file(&self) -> PathBuf {
        self.data_dir.join("config.json")
    }

    /// Create a backup of a file before overwriting
    fn backup_file(&self, file_path: &Path) -> Result<()> {
        if !file_path.exists() {
            return Ok(()); // No file to backup
        }

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;

        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let backup_name = format!("{}_{}.bak", file_name.trim_end_matches(".json"), timestamp);
        let backup_path = self.backup_dir.join(backup_name);

        fs::copy(file_path, backup_path)
            .with_context(|| format!("Failed to backup file: {:?}", file_path))?;

        // Clean up old backups (keep only the last 10)
        self.cleanup_old_backups(file_name)?;

        Ok(())
    }

    /// Clean up old backup files, keeping only the most recent N
    fn cleanup_old_backups(&self, base_filename: &str) -> Result<()> {
        let prefix = base_filename.trim_end_matches(".json");
        let mut backups: Vec<PathBuf> = Vec::new();

        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with(prefix) && filename.ends_with(".bak") {
                    backups.push(path);
                }
            }
        }

        // Sort by modification time (newest first)
        backups.sort_by_key(|p| {
            fs::metadata(p)
                .and_then(|m| m.modified())
                .ok()
                .map(|t| std::cmp::Reverse(t))
        });

        // Keep only the last 10 backups
        const MAX_BACKUPS: usize = 10;
        if backups.len() > MAX_BACKUPS {
            for backup in &backups[MAX_BACKUPS..] {
                fs::remove_file(backup)
                    .with_context(|| format!("Failed to remove old backup: {:?}", backup))?;
            }
        }

        Ok(())
    }

    /// Perform an atomic write to a file
    fn atomic_write(&self, path: &Path, content: &[u8]) -> Result<()> {
        // Create a temporary file in the same directory as the target
        let parent = path.parent()
            .ok_or_else(|| anyhow::anyhow!("File has no parent directory"))?;

        let mut temp_file = NamedTempFile::new_in(parent)
            .with_context(|| "Failed to create temporary file")?;

        // Write content to temporary file
        temp_file.write_all(content)
            .with_context(|| "Failed to write to temporary file")?;

        // Sync to ensure data is written to disk
        temp_file.flush()
            .with_context(|| "Failed to flush temporary file")?;

        // Atomically rename temporary file to target
        temp_file.persist(path)
            .with_context(|| format!("Failed to persist file to: {:?}", path))?;

        Ok(())
    }

    /// Save the complete dividend tracker data
    pub fn save(&self, tracker: &DividendTracker) -> Result<()> {
        self.ensure_directories()?;

        let file_path = self.dividends_file();

        // Backup existing file
        self.backup_file(&file_path)?;

        // Prepare versioned data
        let persisted = PersistedData {
            schema_version: SCHEMA_VERSION,
            data: tracker.clone(),
            metadata: DataMetadata {
                last_saved: Local::now().to_rfc3339(),
                save_count: self.get_save_count()? + 1,
                app_version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        // Serialize to JSON with pretty printing
        let json = serde_json::to_string_pretty(&persisted)
            .with_context(|| "Failed to serialize dividend data")?;

        // Atomic write
        self.atomic_write(&file_path, json.as_bytes())?;

        Ok(())
    }

    /// Load the complete dividend tracker data
    pub fn load(&self) -> Result<DividendTracker> {
        let file_path = self.dividends_file();

        if !file_path.exists() {
            // Return empty tracker if no file exists
            return Ok(DividendTracker::new());
        }

        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read file: {:?}", file_path))?;

        // Try to parse the JSON
        let persisted: PersistedData = match serde_json::from_str(&content) {
            Ok(data) => data,
            Err(e) => {
                // Handle corrupted JSON gracefully
                eprintln!("Warning: Failed to parse JSON: {}", e);
                eprintln!("Creating backup and starting fresh...");

                // Backup the corrupted file
                self.backup_file(&file_path)?;

                // Return empty tracker
                return Ok(DividendTracker::new());
            }
        };

        // Check schema version and migrate if needed
        let data = if persisted.schema_version != SCHEMA_VERSION {
            self.migrate_data(persisted)?
        } else {
            persisted.data
        };

        Ok(data)
    }

    /// Save holdings separately
    pub fn save_holdings(&self, holdings: &HashMap<String, Holding>) -> Result<()> {
        self.ensure_directories()?;

        let file_path = self.holdings_file();

        // Backup existing file
        self.backup_file(&file_path)?;

        // Serialize to JSON with pretty printing
        let json = serde_json::to_string_pretty(&holdings)
            .with_context(|| "Failed to serialize holdings data")?;

        // Atomic write
        self.atomic_write(&file_path, json.as_bytes())?;

        Ok(())
    }

    /// Load holdings
    pub fn load_holdings(&self) -> Result<HashMap<String, Holding>> {
        let file_path = self.holdings_file();

        if !file_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read file: {:?}", file_path))?;

        match serde_json::from_str(&content) {
            Ok(holdings) => Ok(holdings),
            Err(e) => {
                eprintln!("Warning: Failed to parse holdings JSON: {}", e);
                eprintln!("Creating backup and starting fresh...");

                // Backup the corrupted file
                self.backup_file(&file_path)?;

                Ok(HashMap::new())
            }
        }
    }

    /// Save dividends only
    pub fn save_dividends(&self, dividends: &[Dividend]) -> Result<()> {
        self.ensure_directories()?;

        // Load existing data to preserve holdings
        let mut tracker = self.load()?;
        tracker.dividends = dividends.to_vec();

        self.save(&tracker)
    }

    /// Load dividends only
    pub fn load_dividends(&self) -> Result<Vec<Dividend>> {
        let tracker = self.load()?;
        Ok(tracker.dividends)
    }

    /// Export data to CSV format
    pub fn export_to_csv(&self, output_path: &Path) -> Result<()> {
        let tracker = self.load()?;

        let mut wtr = csv::Writer::from_path(output_path)
            .with_context(|| format!("Failed to create CSV file: {:?}", output_path))?;

        // Write header
        wtr.write_record(&[
            "Symbol",
            "Company Name",
            "Ex Date",
            "Pay Date",
            "Amount Per Share",
            "Shares Owned",
            "Total Amount",
            "Dividend Type",
        ])?;

        // Write dividend records
        for dividend in &tracker.dividends {
            wtr.write_record(&[
                &dividend.symbol,
                dividend.company_name.as_deref().unwrap_or(""),
                &dividend.ex_date.to_string(),
                &dividend.pay_date.to_string(),
                &dividend.amount_per_share.to_string(),
                &dividend.shares_owned.to_string(),
                &dividend.total_amount.to_string(),
                &format!("{:?}", dividend.dividend_type),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export holdings to CSV format
    pub fn export_holdings_to_csv(&self, output_path: &Path) -> Result<()> {
        let tracker = self.load()?;
        let holdings = &tracker.holdings;

        let mut wtr = csv::Writer::from_path(output_path)
            .with_context(|| format!("Failed to create CSV file: {:?}", output_path))?;

        // Write header
        wtr.write_record(&["Symbol", "Shares", "Avg Cost Basis", "Current Yield %"])?;

        // Write holding records
        for (symbol, holding) in holdings {
            wtr.write_record(&[
                symbol,
                &holding.shares.to_string(),
                &holding.avg_cost_basis
                    .map(|cb| cb.to_string())
                    .unwrap_or_else(|| "".to_string()),
                &holding.current_yield
                    .map(|y| y.to_string())
                    .unwrap_or_else(|| "".to_string()),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Export all data to human-readable JSON
    pub fn export_to_json(&self, output_path: &Path) -> Result<()> {
        let tracker = self.load()?;

        #[derive(serde::Serialize)]
        struct ExportData {
            dividends: Vec<Dividend>,
            holdings: HashMap<String, Holding>,
            export_date: String,
            total_dividend_records: usize,
            total_holdings: usize,
        }

        let export = ExportData {
            total_dividend_records: tracker.dividends.len(),
            total_holdings: tracker.holdings.len(),
            dividends: tracker.dividends,
            holdings: tracker.holdings,
            export_date: Local::now().to_rfc3339(),
        };

        let json = serde_json::to_string_pretty(&export)
            .with_context(|| "Failed to serialize export data")?;

        fs::write(output_path, json)
            .with_context(|| format!("Failed to write JSON export: {:?}", output_path))?;

        Ok(())
    }

    /// Get the current save count
    fn get_save_count(&self) -> Result<u32> {
        let file_path = self.dividends_file();

        if !file_path.exists() {
            return Ok(0);
        }

        let content = fs::read_to_string(&file_path).ok();

        if let Some(content) = content {
            if let Ok(persisted) = serde_json::from_str::<PersistedData>(&content) {
                return Ok(persisted.metadata.save_count);
            }
        }

        Ok(0)
    }

    /// Migrate data from older schema versions
    fn migrate_data(&self, mut data: PersistedData) -> Result<DividendTracker> {
        // This is where we would handle schema migrations
        // For now, we just update the version and return the data

        if data.schema_version < SCHEMA_VERSION {
            eprintln!("Migrating data from schema version {} to {}",
                     data.schema_version, SCHEMA_VERSION);

            // Future migrations would go here
            // Example:
            // if data.schema_version == 1 {
            //     // Migrate from v1 to v2
            //     data = migrate_v1_to_v2(data)?;
            // }

            data.schema_version = SCHEMA_VERSION;
        }

        Ok(data.data)
    }

    /// Get statistics about the persisted data
    pub fn get_stats(&self) -> Result<DataStats> {
        // Ensure directories exist before accessing them
        self.ensure_directories()?;

        let tracker = self.load()?;

        let dividends_file = self.dividends_file();
        let holdings_file = self.holdings_file();

        let dividends_size = if dividends_file.exists() {
            fs::metadata(&dividends_file)?.len()
        } else {
            0
        };

        let holdings_size = if holdings_file.exists() {
            fs::metadata(&holdings_file)?.len()
        } else {
            0
        };

        let backup_count = fs::read_dir(&self.backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "bak")
                    .unwrap_or(false)
            })
            .count();

        Ok(DataStats {
            dividend_count: tracker.dividends.len(),
            holding_count: tracker.holdings.len(),
            total_size_bytes: dividends_size + holdings_size,
            backup_count,
            data_directory: self.data_dir.clone(),
        })
    }
}

/// Statistics about persisted data
#[derive(Debug)]
pub struct DataStats {
    pub dividend_count: usize,
    pub holding_count: usize,
    pub total_size_bytes: u64,
    pub backup_count: usize,
    pub data_directory: PathBuf,
}

impl Default for PersistenceManager {
    fn default() -> Self {
        PersistenceManager::new().expect("Failed to create default persistence manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use rust_decimal_macros::dec;
    use chrono::NaiveDate;

    #[test]
    fn test_persistence_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PersistenceManager::with_custom_path(temp_dir.path());

        assert!(manager.ensure_directories().is_ok());
        assert!(temp_dir.path().join("backups").exists());
    }

    #[test]
    fn test_save_and_load_empty_tracker() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PersistenceManager::with_custom_path(temp_dir.path());

        let tracker = DividendTracker::new();
        assert!(manager.save(&tracker).is_ok());

        let loaded = manager.load().unwrap();
        assert_eq!(loaded.dividends.len(), 0);
        assert_eq!(loaded.holdings.len(), 0);
    }

    #[test]
    fn test_save_and_load_with_data() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PersistenceManager::with_custom_path(temp_dir.path());

        let mut tracker = DividendTracker::new();

        // Add test dividend
        let dividend = Dividend::new(
            "AAPL".to_string(),
            Some("Apple Inc.".to_string()),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 22).unwrap(),
            dec!(0.94),
            dec!(100),
            crate::models::DividendType::Regular,
        ).unwrap();
        tracker.add_dividend(dividend.clone());

        // Add test holding
        let holding = Holding::new(
            "AAPL".to_string(),
            dec!(100),
            Some(dec!(150.00)),
            Some(dec!(2.5)),
        ).unwrap();
        tracker.add_holding(holding.clone());

        // Save and load
        assert!(manager.save(&tracker).is_ok());
        let loaded = manager.load().unwrap();

        assert_eq!(loaded.dividends.len(), 1);
        assert_eq!(loaded.dividends[0], dividend);
        assert_eq!(loaded.holdings.len(), 1);
        assert_eq!(loaded.holdings["AAPL"], holding);
    }

    #[test]
    fn test_backup_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PersistenceManager::with_custom_path(temp_dir.path());

        let tracker = DividendTracker::new();

        // First save
        manager.save(&tracker).unwrap();

        // Second save should create a backup
        manager.save(&tracker).unwrap();

        // Check that backup exists
        let backup_dir = temp_dir.path().join("backups");
        let backups: Vec<_> = fs::read_dir(&backup_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        assert!(backups.len() > 0);
    }

    #[test]
    fn test_corrupted_json_handling() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PersistenceManager::with_custom_path(temp_dir.path());

        manager.ensure_directories().unwrap();

        // Write corrupted JSON
        let file_path = manager.dividends_file();
        fs::write(&file_path, "{ this is not valid json }").unwrap();

        // Load should handle gracefully
        let loaded = manager.load().unwrap();
        assert_eq!(loaded.dividends.len(), 0);

        // Backup should have been created
        let backup_dir = temp_dir.path().join("backups");
        let backups: Vec<_> = fs::read_dir(&backup_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        assert!(backups.len() > 0);
    }
}