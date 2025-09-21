use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api: ApiSettings,
    pub cache: CacheSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSettings {
    pub alpha_vantage_key: Option<String>,
    pub rate_limit_delay_ms: u64,
    pub max_retries: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheSettings {
    pub enabled: bool,
    pub ttl_hours: u32,
    pub max_size_mb: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api: ApiSettings {
                alpha_vantage_key: None,
                rate_limit_delay_ms: 12000, // 5 calls per minute
                max_retries: 3,
                timeout_seconds: 30,
            },
            cache: CacheSettings {
                enabled: true,
                ttl_hours: 24,
                max_size_mb: 100,
            },
        }
    }
}

impl Config {
    /// Get the configuration directory path
    pub fn config_dir() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("dividend-tracker");

        Ok(dir)
    }

    /// Get the configuration file path
    pub fn config_file() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Load configuration from file or create default
    pub fn load() -> Result<Self> {
        let config_file = Self::config_file()?;

        if config_file.exists() {
            let contents = fs::read_to_string(&config_file)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            let mut config = Config::default();

            // Check environment variable for API key
            if let Ok(api_key) = std::env::var("ALPHA_VANTAGE_API_KEY") {
                config.api.alpha_vantage_key = Some(api_key);
            }

            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir()?;
        fs::create_dir_all(&config_dir)?;

        let config_file = Self::config_file()?;
        let contents = toml::to_string_pretty(&self)?;
        fs::write(config_file, contents)?;

        Ok(())
    }

    /// Get the Alpha Vantage API key
    pub fn get_api_key(&self) -> Result<String> {
        // First check config file
        if let Some(ref key) = self.api.alpha_vantage_key {
            return Ok(key.clone());
        }

        // Then check environment variable
        if let Ok(key) = std::env::var("ALPHA_VANTAGE_API_KEY") {
            return Ok(key);
        }

        Err(anyhow!(
            "No Alpha Vantage API key found. Please set the ALPHA_VANTAGE_API_KEY environment variable \
             or add it to the config file at {:?}",
            Self::config_file()?
        ))
    }
}

/// Initialize configuration for first-time setup
pub fn init_config() -> Result<()> {
    let config = Config::default();
    config.save()?;

    let config_file = Config::config_file()?;
    println!("Configuration file created at: {:?}", config_file);
    println!("Please add your Alpha Vantage API key to the config file or set the ALPHA_VANTAGE_API_KEY environment variable");

    Ok(())
}
