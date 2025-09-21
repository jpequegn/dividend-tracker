use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use reqwest::blocking::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

/// Alpha Vantage API client for fetching dividend data
pub struct AlphaVantageClient {
    client: Client,
    api_key: String,
    cache_dir: PathBuf,
    rate_limit_delay: Duration,
}

/// Response structure for dividend data from Alpha Vantage
#[derive(Debug, Deserialize, Serialize)]
struct DividendResponse {
    #[serde(rename = "Meta Data")]
    meta_data: Option<MetaData>,
    #[serde(rename = "Monthly Adjusted Time Series")]
    monthly_series: Option<HashMap<String, MonthlyData>>,
    #[serde(rename = "Time Series (Daily)")]
    daily_series: Option<HashMap<String, DailyData>>,
    #[serde(rename = "Error Message")]
    error_message: Option<String>,
    #[serde(rename = "Note")]
    note: Option<String>, // Rate limit message
}

#[derive(Debug, Deserialize, Serialize)]
struct MetaData {
    #[serde(rename = "1. Information")]
    information: String,
    #[serde(rename = "2. Symbol")]
    symbol: String,
    #[serde(rename = "3. Last Refreshed")]
    last_refreshed: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct MonthlyData {
    #[serde(rename = "1. open")]
    open: String,
    #[serde(rename = "2. high")]
    high: String,
    #[serde(rename = "3. low")]
    low: String,
    #[serde(rename = "4. close")]
    close: String,
    #[serde(rename = "5. adjusted close")]
    adjusted_close: String,
    #[serde(rename = "6. volume")]
    volume: String,
    #[serde(rename = "7. dividend amount")]
    dividend_amount: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct DailyData {
    #[serde(rename = "1. open")]
    open: String,
    #[serde(rename = "2. high")]
    high: String,
    #[serde(rename = "3. low")]
    low: String,
    #[serde(rename = "4. close")]
    close: String,
    #[serde(rename = "5. adjusted close")]
    adjusted_close: Option<String>,
    #[serde(rename = "6. volume")]
    volume: String,
    #[serde(rename = "7. dividend amount")]
    dividend_amount: Option<String>,
    #[serde(rename = "8. split coefficient")]
    split_coefficient: Option<String>,
}

/// Dividend data extracted from API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DividendData {
    pub symbol: String,
    pub ex_date: NaiveDate,
    pub amount: Decimal,
}

impl AlphaVantageClient {
    /// Create a new Alpha Vantage API client
    pub fn new(api_key: String) -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow!("Could not determine cache directory"))?
            .join("dividend-tracker")
            .join("api_cache");

        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            client: Client::new(),
            api_key,
            cache_dir,
            rate_limit_delay: Duration::from_millis(12000), // 5 calls per minute = 12 seconds between calls
        })
    }

    /// Fetch dividend history for a symbol
    pub fn fetch_dividends(
        &self,
        symbol: &str,
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>,
    ) -> Result<Vec<DividendData>> {
        // Check cache first
        if let Ok(cached_data) = self.get_cached_dividends(symbol) {
            // Filter by date range if specified
            let filtered = self.filter_by_date_range(cached_data, from_date, to_date);
            if !filtered.is_empty() {
                return Ok(filtered);
            }
        }

        // Fetch from API
        let response = self.fetch_from_api(symbol)?;

        // Parse dividend data
        let dividends = self.parse_dividend_response(symbol, response)?;

        // Cache the results
        self.cache_dividends(symbol, &dividends)?;

        // Filter by date range if specified
        Ok(self.filter_by_date_range(dividends, from_date, to_date))
    }

    /// Fetch data from Alpha Vantage API
    fn fetch_from_api(&self, symbol: &str) -> Result<DividendResponse> {
        // Apply rate limiting
        thread::sleep(self.rate_limit_delay);

        let url = format!(
            "https://www.alphavantage.co/query?function=TIME_SERIES_MONTHLY_ADJUSTED&symbol={}&apikey={}",
            symbol, self.api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .context("Failed to send API request")?
            .json::<DividendResponse>()
            .context("Failed to parse API response")?;

        // Check for error messages
        if let Some(error) = response.error_message {
            return Err(anyhow!("API error: {}", error));
        }

        // Check for rate limit message
        if let Some(ref note) = response.note {
            if note.contains("API call frequency") {
                return Err(anyhow!("Rate limit exceeded: {}", note));
            }
        }

        Ok(response)
    }

    /// Parse dividend data from API response
    fn parse_dividend_response(
        &self,
        symbol: &str,
        response: DividendResponse,
    ) -> Result<Vec<DividendData>> {
        let mut dividends = Vec::new();

        if let Some(monthly_series) = response.monthly_series {
            for (date_str, data) in monthly_series {
                let dividend_amount = Decimal::from_str(&data.dividend_amount)
                    .context("Failed to parse dividend amount")?;

                // Only include if dividend is non-zero
                if dividend_amount > Decimal::ZERO {
                    let ex_date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                        .context("Failed to parse date")?;

                    dividends.push(DividendData {
                        symbol: symbol.to_uppercase(),
                        ex_date,
                        amount: dividend_amount,
                    });
                }
            }
        }

        dividends.sort_by(|a, b| b.ex_date.cmp(&a.ex_date)); // Sort by date, newest first
        Ok(dividends)
    }

    /// Get cached dividend data for a symbol
    fn get_cached_dividends(&self, symbol: &str) -> Result<Vec<DividendData>> {
        let cache_file = self
            .cache_dir
            .join(format!("{}.json", symbol.to_uppercase()));

        if !cache_file.exists() {
            return Err(anyhow!("Cache file does not exist"));
        }

        // Check if cache is fresh (less than 24 hours old)
        let metadata = fs::metadata(&cache_file)?;
        let modified = metadata.modified()?;
        let age = modified.elapsed().unwrap_or(Duration::from_secs(u64::MAX));

        if age > Duration::from_secs(86400) {
            // Cache is stale
            return Err(anyhow!("Cache is stale"));
        }

        let contents = fs::read_to_string(&cache_file)?;
        let dividends: Vec<DividendData> = serde_json::from_str(&contents)?;

        Ok(dividends)
    }

    /// Cache dividend data for a symbol
    fn cache_dividends(&self, symbol: &str, dividends: &[DividendData]) -> Result<()> {
        let cache_file = self
            .cache_dir
            .join(format!("{}.json", symbol.to_uppercase()));
        let json = serde_json::to_string_pretty(dividends)?;
        fs::write(cache_file, json)?;
        Ok(())
    }

    /// Filter dividends by date range
    fn filter_by_date_range(
        &self,
        dividends: Vec<DividendData>,
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>,
    ) -> Vec<DividendData> {
        dividends
            .into_iter()
            .filter(|d| {
                if let Some(from) = from_date {
                    if d.ex_date < from {
                        return false;
                    }
                }
                if let Some(to) = to_date {
                    if d.ex_date > to {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// Batch fetch dividends for multiple symbols
    pub fn batch_fetch_dividends(
        &self,
        symbols: &[String],
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>,
        progress_callback: Option<Box<dyn Fn(usize, usize, &str)>>,
    ) -> HashMap<String, Result<Vec<DividendData>>> {
        let mut results = HashMap::new();
        let total = symbols.len();

        for (index, symbol) in symbols.iter().enumerate() {
            if let Some(ref callback) = progress_callback {
                callback(index + 1, total, symbol);
            }

            let result = self.fetch_dividends(symbol, from_date, to_date);
            results.insert(symbol.clone(), result);
        }

        results
    }
}

/// Configuration for API settings
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConfig {
    pub api_key: String,
    pub rate_limit_delay_ms: u64,
}

impl ApiConfig {
    /// Load API configuration from file
    pub fn load() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("dividend-tracker");

        let config_file = config_dir.join("config.json");

        if config_file.exists() {
            let contents = fs::read_to_string(&config_file)?;
            let config: ApiConfig = serde_json::from_str(&contents)?;
            Ok(config)
        } else {
            // Check environment variable
            if let Ok(api_key) = std::env::var("ALPHA_VANTAGE_API_KEY") {
                Ok(ApiConfig {
                    api_key,
                    rate_limit_delay_ms: 12000,
                })
            } else {
                Err(anyhow!(
                    "No API key found. Set ALPHA_VANTAGE_API_KEY environment variable or create config file at {:?}",
                    config_file
                ))
            }
        }
    }

    /// Save API configuration to file
    pub fn save(&self) -> Result<()> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("dividend-tracker");

        fs::create_dir_all(&config_dir)?;

        let config_file = config_dir.join("config.json");
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(config_file, json)?;

        Ok(())
    }
}
