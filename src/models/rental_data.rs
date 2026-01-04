use serde::{Deserialize, Serialize};

/// Stores all financial parameters related to renting a property.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RentalData {
    pub monthly_rent: f64,
    pub amenity_fees: f64,
    pub rent_included_utilities: f64,
    pub rent_non_included_utilities: f64,
    pub renters_insurance: f64,
    pub rent_increase_rate: f64,
    pub security_deposit: f64, // Upfront deposit, typically 1 month's rent
    pub enable_broker_fee: bool, // Toggle for broker's fee (common in NYC)
    pub broker_fee_percent: f64, // Broker's fee as percentage of annual rent (e.g., 15%)
}

impl Default for RentalData {
    /// Default constructor using realistic DC 2-bedroom rental values
    fn default() -> Self {
        let monthly_rent = 2350.0; // Comparable DC 2BR rent
        Self {
            monthly_rent,
            amenity_fees: 50.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 150.0,
            renters_insurance: 25.0,
            rent_increase_rate: 4.0, // DC historical rent increase rate
            security_deposit: monthly_rent, // Default to 1 month's rent
            enable_broker_fee: false, // Disabled by default (not common outside NYC)
            broker_fee_percent: 15.0, // 15% of annual rent when enabled
        }
    }
}

impl RentalData {
    /// Create a new RentalData with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate total monthly rent cost
    pub fn total_monthly_rent_cost(&self) -> f64 {
        self.monthly_rent
            + self.amenity_fees
            + self.rent_non_included_utilities
            + self.renters_insurance
    }

    /// Calculate rent at a specific year (with increases)
    pub fn rent_at_year(&self, year: usize) -> f64 {
        // Apply compound increase rate: rent * (1 + rate/100)^(year-1)
        self.monthly_rent * (1.0 + self.rent_increase_rate / 100.0).powi((year - 1) as i32)
    }

    /// Calculate broker's fee amount (one-time cost)
    pub fn broker_fee_amount(&self) -> f64 {
        if self.enable_broker_fee {
            self.monthly_rent * 12.0 * (self.broker_fee_percent / 100.0)
        } else {
            0.0
        }
    }

    /// Validate all input data
    pub fn is_valid(&self) -> bool {
        self.monthly_rent > 0.0
            && self.monthly_rent < 50000.0
            && self.amenity_fees >= 0.0
            && self.rent_included_utilities >= 0.0
            && self.rent_non_included_utilities >= 0.0
            && self.renters_insurance >= 0.0
            && self.rent_increase_rate >= 0.0
            && self.rent_increase_rate < 50.0
            && self.security_deposit >= 0.0
            && self.broker_fee_percent >= 0.0
            && self.broker_fee_percent <= 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let data = RentalData::new();
        assert_eq!(data.monthly_rent, 2350.0);
        assert_eq!(data.rent_increase_rate, 4.0);
    }

    #[test]
    fn test_total_monthly_cost() {
        let data = RentalData::new();
        let expected = 2350.0 + 50.0 + 150.0 + 25.0;
        assert_eq!(data.total_monthly_rent_cost(), expected);
    }

    #[test]
    fn test_rent_at_year() {
        let data = RentalData::new();
        // Year 1 should be base rent
        assert_eq!(data.rent_at_year(1), 2350.0);
        // Year 2 should be base rent * 1.04
        assert!((data.rent_at_year(2) - 2444.0).abs() < 0.1);
    }

    #[test]
    fn test_validation() {
        let data = RentalData::new();
        assert!(data.is_valid());

        let mut invalid_data = data.clone();
        invalid_data.monthly_rent = -100.0;
        assert!(!invalid_data.is_valid());
    }
}
