use crate::utils::AppConfig;
use serde::{Deserialize, Serialize};

/// Default long-term capital gains rate: 15% federal bracket (most households).
/// Users in states that tax gains (e.g., DC ~6-10%) should add their state rate.
pub const DEFAULT_CAPITAL_GAINS_TAX_RATE: f64 = 15.0;

/// Stores assumptions about investment returns on saved money.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct InvestmentParameters {
    pub annual_return_rate: f64,
    pub analysis_years: usize,
    pub inflation_rate: f64, // Annual inflation rate as percentage (e.g., 3.0%)
    /// Combined (federal + state) long-term capital gains tax rate, as a percentage.
    /// Applied to investment gains when portfolios are cashed out, and to home-sale
    /// gains above the Section 121 exclusion. 0 disables capital gains taxes.
    pub capital_gains_tax_rate: f64,
}

impl Default for InvestmentParameters {
    /// Default constructor (7% return, 30 years, 3% inflation)
    /// Note: 7% is a reasonable stock market index fund return rate
    /// 3% is the 10-year average US inflation rate (2015-2024)
    fn default() -> Self {
        Self {
            annual_return_rate: 7.0,
            analysis_years: AppConfig::DEFAULT_ANALYSIS_YEARS,
            inflation_rate: 3.0, // 10-year US average (2015-2024): 2.86%, rounded to 3%
            capital_gains_tax_rate: DEFAULT_CAPITAL_GAINS_TAX_RATE,
        }
    }
}

impl InvestmentParameters {
    /// Create a new InvestmentParameters with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific values
    pub fn with_values(annual_return_rate: f64, analysis_years: usize) -> Self {
        Self {
            annual_return_rate,
            analysis_years,
            inflation_rate: 3.0,
            capital_gains_tax_rate: DEFAULT_CAPITAL_GAINS_TAX_RATE,
        }
    }

    /// Value of a portfolio after paying capital gains tax on unrealized gains
    /// (value above cost basis). Losses are not taxed (no benefit modeled).
    pub fn after_tax_portfolio_value(&self, value: f64, cost_basis: f64) -> f64 {
        let gain = (value - cost_basis).max(0.0);
        value - gain * (self.capital_gains_tax_rate.clamp(0.0, 100.0) / 100.0)
    }

    /// Get monthly return rate
    pub fn monthly_return_rate(&self) -> f64 {
        self.annual_return_rate / 100.0 / AppConfig::MONTHS_PER_YEAR as f64
    }

    /// Get the inflation multiplier for a given year
    /// Returns (1 + inflation_rate/100)^(year-1)
    pub fn inflation_multiplier(&self, year: usize) -> f64 {
        if year <= 1 {
            1.0
        } else {
            (1.0 + self.inflation_rate / 100.0).powi((year - 1) as i32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let params = InvestmentParameters::new();
        assert_eq!(params.annual_return_rate, 7.0);
        assert_eq!(params.analysis_years, 30);
    }

    #[test]
    fn test_monthly_return_rate() {
        let params = InvestmentParameters::new();
        let expected = 7.0 / 100.0 / 12.0;
        assert!((params.monthly_return_rate() - expected).abs() < 0.0001);
    }

    #[test]
    fn test_after_tax_portfolio_value() {
        let params = InvestmentParameters::new(); // 15% capital gains
                                                  // $100k gain on $200k of contributions: $15k tax
        assert!((params.after_tax_portfolio_value(300000.0, 200000.0) - 285000.0).abs() < 1e-6);
        // Portfolio at a loss: no tax, no credit
        assert_eq!(
            params.after_tax_portfolio_value(150000.0, 200000.0),
            150000.0
        );
        // 0% rate disables the tax
        let mut untaxed = params.clone();
        untaxed.capital_gains_tax_rate = 0.0;
        assert_eq!(
            untaxed.after_tax_portfolio_value(300000.0, 200000.0),
            300000.0
        );
        // Out-of-range rates are clamped (never more than the whole gain)
        untaxed.capital_gains_tax_rate = 250.0;
        assert_eq!(
            untaxed.after_tax_portfolio_value(300000.0, 200000.0),
            200000.0
        );
    }

    #[test]
    fn test_with_values() {
        let params = InvestmentParameters::with_values(5.0, 20);
        assert_eq!(params.annual_return_rate, 5.0);
        assert_eq!(params.analysis_years, 20);
    }
}
