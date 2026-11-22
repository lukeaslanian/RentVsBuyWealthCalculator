use crate::utils::AppConfig;
use serde::{Deserialize, Serialize};

/// Get the default mortgage interest rate - fetched from FRED at build time, or fallback value
pub fn default_interest_rate() -> f64 {
    option_env!("DEFAULT_MORTGAGE_RATE")
        .and_then(|s| s.parse().ok())
        .unwrap_or(5.99)
}

/// Get the date when the default mortgage rate was fetched from FRED (if available)
pub fn default_interest_rate_date() -> Option<&'static str> {
    option_env!("DEFAULT_MORTGAGE_RATE_DATE")
}

/// Check if the default interest rate was fetched from FRED at build time
pub fn is_default_rate_from_fred() -> bool {
    option_env!("MORTGAGE_RATE_FROM_FRED").is_some()
}

/// Tax filing status for determining standard deduction
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum FilingStatus {
    #[default]
    Single,
    MarriedFilingJointly,
}

impl FilingStatus {
    /// Get the standard deduction for the 2026 tax year (IRS Rev. Proc. 2025-32)
    /// Single: $16,100, Married Filing Jointly: $32,200
    pub fn standard_deduction(&self) -> f64 {
        match self {
            FilingStatus::Single => 16100.0,
            FilingStatus::MarriedFilingJointly => 32200.0,
        }
    }

    /// Home-sale capital gain exclusion (IRC Section 121) for a primary residence
    /// owned and lived in for at least 2 of the last 5 years
    pub fn home_sale_exclusion(&self) -> f64 {
        match self {
            FilingStatus::Single => 250000.0,
            FilingStatus::MarriedFilingJointly => 500000.0,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            FilingStatus::Single => "Individual return",
            FilingStatus::MarriedFilingJointly => "Joint return",
        }
    }
}

/// Default household income (approx. MAGI) used for SALT calculations
pub const DEFAULT_HOUSEHOLD_INCOME: f64 = 150000.0;
/// Default effective state/local income tax rate (DC is roughly 6-7% effective for middle incomes)
pub const DEFAULT_STATE_INCOME_TAX_RATE: f64 = 6.5;

/// Calendar tax year corresponding to analysis year 1
pub const TAX_BASE_YEAR: i32 = 2026;

/// SALT deduction floor (and the cap before 2025 / after 2029)
const SALT_CAP_FLOOR: f64 = 10000.0;

/// SALT (state and local tax) deduction cap for a given calendar tax year, per the
/// One Big Beautiful Bill Act (July 2025):
/// - 2025: $40,000, rising 1%/yr through 2029 ($40,400 in 2026 ... $41,624 in 2029)
/// - Reduced by 30% of MAGI above a threshold ($500,000 in 2025, rising 1%/yr),
///   but never below $10,000
/// - 2030 onward: reverts to a flat $10,000 (no phase-down)
pub fn salt_cap(tax_year: i32, magi: f64) -> f64 {
    let (cap, threshold) = match tax_year {
        2025 => (40000.0, 500000.0),
        2026 => (40400.0, 505000.0),
        2027 => (40804.0, 510050.0),
        2028 => (41212.0, 515151.0),
        2029 => (41624.0, 520303.0),
        _ => return SALT_CAP_FLOOR,
    };
    let phase_down = 0.30 * (magi - threshold).max(0.0);
    (cap - phase_down).max(SALT_CAP_FLOOR)
}

/// Number of months HPAP repayment is deferred for moderate-income borrowers (5 years)
pub const HPAP_DEFERRAL_MONTHS: usize = 60;
/// Number of monthly payments once HPAP repayment begins (40-year amortization)
pub const HPAP_AMORTIZATION_MONTHS: usize = 480;
/// Maximum HPAP closing cost assistance per DHCD
pub const HPAP_MAX_CLOSING_COST_ASSISTANCE: f64 = 4000.0;

/// HPAP repayment schedule, determined by the household's income tier.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum HpapRepayment {
    /// Up to 80% MFI: no payments; balance due on sale, refinance, or move-out
    #[default]
    DeferredUntilSale,
    /// 81-110% MFI: deferred 5 years, then 0% principal-only
    /// payments amortized over 40 years; remaining balance due on sale
    FiveYearDeferral40Year,
}

impl HpapRepayment {
    pub fn display_name(&self) -> &'static str {
        match self {
            HpapRepayment::DeferredUntilSale => "Up to 80% MFI: deferred until sale",
            HpapRepayment::FiveYearDeferral40Year => {
                "81-110% MFI: 5-yr deferral, then 40-yr payments"
            }
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
    pub household_income: f64,         // Annual household income (approx. MAGI), drives SALT cap phase-down
    pub state_income_tax_rate: f64,     // Effective state/local income tax rate as % of income (counts toward SALT)

    // DC Home Purchase Assistance Program (HPAP) - 0% interest gap-financing second loan
    pub enable_hpap: bool,                  // Toggle for DC HPAP
    pub hpap_loan_amount: f64,              // Gap financing: covers down payment first, excess reduces first mortgage
    pub hpap_closing_cost_assistance: f64,  // Closing cost assistance (max $4,000), part of the HPAP loan
    pub hpap_repayment: HpapRepayment,      // Repayment schedule based on income tier
}

impl Default for PropertyData {
    /// Default constructor using Washington DC 2-bedroom preset values (January 2026)
    /// Based on current DC condo market data
    fn default() -> Self {
        Self {
            // DC 2-bedroom defaults (updated January 2026)
            home_price: 575000.0,
            down_payment_percent: 3.0,
            interest_rate: default_interest_rate(),
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

            // Tax benefits - enabled by default
            enable_tax_benefits: true,
            filing_status: FilingStatus::Single,
            marginal_tax_rate: 22.0,
            other_itemized_deductions: 0.0,
            hoa_deduction_percent: 0.0,
            household_income: DEFAULT_HOUSEHOLD_INCOME,
            state_income_tax_rate: DEFAULT_STATE_INCOME_TAX_RATE,

            // DC HPAP - disabled by default
            enable_hpap: false,
            hpap_loan_amount: 202000.0,
            hpap_closing_cost_assistance: HPAP_MAX_CLOSING_COST_ASSISTANCE,
            hpap_repayment: HpapRepayment::DeferredUntilSale,
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

    /// Calculate the first mortgage loan amount (reduced by any HPAP gap financing)
    pub fn loan_amount(&self) -> f64 {
        self.loan_amount_before_hpap() - self.hpap_mortgage_reduction()
    }

    /// First mortgage amount before any HPAP gap financing is applied
    fn loan_amount_before_hpap(&self) -> f64 {
        self.home_price - self.down_payment_amount() - self.lender_grant
    }

    /// Portion of the HPAP loan that funds the down payment
    pub fn hpap_down_payment_applied(&self) -> f64 {
        if !self.enable_hpap {
            return 0.0;
        }
        self.hpap_loan_amount.max(0.0).min(self.down_payment_amount())
    }

    /// Portion of the HPAP loan beyond the down payment, which reduces the first mortgage
    /// (gap financing). Capped so the first mortgage never goes below zero.
    pub fn hpap_mortgage_reduction(&self) -> f64 {
        if !self.enable_hpap {
            return 0.0;
        }
        let excess = (self.hpap_loan_amount - self.down_payment_amount()).max(0.0);
        excess.min(self.loan_amount_before_hpap().max(0.0))
    }

    /// Portion of the HPAP closing cost assistance actually used (capped at closing costs owed)
    pub fn hpap_closing_costs_applied(&self) -> f64 {
        if !self.enable_hpap {
            return 0.0;
        }
        self.hpap_closing_cost_assistance
            .max(0.0)
            .min(self.closing_costs_purchase())
    }

    /// Total HPAP second-trust balance at purchase
    pub fn hpap_total_loan(&self) -> f64 {
        self.hpap_down_payment_applied()
            + self.hpap_mortgage_reduction()
            + self.hpap_closing_costs_applied()
    }

    /// Down payment paid out of the buyer's own pocket
    pub fn buyer_cash_down_payment(&self) -> f64 {
        self.down_payment_amount() - self.hpap_down_payment_applied()
    }

    /// Purchase closing costs paid out of the buyer's own pocket
    pub fn buyer_cash_closing_costs(&self) -> f64 {
        self.closing_costs_purchase() - self.hpap_closing_costs_applied()
    }

    /// Monthly HPAP repayment for a given month (1-indexed).
    /// Moderate tier: 0% principal-only payments in months 61-540; otherwise zero.
    pub fn hpap_monthly_payment(&self, month: usize) -> f64 {
        match self.hpap_repayment {
            HpapRepayment::FiveYearDeferral40Year
                if month > HPAP_DEFERRAL_MONTHS
                    && month <= HPAP_DEFERRAL_MONTHS + HPAP_AMORTIZATION_MONTHS =>
            {
                self.hpap_total_loan() / HPAP_AMORTIZATION_MONTHS as f64
            }
            _ => 0.0,
        }
    }

    /// Scheduled monthly HPAP payment once repayment begins (0 if deferred until sale)
    pub fn hpap_scheduled_payment(&self) -> f64 {
        self.hpap_monthly_payment(HPAP_DEFERRAL_MONTHS + 1)
    }

    /// Remaining HPAP balance after the given month's payment (due in full on sale)
    pub fn hpap_balance_after_month(&self, month: usize) -> f64 {
        let total = self.hpap_total_loan();
        match self.hpap_repayment {
            HpapRepayment::DeferredUntilSale => total,
            HpapRepayment::FiveYearDeferral40Year => {
                let payments_made = month
                    .saturating_sub(HPAP_DEFERRAL_MONTHS)
                    .min(HPAP_AMORTIZATION_MONTHS);
                let paid = total / HPAP_AMORTIZATION_MONTHS as f64 * payments_made as f64;
                (total - paid).max(0.0)
            }
        }
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

    /// Calculate closing costs at purchase in dollars (net of seller assistance)
    pub fn closing_costs_purchase(&self) -> f64 {
        (self.home_price * (self.closing_costs_percent_purchase / 100.0)
            - self.seller_closing_assistance)
            .max(0.0)
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
            && self.hpap_loan_amount >= 0.0
            && self.hpap_loan_amount <= self.home_price
            && self.hpap_closing_cost_assistance >= 0.0
            && self.hpap_closing_cost_assistance <= HPAP_MAX_CLOSING_COST_ASSISTANCE
            && self.household_income >= 0.0
            && self.household_income < 100_000_000.0
            && self.state_income_tax_rate >= 0.0
            && self.state_income_tax_rate <= 20.0
    }

    /// Annual state/local income tax paid (counts toward the SALT deduction)
    pub fn annual_state_income_tax(&self) -> f64 {
        self.household_income * (self.state_income_tax_rate / 100.0)
    }

    /// Calculate annual tax savings from homeownership deductions.
    ///
    /// This implements the US tax benefit calculation where:
    /// 1. State/local income tax + property tax are deductible together, up to the SALT cap
    ///    for that tax year (see `salt_cap`)
    /// 2. Mortgage interest is deductible (assumes the loan is within the $750k limit)
    /// 3. A portion of HOA fees may be deductible (based on hoa_deduction_percent)
    /// 4. The benefit is the INCREMENTAL deduction from owning vs. not owning, each taken as
    ///    the better of itemizing or the standard deduction, times the marginal tax rate.
    ///    This avoids crediting the buyer for deductions a renter would also get
    ///    (e.g., state income tax or large charitable gifts that already exceed the standard).
    ///
    /// `year` is the 1-based analysis year (year 1 = `TAX_BASE_YEAR`).
    /// Returns annual tax savings in dollars (0 if tax benefits are disabled or no benefit)
    pub fn calculate_annual_tax_savings(
        &self,
        year: usize,
        annual_mortgage_interest: f64,
        current_home_value: f64,
        annual_hoa_fees: f64,
    ) -> f64 {
        if !self.enable_tax_benefits {
            return 0.0;
        }

        let tax_year = TAX_BASE_YEAR + year.saturating_sub(1) as i32;
        let cap = salt_cap(tax_year, self.household_income);

        let annual_property_tax = current_home_value * (self.property_tax_rate / 100.0);
        let state_income_tax = self.annual_state_income_tax();

        // SALT: income tax + property tax share one cap. Without the home, only income tax.
        let salt_with_home = (state_income_tax + annual_property_tax).min(cap);
        let salt_without_home = state_income_tax.min(cap);

        let hoa_deduction = annual_hoa_fees * (self.hoa_deduction_percent / 100.0);

        let itemized_with_home = salt_with_home
            + annual_mortgage_interest
            + hoa_deduction
            + self.other_itemized_deductions;
        let itemized_without_home = salt_without_home + self.other_itemized_deductions;

        let standard_deduction = self.filing_status.standard_deduction();
        let deduction_with_home = itemized_with_home.max(standard_deduction);
        let deduction_without_home = itemized_without_home.max(standard_deduction);

        let incremental_deduction = (deduction_with_home - deduction_without_home).max(0.0);
        incremental_deduction * (self.marginal_tax_rate / 100.0)
    }

    /// Tax owed on the home sale gain if sold after `months_owned` months.
    ///
    /// Gain = sale price - selling costs - purchase basis (price + purchase closing costs).
    /// The Section 121 exclusion ($250k single / $500k joint) applies once the home has been
    /// owned and lived in for 2+ years. The remainder is taxed at `capital_gains_rate` (%).
    pub fn home_sale_capital_gains_tax(
        &self,
        sale_price: f64,
        selling_costs: f64,
        months_owned: usize,
        capital_gains_rate: f64,
    ) -> f64 {
        let basis = self.home_price + self.closing_costs_purchase();
        let gain = sale_price - selling_costs - basis;
        let exclusion = if months_owned >= 24 {
            self.filing_status.home_sale_exclusion()
        } else {
            0.0
        };
        (gain - exclusion).max(0.0) * (capital_gains_rate.clamp(0.0, 100.0) / 100.0)
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
        assert_eq!(data.interest_rate, default_interest_rate()); // FRED rate at build time, or 5.99 fallback
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

    #[test]
    fn test_tax_savings_only_counts_incremental_housing_deductions() {
        let mut data = PropertyData::new();
        data.filing_status = FilingStatus::Single; // $16,100 standard deduction
        data.marginal_tax_rate = 20.0;
        data.home_price = 500000.0;
        data.property_tax_rate = 1.0; // $5,000 property tax
        data.state_income_tax_rate = 0.0; // isolate housing deductions

        // Below standard deduction overall: no benefit
        assert_eq!(data.calculate_annual_tax_savings(1, 5000.0, 500000.0, 0.0), 0.0);

        // Housing pushes itemized over standard: benefit on the excess only
        // 5,000 + 15,000 = 20,000 -> excess 3,900 -> $780
        let savings = data.calculate_annual_tax_savings(1, 15000.0, 500000.0, 0.0);
        assert!((savings - 780.0).abs() < 1e-6);

        // Non-housing deductions already exceed standard: the renter would itemize too,
        // so the benefit is the full housing deduction, not the total excess
        data.other_itemized_deductions = 20000.0;
        let savings = data.calculate_annual_tax_savings(1, 15000.0, 500000.0, 0.0);
        assert!((savings - 20000.0 * 0.20).abs() < 1e-6);
    }

    #[test]
    fn test_salt_cap_schedule_and_phase_down() {
        assert_eq!(salt_cap(2026, 150000.0), 40400.0);
        assert_eq!(salt_cap(2029, 150000.0), 41624.0);
        // 30% of MAGI above the $505,000 threshold: 40,400 - 0.3 * 50,000
        assert!((salt_cap(2026, 555000.0) - 25400.0).abs() < 1e-6);
        // Never below the $10,000 floor
        assert_eq!(salt_cap(2026, 700000.0), 10000.0);
        // Reverts to $10,000 in 2030
        assert_eq!(salt_cap(2030, 150000.0), 10000.0);
    }

    #[test]
    fn test_state_income_tax_shares_salt_cap() {
        let mut data = PropertyData::new();
        data.filing_status = FilingStatus::Single; // $16,100 standard deduction
        data.marginal_tax_rate = 20.0;
        data.home_price = 500000.0;
        data.property_tax_rate = 1.0; // $5,000 property tax
        data.household_income = 200000.0;
        data.state_income_tax_rate = 10.0; // $20,000 state income tax

        // 2026: renter already itemizes $20k SALT; owning adds the $5k property tax
        let savings = data.calculate_annual_tax_savings(1, 0.0, 500000.0, 0.0);
        assert!((savings - 5000.0 * 0.20).abs() < 1e-6);

        // 2030 (year 5): SALT cap drops to $10k, already used up by income tax alone
        assert_eq!(data.calculate_annual_tax_savings(5, 0.0, 500000.0, 0.0), 0.0);
    }

    #[test]
    fn test_home_sale_capital_gains_tax() {
        let mut data = PropertyData::new();
        data.filing_status = FilingStatus::Single;
        data.home_price = 500000.0;
        data.closing_costs_percent_purchase = 4.0; // basis = 520,000
        data.seller_closing_assistance = 0.0;

        // Gain = 900,000 - 54,000 - 520,000 = 326,000
        // After 2+ years: 326,000 - 250,000 exclusion = 76,000 taxable at 15%
        let tax = data.home_sale_capital_gains_tax(900000.0, 54000.0, 24, 15.0);
        assert!((tax - 11400.0).abs() < 1e-6);
        // Before 2 years: no exclusion
        let tax = data.home_sale_capital_gains_tax(900000.0, 54000.0, 12, 15.0);
        assert!((tax - 48900.0).abs() < 1e-6);
        // Loss: no tax
        assert_eq!(data.home_sale_capital_gains_tax(400000.0, 24000.0, 60, 15.0), 0.0);
    }

    #[test]
    fn test_hpap_disabled_has_no_effect() {
        let data = PropertyData::new();
        assert!(!data.enable_hpap);
        assert_eq!(data.hpap_total_loan(), 0.0);
        assert_eq!(data.buyer_cash_down_payment(), data.down_payment_amount());
        assert_eq!(data.buyer_cash_closing_costs(), data.closing_costs_purchase());
        assert_eq!(data.loan_amount(), 575000.0 - 575000.0 * 0.03);
        assert_eq!(data.hpap_balance_after_month(12), 0.0);
    }

    #[test]
    fn test_hpap_gap_financing() {
        let mut data = PropertyData::new();
        data.enable_hpap = true;
        data.hpap_loan_amount = 100000.0;
        data.hpap_closing_cost_assistance = 4000.0;

        let down_payment = 575000.0 * 0.03; // 17,250
        assert_eq!(data.hpap_down_payment_applied(), down_payment);
        assert_eq!(data.buyer_cash_down_payment(), 0.0);
        assert_eq!(data.hpap_mortgage_reduction(), 100000.0 - down_payment);
        assert_eq!(data.loan_amount(), 575000.0 - 100000.0);
        assert_eq!(data.buyer_cash_closing_costs(), 575000.0 * 0.04 - 4000.0);
        assert_eq!(data.hpap_total_loan(), 104000.0);
    }

    #[test]
    fn test_hpap_smaller_than_down_payment() {
        let mut data = PropertyData::new();
        data.enable_hpap = true;
        data.down_payment_percent = 20.0; // 115,000
        data.hpap_loan_amount = 80800.0;

        assert_eq!(data.hpap_down_payment_applied(), 80800.0);
        assert_eq!(data.hpap_mortgage_reduction(), 0.0);
        assert_eq!(data.buyer_cash_down_payment(), 115000.0 - 80800.0);
        assert_eq!(data.loan_amount(), 575000.0 - 115000.0);
    }

    #[test]
    fn test_hpap_deferred_until_sale() {
        let mut data = PropertyData::new();
        data.enable_hpap = true;
        data.hpap_loan_amount = 100000.0;
        data.hpap_repayment = HpapRepayment::DeferredUntilSale;

        let total = data.hpap_total_loan();
        for month in [1, 60, 61, 360, 600] {
            assert_eq!(data.hpap_monthly_payment(month), 0.0);
            assert_eq!(data.hpap_balance_after_month(month), total);
        }
    }

    #[test]
    fn test_hpap_five_year_deferral_forty_year_amortization() {
        let mut data = PropertyData::new();
        data.enable_hpap = true;
        data.hpap_loan_amount = 96000.0;
        data.hpap_closing_cost_assistance = 0.0;
        data.hpap_repayment = HpapRepayment::FiveYearDeferral40Year;

        let total = data.hpap_total_loan();
        assert_eq!(total, 96000.0);
        let payment = total / 480.0; // $200/mo

        assert_eq!(data.hpap_monthly_payment(60), 0.0);
        assert_eq!(data.hpap_monthly_payment(61), payment);
        assert_eq!(data.hpap_monthly_payment(540), payment);
        assert_eq!(data.hpap_monthly_payment(541), 0.0);
        assert_eq!(data.hpap_scheduled_payment(), payment);

        assert_eq!(data.hpap_balance_after_month(60), total);
        assert_eq!(data.hpap_balance_after_month(72), total - 12.0 * payment);
        assert!(data.hpap_balance_after_month(540).abs() < 1e-6);
    }
}
