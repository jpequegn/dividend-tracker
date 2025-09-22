use anyhow::{Result, bail};
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Module for core data structures used in dividend tracking

/// Represents different types of dividend payments
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DividendType {
    /// Regular quarterly or annual dividend
    Regular,
    /// Special one-time dividend payment
    Special,
    /// Return of capital distribution
    ReturnOfCapital,
    /// Stock dividend (shares instead of cash)
    Stock,
    /// Spin-off distribution
    SpinOff,
}

/// Represents a dividend payment record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dividend {
    /// Stock symbol (e.g., AAPL, MSFT)
    pub symbol: String,
    /// Optional company name for display purposes
    pub company_name: Option<String>,
    /// Ex-dividend date (when stock goes ex-dividend)
    pub ex_date: NaiveDate,
    /// Payment date (when dividend is actually paid)
    pub pay_date: NaiveDate,
    /// Dividend amount per share
    pub amount_per_share: Decimal,
    /// Number of shares owned at record date
    pub shares_owned: Decimal,
    /// Total dividend payment (calculated: amount_per_share * shares_owned)
    pub total_amount: Decimal,
    /// Type of dividend payment
    pub dividend_type: DividendType,
}

/// Represents a stock holding in the portfolio
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Holding {
    /// Stock symbol (e.g., AAPL, MSFT)
    pub symbol: String,
    /// Number of shares currently owned
    pub shares: Decimal,
    /// Average cost basis per share (optional for tracking gains/losses)
    pub avg_cost_basis: Option<Decimal>,
    /// Current dividend yield percentage (optional for display)
    pub current_yield: Option<Decimal>,
}

/// Main data structure for managing dividend and portfolio data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DividendTracker {
    /// Collection of dividend payment records
    pub dividends: Vec<Dividend>,
    /// Map of stock symbols to current holdings
    pub holdings: HashMap<String, Holding>,
}

// Implementation blocks for constructor methods and validation
impl Dividend {
    /// Create a new dividend record with validation
    pub fn new(
        symbol: String,
        company_name: Option<String>,
        ex_date: NaiveDate,
        pay_date: NaiveDate,
        amount_per_share: Decimal,
        shares_owned: Decimal,
        dividend_type: DividendType,
    ) -> Result<Self> {
        // Validation checks
        if symbol.trim().is_empty() {
            bail!("Symbol cannot be empty");
        }

        if amount_per_share <= Decimal::ZERO {
            bail!("Amount per share must be positive");
        }

        if shares_owned <= Decimal::ZERO {
            bail!("Shares owned must be positive");
        }

        if pay_date < ex_date {
            bail!("Pay date cannot be before ex-dividend date");
        }

        let total_amount = amount_per_share * shares_owned;

        Ok(Dividend {
            symbol: symbol.trim().to_uppercase(),
            company_name,
            ex_date,
            pay_date,
            amount_per_share,
            shares_owned,
            total_amount,
            dividend_type,
        })
    }
}

impl Holding {
    /// Create a new holding record with validation
    pub fn new(
        symbol: String,
        shares: Decimal,
        avg_cost_basis: Option<Decimal>,
        current_yield: Option<Decimal>,
    ) -> Result<Self> {
        // Validation checks
        if symbol.trim().is_empty() {
            bail!("Symbol cannot be empty");
        }

        if shares <= Decimal::ZERO {
            bail!("Shares must be positive");
        }

        if let Some(cost_basis) = avg_cost_basis {
            if cost_basis <= Decimal::ZERO {
                bail!("Average cost basis must be positive if provided");
            }
        }

        if let Some(yield_pct) = current_yield {
            if yield_pct < Decimal::ZERO {
                bail!("Current yield cannot be negative");
            }
        }

        Ok(Holding {
            symbol: symbol.trim().to_uppercase(),
            shares,
            avg_cost_basis,
            current_yield,
        })
    }
}

impl DividendTracker {
    /// Create a new dividend tracker
    pub fn new() -> Self {
        DividendTracker {
            dividends: Vec::new(),
            holdings: HashMap::new(),
        }
    }

    /// Add a dividend record
    pub fn add_dividend(&mut self, dividend: Dividend) {
        self.dividends.push(dividend);
    }

    /// Add or update a holding
    pub fn add_holding(&mut self, holding: Holding) {
        self.holdings.insert(holding.symbol.clone(), holding);
    }

    /// Get dividends for a specific symbol
    pub fn get_dividends_for_symbol(&self, symbol: &str) -> Vec<&Dividend> {
        let symbol = symbol.trim().to_uppercase();
        self.dividends
            .iter()
            .filter(|div| div.symbol == symbol)
            .collect()
    }

    /// Get total dividend income for a year
    pub fn get_total_income_for_year(&self, year: i32) -> Decimal {
        self.dividends
            .iter()
            .filter(|div| div.pay_date.year() == year)
            .map(|div| div.total_amount)
            .sum()
    }
}

impl Default for DividendTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents an upcoming dividend calendar entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DividendCalendarEntry {
    /// Stock symbol
    pub symbol: String,
    /// Company name
    pub company_name: Option<String>,
    /// Ex-dividend date
    pub ex_date: NaiveDate,
    /// Payment date (estimated or confirmed)
    pub pay_date: Option<NaiveDate>,
    /// Dividend amount per share (estimated or confirmed)
    pub estimated_amount: Option<Decimal>,
    /// Whether this is an estimate based on historical data
    pub is_estimated: bool,
    /// Frequency of dividend payments (quarterly, monthly, annual)
    pub frequency: Option<DividendFrequency>,
    /// Days until ex-date
    pub days_until_ex: i64,
}

/// Dividend payment frequency
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DividendFrequency {
    Monthly,
    Quarterly,
    SemiAnnual,
    Annual,
    Irregular,
}

/// Represents a dividend notification alert
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DividendAlert {
    /// Stock symbol
    pub symbol: String,
    /// Alert type
    pub alert_type: AlertType,
    /// Ex-dividend date
    pub ex_date: NaiveDate,
    /// Estimated dividend amount
    pub estimated_amount: Option<Decimal>,
    /// Number of shares owned
    pub shares_owned: Option<Decimal>,
    /// Estimated income from this dividend
    pub estimated_income: Option<Decimal>,
    /// Alert message
    pub message: String,
}

/// Types of dividend alerts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertType {
    /// Ex-date is tomorrow
    ExDateTomorrow,
    /// Ex-date is within a week
    ExDateThisWeek,
    /// Ex-date is within a month
    ExDateThisMonth,
    /// New dividend announced
    NewDividendAnnounced,
    /// Dividend increase
    DividendIncrease,
    /// Dividend cut
    DividendCut,
}

impl DividendCalendarEntry {
    /// Create a new calendar entry
    pub fn new(
        symbol: String,
        company_name: Option<String>,
        ex_date: NaiveDate,
        pay_date: Option<NaiveDate>,
        estimated_amount: Option<Decimal>,
        is_estimated: bool,
    ) -> Self {
        let today = chrono::Local::now().naive_local().date();
        let days_until_ex = (ex_date - today).num_days();

        DividendCalendarEntry {
            symbol: symbol.trim().to_uppercase(),
            company_name,
            ex_date,
            pay_date,
            estimated_amount,
            is_estimated,
            frequency: None,
            days_until_ex,
        }
    }

    /// Check if ex-date is upcoming (within specified days)
    pub fn is_upcoming(&self, days: i64) -> bool {
        self.days_until_ex >= 0 && self.days_until_ex <= days
    }

    /// Get alert type based on days until ex-date
    pub fn get_alert_type(&self) -> Option<AlertType> {
        match self.days_until_ex {
            0..=1 => Some(AlertType::ExDateTomorrow),
            2..=7 => Some(AlertType::ExDateThisWeek),
            8..=30 => Some(AlertType::ExDateThisMonth),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    #[test]
    fn test_dividend_creation_valid() {
        let dividend = Dividend::new(
            "AAPL".to_string(),
            Some("Apple Inc.".to_string()),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 22).unwrap(),
            dec!(0.94),
            dec!(100),
            DividendType::Regular,
        );

        assert!(dividend.is_ok());
        let dividend = dividend.unwrap();
        assert_eq!(dividend.symbol, "AAPL");
        assert_eq!(dividend.total_amount, dec!(94.0));
    }

    #[test]
    fn test_dividend_creation_invalid_symbol() {
        let result = Dividend::new(
            "".to_string(),
            None,
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 22).unwrap(),
            dec!(0.94),
            dec!(100),
            DividendType::Regular,
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Symbol cannot be empty"));
    }

    #[test]
    fn test_dividend_creation_negative_amount() {
        let result = Dividend::new(
            "AAPL".to_string(),
            None,
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 22).unwrap(),
            dec!(-0.94),
            dec!(100),
            DividendType::Regular,
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Amount per share must be positive"));
    }

    #[test]
    fn test_dividend_creation_invalid_dates() {
        let result = Dividend::new(
            "AAPL".to_string(),
            None,
            NaiveDate::from_ymd_opt(2024, 2, 22).unwrap(), // Ex-date after pay date
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            dec!(0.94),
            dec!(100),
            DividendType::Regular,
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Pay date cannot be before ex-dividend date"));
    }

    #[test]
    fn test_holding_creation_valid() {
        let holding = Holding::new(
            "MSFT".to_string(),
            dec!(50),
            Some(dec!(150.00)),
            Some(dec!(2.5)),
        );

        assert!(holding.is_ok());
        let holding = holding.unwrap();
        assert_eq!(holding.symbol, "MSFT");
        assert_eq!(holding.shares, dec!(50));
    }

    #[test]
    fn test_holding_creation_invalid_shares() {
        let result = Holding::new(
            "MSFT".to_string(),
            dec!(0),
            Some(dec!(150.00)),
            Some(dec!(2.5)),
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Shares must be positive"));
    }

    #[test]
    fn test_holding_creation_negative_yield() {
        let result = Holding::new(
            "MSFT".to_string(),
            dec!(50),
            Some(dec!(150.00)),
            Some(dec!(-2.5)),
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Current yield cannot be negative"));
    }

    #[test]
    fn test_dividend_tracker_new() {
        let tracker = DividendTracker::new();
        assert!(tracker.dividends.is_empty());
        assert!(tracker.holdings.is_empty());
    }

    #[test]
    fn test_dividend_tracker_add_dividend() {
        let mut tracker = DividendTracker::new();
        let dividend = Dividend::new(
            "AAPL".to_string(),
            None,
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 22).unwrap(),
            dec!(0.94),
            dec!(100),
            DividendType::Regular,
        )
        .unwrap();

        tracker.add_dividend(dividend);
        assert_eq!(tracker.dividends.len(), 1);
    }

    #[test]
    fn test_dividend_tracker_add_holding() {
        let mut tracker = DividendTracker::new();
        let holding = Holding::new(
            "AAPL".to_string(),
            dec!(100),
            Some(dec!(150.00)),
            Some(dec!(2.5)),
        )
        .unwrap();

        tracker.add_holding(holding);
        assert_eq!(tracker.holdings.len(), 1);
        assert!(tracker.holdings.contains_key("AAPL"));
    }

    #[test]
    fn test_dividend_tracker_get_dividends_for_symbol() {
        let mut tracker = DividendTracker::new();

        let dividend1 = Dividend::new(
            "AAPL".to_string(),
            None,
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 22).unwrap(),
            dec!(0.94),
            dec!(100),
            DividendType::Regular,
        )
        .unwrap();

        let dividend2 = Dividend::new(
            "MSFT".to_string(),
            None,
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 22).unwrap(),
            dec!(0.75),
            dec!(50),
            DividendType::Regular,
        )
        .unwrap();

        tracker.add_dividend(dividend1);
        tracker.add_dividend(dividend2);

        let aapl_dividends = tracker.get_dividends_for_symbol("AAPL");
        assert_eq!(aapl_dividends.len(), 1);
        assert_eq!(aapl_dividends[0].symbol, "AAPL");
    }

    #[test]
    fn test_dividend_tracker_get_total_income_for_year() {
        let mut tracker = DividendTracker::new();

        let dividend1 = Dividend::new(
            "AAPL".to_string(),
            None,
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 22).unwrap(),
            dec!(0.94),
            dec!(100),
            DividendType::Regular,
        )
        .unwrap();

        let dividend2 = Dividend::new(
            "MSFT".to_string(),
            None,
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 22).unwrap(),
            dec!(0.75),
            dec!(50),
            DividendType::Regular,
        )
        .unwrap();

        tracker.add_dividend(dividend1);
        tracker.add_dividend(dividend2);

        let total_2024 = tracker.get_total_income_for_year(2024);
        assert_eq!(total_2024, dec!(131.5)); // 94.0 + 37.5
    }

    #[test]
    fn test_symbol_normalization() {
        let dividend = Dividend::new(
            " aapl ".to_string(),
            None,
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 22).unwrap(),
            dec!(0.94),
            dec!(100),
            DividendType::Regular,
        )
        .unwrap();

        assert_eq!(dividend.symbol, "AAPL");
    }

    #[test]
    fn test_serde_serialization() {
        let dividend = Dividend::new(
            "AAPL".to_string(),
            Some("Apple Inc.".to_string()),
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 22).unwrap(),
            dec!(0.94),
            dec!(100),
            DividendType::Regular,
        )
        .unwrap();

        let json = serde_json::to_string(&dividend);
        assert!(json.is_ok());

        let deserialized: Result<Dividend, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.unwrap(), dividend);
    }
}
