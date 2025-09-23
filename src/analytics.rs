use anyhow::{anyhow, Result};
use chrono::{Datelike, Local, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

use crate::models::{Dividend, DividendTracker, Holding};

/// Analytics summary for dividend data
#[derive(Debug, Clone)]
pub struct DividendAnalytics {
    pub total_dividends: Decimal,
    pub total_payments: usize,
    pub unique_symbols: usize,
    pub monthly_breakdown: HashMap<u32, MonthlyDividendSummary>,
    pub quarterly_breakdown: HashMap<String, QuarterlyDividendSummary>,
    pub top_payers: Vec<StockDividendSummary>,
    pub frequency_analysis: FrequencyAnalysis,
    pub consistency_analysis: ConsistencyAnalysis,
    pub yield_analysis: Option<YieldAnalysis>,
    pub growth_analysis: Option<GrowthAnalysis>,
}

#[derive(Debug, Clone)]
pub struct MonthlyDividendSummary {
    pub month: u32,
    pub total_amount: Decimal,
    pub payment_count: usize,
    pub unique_symbols: usize,
    pub top_symbol: Option<String>,
    pub top_amount: Decimal,
}

#[derive(Debug, Clone)]
pub struct QuarterlyDividendSummary {
    pub quarter: String,
    pub total_amount: Decimal,
    pub payment_count: usize,
    pub unique_symbols: usize,
    pub months: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct StockDividendSummary {
    pub symbol: String,
    pub total_amount: Decimal,
    pub payment_count: usize,
    pub average_amount: Decimal,
    pub first_payment: NaiveDate,
    pub last_payment: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct FrequencyAnalysis {
    pub monthly_payers: Vec<String>,
    pub quarterly_payers: Vec<String>,
    pub semi_annual_payers: Vec<String>,
    pub annual_payers: Vec<String>,
    pub irregular_payers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ConsistencyAnalysis {
    pub consistent_payers: Vec<ConsistentPayer>,
    pub inconsistent_payers: Vec<String>,
    pub average_consistency_score: f64,
}

#[derive(Debug, Clone)]
pub struct ConsistentPayer {
    pub symbol: String,
    pub consistency_score: f64,
    pub payment_intervals: Vec<i64>,
    pub expected_frequency: String,
}

#[derive(Debug, Clone)]
pub struct YieldAnalysis {
    pub average_yield: Decimal,
    pub stock_yields: Vec<StockYield>,
    pub highest_yielding: Option<StockYield>,
    pub lowest_yielding: Option<StockYield>,
}

#[derive(Debug, Clone)]
pub struct StockYield {
    pub symbol: String,
    pub annual_dividend: Decimal,
    pub cost_basis: Decimal,
    pub shares: Decimal,
    pub yield_percent: Decimal,
}

#[derive(Debug, Clone)]
pub struct GrowthAnalysis {
    pub year_over_year: Vec<YearlyGrowth>,
    pub total_growth_rate: Decimal,
    pub average_annual_growth: Decimal,
    pub best_year: Option<YearlyGrowth>,
    pub worst_year: Option<YearlyGrowth>,
}

#[derive(Debug, Clone)]
pub struct YearlyGrowth {
    pub year: i32,
    pub total_dividends: Decimal,
    pub growth_rate: Option<Decimal>,
    pub payment_count: usize,
}

impl DividendAnalytics {
    /// Generate comprehensive analytics from dividend tracker data
    pub fn generate(
        tracker: &DividendTracker,
        year_filter: Option<i32>,
        quarter_filter: Option<&str>,
    ) -> Result<Self> {
        let current_year = Local::now().year();
        let target_year = year_filter.unwrap_or(current_year);

        // Filter dividends based on criteria
        let mut filtered_dividends = Vec::new();
        for div in &tracker.dividends {
            // Check year filter
            if let Some(year) = year_filter {
                if div.ex_date.year() != year {
                    continue;
                }
            }

            // Check quarter filter
            if let Some(quarter) = quarter_filter {
                if !Self::is_in_quarter(div.ex_date, quarter)? {
                    continue;
                }
            }

            filtered_dividends.push(div);
        }

        let total_dividends: Decimal = filtered_dividends.iter().map(|d| d.total_amount).sum();
        let total_payments = filtered_dividends.len();
        let unique_symbols = filtered_dividends
            .iter()
            .map(|d| &d.symbol)
            .collect::<std::collections::HashSet<_>>()
            .len();

        let monthly_breakdown = Self::calculate_monthly_breakdown(&filtered_dividends, target_year)?;
        let quarterly_breakdown = Self::calculate_quarterly_breakdown(&filtered_dividends, target_year)?;
        let top_payers = Self::calculate_top_payers(&tracker.dividends)?;
        let frequency_analysis = Self::analyze_frequency(&tracker.dividends)?;
        let consistency_analysis = Self::analyze_consistency(&tracker.dividends)?;
        let yield_analysis = Self::analyze_yields(tracker)?;
        let growth_analysis = Self::analyze_growth(&tracker.dividends)?;

        Ok(DividendAnalytics {
            total_dividends,
            total_payments,
            unique_symbols,
            monthly_breakdown,
            quarterly_breakdown,
            top_payers,
            frequency_analysis,
            consistency_analysis,
            yield_analysis,
            growth_analysis,
        })
    }

    fn is_in_quarter(date: NaiveDate, quarter: &str) -> Result<bool> {
        // Parse quarter format like "Q1-2024"
        let parts: Vec<&str> = quarter.split('-').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid quarter format. Use Q1-2024, Q2-2024, etc."));
        }

        let quarter_num = match parts[0] {
            "Q1" => 1,
            "Q2" => 2,
            "Q3" => 3,
            "Q4" => 4,
            _ => return Err(anyhow!("Invalid quarter. Use Q1, Q2, Q3, or Q4")),
        };

        let year: i32 = parts[1].parse()
            .map_err(|_| anyhow!("Invalid year in quarter format"))?;

        let month = date.month();
        let date_year = date.year();

        if date_year != year {
            return Ok(false);
        }

        let in_quarter = match quarter_num {
            1 => month >= 1 && month <= 3,
            2 => month >= 4 && month <= 6,
            3 => month >= 7 && month <= 9,
            4 => month >= 10 && month <= 12,
            _ => false,
        };

        Ok(in_quarter)
    }

    fn calculate_monthly_breakdown(
        dividends: &[&Dividend],
        year: i32,
    ) -> Result<HashMap<u32, MonthlyDividendSummary>> {
        let mut monthly_data: HashMap<u32, Vec<&Dividend>> = HashMap::new();

        for dividend in dividends {
            if dividend.ex_date.year() == year {
                monthly_data
                    .entry(dividend.ex_date.month())
                    .or_insert_with(Vec::new)
                    .push(dividend);
            }
        }

        let mut breakdown = HashMap::new();
        for (month, month_dividends) in monthly_data {
            let total_amount: Decimal = month_dividends.iter().map(|d| d.total_amount).sum();
            let payment_count = month_dividends.len();
            let unique_symbols = month_dividends
                .iter()
                .map(|d| &d.symbol)
                .collect::<std::collections::HashSet<_>>()
                .len();

            // Find top paying stock for the month
            let mut symbol_totals: HashMap<&str, Decimal> = HashMap::new();
            for dividend in &month_dividends {
                *symbol_totals.entry(&dividend.symbol).or_insert(dec!(0)) += dividend.total_amount;
            }

            let (top_symbol, top_amount) = symbol_totals
                .iter()
                .max_by_key(|(_, amount)| *amount)
                .map(|(sym, amt)| (Some(sym.to_string()), *amt))
                .unwrap_or((None, dec!(0)));

            breakdown.insert(
                month,
                MonthlyDividendSummary {
                    month,
                    total_amount,
                    payment_count,
                    unique_symbols,
                    top_symbol,
                    top_amount,
                },
            );
        }

        Ok(breakdown)
    }

    fn calculate_quarterly_breakdown(
        dividends: &[&Dividend],
        year: i32,
    ) -> Result<HashMap<String, QuarterlyDividendSummary>> {
        let mut breakdown = HashMap::new();

        for quarter_num in 1..=4 {
            let quarter_name = format!("Q{}-{}", quarter_num, year);
            let months = match quarter_num {
                1 => vec![1, 2, 3],
                2 => vec![4, 5, 6],
                3 => vec![7, 8, 9],
                4 => vec![10, 11, 12],
                _ => continue,
            };

            let quarter_dividends: Vec<&Dividend> = dividends
                .iter()
                .filter(|d| {
                    d.ex_date.year() == year && months.contains(&d.ex_date.month())
                })
                .copied()
                .collect();

            if !quarter_dividends.is_empty() {
                let total_amount: Decimal = quarter_dividends.iter().map(|d| d.total_amount).sum();
                let payment_count = quarter_dividends.len();
                let unique_symbols = quarter_dividends
                    .iter()
                    .map(|d| &d.symbol)
                    .collect::<std::collections::HashSet<_>>()
                    .len();

                breakdown.insert(
                    quarter_name.clone(),
                    QuarterlyDividendSummary {
                        quarter: quarter_name,
                        total_amount,
                        payment_count,
                        unique_symbols,
                        months,
                    },
                );
            }
        }

        Ok(breakdown)
    }

    fn calculate_top_payers(dividends: &[Dividend]) -> Result<Vec<StockDividendSummary>> {
        let mut stock_summaries: HashMap<String, Vec<&Dividend>> = HashMap::new();

        for dividend in dividends {
            stock_summaries
                .entry(dividend.symbol.clone())
                .or_insert_with(Vec::new)
                .push(dividend);
        }

        let mut summaries: Vec<StockDividendSummary> = stock_summaries
            .into_iter()
            .map(|(symbol, dividends)| {
                let total_amount: Decimal = dividends.iter().map(|d| d.total_amount).sum();
                let payment_count = dividends.len();
                let average_amount = if payment_count > 0 {
                    total_amount / Decimal::from(payment_count)
                } else {
                    dec!(0)
                };

                let dates: Vec<NaiveDate> = dividends.iter().map(|d| d.ex_date).collect();
                let first_payment = *dates.iter().min().unwrap();
                let last_payment = *dates.iter().max().unwrap();

                StockDividendSummary {
                    symbol,
                    total_amount,
                    payment_count,
                    average_amount,
                    first_payment,
                    last_payment,
                }
            })
            .collect();

        summaries.sort_by(|a, b| b.total_amount.cmp(&a.total_amount));
        Ok(summaries)
    }

    fn analyze_frequency(dividends: &[Dividend]) -> Result<FrequencyAnalysis> {
        let mut stock_payments: HashMap<String, Vec<NaiveDate>> = HashMap::new();

        for dividend in dividends {
            stock_payments
                .entry(dividend.symbol.clone())
                .or_insert_with(Vec::new)
                .push(dividend.ex_date);
        }

        let mut monthly_payers = Vec::new();
        let mut quarterly_payers = Vec::new();
        let mut semi_annual_payers = Vec::new();
        let mut annual_payers = Vec::new();
        let mut irregular_payers = Vec::new();

        for (symbol, mut dates) in stock_payments {
            dates.sort();

            if dates.len() < 2 {
                irregular_payers.push(symbol);
                continue;
            }

            let intervals: Vec<i64> = dates
                .windows(2)
                .map(|window| (window[1] - window[0]).num_days())
                .collect();

            let average_interval = intervals.iter().sum::<i64>() as f64 / intervals.len() as f64;

            // Classify based on average interval
            match average_interval.round() as i64 {
                20..=40 => monthly_payers.push(symbol),    // ~30 days
                80..=100 => quarterly_payers.push(symbol), // ~90 days
                170..=200 => semi_annual_payers.push(symbol), // ~180 days
                350..=380 => annual_payers.push(symbol),   // ~365 days
                _ => irregular_payers.push(symbol),
            }
        }

        Ok(FrequencyAnalysis {
            monthly_payers,
            quarterly_payers,
            semi_annual_payers,
            annual_payers,
            irregular_payers,
        })
    }

    fn analyze_consistency(dividends: &[Dividend]) -> Result<ConsistencyAnalysis> {
        let mut stock_payments: HashMap<String, Vec<NaiveDate>> = HashMap::new();

        for dividend in dividends {
            stock_payments
                .entry(dividend.symbol.clone())
                .or_insert_with(Vec::new)
                .push(dividend.ex_date);
        }

        let mut consistent_payers = Vec::new();
        let mut inconsistent_payers = Vec::new();
        let mut total_consistency_score = 0.0;
        let mut stock_count = 0;

        for (symbol, mut dates) in stock_payments {
            dates.sort();

            if dates.len() < 3 {
                inconsistent_payers.push(symbol);
                continue;
            }

            let intervals: Vec<i64> = dates
                .windows(2)
                .map(|window| (window[1] - window[0]).num_days())
                .collect();

            // Calculate consistency score (lower variance = higher consistency)
            let mean_interval = intervals.iter().sum::<i64>() as f64 / intervals.len() as f64;
            let variance = intervals
                .iter()
                .map(|interval| {
                    let diff = *interval as f64 - mean_interval;
                    diff * diff
                })
                .sum::<f64>()
                / intervals.len() as f64;

            let std_deviation = variance.sqrt();
            let consistency_score = if mean_interval > 0.0 {
                100.0 * (1.0 - (std_deviation / mean_interval).min(1.0))
            } else {
                0.0
            };

            total_consistency_score += consistency_score;
            stock_count += 1;

            let expected_frequency = match mean_interval.round() as i64 {
                20..=40 => "Monthly".to_string(),
                80..=100 => "Quarterly".to_string(),
                170..=200 => "Semi-Annual".to_string(),
                350..=380 => "Annual".to_string(),
                _ => "Irregular".to_string(),
            };

            if consistency_score >= 70.0 {
                consistent_payers.push(ConsistentPayer {
                    symbol,
                    consistency_score,
                    payment_intervals: intervals,
                    expected_frequency,
                });
            } else {
                inconsistent_payers.push(symbol);
            }
        }

        let average_consistency_score = if stock_count > 0 {
            total_consistency_score / stock_count as f64
        } else {
            0.0
        };

        Ok(ConsistencyAnalysis {
            consistent_payers,
            inconsistent_payers,
            average_consistency_score,
        })
    }

    fn analyze_yields(tracker: &DividendTracker) -> Result<Option<YieldAnalysis>> {
        // Only analyze yields if we have holdings with cost basis
        let holdings_with_cost: Vec<(&String, &Holding)> = tracker
            .holdings
            .iter()
            .filter(|(_, holding)| holding.avg_cost_basis.is_some())
            .collect();

        if holdings_with_cost.is_empty() {
            return Ok(None);
        }

        let mut stock_yields = Vec::new();
        let current_year = Local::now().year();

        for (symbol, holding) in holdings_with_cost {
            if let Some(cost_basis) = holding.avg_cost_basis {
                // Calculate annual dividend for this stock
                let annual_dividend: Decimal = tracker
                    .dividends
                    .iter()
                    .filter(|d| {
                        d.symbol == *symbol && d.ex_date.year() == current_year
                    })
                    .map(|d| d.amount_per_share)
                    .sum();

                if annual_dividend > dec!(0) {
                    let yield_percent = (annual_dividend / cost_basis) * dec!(100);

                    stock_yields.push(StockYield {
                        symbol: symbol.clone(),
                        annual_dividend,
                        cost_basis,
                        shares: holding.shares,
                        yield_percent,
                    });
                }
            }
        }

        if stock_yields.is_empty() {
            return Ok(None);
        }

        stock_yields.sort_by(|a, b| b.yield_percent.cmp(&a.yield_percent));

        let average_yield = stock_yields.iter().map(|y| y.yield_percent).sum::<Decimal>()
            / Decimal::from(stock_yields.len());

        let highest_yielding = stock_yields.first().cloned();
        let lowest_yielding = stock_yields.last().cloned();

        Ok(Some(YieldAnalysis {
            average_yield,
            stock_yields,
            highest_yielding,
            lowest_yielding,
        }))
    }

    fn analyze_growth(dividends: &[Dividend]) -> Result<Option<GrowthAnalysis>> {
        let mut yearly_totals: HashMap<i32, (Decimal, usize)> = HashMap::new();

        for dividend in dividends {
            let year = dividend.ex_date.year();
            let entry = yearly_totals.entry(year).or_insert((dec!(0), 0));
            entry.0 += dividend.total_amount;
            entry.1 += 1;
        }

        if yearly_totals.len() < 2 {
            return Ok(None);
        }

        let mut yearly_growth: Vec<YearlyGrowth> = yearly_totals
            .into_iter()
            .map(|(year, (total, count))| YearlyGrowth {
                year,
                total_dividends: total,
                growth_rate: None,
                payment_count: count,
            })
            .collect();

        yearly_growth.sort_by_key(|y| y.year);

        // Calculate growth rates
        for i in 1..yearly_growth.len() {
            let current = yearly_growth[i].total_dividends;
            let previous = yearly_growth[i - 1].total_dividends;

            if previous > dec!(0) {
                let growth_rate = ((current - previous) / previous) * dec!(100);
                yearly_growth[i].growth_rate = Some(growth_rate);
            }
        }

        let total_growth_rate = if let (Some(first), Some(last)) = (yearly_growth.first(), yearly_growth.last()) {
            if first.total_dividends > dec!(0) {
                ((last.total_dividends - first.total_dividends) / first.total_dividends) * dec!(100)
            } else {
                dec!(0)
            }
        } else {
            dec!(0)
        };

        let growth_rates: Vec<Decimal> = yearly_growth
            .iter()
            .filter_map(|y| y.growth_rate)
            .collect();

        let average_annual_growth = if !growth_rates.is_empty() {
            growth_rates.iter().sum::<Decimal>() / Decimal::from(growth_rates.len())
        } else {
            dec!(0)
        };

        let best_year = yearly_growth
            .iter()
            .filter(|y| y.growth_rate.is_some())
            .max_by_key(|y| y.growth_rate.unwrap())
            .cloned();

        let worst_year = yearly_growth
            .iter()
            .filter(|y| y.growth_rate.is_some())
            .min_by_key(|y| y.growth_rate.unwrap())
            .cloned();

        Ok(Some(GrowthAnalysis {
            year_over_year: yearly_growth,
            total_growth_rate,
            average_annual_growth,
            best_year,
            worst_year,
        }))
    }

    /// Export analytics data to CSV
    pub fn export_to_csv(&self, file_path: &str) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(file_path)?;

        // Write headers
        writeln!(
            file,
            "Metric,Value,Details"
        )?;

        // Summary data
        writeln!(
            file,
            "Total Dividends,${:.2},",
            self.total_dividends
        )?;
        writeln!(
            file,
            "Total Payments,{},",
            self.total_payments
        )?;
        writeln!(
            file,
            "Unique Symbols,{},",
            self.unique_symbols
        )?;

        // Monthly breakdown
        let mut months: Vec<_> = self.monthly_breakdown.keys().collect();
        months.sort();
        for month in months {
            let summary = &self.monthly_breakdown[month];
            writeln!(
                file,
                "Month {} Total,${:.2},{} payments from {} stocks",
                month,
                summary.total_amount,
                summary.payment_count,
                summary.unique_symbols
            )?;
        }

        // Top payers
        for (i, payer) in self.top_payers.iter().take(10).enumerate() {
            writeln!(
                file,
                "Top Payer #{},{},${:.2} from {} payments",
                i + 1,
                payer.symbol,
                payer.total_amount,
                payer.payment_count
            )?;
        }

        Ok(())
    }
}