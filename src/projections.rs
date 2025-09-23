use anyhow::{anyhow, Result};
use chrono::{Datelike, Local, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

use crate::models::{Dividend, DividendTracker, Holding};

/// Projection method for calculating future dividend income
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectionMethod {
    /// Use last 12 months of actual dividend payments
    Last12Months,
    /// Use average of last 2-3 years of payments
    AverageYears(u32), // number of years to average
    /// Use current indicated annual yield rates
    CurrentYield,
}

/// Growth scenario assumptions for dividend projections
#[derive(Debug, Clone, PartialEq)]
pub enum GrowthScenario {
    /// Conservative growth scenario (1-3% annual growth)
    Conservative,
    /// Moderate growth scenario (3-7% annual growth)
    Moderate,
    /// Optimistic growth scenario (7-12% annual growth)
    Optimistic,
    /// Custom growth rate
    Custom(Decimal),
}

impl GrowthScenario {
    /// Get the annual growth rate as a decimal (e.g., 0.05 for 5%)
    pub fn get_growth_rate(&self) -> Decimal {
        match self {
            GrowthScenario::Conservative => dec!(0.02), // 2%
            GrowthScenario::Moderate => dec!(0.05),     // 5%
            GrowthScenario::Optimistic => dec!(0.09),   // 9%
            GrowthScenario::Custom(rate) => *rate,
        }
    }

    /// Get the display name for the scenario
    pub fn name(&self) -> String {
        match self {
            GrowthScenario::Conservative => "Conservative (2%)".to_string(),
            GrowthScenario::Moderate => "Moderate (5%)".to_string(),
            GrowthScenario::Optimistic => "Optimistic (9%)".to_string(),
            GrowthScenario::Custom(rate) => format!("Custom ({:.1}%)", rate * dec!(100)),
        }
    }
}

/// Monthly projected dividend income
#[derive(Debug, Clone)]
pub struct MonthlyProjection {
    pub month: u32,
    pub month_name: String,
    pub projected_amount: Decimal,
    pub payment_count: usize,
    pub top_payers: Vec<String>, // symbols of top contributing stocks
}

/// Annual dividend projection summary
#[derive(Debug, Clone)]
pub struct DividendProjection {
    /// Target year for projection
    pub year: i32,
    /// Total projected annual dividend income
    pub total_projected_income: Decimal,
    /// Projection method used
    pub method: ProjectionMethod,
    /// Growth scenario applied
    pub growth_scenario: GrowthScenario,
    /// Monthly breakdown of projected income
    pub monthly_projections: HashMap<u32, MonthlyProjection>,
    /// Individual stock projections
    pub stock_projections: Vec<StockProjection>,
    /// Metadata about the projection
    pub metadata: ProjectionMetadata,
}

/// Individual stock dividend projection
#[derive(Debug, Clone)]
pub struct StockProjection {
    pub symbol: String,
    pub current_shares: Decimal,
    pub projected_annual_dividend: Decimal,
    pub historical_dividend_per_share: Decimal,
    pub projected_dividend_per_share: Decimal,
    pub growth_applied: Decimal, // growth rate applied
    pub payment_frequency: PaymentFrequency,
    /// Expected payment months based on historical data
    pub payment_months: Vec<u32>,
}

/// Dividend payment frequency analysis
#[derive(Debug, Clone, PartialEq)]
pub enum PaymentFrequency {
    Monthly,
    Quarterly,
    SemiAnnual,
    Annual,
    Irregular,
}

impl PaymentFrequency {
    /// Get expected number of payments per year
    pub fn payments_per_year(&self) -> u32 {
        match self {
            PaymentFrequency::Monthly => 12,
            PaymentFrequency::Quarterly => 4,
            PaymentFrequency::SemiAnnual => 2,
            PaymentFrequency::Annual => 1,
            PaymentFrequency::Irregular => 1, // conservative assumption
        }
    }

    /// Get display name
    pub fn name(&self) -> &str {
        match self {
            PaymentFrequency::Monthly => "Monthly",
            PaymentFrequency::Quarterly => "Quarterly",
            PaymentFrequency::SemiAnnual => "Semi-Annual",
            PaymentFrequency::Annual => "Annual",
            PaymentFrequency::Irregular => "Irregular",
        }
    }
}

/// Metadata about the projection calculation
#[derive(Debug, Clone)]
pub struct ProjectionMetadata {
    /// When the projection was calculated
    pub calculated_at: String,
    /// Number of historical data points used
    pub data_points_used: usize,
    /// Date range of historical data used
    pub historical_range: (Option<NaiveDate>, Option<NaiveDate>),
    /// Stocks included in projection
    pub stocks_included: usize,
    /// Stocks excluded (no historical data)
    pub stocks_excluded: Vec<String>,
    /// Confidence score (0-100)
    pub confidence_score: u32,
}

/// Main projection engine
pub struct ProjectionEngine;

impl ProjectionEngine {
    /// Generate dividend projections for the next year
    pub fn generate_projection(
        tracker: &DividendTracker,
        method: ProjectionMethod,
        growth_scenario: GrowthScenario,
        target_year: Option<i32>,
    ) -> Result<DividendProjection> {
        let current_year = Local::now().year();
        let projection_year = target_year.unwrap_or(current_year + 1);

        // Validate we have holdings to project
        if tracker.holdings.is_empty() {
            return Err(anyhow!("No holdings found. Add holdings first to generate projections."));
        }

        // Generate individual stock projections
        let stock_projections = Self::generate_stock_projections(
            tracker,
            &method,
            &growth_scenario,
            projection_year,
        )?;

        // Calculate monthly breakdown
        let monthly_projections = Self::calculate_monthly_projections(&stock_projections)?;

        // Calculate total projected income
        let total_projected_income: Decimal = stock_projections
            .iter()
            .map(|sp| sp.projected_annual_dividend)
            .sum();

        // Generate metadata
        let metadata = Self::generate_metadata(tracker, &method, &stock_projections)?;

        Ok(DividendProjection {
            year: projection_year,
            total_projected_income,
            method,
            growth_scenario,
            monthly_projections,
            stock_projections,
            metadata,
        })
    }

    /// Generate projections for individual stocks
    fn generate_stock_projections(
        tracker: &DividendTracker,
        method: &ProjectionMethod,
        growth_scenario: &GrowthScenario,
        target_year: i32,
    ) -> Result<Vec<StockProjection>> {
        let mut projections = Vec::new();

        for (symbol, holding) in &tracker.holdings {
            if let Some(projection) = Self::project_stock_dividend(
                symbol,
                holding,
                &tracker.dividends,
                method,
                growth_scenario,
                target_year,
            )? {
                projections.push(projection);
            }
        }

        Ok(projections)
    }

    /// Project dividend for a single stock
    fn project_stock_dividend(
        symbol: &str,
        holding: &Holding,
        all_dividends: &[Dividend],
        method: &ProjectionMethod,
        growth_scenario: &GrowthScenario,
        target_year: i32,
    ) -> Result<Option<StockProjection>> {
        // Get historical dividends for this stock
        let historical_dividends: Vec<&Dividend> = all_dividends
            .iter()
            .filter(|d| d.symbol == symbol)
            .collect();

        if historical_dividends.is_empty() {
            // No historical data, cannot project
            return Ok(None);
        }

        // Calculate historical dividend per share based on method
        let historical_dividend_per_share = match method {
            ProjectionMethod::Last12Months => {
                Self::calculate_last_12_months_dividend(symbol, &historical_dividends)?
            }
            ProjectionMethod::AverageYears(years) => {
                Self::calculate_average_years_dividend(symbol, &historical_dividends, *years)?
            }
            ProjectionMethod::CurrentYield => {
                Self::calculate_current_yield_dividend(holding, &historical_dividends)?
            }
        };

        // Apply growth scenario
        let growth_rate = growth_scenario.get_growth_rate();
        let projected_dividend_per_share = historical_dividend_per_share * (dec!(1) + growth_rate);

        // Calculate total projected annual dividend
        let projected_annual_dividend = projected_dividend_per_share * holding.shares;

        // Analyze payment frequency and months
        let (payment_frequency, payment_months) = Self::analyze_payment_pattern(&historical_dividends)?;

        Ok(Some(StockProjection {
            symbol: symbol.to_string(),
            current_shares: holding.shares,
            projected_annual_dividend,
            historical_dividend_per_share,
            projected_dividend_per_share,
            growth_applied: growth_rate,
            payment_frequency,
            payment_months,
        }))
    }

    /// Calculate dividend based on last 12 months of payments
    fn calculate_last_12_months_dividend(
        _symbol: &str,
        dividends: &[&Dividend],
    ) -> Result<Decimal> {
        let cutoff_date = Local::now().naive_local().date() - chrono::Duration::days(365);

        let recent_dividends: Vec<&Dividend> = dividends
            .iter()
            .filter(|d| d.ex_date >= cutoff_date)
            .cloned()
            .collect();

        if recent_dividends.is_empty() {
            return Ok(dec!(0));
        }

        // Sum up all dividend payments in the last 12 months
        let total: Decimal = recent_dividends
            .iter()
            .map(|d| d.amount_per_share)
            .sum();

        Ok(total)
    }

    /// Calculate dividend based on average of last N years
    fn calculate_average_years_dividend(
        _symbol: &str,
        dividends: &[&Dividend],
        years: u32,
    ) -> Result<Decimal> {
        let current_year = Local::now().year();
        let start_year = current_year - years as i32;

        let mut yearly_totals: HashMap<i32, Decimal> = HashMap::new();

        // Group dividends by year and sum them
        for dividend in dividends {
            let year = dividend.ex_date.year();
            if year >= start_year && year < current_year {
                *yearly_totals.entry(year).or_insert(dec!(0)) += dividend.amount_per_share;
            }
        }

        if yearly_totals.is_empty() {
            return Ok(dec!(0));
        }

        // Calculate average
        let total: Decimal = yearly_totals.values().sum();
        let count = yearly_totals.len() as u32;

        Ok(total / Decimal::from(count))
    }

    /// Calculate dividend based on current yield
    fn calculate_current_yield_dividend(
        holding: &Holding,
        dividends: &[&Dividend],
    ) -> Result<Decimal> {
        // If holding has current_yield, use that
        if let Some(yield_rate) = holding.current_yield {
            if let Some(cost_basis) = holding.avg_cost_basis {
                return Ok(cost_basis * yield_rate / dec!(100));
            }
        }

        // Fallback to most recent dividend payment annualized
        if let Some(recent_dividend) = dividends.iter().max_by_key(|d| d.ex_date) {
            // Estimate annual dividend by analyzing payment frequency
            let (frequency, _) = Self::analyze_payment_pattern(dividends)?;
            let payments_per_year = Decimal::from(frequency.payments_per_year());
            return Ok(recent_dividend.amount_per_share * payments_per_year);
        }

        Ok(dec!(0))
    }

    /// Analyze payment pattern to determine frequency and typical months
    fn analyze_payment_pattern(dividends: &[&Dividend]) -> Result<(PaymentFrequency, Vec<u32>)> {
        if dividends.is_empty() {
            return Ok((PaymentFrequency::Irregular, vec![]));
        }

        // Sort dividends by ex_date
        let mut sorted_dividends = dividends.to_vec();
        sorted_dividends.sort_by_key(|d| d.ex_date);

        // Extract months of payments
        let payment_months: Vec<u32> = sorted_dividends
            .iter()
            .map(|d| d.ex_date.month())
            .collect();

        // Count unique months
        let unique_months: std::collections::HashSet<u32> = payment_months.iter().cloned().collect();

        // Determine frequency based on pattern
        let frequency = match unique_months.len() {
            1 => PaymentFrequency::Annual,
            2 => PaymentFrequency::SemiAnnual,
            3..=4 => PaymentFrequency::Quarterly,
            5..=12 => {
                if payment_months.len() >= 10 {
                    PaymentFrequency::Monthly
                } else {
                    PaymentFrequency::Quarterly
                }
            }
            _ => PaymentFrequency::Irregular,
        };

        // Return sorted unique months
        let mut months: Vec<u32> = unique_months.into_iter().collect();
        months.sort();

        Ok((frequency, months))
    }

    /// Calculate monthly breakdown of projected dividends
    fn calculate_monthly_projections(
        stock_projections: &[StockProjection],
    ) -> Result<HashMap<u32, MonthlyProjection>> {
        let mut monthly_totals: HashMap<u32, Decimal> = HashMap::new();
        let mut monthly_counts: HashMap<u32, usize> = HashMap::new();
        let mut monthly_payers: HashMap<u32, Vec<String>> = HashMap::new();

        for stock in stock_projections {
            // Distribute annual dividend across payment months
            let dividend_per_payment = stock.projected_annual_dividend /
                Decimal::from(stock.payment_months.len().max(1));

            for &month in &stock.payment_months {
                *monthly_totals.entry(month).or_insert(dec!(0)) += dividend_per_payment;
                *monthly_counts.entry(month).or_insert(0) += 1;
                monthly_payers.entry(month).or_insert_with(Vec::new).push(stock.symbol.clone());
            }
        }

        let mut projections = HashMap::new();

        for month in 1..=12 {
            let amount = monthly_totals.get(&month).cloned().unwrap_or(dec!(0));
            let count = monthly_counts.get(&month).cloned().unwrap_or(0);
            let payers = monthly_payers.get(&month).cloned().unwrap_or_default();

            let month_name = match month {
                1 => "January",
                2 => "February",
                3 => "March",
                4 => "April",
                5 => "May",
                6 => "June",
                7 => "July",
                8 => "August",
                9 => "September",
                10 => "October",
                11 => "November",
                12 => "December",
                _ => "Unknown",
            }.to_string();

            projections.insert(month, MonthlyProjection {
                month,
                month_name,
                projected_amount: amount,
                payment_count: count,
                top_payers: payers,
            });
        }

        Ok(projections)
    }

    /// Generate metadata about the projection
    fn generate_metadata(
        tracker: &DividendTracker,
        method: &ProjectionMethod,
        stock_projections: &[StockProjection],
    ) -> Result<ProjectionMetadata> {
        let data_points_used = tracker.dividends.len();

        let historical_range = if !tracker.dividends.is_empty() {
            let min_date = tracker.dividends.iter().map(|d| d.ex_date).min();
            let max_date = tracker.dividends.iter().map(|d| d.ex_date).max();
            (min_date, max_date)
        } else {
            (None, None)
        };

        let stocks_included = stock_projections.len();
        let stocks_excluded: Vec<String> = tracker.holdings
            .keys()
            .filter(|symbol| !stock_projections.iter().any(|sp| sp.symbol == **symbol))
            .cloned()
            .collect();

        // Calculate confidence score based on data availability
        let confidence_score = match method {
            ProjectionMethod::Last12Months => {
                if data_points_used >= 20 && stocks_excluded.is_empty() {
                    95
                } else if data_points_used >= 10 {
                    80
                } else {
                    60
                }
            }
            ProjectionMethod::AverageYears(_) => {
                if data_points_used >= 30 && stocks_excluded.is_empty() {
                    90
                } else if data_points_used >= 15 {
                    75
                } else {
                    55
                }
            }
            ProjectionMethod::CurrentYield => {
                if stocks_excluded.is_empty() {
                    85
                } else {
                    65
                }
            }
        };

        Ok(ProjectionMetadata {
            calculated_at: Local::now().to_rfc3339(),
            data_points_used,
            historical_range,
            stocks_included,
            stocks_excluded,
            confidence_score,
        })
    }

    /// Export projections to CSV format
    pub fn export_to_csv(projection: &DividendProjection, output_path: &str) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(output_path)?;

        // Write header
        writeln!(file, "Type,Symbol,Month,Amount,Details")?;

        // Write summary
        writeln!(file, "Summary,Portfolio,Annual,{:.2},Total Projected Income for {}",
                projection.total_projected_income, projection.year)?;

        // Write stock projections
        for stock in &projection.stock_projections {
            writeln!(file, "Stock,{},Annual,{:.2},Projected annual dividend",
                    stock.symbol, stock.projected_annual_dividend)?;
        }

        // Write monthly breakdown
        for month in 1..=12 {
            if let Some(monthly) = projection.monthly_projections.get(&month) {
                writeln!(file, "Monthly,Portfolio,{},{:.2},{}",
                        monthly.month_name, monthly.projected_amount,
                        monthly.top_payers.join("|"))?;
            }
        }

        // Write metadata
        writeln!(file, "Metadata,Method,,-,{:?}", projection.method)?;
        writeln!(file, "Metadata,Growth,,-,{}", projection.growth_scenario.name())?;
        writeln!(file, "Metadata,Confidence,,-,{}%", projection.metadata.confidence_score)?;

        Ok(())
    }

    /// Export projections to JSON format
    pub fn export_to_json(projection: &DividendProjection, output_path: &str) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        #[derive(serde::Serialize)]
        struct ExportProjection<'a> {
            year: i32,
            total_projected_income: Decimal,
            method: String,
            growth_scenario: String,
            stock_projections: &'a Vec<StockProjection>,
            monthly_breakdown: Vec<MonthlyExport>,
            metadata: &'a ProjectionMetadata,
        }

        #[derive(serde::Serialize)]
        struct MonthlyExport {
            month: u32,
            month_name: String,
            projected_amount: Decimal,
            payment_count: usize,
        }

        let monthly_breakdown: Vec<MonthlyExport> = (1..=12)
            .map(|month| {
                let monthly = projection.monthly_projections.get(&month);
                MonthlyExport {
                    month,
                    month_name: monthly.map(|m| m.month_name.clone()).unwrap_or_else(|| "Unknown".to_string()),
                    projected_amount: monthly.map(|m| m.projected_amount).unwrap_or(dec!(0)),
                    payment_count: monthly.map(|m| m.payment_count).unwrap_or(0),
                }
            })
            .collect();

        let export = ExportProjection {
            year: projection.year,
            total_projected_income: projection.total_projected_income,
            method: format!("{:?}", projection.method),
            growth_scenario: projection.growth_scenario.name(),
            stock_projections: &projection.stock_projections,
            monthly_breakdown,
            metadata: &projection.metadata,
        };

        let json = serde_json::to_string_pretty(&export)?;
        let mut file = File::create(output_path)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }
}

// Implement serde traits for JSON export
impl serde::Serialize for StockProjection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("StockProjection", 7)?;
        state.serialize_field("symbol", &self.symbol)?;
        state.serialize_field("current_shares", &self.current_shares)?;
        state.serialize_field("projected_annual_dividend", &self.projected_annual_dividend)?;
        state.serialize_field("historical_dividend_per_share", &self.historical_dividend_per_share)?;
        state.serialize_field("projected_dividend_per_share", &self.projected_dividend_per_share)?;
        state.serialize_field("growth_applied", &self.growth_applied)?;
        state.serialize_field("payment_frequency", self.payment_frequency.name())?;
        state.end()
    }
}

impl serde::Serialize for ProjectionMetadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ProjectionMetadata", 6)?;
        state.serialize_field("calculated_at", &self.calculated_at)?;
        state.serialize_field("data_points_used", &self.data_points_used)?;
        state.serialize_field("stocks_included", &self.stocks_included)?;
        state.serialize_field("stocks_excluded", &self.stocks_excluded)?;
        state.serialize_field("confidence_score", &self.confidence_score)?;
        state.serialize_field("historical_range_start", &self.historical_range.0)?;
        state.serialize_field("historical_range_end", &self.historical_range.1)?;
        state.end()
    }
}