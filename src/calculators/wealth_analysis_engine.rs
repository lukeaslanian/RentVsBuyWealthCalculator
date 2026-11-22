use crate::calculators::{BreakEvenAnalyzer, LoanSchedule, RefinancePlan};
#[cfg(test)]
use crate::models::CityPreset;
use crate::models::{FinancialResults, InvestmentParameters, PropertyData, RentalData};
use crate::utils::AppConfig;

/// The main calculation engine that performs all financial analysis
/// Implements Algorithm 2 from the proposal (Section 4.1): Wealth Accumulation Calculator
pub struct WealthAnalysisEngine {
    property_data: PropertyData,
    rental_data: RentalData,
    investment_params: InvestmentParameters,
    results: Option<FinancialResults>,
}

impl WealthAnalysisEngine {
    /// Create a new WealthAnalysisEngine
    pub fn new(
        property_data: PropertyData,
        rental_data: RentalData,
        investment_params: InvestmentParameters,
    ) -> Self {
        Self {
            property_data,
            rental_data,
            investment_params,
            results: None,
        }
    }

    /// Run the complete analysis
    /// This is the main method that orchestrates all calculations
    pub fn run_analysis(&mut self) {
        // Initialize results object
        let mut results = FinancialResults::new(self.investment_params.analysis_years);

        // First-mortgage schedule (effective rate after points), switching to the new loan
        // after a refinance if one is planned
        let schedule = LoanSchedule::from_property_data(&self.property_data);
        results.monthly_mortgage_payment = schedule.original_payment();
        if let Some(plan) = schedule.refinance_plan() {
            results.refinance_monthly_payment = Some(plan.new_payment);
            results.refinance_after_years = Some(plan.after_month / AppConfig::MONTHS_PER_YEAR);
        }

        // Run all calculations
        self.calculate_investment_growth(&mut results, &schedule);
        self.calculate_total_wealth(&mut results);
        self.find_break_even_point(&mut results);

        self.results = Some(results);
    }

    /// Get the results of the analysis
    pub fn results(&self) -> Option<&FinancialResults> {
        self.results.as_ref()
    }

    /// Calculate investment portfolio growth for both buyer and renter
    ///
    /// FINANCIAL LOGIC:
    /// - Buyer: Pays down payment upfront (no portfolio).
    ///   Each month, if renting costs MORE than owning, the buyer invests the difference.
    ///   Buyer also builds home equity through mortgage payments and appreciation.
    ///   If tax benefits are enabled, annual tax savings are added to the buyer's portfolio.
    ///
    /// - Renter: Invests the saved down payment and closing costs immediately.
    ///   Each month, if owning costs MORE than renting, the renter invests the difference.
    ///   Renter has no equity but grows portfolio faster when renting is cheaper.
    ///
    /// Both portfolios grow with compound interest monthly.
    fn calculate_investment_growth(&self, results: &mut FinancialResults, schedule: &LoanSchedule) {
        let refinance = schedule.refinance_plan();
        let total_months = self.investment_params.analysis_years * AppConfig::MONTHS_PER_YEAR;

        // STEP 1: Initialize portfolios
        // Buyer starts at $0 (spent down payment), Renter starts with down payment invested.
        // Only the buyer's own cash counts: any DC HPAP assistance covering the down payment
        // or closing costs is a loan, not buyer cash (it is repaid via payments / at sale).
        let buy_start_capital = self.property_data.buyer_cash_at_closing();

        // Renter's initial costs: security deposit + broker fee (if enabled)
        let security_deposit = self.rental_data.security_deposit;
        let broker_fee = self.rental_data.broker_fee_amount();
        let rent_initial_costs = security_deposit + broker_fee;

        // Whoever spends less upfront invests the difference. Normally that's the renter
        // (they didn't spend a down payment), but with low-cash-down buying (e.g., HPAP)
        // the renter's deposit + broker fee can exceed the buyer's cash at closing.
        let upfront_difference = buy_start_capital - rent_initial_costs;

        let mut running_buy_portfolio = (-upfront_difference).max(0.0);
        let mut running_rent_portfolio = upfront_difference.max(0.0);

        // Cost basis (total contributions) for capital gains tax on liquidation
        let mut buy_cost_basis = running_buy_portfolio;
        let mut rent_cost_basis = running_rent_portfolio;

        // STEP 2: Track cumulative costs
        let mut buy_cumulative_cost = buy_start_capital;
        let mut rent_cumulative_cost = rent_initial_costs;

        // STEP 3: Track home value with appreciation
        let mut current_home_value = self.property_data.home_price;
        // Taxable assessed value: starts at the purchase price; with an assessment cap it is
        // reassessed once a year and can grow no faster than the cap
        let mut assessed_value = self.property_data.home_price;
        // Home value when refinancing (the new lender's appraisal, used for PMI)
        let mut refinance_home_value: Option<f64> = None;
        let monthly_appreciation_rate =
            self.property_data.home_appreciation_rate / 100.0 / AppConfig::MONTHS_PER_YEAR as f64;

        // STEP 4: MAIN SIMULATION LOOP
        for month in 1..=total_months {
            let year = (month - 1) / AppConfig::MONTHS_PER_YEAR + 1;

            // Apply monthly home appreciation
            current_home_value *= 1.0 + monthly_appreciation_rate;

            // Property tax base: capped assessments step up once a year; uncapped ones
            // follow market value
            if !self.property_data.enable_assessment_cap {
                assessed_value = current_home_value;
            } else if month > 1 && (month - 1) % AppConfig::MONTHS_PER_YEAR == 0 {
                assessed_value = self
                    .property_data
                    .next_assessed_value(assessed_value, current_home_value);
            }

            // Remaining balance must be computed before ownership costs so PMI drop-off can be applied
            let month_index = month - 1;
            let remaining_balance = schedule.remaining_balance(month);
            // PMI eligibility is based on LTV at the START of the month (before this payment is made)
            let pre_payment_balance = schedule.remaining_balance(month - 1);
            let pmi = self.monthly_pmi(month, pre_payment_balance, refinance, refinance_home_value);

            // Calculate this month's costs for both scenarios
            // HPAP repayment (0% interest, moderate-income tier only) is an ownership cost,
            // and so is any cash paid at a refinance (closing costs and/or HPAP payoff)
            let mut ownership_costs = self.calculate_monthly_ownership_costs(
                current_home_value,
                assessed_value,
                year,
                schedule.monthly_payment(month),
                pmi,
            ) + self.hpap_payment(month, refinance);
            if let Some(plan) = refinance {
                if month == plan.after_month {
                    ownership_costs += plan.cash_due();
                    refinance_home_value = Some(current_home_value);
                }
            }
            let rental_costs = self.calculate_monthly_rental_costs(year);

            // Track total out-of-pocket spending to date
            buy_cumulative_cost += ownership_costs;
            rent_cumulative_cost += rental_costs;

            // CRITICAL STEP: Update investment portfolios (and their cost basis)
            self.update_portfolios(
                &mut running_buy_portfolio,
                &mut running_rent_portfolio,
                &mut buy_cost_basis,
                &mut rent_cost_basis,
                ownership_costs,
                rental_costs,
            );

            let is_year_end = month % AppConfig::MONTHS_PER_YEAR == 0;

            // At the end of each year, apply homeowner tax benefits.
            // Tax savings are added to the buyer's portfolio (as if they're investing their refund).
            if is_year_end && self.property_data.enable_tax_benefits {
                let annual_interest = schedule.interest_paid_in_year(year);
                let annual_hoa_fees = self.property_data.hoa_fee_at_year(year) * 12.0;
                let tax_savings = self.property_data.calculate_annual_tax_savings(
                    year,
                    annual_interest,
                    assessed_value,
                    annual_hoa_fees,
                );

                running_buy_portfolio += tax_savings;
                buy_cost_basis += tax_savings;
                // Reduce cumulative costs by tax savings (they get money back)
                buy_cumulative_cost -= tax_savings;
            }

            // Net worth if everything were cashed out this month.
            // Equity is net of all liens: first mortgage + any outstanding HPAP balance
            let equity =
                current_home_value - remaining_balance - self.hpap_balance(month, refinance);
            let selling_costs =
                current_home_value * (self.property_data.closing_costs_percent_sale / 100.0);
            let home_sale_tax = self.property_data.home_sale_capital_gains_tax(
                current_home_value,
                selling_costs,
                month,
                self.investment_params.capital_gains_tax_rate,
            );
            let buy_portfolio_after_tax = self
                .investment_params
                .after_tax_portfolio_value(running_buy_portfolio, buy_cost_basis);
            let rent_portfolio_after_tax = self
                .investment_params
                .after_tax_portfolio_value(running_rent_portfolio, rent_cost_basis);

            // Store monthly data for charts (0-indexed). Portfolio series are pre-tax balances;
            // total wealth is after selling costs and capital gains taxes.
            results.monthly_buy_cumulative_costs[month_index] = buy_cumulative_cost;
            results.monthly_rent_cumulative_costs[month_index] = rent_cumulative_cost;
            results.monthly_home_equity[month_index] = equity;
            results.monthly_buy_investment_portfolio[month_index] = running_buy_portfolio;
            results.monthly_rent_investment_portfolio[month_index] = running_rent_portfolio;
            results.monthly_buy_total_wealth[month_index] =
                equity - selling_costs - home_sale_tax + buy_portfolio_after_tax;
            results.monthly_rent_total_wealth[month_index] =
                rent_portfolio_after_tax + security_deposit;
            results.monthly_buy_costs[month_index] = ownership_costs;
            results.monthly_rent_costs[month_index] = rental_costs;

            // Store yearly snapshot
            if is_year_end {
                self.store_yearly_results(
                    results,
                    month,
                    year,
                    current_home_value,
                    buy_cumulative_cost,
                    rent_cumulative_cost,
                    ownership_costs,
                    rental_costs,
                    running_buy_portfolio,
                    running_rent_portfolio,
                    schedule,
                    refinance,
                );
            }
        }
    }

    fn calculate_monthly_ownership_costs(
        &self,
        current_home_value: f64,
        assessed_value: f64,
        year: usize,
        monthly_mortgage: f64,
        pmi: f64,
    ) -> f64 {
        // Get inflation multiplier for this year
        let inflation_mult = self.investment_params.inflation_multiplier(year);

        let monthly_property_tax = ((assessed_value * self.property_data.property_tax_rate)
            / 100.0)
            / AppConfig::MONTHS_PER_YEAR as f64;

        // Insurance grows with inflation
        let monthly_insurance = (self.property_data.home_insurance_annual * inflation_mult)
            / AppConfig::MONTHS_PER_YEAR as f64;

        let monthly_maintenance = ((current_home_value * self.property_data.maintenance_percent)
            / 100.0)
            / AppConfig::MONTHS_PER_YEAR as f64;

        let current_hoa_fee = self.property_data.hoa_fee_at_year(year);

        // Utilities grow with inflation
        let utilities = self.property_data.non_included_utilities * inflation_mult;

        monthly_mortgage
            + monthly_property_tax
            + monthly_insurance
            + monthly_maintenance
            + current_hoa_fee
            + utilities
            + pmi
    }

    /// PMI due in `month`. Original loan: rate x loan (0 if it starts at or below 80% LTV),
    /// until the balance falls to pmi_drop_off_ltv of the purchase price. After a refinance,
    /// the same rules apply to the new loan against the home's value at the refinance.
    fn monthly_pmi(
        &self,
        month: usize,
        pre_payment_balance: f64,
        refinance: Option<&RefinancePlan>,
        refinance_home_value: Option<f64>,
    ) -> f64 {
        let (monthly_pmi, value_basis) = match (refinance, refinance_home_value) {
            (Some(plan), Some(home_value)) if month > plan.after_month && home_value > 0.0 => {
                let starts_above_80 =
                    plan.new_principal / home_value * 100.0 > AppConfig::PMI_REQUIRED_ABOVE_LTV;
                let pmi = if starts_above_80 {
                    plan.new_principal * self.property_data.pmi_rate_percent / 100.0 / 12.0
                } else {
                    0.0
                };
                (pmi, home_value)
            }
            _ => (
                self.property_data.monthly_pmi(),
                self.property_data.home_price,
            ),
        };
        let ltv = pre_payment_balance / value_basis * 100.0;
        if monthly_pmi > 0.0 && ltv > self.property_data.pmi_drop_off_ltv {
            monthly_pmi
        } else {
            0.0
        }
    }

    /// HPAP payment due in `month` (none after a refinance that pays HPAP off)
    fn hpap_payment(&self, month: usize, refinance: Option<&RefinancePlan>) -> f64 {
        match refinance {
            Some(plan) if plan.pays_off_hpap(&self.property_data) && month > plan.after_month => {
                0.0
            }
            _ => self.property_data.hpap_monthly_payment(month),
        }
    }

    /// HPAP balance after `month` (zero once a refinance pays HPAP off)
    fn hpap_balance(&self, month: usize, refinance: Option<&RefinancePlan>) -> f64 {
        match refinance {
            Some(plan) if plan.pays_off_hpap(&self.property_data) && month >= plan.after_month => {
                0.0
            }
            _ => self.property_data.hpap_balance_after_month(month),
        }
    }

    fn calculate_monthly_rental_costs(&self, year: usize) -> f64 {
        // Get inflation multiplier for this year
        let inflation_mult = self.investment_params.inflation_multiplier(year);

        let current_rent = self.rental_data.rent_at_year(year);

        // Amenity fees, utilities, and renters insurance grow with inflation
        let amenity_fees = self.rental_data.amenity_fees * inflation_mult;
        let utilities = self.rental_data.rent_non_included_utilities * inflation_mult;
        let renters_insurance = self.rental_data.renters_insurance * inflation_mult;

        current_rent + amenity_fees + utilities + renters_insurance
    }

    /// Update both investment portfolios with compound interest and monthly savings
    ///
    /// KEY: Whoever pays LESS this month gets to invest the difference.
    fn update_portfolios(
        &self,
        buy_portfolio: &mut f64,
        rent_portfolio: &mut f64,
        buy_cost_basis: &mut f64,
        rent_cost_basis: &mut f64,
        ownership_costs: f64,
        rental_costs: f64,
    ) {
        let monthly_investment_rate = self.investment_params.monthly_return_rate();

        // STEP 1: Apply compound interest to existing portfolios
        *buy_portfolio *= 1.0 + monthly_investment_rate;
        *rent_portfolio *= 1.0 + monthly_investment_rate;

        // STEP 2: Add monthly savings (whoever pays less invests the difference)
        if ownership_costs < rental_costs {
            // Buying is cheaper this month - buyer invests the savings
            *buy_portfolio += rental_costs - ownership_costs;
            *buy_cost_basis += rental_costs - ownership_costs;
        } else if rental_costs < ownership_costs {
            // Renting is cheaper this month - renter invests the savings
            *rent_portfolio += ownership_costs - rental_costs;
            *rent_cost_basis += ownership_costs - rental_costs;
        }
        // If costs are exactly equal, neither portfolio gets additional contribution
    }

    #[allow(clippy::too_many_arguments)]
    fn store_yearly_results(
        &self,
        results: &mut FinancialResults,
        month: usize,
        year: usize,
        current_home_value: f64,
        buy_cumulative_cost: f64,
        rent_cumulative_cost: f64,
        current_ownership_costs: f64,
        current_rental_costs: f64,
        running_buy_portfolio: f64,
        running_rent_portfolio: f64,
        schedule: &LoanSchedule,
        refinance: Option<&RefinancePlan>,
    ) {
        let year_index = year - 1;

        // Calculate equity (net of first mortgage and any outstanding HPAP balance)
        let remaining_balance = schedule.remaining_balance(month);
        let equity = current_home_value - remaining_balance - self.hpap_balance(month, refinance);

        results.buy_cumulative_costs[year_index] = buy_cumulative_cost;
        results.rent_cumulative_costs[year_index] = rent_cumulative_cost;
        results.home_equity[year_index] = equity;
        results.home_value[year_index] = current_home_value;
        results.mortgage_balance[year_index] = remaining_balance;
        results.buy_investment_portfolio[year_index] = running_buy_portfolio;
        results.rent_investment_portfolio[year_index] = running_rent_portfolio;
        results.buy_monthly_costs[year_index] = current_ownership_costs;
        results.rent_monthly_costs[year_index] = current_rental_costs;
    }

    /// Yearly total wealth for both scenarios, taken from the year-end monthly values
    /// (after selling costs, HPAP payoff, and capital gains taxes on liquidation)
    fn calculate_total_wealth(&self, results: &mut FinancialResults) {
        let years = self.investment_params.analysis_years;

        for year in 0..years {
            let year_end_month_index = (year + 1) * AppConfig::MONTHS_PER_YEAR - 1;
            results.buy_total_wealth[year] = results.monthly_buy_total_wealth[year_end_month_index];
            results.rent_total_wealth[year] =
                results.monthly_rent_total_wealth[year_end_month_index];
        }
    }

    /// Find the break-even point where buying becomes better than renting
    fn find_break_even_point(&self, results: &mut FinancialResults) {
        let analyzer =
            BreakEvenAnalyzer::new(&results.buy_total_wealth, &results.rent_total_wealth);

        // Find all break-even years and set the description
        results.all_break_even_years = analyzer.find_all_break_even_years();
        results.break_even_description = analyzer.break_even_description();

        // Keep the old single break-even year for backward compatibility
        results.break_even_year = analyzer.find_break_even_year();
    }

    /// Calculate home value at a specific year with appreciation
    pub fn calculate_home_value_at_year(&self, year: usize) -> f64 {
        let appreciation_rate = self.property_data.home_appreciation_rate / 100.0;
        self.property_data.home_price * (1.0 + appreciation_rate).powi(year as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_analysis() {
        let property_data = PropertyData::new();
        let rental_data = RentalData::new();
        let investment_params = InvestmentParameters::new();

        let mut engine = WealthAnalysisEngine::new(property_data, rental_data, investment_params);
        engine.run_analysis();

        let results = engine.results().expect("Results should be present");

        // Check that arrays are populated
        assert_eq!(results.buy_total_wealth.len(), 30);
        assert_eq!(results.rent_total_wealth.len(), 30);

        // Check that monthly mortgage payment was calculated
        assert!(results.monthly_mortgage_payment > 0.0);

        // Check that some wealth accumulates
        assert!(results.buy_total_wealth[29] > 0.0 || results.rent_total_wealth[29] > 0.0);
    }

    fn run(property_data: PropertyData) -> FinancialResults {
        let mut engine = WealthAnalysisEngine::new(
            property_data,
            RentalData::new(),
            InvestmentParameters::new(),
        );
        engine.run_analysis();
        engine.results().expect("Results should be present").clone()
    }

    /// Default DC 2BR with HPAP on for a 1-person household earning $70,000
    /// (51-80% MFI: $161,600 max, no monthly payments)
    fn hpap_property_data() -> PropertyData {
        let mut data = PropertyData::new();
        data.enable_hpap = true;
        data.hpap_enter_own_max = false; // use the DHCD table
        data.household_income = 70000.0;
        data.hpap_household_size = 1;
        data
    }

    #[test]
    fn test_hpap_disabled_matches_baseline() {
        let baseline = run(PropertyData::new());
        let mut toggled = PropertyData::new();
        toggled.hpap_household_size = 4;
        toggled.hpap_liquid_assets = 50000.0;
        toggled.hpap_closing_cost_assistance = 1000.0;
        // enable_hpap stays false: other HPAP fields must be ignored
        assert_eq!(baseline, run(toggled));
    }

    #[test]
    fn test_hpap_lowers_buyer_cash_not_first_mortgage() {
        let mut no_hpap = hpap_property_data();
        no_hpap.enable_hpap = false;
        let baseline = run(no_hpap);
        let with_hpap_data = hpap_property_data();
        let with_hpap = run(with_hpap_data.clone());

        // Same down payment = same first mortgage
        assert_eq!(
            with_hpap.monthly_mortgage_payment,
            baseline.monthly_mortgage_payment
        );
        // Less buyer cash, so the renter starts with less invested
        assert!(
            with_hpap.monthly_rent_investment_portfolio[0]
                < baseline.monthly_rent_investment_portfolio[0]
        );
        // Deferred tier: full HPAP balance comes out of equity
        let expected_equity_year1 = with_hpap.home_value[0]
            - with_hpap.mortgage_balance[0]
            - with_hpap_data.hpap_total_loan();
        assert!((with_hpap.home_equity[0] - expected_equity_year1).abs() < 1e-6);
    }

    #[test]
    fn test_full_hpap_lowers_first_mortgage() {
        let baseline = run(hpap_property_data());
        let mut full = hpap_property_data();
        full.down_payment_percent = full.hpap_full_use_down_payment_percent().unwrap();
        let full_results = run(full);
        assert!(full_results.monthly_mortgage_payment < baseline.monthly_mortgage_payment);
    }

    #[test]
    fn test_hpap_moderate_tier_payments_start_in_year_six() {
        let mut moderate = hpap_property_data();
        moderate.household_income = 120000.0; // 81-110% MFI
        let mut no_hpap = moderate.clone();
        no_hpap.enable_hpap = false;

        let moderate_results = run(moderate.clone());
        let baseline = run(no_hpap);

        // First mortgage is identical, so monthly costs differ only by the HPAP payment
        assert_eq!(
            moderate_results.monthly_buy_costs[59],
            baseline.monthly_buy_costs[59]
        );
        let diff = moderate_results.monthly_buy_costs[60] - baseline.monthly_buy_costs[60];
        assert!((diff - moderate.hpap_scheduled_payment()).abs() < 1e-6);
        assert!(moderate.hpap_scheduled_payment() > 0.0);
    }

    #[test]
    fn test_buyer_invests_upfront_surplus_when_renter_spends_more() {
        let mut property_data = hpap_property_data();
        property_data.hpap_liquid_assets = 0.0; // $500 required contribution
        property_data.closing_costs_percent_purchase = 0.5; // fully covered by assistance

        let results = run(property_data.clone());
        // Buyer pays only the $500 contribution; renter pays a $3,200 deposit up front,
        // so the buyer starts with the difference invested
        assert!((property_data.buyer_cash_at_closing() - 500.0).abs() < 1e-6);
        assert!(results.monthly_buy_investment_portfolio[0] > 0.0);
    }

    #[test]
    fn test_assessment_cap_lowers_property_tax_over_time() {
        let mut capped = PropertyData::new();
        capped.home_appreciation_rate = 6.0;
        capped.enable_assessment_cap = true;
        capped.assessment_cap_percent = 2.0;
        let mut uncapped = capped.clone();
        uncapped.enable_assessment_cap = false;

        let capped_results = run(capped.clone());
        let uncapped_results = run(uncapped);

        // Year 1 with a cap: taxed on the purchase price
        let month1_tax_diff =
            uncapped_results.monthly_buy_costs[0] - capped_results.monthly_buy_costs[0];
        let appreciated = capped.home_price * (1.0 + 0.06 / 12.0);
        let expected = (appreciated - capped.home_price) * capped.property_tax_rate / 100.0 / 12.0;
        assert!((month1_tax_diff - expected).abs() < 1e-6);

        // Year 10: the capped owner pays noticeably less each month
        assert!(
            uncapped_results.monthly_buy_costs[119] - capped_results.monthly_buy_costs[119] > 100.0
        );
    }

    #[test]
    fn test_no_pmi_when_first_mortgage_starts_at_or_below_80_ltv() {
        let mut twenty_down = PropertyData::new();
        twenty_down.down_payment_percent = 20.0;
        let mut no_pmi = twenty_down.clone();
        no_pmi.pmi_rate_percent = 0.0;
        assert_eq!(
            run(twenty_down).monthly_buy_costs[0],
            run(no_pmi).monthly_buy_costs[0]
        );

        // 3% down: PMI applies at rate x loan
        let three_down = PropertyData::new();
        let mut three_down_no_pmi = three_down.clone();
        three_down_no_pmi.pmi_rate_percent = 0.0;
        let diff = run(three_down.clone()).monthly_buy_costs[0]
            - run(three_down_no_pmi).monthly_buy_costs[0];
        assert!((diff - three_down.monthly_pmi()).abs() < 1e-6);
        assert!(diff > 200.0);
    }

    /// Every number in the results must be finite (no NaN or infinity)
    fn assert_all_finite(results: &FinancialResults, label: &str) {
        let series: [(&str, &Vec<f64>); 20] = [
            ("buy_cumulative_costs", &results.buy_cumulative_costs),
            ("rent_cumulative_costs", &results.rent_cumulative_costs),
            ("home_equity", &results.home_equity),
            ("home_value", &results.home_value),
            ("mortgage_balance", &results.mortgage_balance),
            (
                "buy_investment_portfolio",
                &results.buy_investment_portfolio,
            ),
            (
                "rent_investment_portfolio",
                &results.rent_investment_portfolio,
            ),
            ("buy_total_wealth", &results.buy_total_wealth),
            ("rent_total_wealth", &results.rent_total_wealth),
            ("buy_monthly_costs", &results.buy_monthly_costs),
            ("rent_monthly_costs", &results.rent_monthly_costs),
            (
                "monthly_buy_cumulative_costs",
                &results.monthly_buy_cumulative_costs,
            ),
            (
                "monthly_rent_cumulative_costs",
                &results.monthly_rent_cumulative_costs,
            ),
            (
                "monthly_buy_total_wealth",
                &results.monthly_buy_total_wealth,
            ),
            (
                "monthly_rent_total_wealth",
                &results.monthly_rent_total_wealth,
            ),
            ("monthly_home_equity", &results.monthly_home_equity),
            (
                "monthly_buy_investment_portfolio",
                &results.monthly_buy_investment_portfolio,
            ),
            (
                "monthly_rent_investment_portfolio",
                &results.monthly_rent_investment_portfolio,
            ),
            ("monthly_buy_costs", &results.monthly_buy_costs),
            ("monthly_rent_costs", &results.monthly_rent_costs),
        ];
        for (name, values) in series {
            assert!(
                values.iter().all(|v| v.is_finite()),
                "{label}: non-finite value in {name}"
            );
        }
        assert!(
            results.monthly_mortgage_payment.is_finite(),
            "{label}: mortgage payment"
        );
    }

    /// A named tweak to the inputs, for scenario sweeps
    type HpapVariant = (&'static str, fn(&mut PropertyData));
    type EdgeVariant = (&'static str, fn(&mut PropertyData, &mut RentalData));

    #[test]
    fn test_all_presets_valid_and_finite_across_scenarios() {
        // HPAP setups to try on every preset
        let hpap_variants: [HpapVariant; 4] = [
            ("hpap off", |_| {}),
            ("hpap own max", |p| {
                p.enable_hpap = true;
                p.hpap_enter_own_max = true;
                p.hpap_own_max = 80000.0;
            }),
            ("hpap own max, full use", |p| {
                p.enable_hpap = true;
                p.hpap_enter_own_max = true;
                p.hpap_own_max = 202000.0;
                p.hpap_own_repayment = crate::models::HpapRepayment::FiveYearDeferral40Year;
                p.down_payment_percent = p.hpap_full_use_down_payment_percent().unwrap();
            }),
            ("hpap table", |p| {
                p.enable_hpap = true;
                p.hpap_enter_own_max = false;
                p.household_income = 70000.0;
            }),
        ];
        // Edge-case inputs to try on every preset
        let edge_variants: [EdgeVariant; 11] = [
            ("as preset", |_, _| {}),
            ("0% down", |p, _| p.down_payment_percent = 0.0),
            ("100% down", |p, _| p.down_payment_percent = 100.0),
            ("0% interest", |p, _| p.interest_rate = 0.0),
            ("negative appreciation", |p, _| {
                p.home_appreciation_rate = -5.0
            }),
            ("0 rent growth", |_, r| r.rent_increase_rate = 0.0),
            ("2 points", |p, _| p.mortgage_points = 2.0),
            ("no tax benefits", |p, _| p.enable_tax_benefits = false),
            ("zero cap", |p, _| {
                p.enable_assessment_cap = true;
                p.assessment_cap_percent = 0.0;
            }),
            ("refinance 15yr cash costs", |p, _| {
                p.enable_refinance = true;
                p.refinance_after_years = 3;
                p.refinance_term = crate::models::RefinanceTerm::Years15;
                p.refinance_cost_payment = crate::models::RefinanceCostPayment::PayCash;
            }),
            ("refinance year 29", |p, _| {
                p.enable_refinance = true;
                p.refinance_after_years = 29;
                p.refinance_term = crate::models::RefinanceTerm::KeepRemaining;
            }),
        ];

        for preset in CityPreset::all_presets() {
            assert!(
                preset.to_property_data().is_valid(),
                "{}",
                preset.display_name()
            );
            assert!(
                preset.to_rental_data().is_valid(),
                "{}",
                preset.display_name()
            );

            for (hpap_label, hpap) in hpap_variants {
                for (edge_label, edge) in edge_variants {
                    let mut p = preset.to_property_data();
                    let mut r = preset.to_rental_data();
                    hpap(&mut p);
                    edge(&mut p, &mut r);
                    let label = format!(
                        "{} / {} / {}",
                        preset.display_name(),
                        hpap_label,
                        edge_label
                    );
                    let mut engine =
                        WealthAnalysisEngine::new(p, r, preset.to_investment_parameters());
                    engine.run_analysis();
                    assert_all_finite(engine.results().expect("results"), &label);
                }
            }
        }
    }

    fn refi_property_data() -> PropertyData {
        let mut data = PropertyData::new();
        data.interest_rate = 7.0;
        data.enable_refinance = true;
        data.refinance_after_years = 5;
        data.refinance_rate = 5.0;
        data
    }

    #[test]
    fn test_refinance_lowers_payments_after_refi_month() {
        let mut no_refi = refi_property_data();
        no_refi.enable_refinance = false;
        let base = run(no_refi);
        let refi = run(refi_property_data());

        // Same until the refinance (closing costs rolled in: no cash that month)
        assert_eq!(base.monthly_buy_costs[59], refi.monthly_buy_costs[59]);
        // Lower payment afterwards
        assert!(refi.monthly_buy_costs[60] < base.monthly_buy_costs[60] - 300.0);
        assert!(refi.refinance_monthly_payment.unwrap() < refi.monthly_mortgage_payment);
        assert_eq!(refi.refinance_after_years, Some(5));
        assert!(base.refinance_monthly_payment.is_none());
    }

    #[test]
    fn test_refinance_cash_costs_hit_refi_month() {
        let mut no_refi = refi_property_data();
        no_refi.enable_refinance = false;
        let mut cash = refi_property_data();
        cash.refinance_cost_payment = crate::models::RefinanceCostPayment::PayCash;
        let plan = RefinancePlan::from_property_data(&cash).unwrap();

        let base = run(no_refi);
        let cash_results = run(cash);
        let extra = cash_results.monthly_buy_costs[59] - base.monthly_buy_costs[59];
        assert!((extra - plan.cash_due()).abs() < 1e-6);
        assert!(plan.cash_due() > 0.0);
    }

    #[test]
    fn test_refinance_rolls_hpap_into_new_loan() {
        let mut data = refi_property_data();
        data.enable_hpap = true;
        data.hpap_enter_own_max = true;
        data.hpap_own_max = 60000.0;
        data.hpap_own_repayment = crate::models::HpapRepayment::FiveYearDeferral40Year;
        data.down_payment_percent = data.hpap_full_use_down_payment_percent().unwrap();
        data.hpap_at_refinance = crate::models::HpapAtRefinance::RollIntoLoan;
        let results = run(data);

        // After the refinance, equity is just value - new mortgage (no separate HPAP lien)
        for year_index in 5..10 {
            let expected = results.home_value[year_index] - results.mortgage_balance[year_index];
            assert!((results.home_equity[year_index] - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn test_refinance_keeps_hpap_when_approved() {
        let mut data = refi_property_data();
        data.enable_hpap = true;
        data.hpap_enter_own_max = true;
        data.hpap_own_max = 60000.0;
        data.down_payment_percent = data.hpap_full_use_down_payment_percent().unwrap();
        data.hpap_at_refinance = crate::models::HpapAtRefinance::Keep;
        let hpap_total = data.hpap_total_loan();
        let results = run(data);

        // HPAP stays as a separate lien after the refinance
        let expected = results.home_value[9] - results.mortgage_balance[9] - hpap_total;
        assert!((results.home_equity[9] - expected).abs() < 1e-6);
    }

    fn run_with_cg_rate(capital_gains_tax_rate: f64) -> FinancialResults {
        let mut params = InvestmentParameters::new();
        params.capital_gains_tax_rate = capital_gains_tax_rate;
        let mut engine = WealthAnalysisEngine::new(PropertyData::new(), RentalData::new(), params);
        engine.run_analysis();
        engine.results().expect("Results should be present").clone()
    }

    #[test]
    fn test_zero_capital_gains_rate_matches_pre_tax_wealth() {
        let results = run_with_cg_rate(0.0);
        let property_data = PropertyData::new();
        for year in 0..results.buy_total_wealth.len() {
            let selling_costs =
                results.home_value[year] * property_data.closing_costs_percent_sale / 100.0;
            let expected_buy =
                results.home_equity[year] + results.buy_investment_portfolio[year] - selling_costs;
            let expected_rent =
                results.rent_investment_portfolio[year] + RentalData::new().security_deposit;
            assert!((results.buy_total_wealth[year] - expected_buy).abs() < 1e-6);
            assert!((results.rent_total_wealth[year] - expected_rent).abs() < 1e-6);
        }
    }

    #[test]
    fn test_capital_gains_tax_reduces_portfolio_wealth() {
        let untaxed = run_with_cg_rate(0.0);
        let taxed = run_with_cg_rate(15.0);
        let last = taxed.rent_total_wealth.len() - 1;

        // Pre-tax portfolio balances are unchanged; only liquidation value drops
        assert_eq!(
            untaxed.rent_investment_portfolio[last],
            taxed.rent_investment_portfolio[last]
        );
        assert!(taxed.rent_total_wealth[last] < untaxed.rent_total_wealth[last]);
        // Renter's portfolio tax can't exceed 15% of the whole balance
        assert!(
            untaxed.rent_total_wealth[last] - taxed.rent_total_wealth[last]
                < 0.15 * untaxed.rent_investment_portfolio[last]
        );
        assert!(taxed.buy_total_wealth[last] <= untaxed.buy_total_wealth[last]);
    }

    #[test]
    fn test_home_value_appreciation() {
        let property_data = PropertyData::new();
        let rental_data = RentalData::new();
        let investment_params = InvestmentParameters::new();

        let engine = WealthAnalysisEngine::new(property_data, rental_data, investment_params);

        // At year 0, value should equal initial price
        assert!((engine.calculate_home_value_at_year(0) - 575000.0).abs() < 1.0);

        // After 10 years at 3.5% appreciation, value should be higher
        let value_year_10 = engine.calculate_home_value_at_year(10);
        assert!(value_year_10 > 575000.0);
    }
}
