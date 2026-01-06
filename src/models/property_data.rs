use crate::utils::AppConfig;
use serde::{Deserialize, Serialize};

/// Tax filing status for determining standard deduction
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum FilingStatus {
    #[default]
    Single,
    MarriedFilingJointly,
}

impl FilingStatus {
    /// Get the standard deduction for 2025 tax year
    /// Updated per IRS inflation adjustments and the One Big Beautiful Bill Act (July 2025)
    /// Single: $15,750, Married Filing Jointly: $31,500
    pub fn standard_deduction(&self) -> f64 {
        match self {
            FilingStatus::Single => 15750.0,
            FilingStatus::MarriedFilingJointly => 31500.0,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            FilingStatus::Single => "Individual return",
            FilingStatus::MarriedFilingJointly => "Joint return",
        }
    }
}

/// Stores all financial parameters related to purchasing and owning a home.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PropertyData {
    pub home_price: f64,
    pub down_payment_percent: f64,
    pub interest_rate: f64,
    pub mortgage_points: f64, // Discount points (1 point = 1% of loan, reduces rate ~0.25% per point)
    pub property_tax_rate: f64,
    pub home_insurance_annual: f64,
    pub maintenance_percent: f64,
    pub hoa_fee: f64,
    pub hoa_fee_increase_rate: f64,
    pub non_included_utilities: f64,
    pub lender_grant: f64,
    pub closing_costs_percent_purchase: f64, // As percentage of home price
    pub seller_closing_assistance: f64,
    pub closing_costs_percent_sale: f64, // As percentage of home price (typically 6-8%)
    pub home_appreciation_rate: f64,
    pub monthly_pmi: f64, // Monthly PMI (private mortgage insurance) payment
    pub pmi_drop_off_ltv: f64, // LTV percentage when PMI drops off (typically 78%)

    // Tax benefit parameters
    pub enable_tax_benefits: bool, // Toggle to enable/disable tax benefit calculations
    pub filing_status: FilingStatus, // Individual or Joint return
    pub marginal_tax_rate: f64,    // Marginal tax rate as percentage (e.g., 22%)
    pub other_itemized_deductions: f64, // Other itemized deductions in dollars (e.g., charitable contributions)
    pub hoa_deduction_percent: f64,     // Percentage of HOA fees that are tax deductible (0-100%)
}

impl Default for PropertyData {
    /// Default constructor using Washington DC 2-bedroom preset values (January 2026)
    /// Based on current DC condo market data
    fn default() -> Self {
        Self {
            // DC 2-bedroom defaults (updated January 2026)
            home_price: 575000.0,
            down_payment_percent: 3.0,
            interest_rate: 5.99,
            mortgage_points: 0.0,
            property_tax_rate: 0.835, // DC property tax rate
            home_insurance_annual: 1700.0,
            maintenance_percent: 1.0,
            hoa_fee: 650.0,
            hoa_fee_increase_rate: 3.0,
            non_included_utilities: 200.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 4.0,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 6.0,
            home_appreciation_rate: 3.5,
            monthly_pmi: 65.0,
            pmi_drop_off_ltv: 78.0,

            // Tax benefits - disabled by default
            enable_tax_benefits: false,
            filing_status: FilingStatus::Single,
            marginal_tax_rate: 22.0,
            other_itemized_deductions: 0.0,
            hoa_deduction_percent: 0.0,
        }
    }
}

impl PropertyData {
    /// Create a new PropertyData with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate the down payment amount
    pub fn down_payment_amount(&self) -> f64 {
        self.home_price * (self.down_payment_percent / 100.0)
    }

    /// Calculate the loan amount
    pub fn loan_amount(&self) -> f64 {
        self.home_price - self.down_payment_amount() - self.lender_grant
    }

    /// Calculate the cost of mortgage points
    /// 1 point = 1% of loan amount
    pub fn mortgage_points_cost(&self) -> f64 {
        self.loan_amount() * (self.mortgage_points / 100.0)
    }

    /// Calculate the effective interest rate after points buydown
    /// Typically each point reduces rate by 0.25%
    pub fn effective_interest_rate(&self) -> f64 {
        // Each point typically reduces interest rate by 0.25%
        let rate_reduction = self.mortgage_points * 0.25;
        (self.interest_rate - rate_reduction).max(0.0)
    }

    /// Calculate total monthly ownership cost (not including mortgage)
    pub fn total_monthly_ownership_cost(&self) -> f64 {
        let monthly_property_tax = ((self.home_price * self.property_tax_rate) / 100.0)
            / AppConfig::MONTHS_PER_YEAR as f64;
        let monthly_insurance = self.home_insurance_annual / AppConfig::MONTHS_PER_YEAR as f64;
        let monthly_maintenance = ((self.home_price * self.maintenance_percent) / 100.0)
            / AppConfig::MONTHS_PER_YEAR as f64;
        let monthly_utilities = self.non_included_utilities;

        monthly_property_tax
            + monthly_insurance
            + monthly_maintenance
            + self.hoa_fee
            + monthly_utilities
    }

    /// Calculate closing costs at purchase in dollars
    pub fn closing_costs_purchase(&self) -> f64 {
        self.home_price * (self.closing_costs_percent_purchase / 100.0)
    }

    /// Calculate closing costs at sale in dollars
    pub fn closing_costs_sale(&self) -> f64 {
        self.home_price * (self.closing_costs_percent_sale / 100.0)
    }

    /// Calculate HOA fee at a specific year (with increases)
    pub fn hoa_fee_at_year(&self, year: usize) -> f64 {
        // Apply compound increase rate: hoaFee * (1 + rate/100)^(year-1)
        self.hoa_fee * (1.0 + self.hoa_fee_increase_rate / 100.0).powi((year - 1) as i32)
    }

    /// Validate all input data
    pub fn is_valid(&self) -> bool {
        self.home_price > 0.0
            && self.home_price < 10_000_000.0
            && self.down_payment_percent >= 0.0
            && self.down_payment_percent <= 100.0
            && self.interest_rate > 0.0
            && self.interest_rate < 20.0
            && self.mortgage_points >= 0.0
            && self.mortgage_points <= 10.0
            && self.property_tax_rate >= 0.0
            && self.property_tax_rate < 10.0
            && self.home_insurance_annual >= 0.0
            && self.maintenance_percent >= 0.0
            && self.maintenance_percent < 10.0
            && self.hoa_fee >= 0.0
            && self.hoa_fee_increase_rate >= 0.0
            && self.hoa_fee_increase_rate < 50.0
            && self.non_included_utilities >= 0.0
            && self.lender_grant >= 0.0
            && self.closing_costs_percent_purchase >= 0.0
            && self.closing_costs_percent_purchase <= 20.0
            && self.seller_closing_assistance >= 0.0
            && self.closing_costs_percent_sale >= 0.0
            && self.closing_costs_percent_sale <= 20.0
            && self.home_appreciation_rate >= -10.0
            && self.home_appreciation_rate < 50.0
            && self.marginal_tax_rate >= 0.0
            && self.marginal_tax_rate <= 50.0
            && self.other_itemized_deductions >= 0.0
            && self.hoa_deduction_percent >= 0.0
            && self.hoa_deduction_percent <= 100.0
    }

    /// Calculate annual tax savings from homeownership deductions.
    ///
    /// This implements the US tax benefit calculation where:
    /// 1. Property taxes (capped at $10,000 SALT limit) and mortgage interest are deductible
    /// 2. A portion of HOA fees may be deductible (based on hoa_deduction_percent)
    /// 3. Only the amount exceeding the standard deduction provides a tax benefit
    /// 4. The benefit is the excess multiplied by the marginal tax rate
    ///
    /// Returns annual tax savings in dollars (0 if tax benefits are disabled or no benefit)
    pub fn calculate_annual_tax_savings(
        &self,
        annual_mortgage_interest: f64,
        current_home_value: f64,
        annual_hoa_fees: f64,
    ) -> f64 {
        if !self.enable_tax_benefits {
            return 0.0;
        }

        // Calculate annual property tax on current home value
        let annual_property_tax = current_home_value * (self.property_tax_rate / 100.0);

        // SALT deduction is capped at $10,000 for property taxes
        let salt_deduction = annual_property_tax.min(10000.0);

        // Calculate deductible portion of HOA fees
        let hoa_deduction = annual_hoa_fees * (self.hoa_deduction_percent / 100.0);

        // Total itemized deductions for housing
        // Mortgage interest is fully deductible (up to $750k loan limit, which we assume is met)
        let housing_deductions = salt_deduction + annual_mortgage_interest + hoa_deduction;

        // Add other itemized deductions
        let total_itemized = housing_deductions + self.other_itemized_deductions;

        // Get standard deduction based on filing status
        let standard_deduction = self.filing_status.standard_deduction();

        // Only benefit from itemizing if it exceeds standard deduction
        if total_itemized > standard_deduction {
            let excess_deduction = total_itemized - standard_deduction;
            excess_deduction * (self.marginal_tax_rate / 100.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let data = PropertyData::new();
        assert_eq!(data.home_price, 575000.0);
        assert_eq!(data.down_payment_percent, 3.0);
        assert_eq!(data.interest_rate, 5.99);
    }

    #[test]
    fn test_down_payment_calculation() {
        let data = PropertyData::new();
        assert_eq!(data.down_payment_amount(), 575000.0 * 0.03);
    }

    #[test]
    fn test_loan_amount() {
        let data = PropertyData::new();
        let expected_loan = 575000.0 - (575000.0 * 0.03);
        assert_eq!(data.loan_amount(), expected_loan);
    }

    #[test]
    fn test_validation() {
        let data = PropertyData::new();
        assert!(data.is_valid());

        let mut invalid_data = data.clone();
        invalid_data.home_price = -100.0;
        assert!(!invalid_data.is_valid());
    }
}
