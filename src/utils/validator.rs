use crate::utils::AppConfig;

/// Data validation utilities
pub struct DataValidator;

impl DataValidator {
    /// Validate a home price
    pub fn is_valid_home_price(value: f64) -> bool {
        value >= 1.0 && value <= 100_000_000.0
    }

    /// Validate a percentage within a range
    pub fn is_valid_percent(value: f64, min: f64, max: f64) -> bool {
        value >= min && value <= max
    }

    /// Validate a currency value (must be non-negative)
    pub fn is_valid_currency(value: f64, min: f64) -> bool {
        value >= min
    }

    /// Validate interest rate
    pub fn is_valid_interest_rate(value: f64) -> bool {
        value >= AppConfig::MIN_INTEREST_RATE && value <= AppConfig::MAX_INTEREST_RATE
    }

    /// Validate analysis years
    pub fn is_valid_analysis_years(value: usize) -> bool {
        value >= AppConfig::MIN_ANALYSIS_YEARS && value <= AppConfig::MAX_ANALYSIS_YEARS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_price_validation() {
        assert!(DataValidator::is_valid_home_price(100000.0));
        assert!(!DataValidator::is_valid_home_price(-1.0));
        assert!(!DataValidator::is_valid_home_price(200_000_000.0));
    }

    #[test]
    fn test_percent_validation() {
        assert!(DataValidator::is_valid_percent(5.0, 0.0, 100.0));
        assert!(!DataValidator::is_valid_percent(-1.0, 0.0, 100.0));
        assert!(!DataValidator::is_valid_percent(101.0, 0.0, 100.0));
    }

    #[test]
    fn test_currency_validation() {
        assert!(DataValidator::is_valid_currency(1000.0, 0.0));
        assert!(!DataValidator::is_valid_currency(-100.0, 0.0));
    }
}
