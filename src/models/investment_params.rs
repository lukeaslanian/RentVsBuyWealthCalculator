use crate::utils::AppConfig;
use serde::{Deserialize, Serialize};

/// Stores assumptions about investment returns on saved money.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct InvestmentParameters {
    pub annual_return_rate: f64,
    pub analysis_years: usize,
    pub inflation_rate: f64, // Annual inflation rate as percentage (e.g., 3.0%)
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
        }
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
    fn test_with_values() {
        let params = InvestmentParameters::with_values(5.0, 20);
        assert_eq!(params.annual_return_rate, 5.0);
        assert_eq!(params.analysis_years, 20);
    }
}
