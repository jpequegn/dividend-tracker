use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::{Dividend, DividendTracker, TaxClassification};

/// Tax summary for a specific tax year
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxSummary {
    /// Tax year
    pub tax_year: i32,
    /// Total dividend income for the year
    pub total_dividend_income: Decimal,
    /// Qualified dividend income (eligible for capital gains rates)
    pub qualified_dividends: Decimal,
    /// Non-qualified dividend income (taxed as ordinary income)
    pub non_qualified_dividends: Decimal,
    /// Return of capital (not taxable as income)
    pub return_of_capital: Decimal,
    /// Tax-free dividends
    pub tax_free_dividends: Decimal,
    /// Foreign dividends with breakdown
    pub foreign_dividends: ForeignDividendSummary,
    /// Breakdown by stock symbol
    pub by_symbol: HashMap<String, SymbolTaxSummary>,
    /// Tax lot breakdown (if available)
    pub tax_lots: Vec<TaxLotSummary>,
    /// Estimated tax information
    pub estimated_tax: Option<EstimatedTax>,
}

/// Foreign dividend summary with withholding tax information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignDividendSummary {
    /// Total foreign dividend income
    pub total_foreign_income: Decimal,
    /// Total withholding tax paid
    pub total_withholding_tax: Decimal,
    /// Net foreign dividend income (after withholding)
    pub net_foreign_income: Decimal,
    /// Breakdown by country (if available)
    pub by_country: HashMap<String, CountryTaxSummary>,
}

/// Tax summary for a specific country
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryTaxSummary {
    /// Country code or name
    pub country: String,
    /// Total dividend income from this country
    pub dividend_income: Decimal,
    /// Withholding tax paid to this country
    pub withholding_tax: Decimal,
    /// Net income after withholding
    pub net_income: Decimal,
}

/// Tax summary for a specific stock symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolTaxSummary {
    /// Stock symbol
    pub symbol: String,
    /// Company name (if available)
    pub company_name: Option<String>,
    /// Total dividend income from this symbol
    pub total_income: Decimal,
    /// Qualified dividend amount
    pub qualified_amount: Decimal,
    /// Non-qualified dividend amount
    pub non_qualified_amount: Decimal,
    /// Return of capital amount
    pub return_of_capital_amount: Decimal,
    /// Number of dividend payments
    pub payment_count: usize,
    /// First payment date
    pub first_payment: Option<NaiveDate>,
    /// Last payment date
    pub last_payment: Option<NaiveDate>,
}

/// Tax lot summary for cost basis tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxLotSummary {
    /// Tax lot identifier
    pub tax_lot_id: String,
    /// Stock symbol
    pub symbol: String,
    /// Total dividend income from this lot
    pub dividend_income: Decimal,
    /// Number of shares in the lot
    pub shares: Option<Decimal>,
    /// Purchase date of the lot
    pub purchase_date: Option<NaiveDate>,
    /// Cost basis per share
    pub cost_basis_per_share: Option<Decimal>,
}

/// Estimated tax calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimatedTax {
    /// Estimated tax on qualified dividends (capital gains rates)
    pub qualified_tax: Decimal,
    /// Estimated tax on non-qualified dividends (ordinary rates)
    pub non_qualified_tax: Decimal,
    /// Total estimated tax
    pub total_estimated_tax: Decimal,
    /// Tax bracket used for ordinary income
    pub ordinary_tax_bracket: Decimal,
    /// Capital gains tax rate used
    pub capital_gains_rate: Decimal,
    /// Marginal tax rate assumptions
    pub tax_assumptions: TaxAssumptions,
}

/// Tax rate assumptions for calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxAssumptions {
    /// Filing status (Single, MarriedFilingJointly, etc.)
    pub filing_status: FilingStatus,
    /// Estimated annual income bracket
    pub income_bracket: IncomeBracket,
    /// Year for tax rates
    pub tax_year: i32,
}

/// Tax filing status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilingStatus {
    Single,
    MarriedFilingJointly,
    MarriedFilingSeparately,
    HeadOfHousehold,
}

/// Income bracket for tax rate estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncomeBracket {
    Low,      // Under $44,725 (2023 rates for single)
    Medium,   // $44,725 - $95,375
    High,     // $95,375 - $182,050
    VeryHigh, // Over $182,050
}

/// 1099-DIV style report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form1099DIV {
    /// Tax year
    pub tax_year: i32,
    /// Payer information (aggregated by company)
    pub payers: Vec<PayerInfo>,
    /// Summary totals
    pub summary: Form1099Summary,
}

/// Payer information for 1099-DIV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayerInfo {
    /// Company/Payer name
    pub payer_name: String,
    /// Stock symbol(s)
    pub symbols: Vec<String>,
    /// Box 1a: Total ordinary dividends
    pub total_ordinary_dividends: Decimal,
    /// Box 1b: Qualified dividends
    pub qualified_dividends: Decimal,
    /// Box 2a: Total capital gain distributions
    pub capital_gain_distributions: Decimal,
    /// Box 3: Non-dividend distributions
    pub non_dividend_distributions: Decimal,
    /// Box 4: Federal income tax withheld
    pub federal_tax_withheld: Decimal,
    /// Box 6: Foreign tax paid
    pub foreign_tax_paid: Decimal,
    /// Box 7: Foreign country or U.S. possession
    pub foreign_country: Option<String>,
}

/// Summary totals for 1099-DIV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form1099Summary {
    /// Total across all payers - Box 1a
    pub total_ordinary_dividends: Decimal,
    /// Total across all payers - Box 1b
    pub total_qualified_dividends: Decimal,
    /// Total across all payers - Box 2a
    pub total_capital_gain_distributions: Decimal,
    /// Total across all payers - Box 3
    pub total_non_dividend_distributions: Decimal,
    /// Total across all payers - Box 4
    pub total_federal_tax_withheld: Decimal,
    /// Total across all payers - Box 6
    pub total_foreign_tax_paid: Decimal,
}

/// Tax analysis engine
pub struct TaxAnalyzer;

impl TaxAnalyzer {
    /// Generate comprehensive tax summary for a given year
    pub fn generate_tax_summary(
        tracker: &DividendTracker,
        tax_year: i32,
        tax_assumptions: Option<TaxAssumptions>,
    ) -> Result<TaxSummary> {
        // Filter dividends for the tax year (by pay date)
        let tax_year_dividends: Vec<&Dividend> = tracker
            .dividends
            .iter()
            .filter(|d| d.pay_date.year() == tax_year)
            .collect();

        if tax_year_dividends.is_empty() {
            return Ok(TaxSummary {
                tax_year,
                total_dividend_income: dec!(0),
                qualified_dividends: dec!(0),
                non_qualified_dividends: dec!(0),
                return_of_capital: dec!(0),
                tax_free_dividends: dec!(0),
                foreign_dividends: ForeignDividendSummary {
                    total_foreign_income: dec!(0),
                    total_withholding_tax: dec!(0),
                    net_foreign_income: dec!(0),
                    by_country: HashMap::new(),
                },
                by_symbol: HashMap::new(),
                tax_lots: Vec::new(),
                estimated_tax: None,
            });
        }

        // Calculate totals by tax classification
        let mut qualified_total = dec!(0);
        let mut non_qualified_total = dec!(0);
        let mut return_of_capital_total = dec!(0);
        let mut tax_free_total = dec!(0);
        let mut foreign_total = dec!(0);
        let mut total_withholding = dec!(0);

        let mut by_symbol: HashMap<String, SymbolTaxSummary> = HashMap::new();
        let mut tax_lots: Vec<TaxLotSummary> = Vec::new();

        for dividend in &tax_year_dividends {
            // Add to appropriate total based on tax classification
            match dividend.tax_classification {
                TaxClassification::Qualified => qualified_total += dividend.total_amount,
                TaxClassification::NonQualified => non_qualified_total += dividend.total_amount,
                TaxClassification::ReturnOfCapital => return_of_capital_total += dividend.total_amount,
                TaxClassification::TaxFree => tax_free_total += dividend.total_amount,
                TaxClassification::Foreign => {
                    foreign_total += dividend.total_amount;
                    if let Some(withholding) = dividend.withholding_tax {
                        total_withholding += withholding;
                    }
                }
                TaxClassification::Unknown => {
                    // For unknown classification, assume qualified for common stocks
                    qualified_total += dividend.total_amount;
                }
            }

            // Update symbol summary
            let symbol_summary = by_symbol.entry(dividend.symbol.clone()).or_insert(SymbolTaxSummary {
                symbol: dividend.symbol.clone(),
                company_name: dividend.company_name.clone(),
                total_income: dec!(0),
                qualified_amount: dec!(0),
                non_qualified_amount: dec!(0),
                return_of_capital_amount: dec!(0),
                payment_count: 0,
                first_payment: None,
                last_payment: None,
            });

            symbol_summary.total_income += dividend.total_amount;
            symbol_summary.payment_count += 1;

            // Update first/last payment dates
            if symbol_summary.first_payment.is_none() || dividend.pay_date < symbol_summary.first_payment.unwrap() {
                symbol_summary.first_payment = Some(dividend.pay_date);
            }
            if symbol_summary.last_payment.is_none() || dividend.pay_date > symbol_summary.last_payment.unwrap() {
                symbol_summary.last_payment = Some(dividend.pay_date);
            }

            // Update classification amounts
            match dividend.tax_classification {
                TaxClassification::Qualified => symbol_summary.qualified_amount += dividend.total_amount,
                TaxClassification::NonQualified => symbol_summary.non_qualified_amount += dividend.total_amount,
                TaxClassification::ReturnOfCapital => symbol_summary.return_of_capital_amount += dividend.total_amount,
                TaxClassification::Unknown => symbol_summary.qualified_amount += dividend.total_amount, // Assume qualified
                _ => {} // Other classifications don't go into these buckets
            }

            // Handle tax lots if available
            if let Some(tax_lot_id) = &dividend.tax_lot_id {
                // Check if we already have a summary for this tax lot
                if let Some(existing_lot) = tax_lots.iter_mut().find(|lot| lot.tax_lot_id == *tax_lot_id) {
                    existing_lot.dividend_income += dividend.total_amount;
                } else {
                    tax_lots.push(TaxLotSummary {
                        tax_lot_id: tax_lot_id.clone(),
                        symbol: dividend.symbol.clone(),
                        dividend_income: dividend.total_amount,
                        shares: None, // Would need additional data
                        purchase_date: None, // Would need additional data
                        cost_basis_per_share: None, // Would need additional data
                    });
                }
            }
        }

        let total_dividend_income = qualified_total + non_qualified_total + return_of_capital_total + tax_free_total + foreign_total;

        // Create foreign dividend summary
        let foreign_dividends = ForeignDividendSummary {
            total_foreign_income: foreign_total,
            total_withholding_tax: total_withholding,
            net_foreign_income: foreign_total - total_withholding,
            by_country: HashMap::new(), // Would need country data in dividends
        };

        // Calculate estimated tax if assumptions provided
        let estimated_tax = if let Some(assumptions) = tax_assumptions {
            Some(Self::calculate_estimated_tax(
                qualified_total,
                non_qualified_total,
                &assumptions,
            )?)
        } else {
            None
        };

        Ok(TaxSummary {
            tax_year,
            total_dividend_income,
            qualified_dividends: qualified_total,
            non_qualified_dividends: non_qualified_total,
            return_of_capital: return_of_capital_total,
            tax_free_dividends: tax_free_total,
            foreign_dividends,
            by_symbol,
            tax_lots,
            estimated_tax,
        })
    }

    /// Calculate estimated tax based on dividend income and tax assumptions
    pub fn calculate_estimated_tax(
        qualified_amount: Decimal,
        non_qualified_amount: Decimal,
        assumptions: &TaxAssumptions,
    ) -> Result<EstimatedTax> {
        // Get tax rates based on assumptions
        let (ordinary_rate, capital_gains_rate) = Self::get_tax_rates(assumptions)?;

        // Calculate taxes
        let qualified_tax = qualified_amount * capital_gains_rate;
        let non_qualified_tax = non_qualified_amount * ordinary_rate;
        let total_estimated_tax = qualified_tax + non_qualified_tax;

        Ok(EstimatedTax {
            qualified_tax,
            non_qualified_tax,
            total_estimated_tax,
            ordinary_tax_bracket: ordinary_rate,
            capital_gains_rate,
            tax_assumptions: assumptions.clone(),
        })
    }

    /// Get tax rates based on filing status and income bracket
    /// Note: These are approximate 2023 tax rates for estimation purposes
    fn get_tax_rates(assumptions: &TaxAssumptions) -> Result<(Decimal, Decimal)> {
        let (ordinary_rate, capital_gains_rate) = match (&assumptions.filing_status, &assumptions.income_bracket) {
            (FilingStatus::Single, IncomeBracket::Low) => (dec!(0.12), dec!(0.0)),    // 12% ordinary, 0% capital gains
            (FilingStatus::Single, IncomeBracket::Medium) => (dec!(0.22), dec!(0.15)), // 22% ordinary, 15% capital gains
            (FilingStatus::Single, IncomeBracket::High) => (dec!(0.24), dec!(0.15)),   // 24% ordinary, 15% capital gains
            (FilingStatus::Single, IncomeBracket::VeryHigh) => (dec!(0.32), dec!(0.20)), // 32% ordinary, 20% capital gains

            (FilingStatus::MarriedFilingJointly, IncomeBracket::Low) => (dec!(0.12), dec!(0.0)),
            (FilingStatus::MarriedFilingJointly, IncomeBracket::Medium) => (dec!(0.22), dec!(0.15)),
            (FilingStatus::MarriedFilingJointly, IncomeBracket::High) => (dec!(0.24), dec!(0.15)),
            (FilingStatus::MarriedFilingJointly, IncomeBracket::VeryHigh) => (dec!(0.32), dec!(0.20)),

            (FilingStatus::MarriedFilingSeparately, IncomeBracket::Low) => (dec!(0.12), dec!(0.0)),
            (FilingStatus::MarriedFilingSeparately, IncomeBracket::Medium) => (dec!(0.22), dec!(0.15)),
            (FilingStatus::MarriedFilingSeparately, IncomeBracket::High) => (dec!(0.24), dec!(0.15)),
            (FilingStatus::MarriedFilingSeparately, IncomeBracket::VeryHigh) => (dec!(0.32), dec!(0.20)),

            (FilingStatus::HeadOfHousehold, IncomeBracket::Low) => (dec!(0.12), dec!(0.0)),
            (FilingStatus::HeadOfHousehold, IncomeBracket::Medium) => (dec!(0.22), dec!(0.15)),
            (FilingStatus::HeadOfHousehold, IncomeBracket::High) => (dec!(0.24), dec!(0.15)),
            (FilingStatus::HeadOfHousehold, IncomeBracket::VeryHigh) => (dec!(0.32), dec!(0.20)),
        };

        Ok((ordinary_rate, capital_gains_rate))
    }

    /// Generate 1099-DIV style report
    pub fn generate_1099_div_report(
        tracker: &DividendTracker,
        tax_year: i32,
    ) -> Result<Form1099DIV> {
        let tax_summary = Self::generate_tax_summary(tracker, tax_year, None)?;

        let mut payers: Vec<PayerInfo> = Vec::new();

        // Group by symbol/company for 1099-DIV format
        for (symbol, symbol_summary) in &tax_summary.by_symbol {
            let payer_name = symbol_summary.company_name.clone()
                .unwrap_or_else(|| format!("{} Corporation", symbol));

            let payer = PayerInfo {
                payer_name,
                symbols: vec![symbol.clone()],
                total_ordinary_dividends: symbol_summary.qualified_amount + symbol_summary.non_qualified_amount,
                qualified_dividends: symbol_summary.qualified_amount,
                capital_gain_distributions: dec!(0), // Would need separate tracking
                non_dividend_distributions: symbol_summary.return_of_capital_amount,
                federal_tax_withheld: dec!(0), // Would need separate tracking
                foreign_tax_paid: dec!(0), // Would need foreign dividend details
                foreign_country: None,
            };

            payers.push(payer);
        }

        // Calculate summary totals
        let summary = Form1099Summary {
            total_ordinary_dividends: tax_summary.qualified_dividends + tax_summary.non_qualified_dividends,
            total_qualified_dividends: tax_summary.qualified_dividends,
            total_capital_gain_distributions: dec!(0),
            total_non_dividend_distributions: tax_summary.return_of_capital,
            total_federal_tax_withheld: dec!(0),
            total_foreign_tax_paid: tax_summary.foreign_dividends.total_withholding_tax,
        };

        Ok(Form1099DIV {
            tax_year,
            payers,
            summary,
        })
    }

    /// Export tax summary to CSV format
    pub fn export_tax_summary_csv(summary: &TaxSummary, file_path: &str) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(file_path)?;

        // Write header
        writeln!(file, "Tax Year,{}", summary.tax_year)?;
        writeln!(file, "")?;

        // Write summary section
        writeln!(file, "Summary")?;
        writeln!(file, "Category,Amount")?;
        writeln!(file, "Total Dividend Income,{}", summary.total_dividend_income)?;
        writeln!(file, "Qualified Dividends,{}", summary.qualified_dividends)?;
        writeln!(file, "Non-Qualified Dividends,{}", summary.non_qualified_dividends)?;
        writeln!(file, "Return of Capital,{}", summary.return_of_capital)?;
        writeln!(file, "Tax-Free Dividends,{}", summary.tax_free_dividends)?;
        writeln!(file, "Foreign Dividends,{}", summary.foreign_dividends.total_foreign_income)?;
        writeln!(file, "")?;

        // Write by-symbol breakdown
        writeln!(file, "By Symbol")?;
        writeln!(file, "Symbol,Company,Total Income,Qualified,Non-Qualified,Return of Capital,Payments")?;

        for (symbol, symbol_summary) in &summary.by_symbol {
            writeln!(
                file,
                "{},{},{},{},{},{},{}",
                symbol,
                symbol_summary.company_name.as_deref().unwrap_or(""),
                symbol_summary.total_income,
                symbol_summary.qualified_amount,
                symbol_summary.non_qualified_amount,
                symbol_summary.return_of_capital_amount,
                symbol_summary.payment_count
            )?;
        }

        if let Some(estimated_tax) = &summary.estimated_tax {
            writeln!(file, "")?;
            writeln!(file, "Estimated Tax")?;
            writeln!(file, "Tax Type,Amount")?;
            writeln!(file, "Tax on Qualified Dividends,{}", estimated_tax.qualified_tax)?;
            writeln!(file, "Tax on Non-Qualified Dividends,{}", estimated_tax.non_qualified_tax)?;
            writeln!(file, "Total Estimated Tax,{}", estimated_tax.total_estimated_tax)?;
        }

        Ok(())
    }

    /// Export 1099-DIV report to CSV format
    pub fn export_1099_div_csv(report: &Form1099DIV, file_path: &str) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(file_path)?;

        // Write header
        writeln!(file, "1099-DIV Tax Report for {}", report.tax_year)?;
        writeln!(file, "")?;

        // Write summary
        writeln!(file, "Summary Totals")?;
        writeln!(file, "Box 1a - Total Ordinary Dividends,{}", report.summary.total_ordinary_dividends)?;
        writeln!(file, "Box 1b - Qualified Dividends,{}", report.summary.total_qualified_dividends)?;
        writeln!(file, "Box 2a - Total Capital Gain Distributions,{}", report.summary.total_capital_gain_distributions)?;
        writeln!(file, "Box 3 - Non-dividend Distributions,{}", report.summary.total_non_dividend_distributions)?;
        writeln!(file, "Box 4 - Federal Income Tax Withheld,{}", report.summary.total_federal_tax_withheld)?;
        writeln!(file, "Box 6 - Foreign Tax Paid,{}", report.summary.total_foreign_tax_paid)?;
        writeln!(file, "")?;

        // Write payer details
        writeln!(file, "Payer Details")?;
        writeln!(file, "Payer Name,Symbol,Box 1a (Ordinary),Box 1b (Qualified),Box 2a (Capital Gains),Box 3 (Non-dividend),Box 4 (Fed Tax),Box 6 (Foreign Tax)")?;

        for payer in &report.payers {
            writeln!(
                file,
                "{},{},{},{},{},{},{},{}",
                payer.payer_name,
                payer.symbols.join(";"),
                payer.total_ordinary_dividends,
                payer.qualified_dividends,
                payer.capital_gain_distributions,
                payer.non_dividend_distributions,
                payer.federal_tax_withheld,
                payer.foreign_tax_paid
            )?;
        }

        Ok(())
    }
}