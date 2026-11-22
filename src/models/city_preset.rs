use super::property_data::default_interest_rate;
use super::{
    FilingStatus, HpapRepayment, InvestmentParameters, PropertyData, RentalData,
    DEFAULT_HOUSEHOLD_INCOME, DEFAULT_PMI_RATE_PERCENT,
    DEFAULT_HPAP_HOUSEHOLD_SIZE, DEFAULT_HPAP_LIQUID_ASSETS, HPAP_MAX_CLOSING_COST_ASSISTANCE,
    HPAP_MIN_CONTRIBUTION,
};
use serde::{Deserialize, Serialize};

/// Assessment cap shown when a user turns the cap on for a city without one
pub const DEFAULT_ASSESSMENT_CAP_PERCENT: f64 = 2.0;

/// Pre-populated default values for a specific city and property size.
/// This struct is immutable.
///
/// Values updated November 2026:
/// - Rents: Zumper city rent pages (average asking rent by bedroom, Nov 21 2026)
/// - Manhattan prices: Corcoran Report Q3 2026, resale condo median price by bedroom
/// - San Francisco prices: spring/summer 2026 condo medians by bedroom (studio estimated)
/// - DC and Boston prices: citywide 2026 condo medians (~$500K DC, ~$775K Boston);
///   the split by bedroom count is estimated
/// - Property tax: DC Class 1 0.85%; SF FY2026-27 1.1816%; Boston FY2026 $12.40/$1,000
///   minus the ~$4,354 owner-occupant residential exemption (effective rate per price);
///   Manhattan ~0.9% effective (estimate)
/// - Assessment caps: SF Prop 13 2%/yr; DC homestead 10%/yr; none for Boston/Manhattan
/// - Loans above ~$1.2M are jumbo, so those presets use 20% down (no PMI)
/// - Appreciation and rent-growth rates are long-run assumptions, not current trends
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CityPreset {
    pub city_name: String,
    pub bedroom_type: String,

    // Property data fields
    pub home_price: f64,
    pub down_payment_percent: f64,
    pub interest_rate: f64,
    pub property_tax_rate: f64,
    /// Yearly cap on assessed-value growth, if the city has one (SF Prop 13, DC homestead)
    pub assessment_cap_percent: Option<f64>,
    pub home_insurance_annual: f64,
    pub maintenance_percent: f64,
    pub hoa_fee: f64,
    pub included_utilities: f64,
    pub non_included_utilities: f64,
    pub lender_grant: f64,
    pub closing_costs_percent_purchase: f64,
    pub seller_closing_assistance: f64,
    pub closing_costs_percent_sale: f64,
    pub home_appreciation_rate: f64,

    // Rental data fields
    pub monthly_rent: f64,
    pub amenity_fees: f64,
    pub rent_included_utilities: f64,
    pub rent_non_included_utilities: f64,
    pub renters_insurance: f64,
    pub rent_increase_rate: f64,

    // Effective state + local income tax rate (% of income) at the default household income
    pub state_income_tax_rate: f64,

    // Investment parameters
    pub annual_return_rate: f64,
}

impl CityPreset {
    /// Get display name for the preset
    pub fn display_name(&self) -> String {
        format!("{} - {} Bedroom", self.city_name, self.bedroom_type)
    }

    /// Convert to PropertyData
    pub fn to_property_data(&self) -> PropertyData {
        PropertyData {
            home_price: self.home_price,
            down_payment_percent: self.down_payment_percent,
            interest_rate: self.interest_rate,
            mortgage_points: 0.0,
            property_tax_rate: self.property_tax_rate,
            enable_assessment_cap: self.assessment_cap_percent.is_some(),
            assessment_cap_percent: self
                .assessment_cap_percent
                .unwrap_or(DEFAULT_ASSESSMENT_CAP_PERCENT),
            home_insurance_annual: self.home_insurance_annual,
            maintenance_percent: self.maintenance_percent,
            hoa_fee: self.hoa_fee,
            hoa_fee_increase_rate: 3.0,
            non_included_utilities: self.non_included_utilities,
            lender_grant: self.lender_grant,
            closing_costs_percent_purchase: self.closing_costs_percent_purchase,
            seller_closing_assistance: self.seller_closing_assistance,
            closing_costs_percent_sale: self.closing_costs_percent_sale,
            home_appreciation_rate: self.home_appreciation_rate,
            pmi_rate_percent: DEFAULT_PMI_RATE_PERCENT, // $0 automatically at 20% down
            pmi_drop_off_ltv: 78.0,
            // Tax benefits - enabled by default, user can disable
            enable_tax_benefits: true,
            filing_status: FilingStatus::Single,
            marginal_tax_rate: 22.0,
            other_itemized_deductions: 0.0,
            hoa_deduction_percent: 0.0,
            household_income: DEFAULT_HOUSEHOLD_INCOME,
            state_income_tax_rate: self.state_income_tax_rate,
            // DC HPAP - off by default, user can enable
            enable_hpap: false,
            hpap_enter_own_max: true,
            hpap_own_max: 0.0,
            hpap_own_repayment: HpapRepayment::DeferredUntilSale,
            hpap_own_contribution: HPAP_MIN_CONTRIBUTION,
            hpap_household_size: DEFAULT_HPAP_HOUSEHOLD_SIZE,
            hpap_liquid_assets: DEFAULT_HPAP_LIQUID_ASSETS,
            hpap_closing_cost_assistance: HPAP_MAX_CLOSING_COST_ASSISTANCE,
        }
    }

    /// Convert to RentalData
    pub fn to_rental_data(&self) -> RentalData {
        RentalData {
            monthly_rent: self.monthly_rent,
            amenity_fees: self.amenity_fees,
            rent_included_utilities: self.rent_included_utilities,
            rent_non_included_utilities: self.rent_non_included_utilities,
            renters_insurance: self.renters_insurance,
            rent_increase_rate: self.rent_increase_rate,
            security_deposit: self.monthly_rent, // Default to 1 month's rent
            enable_broker_fee: false,
            broker_fee_percent: 15.0,
        }
    }

    /// Convert to InvestmentParameters
    pub fn to_investment_parameters(&self) -> InvestmentParameters {
        InvestmentParameters::with_values(self.annual_return_rate, 30)
    }

    // ===== WASHINGTON DC PRESETS =====

    pub fn dc_studio() -> Self {
        Self {
            city_name: "Washington DC".to_string(),
            bedroom_type: "Studio".to_string(),
            home_price: 325000.0,
            down_payment_percent: 3.0,
            interest_rate: default_interest_rate(),
            property_tax_rate: 0.85,
            assessment_cap_percent: Some(10.0),
            home_insurance_annual: 1200.0,
            maintenance_percent: 1.0,
            hoa_fee: 450.0,
            included_utilities: 0.0,
            non_included_utilities: 150.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 4.0,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 6.0,
            home_appreciation_rate: 3.5,
            monthly_rent: 1846.0,
            amenity_fees: 50.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 120.0,
            renters_insurance: 20.0,
            rent_increase_rate: 4.0,
            state_income_tax_rate: 6.5,
            annual_return_rate: 7.0,
        }
    }

    pub fn dc_one_bedroom() -> Self {
        Self {
            city_name: "Washington DC".to_string(),
            bedroom_type: "1".to_string(),
            home_price: 425000.0,
            down_payment_percent: 3.0,
            interest_rate: default_interest_rate(),
            property_tax_rate: 0.85,
            assessment_cap_percent: Some(10.0),
            home_insurance_annual: 1400.0,
            maintenance_percent: 1.0,
            hoa_fee: 550.0,
            included_utilities: 0.0,
            non_included_utilities: 175.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 4.0,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 6.0,
            home_appreciation_rate: 3.5,
            monthly_rent: 2150.0,
            amenity_fees: 50.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 135.0,
            renters_insurance: 22.0,
            rent_increase_rate: 4.0,
            state_income_tax_rate: 6.5,
            annual_return_rate: 7.0,
        }
    }

    pub fn dc_two_bedroom() -> Self {
        Self {
            city_name: "Washington DC".to_string(),
            bedroom_type: "2".to_string(),
            home_price: 575000.0,
            down_payment_percent: 3.0,
            interest_rate: default_interest_rate(),
            property_tax_rate: 0.85,
            assessment_cap_percent: Some(10.0),
            home_insurance_annual: 1700.0,
            maintenance_percent: 1.0,
            hoa_fee: 650.0,
            included_utilities: 0.0,
            non_included_utilities: 200.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 4.0,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 6.0,
            home_appreciation_rate: 3.5,
            monthly_rent: 2900.0,
            amenity_fees: 75.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 150.0,
            renters_insurance: 25.0,
            rent_increase_rate: 4.0,
            state_income_tax_rate: 6.5,
            annual_return_rate: 7.0,
        }
    }

    pub fn dc_three_bedroom() -> Self {
        Self {
            city_name: "Washington DC".to_string(),
            bedroom_type: "3".to_string(),
            home_price: 750000.0,
            down_payment_percent: 3.0,
            interest_rate: default_interest_rate(),
            property_tax_rate: 0.85,
            assessment_cap_percent: Some(10.0),
            home_insurance_annual: 2100.0,
            maintenance_percent: 1.0,
            hoa_fee: 800.0,
            included_utilities: 0.0,
            non_included_utilities: 250.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 4.0,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 6.0,
            home_appreciation_rate: 3.5,
            monthly_rent: 3575.0,
            amenity_fees: 100.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 180.0,
            renters_insurance: 30.0,
            rent_increase_rate: 4.0,
            state_income_tax_rate: 6.5,
            annual_return_rate: 7.0,
        }
    }

    // ===== BOSTON PRESETS =====

    pub fn boston_studio() -> Self {
        Self {
            city_name: "Boston".to_string(),
            bedroom_type: "Studio".to_string(),
            home_price: 475000.0,
            down_payment_percent: 3.0,
            interest_rate: default_interest_rate(),
            property_tax_rate: 0.32,
            assessment_cap_percent: None,
            home_insurance_annual: 1400.0,
            maintenance_percent: 1.0,
            hoa_fee: 400.0,
            included_utilities: 0.0,
            non_included_utilities: 175.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 4.0,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 6.0,
            home_appreciation_rate: 4.0,
            monthly_rent: 2475.0,
            amenity_fees: 50.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 150.0,
            renters_insurance: 25.0,
            rent_increase_rate: 4.0,
            state_income_tax_rate: 4.9,
            annual_return_rate: 7.0,
        }
    }

    pub fn boston_one_bedroom() -> Self {
        Self {
            city_name: "Boston".to_string(),
            bedroom_type: "1".to_string(),
            home_price: 625000.0,
            down_payment_percent: 3.0,
            interest_rate: default_interest_rate(),
            property_tax_rate: 0.54,
            assessment_cap_percent: None,
            home_insurance_annual: 1600.0,
            maintenance_percent: 1.0,
            hoa_fee: 500.0,
            included_utilities: 0.0,
            non_included_utilities: 185.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 4.0,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 6.0,
            home_appreciation_rate: 4.0,
            monthly_rent: 2900.0,
            amenity_fees: 60.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 160.0,
            renters_insurance: 27.0,
            rent_increase_rate: 4.0,
            state_income_tax_rate: 4.9,
            annual_return_rate: 7.0,
        }
    }

    pub fn boston_two_bedroom() -> Self {
        Self {
            city_name: "Boston".to_string(),
            bedroom_type: "2".to_string(),
            home_price: 850000.0,
            down_payment_percent: 3.0,
            interest_rate: default_interest_rate(),
            property_tax_rate: 0.73,
            assessment_cap_percent: None,
            home_insurance_annual: 1900.0,
            maintenance_percent: 1.0,
            hoa_fee: 600.0,
            included_utilities: 0.0,
            non_included_utilities: 200.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 4.0,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 6.0,
            home_appreciation_rate: 4.0,
            monthly_rent: 3480.0,
            amenity_fees: 75.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 175.0,
            renters_insurance: 30.0,
            rent_increase_rate: 4.0,
            state_income_tax_rate: 4.9,
            annual_return_rate: 7.0,
        }
    }

    pub fn boston_three_bedroom() -> Self {
        Self {
            city_name: "Boston".to_string(),
            bedroom_type: "3".to_string(),
            home_price: 1150000.0,
            down_payment_percent: 3.0,
            interest_rate: default_interest_rate(),
            property_tax_rate: 0.86,
            assessment_cap_percent: None,
            home_insurance_annual: 2300.0,
            maintenance_percent: 1.0,
            hoa_fee: 750.0,
            included_utilities: 0.0,
            non_included_utilities: 225.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 4.0,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 6.0,
            home_appreciation_rate: 4.0,
            monthly_rent: 3750.0,
            amenity_fees: 100.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 200.0,
            renters_insurance: 35.0,
            rent_increase_rate: 4.0,
            state_income_tax_rate: 4.9,
            annual_return_rate: 7.0,
        }
    }

    // ===== NYC PRESETS =====

    pub fn nyc_studio() -> Self {
        Self {
            city_name: "New York City".to_string(),
            bedroom_type: "Studio".to_string(),
            home_price: 644000.0,
            down_payment_percent: 3.0,
            interest_rate: default_interest_rate(),
            property_tax_rate: 0.9,
            assessment_cap_percent: None,
            home_insurance_annual: 1500.0,
            maintenance_percent: 1.0,
            hoa_fee: 900.0,
            included_utilities: 0.0,
            non_included_utilities: 175.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 4.0,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 7.5,
            home_appreciation_rate: 3.5,
            monthly_rent: 4100.0,
            amenity_fees: 75.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 150.0,
            renters_insurance: 30.0,
            rent_increase_rate: 3.5,
            state_income_tax_rate: 8.9,
            annual_return_rate: 7.0,
        }
    }

    pub fn nyc_one_bedroom() -> Self {
        Self {
            city_name: "New York City".to_string(),
            bedroom_type: "1".to_string(),
            home_price: 990000.0,
            down_payment_percent: 3.0,
            interest_rate: default_interest_rate(),
            property_tax_rate: 0.9,
            assessment_cap_percent: None,
            home_insurance_annual: 1800.0,
            maintenance_percent: 1.0,
            hoa_fee: 1200.0,
            included_utilities: 0.0,
            non_included_utilities: 200.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 4.0,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 7.5,
            home_appreciation_rate: 3.5,
            monthly_rent: 5362.0,
            amenity_fees: 100.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 175.0,
            renters_insurance: 32.0,
            rent_increase_rate: 3.5,
            state_income_tax_rate: 8.9,
            annual_return_rate: 7.0,
        }
    }

    pub fn nyc_two_bedroom() -> Self {
        Self {
            city_name: "New York City".to_string(),
            bedroom_type: "2".to_string(),
            home_price: 1950000.0,
            down_payment_percent: 20.0,
            interest_rate: default_interest_rate(),
            property_tax_rate: 0.9,
            assessment_cap_percent: None,
            home_insurance_annual: 2200.0,
            maintenance_percent: 1.0,
            hoa_fee: 1800.0,
            included_utilities: 0.0,
            non_included_utilities: 225.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 5.0,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 7.5,
            home_appreciation_rate: 3.5,
            monthly_rent: 6850.0,
            amenity_fees: 125.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 200.0,
            renters_insurance: 35.0,
            rent_increase_rate: 3.5,
            state_income_tax_rate: 8.9,
            annual_return_rate: 7.0,
        }
    }

    pub fn nyc_three_bedroom() -> Self {
        Self {
            city_name: "New York City".to_string(),
            bedroom_type: "3".to_string(),
            home_price: 4000000.0,
            down_payment_percent: 20.0,
            interest_rate: default_interest_rate(),
            property_tax_rate: 0.9,
            assessment_cap_percent: None,
            home_insurance_annual: 2800.0,
            maintenance_percent: 1.0,
            hoa_fee: 3000.0,
            included_utilities: 0.0,
            non_included_utilities: 275.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 5.5,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 7.5,
            home_appreciation_rate: 3.5,
            monthly_rent: 7595.0,
            amenity_fees: 150.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 225.0,
            renters_insurance: 40.0,
            rent_increase_rate: 3.5,
            state_income_tax_rate: 8.9,
            annual_return_rate: 7.0,
        }
    }

    // ===== SAN FRANCISCO PRESETS =====

    pub fn sf_studio() -> Self {
        Self {
            city_name: "San Francisco".to_string(),
            bedroom_type: "Studio".to_string(),
            home_price: 500000.0,
            down_payment_percent: 3.0,
            interest_rate: default_interest_rate(),
            property_tax_rate: 1.18,
            assessment_cap_percent: Some(2.0),
            home_insurance_annual: 1500.0,
            maintenance_percent: 1.0,
            hoa_fee: 500.0,
            included_utilities: 0.0,
            non_included_utilities: 160.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 4.0,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 6.0,
            home_appreciation_rate: 4.0,
            monthly_rent: 2880.0,
            amenity_fees: 50.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 150.0,
            renters_insurance: 30.0,
            rent_increase_rate: 4.0,
            state_income_tax_rate: 6.9,
            annual_return_rate: 7.0,
        }
    }

    pub fn sf_one_bedroom() -> Self {
        Self {
            city_name: "San Francisco".to_string(),
            bedroom_type: "1".to_string(),
            home_price: 775000.0,
            down_payment_percent: 3.0,
            interest_rate: default_interest_rate(),
            property_tax_rate: 1.18,
            assessment_cap_percent: Some(2.0),
            home_insurance_annual: 1800.0,
            maintenance_percent: 1.0,
            hoa_fee: 600.0,
            included_utilities: 0.0,
            non_included_utilities: 170.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 4.0,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 6.0,
            home_appreciation_rate: 4.0,
            monthly_rent: 4295.0,
            amenity_fees: 60.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 160.0,
            renters_insurance: 32.0,
            rent_increase_rate: 4.0,
            state_income_tax_rate: 6.9,
            annual_return_rate: 7.0,
        }
    }

    pub fn sf_two_bedroom() -> Self {
        Self {
            city_name: "San Francisco".to_string(),
            bedroom_type: "2".to_string(),
            home_price: 1400000.0,
            down_payment_percent: 20.0,
            interest_rate: default_interest_rate(),
            property_tax_rate: 1.18,
            assessment_cap_percent: Some(2.0),
            home_insurance_annual: 2100.0,
            maintenance_percent: 1.0,
            hoa_fee: 700.0,
            included_utilities: 0.0,
            non_included_utilities: 180.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 4.0,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 6.0,
            home_appreciation_rate: 4.0,
            monthly_rent: 6118.0,
            amenity_fees: 75.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 170.0,
            renters_insurance: 35.0,
            rent_increase_rate: 4.0,
            state_income_tax_rate: 6.9,
            annual_return_rate: 7.0,
        }
    }

    pub fn sf_three_bedroom() -> Self {
        Self {
            city_name: "San Francisco".to_string(),
            bedroom_type: "3".to_string(),
            home_price: 1850000.0,
            down_payment_percent: 20.0,
            interest_rate: default_interest_rate(),
            property_tax_rate: 1.18,
            assessment_cap_percent: Some(2.0),
            home_insurance_annual: 2500.0,
            maintenance_percent: 1.0,
            hoa_fee: 850.0,
            included_utilities: 0.0,
            non_included_utilities: 200.0,
            lender_grant: 0.0,
            closing_costs_percent_purchase: 4.0,
            seller_closing_assistance: 0.0,
            closing_costs_percent_sale: 6.0,
            home_appreciation_rate: 4.0,
            monthly_rent: 7500.0,
            amenity_fees: 100.0,
            rent_included_utilities: 0.0,
            rent_non_included_utilities: 190.0,
            renters_insurance: 40.0,
            rent_increase_rate: 4.0,
            state_income_tax_rate: 6.9,
            annual_return_rate: 7.0,
        }
    }

    /// Get all available presets
    pub fn all_presets() -> Vec<CityPreset> {
        vec![
            Self::dc_studio(),
            Self::dc_one_bedroom(),
            Self::dc_two_bedroom(),
            Self::dc_three_bedroom(),
            Self::boston_studio(),
            Self::boston_one_bedroom(),
            Self::boston_two_bedroom(),
            Self::boston_three_bedroom(),
            Self::nyc_studio(),
            Self::nyc_one_bedroom(),
            Self::nyc_two_bedroom(),
            Self::nyc_three_bedroom(),
            Self::sf_studio(),
            Self::sf_one_bedroom(),
            Self::sf_two_bedroom(),
            Self::sf_three_bedroom(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_presets_count() {
        let presets = CityPreset::all_presets();
        assert_eq!(presets.len(), 16);
    }

    #[test]
    fn test_dc_two_bedroom() {
        let preset = CityPreset::dc_two_bedroom();
        assert_eq!(preset.city_name, "Washington DC");
        assert_eq!(preset.home_price, 575000.0);
        assert_eq!(preset.monthly_rent, 2900.0);
    }

    #[test]
    fn test_preset_pmi_and_jumbo_down_payment() {
        // 3% down on $575,000: loan $557,750 * 0.5% / 12 = ~$232/mo
        let dc = CityPreset::dc_two_bedroom().to_property_data();
        assert!((dc.monthly_pmi() - 232.40).abs() < 0.01);
        // Jumbo presets use 20% down and have no PMI
        let nyc = CityPreset::nyc_two_bedroom();
        assert_eq!(nyc.down_payment_percent, 20.0);
        assert_eq!(nyc.to_property_data().monthly_pmi(), 0.0);
        // Every preset's 3%-down loan stays under the ~$1.2M jumbo threshold
        for preset in CityPreset::all_presets() {
            if preset.down_payment_percent < 20.0 {
                assert!(preset.home_price * 0.97 < 1_200_000.0, "{}", preset.display_name());
            }
        }
    }

    #[test]
    fn test_to_property_data() {
        let preset = CityPreset::dc_two_bedroom();
        let property_data = preset.to_property_data();
        assert_eq!(property_data.home_price, 575000.0);
        assert_eq!(property_data.interest_rate, default_interest_rate()); // FRED rate at build time, or 5.99 fallback
    }

    #[test]
    fn test_to_rental_data() {
        let preset = CityPreset::dc_two_bedroom();
        let rental_data = preset.to_rental_data();
        assert_eq!(rental_data.monthly_rent, 2900.0);
        assert_eq!(rental_data.rent_increase_rate, 4.0);
    }

    #[test]
    fn test_app_defaults_match_dc_two_bedroom_preset() {
        let preset = CityPreset::dc_two_bedroom();
        assert_eq!(preset.to_property_data(), PropertyData::new());
        assert_eq!(preset.to_rental_data(), RentalData::new());
    }

    #[test]
    fn test_display_name() {
        let preset = CityPreset::sf_three_bedroom();
        assert_eq!(preset.display_name(), "San Francisco - 3 Bedroom");
    }
}
