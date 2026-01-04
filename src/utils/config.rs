/// Application configuration constants.
/// Centralizes all numbers & configuration values.
pub struct AppConfig;

impl AppConfig {
    // ===== FINANCIAL CONSTANTS =====

    /// Months per year
    pub const MONTHS_PER_YEAR: usize = 12;

    /// Default analysis period in years
    pub const DEFAULT_ANALYSIS_YEARS: usize = 30;

    /// Minimum analysis period in years
    pub const MIN_ANALYSIS_YEARS: usize = 1;

    /// Maximum analysis period in years
    pub const MAX_ANALYSIS_YEARS: usize = 30;

    /// Default mortgage term in years
    pub const DEFAULT_MORTGAGE_TERM_YEARS: usize = 30;

    // ===== PROPERTY DATA VALIDATION RANGES =====

    /// Minimum interest rate (%)
    pub const MIN_INTEREST_RATE: f64 = 0.0;

    /// Maximum interest rate (%)
    pub const MAX_INTEREST_RATE: f64 = 30.0;
}
