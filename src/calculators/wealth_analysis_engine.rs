use crate::calculators::{BreakEvenAnalyzer, MortgageCalculator};
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

        // Calculate mortgage payment using effective interest rate (after points buydown)
        let mortgage_calc = MortgageCalculator::new(
            self.property_data.loan_amount(),
            self.property_data.effective_interest_rate(),
            AppConfig::DEFAULT_MORTGAGE_TERM_YEARS,
        );
        results.monthly_mortgage_payment = mortgage_calc.calculate_monthly_payment();

        // Run all calculations
        self.calculate_investment_growth(&mut results, &mortgage_calc);
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
    fn calculate_investment_growth(
        &self,
        results: &mut FinancialResults,
        mortgage_calc: &MortgageCalculator,
    ) {
        let total_months = self.investment_params.analysis_years * AppConfig::MONTHS_PER_YEAR;

        // STEP 1: Initialize portfolios
        // Buyer starts at $0 (spent down payment), Renter starts with down payment invested
        let down_payment = self.property_data.down_payment_amount();
        let purchase_closing_costs = self.property_data.closing_costs_purchase();
        let mortgage_points_cost = self.property_data.mortgage_points_cost();
        let buy_start_capital = down_payment + purchase_closing_costs + mortgage_points_cost;

        // Renter's initial costs: security deposit + broker fee (if enabled)
        let security_deposit = self.rental_data.security_deposit;
        let broker_fee = self.rental_data.broker_fee_amount();
        let rent_initial_costs = security_deposit + broker_fee;

        // Renter invests the difference between buyer's upfront costs and their own
        // (they get to invest what they didn't spend on a down payment)
        let rent_start_capital = buy_start_capital - rent_initial_costs;

        let mut running_buy_portfolio = 0.0;
        let mut running_rent_portfolio = rent_start_capital.max(0.0);

        // STEP 2: Track cumulative costs
        let mut buy_cumulative_cost = buy_start_capital;
        let mut rent_cumulative_cost = rent_initial_costs;

        // STEP 3: Track home value with appreciation
        let mut current_home_value = self.property_data.home_price;
        let monthly_appreciation_rate =
            self.property_data.home_appreciation_rate / 100.0 / AppConfig::MONTHS_PER_YEAR as f64;

        // STEP 4: MAIN SIMULATION LOOP
        for month in 1..=total_months {
            let year = (month - 1) / AppConfig::MONTHS_PER_YEAR + 1;

            // Apply monthly home appreciation
            current_home_value *= 1.0 + monthly_appreciation_rate;

            // Calculate this month's costs for both scenarios
            let ownership_costs = self.calculate_monthly_ownership_costs(
                current_home_value,
                year,
                results.monthly_mortgage_payment,
            );
            let rental_costs = self.calculate_monthly_rental_costs(year);

            // Track total out-of-pocket spending to date
            buy_cumulative_cost += ownership_costs;
            rent_cumulative_cost += rental_costs;

            // CRITICAL STEP: Update investment portfolios
            self.update_portfolios(
                &mut running_buy_portfolio,
                &mut running_rent_portfolio,
                ownership_costs,
                rental_costs,
            );

            // Store monthly data for charts (0-indexed)
            let month_index = month - 1;
            let remaining_balance = mortgage_calc.calculate_remaining_balance(month);
            let equity = current_home_value - remaining_balance;
            let selling_costs =
                current_home_value * (self.property_data.closing_costs_percent_sale / 100.0);

            results.monthly_buy_cumulative_costs[month_index] = buy_cumulative_cost;
            results.monthly_rent_cumulative_costs[month_index] = rent_cumulative_cost;
            results.monthly_home_equity[month_index] = equity;
            results.monthly_buy_investment_portfolio[month_index] = running_buy_portfolio;
            results.monthly_rent_investment_portfolio[month_index] = running_rent_portfolio;
            results.monthly_buy_total_wealth[month_index] =
                equity + running_buy_portfolio - selling_costs;
            results.monthly_rent_total_wealth[month_index] = running_rent_portfolio;
            results.monthly_buy_costs[month_index] = ownership_costs;
            results.monthly_rent_costs[month_index] = rental_costs;

            // At the end of each year, apply tax benefits and store yearly snapshot
            if month % AppConfig::MONTHS_PER_YEAR == 0 {
                // Calculate and apply tax benefits for homeowner
                // Tax savings are added to the buyer's portfolio (as if they're investing their tax refund)
                if self.property_data.enable_tax_benefits {
                    let annual_interest = mortgage_calc.calculate_interest_paid_in_year(year);
                    // Calculate annual HOA fees for this year (monthly fee * 12)
                    let annual_hoa_fees = self.property_data.hoa_fee_at_year(year) * 12.0;
                    let tax_savings = self.property_data.calculate_annual_tax_savings(
                        annual_interest,
                        current_home_value,
                        annual_hoa_fees,
                    );

                    // Add tax savings to buyer's portfolio
                    running_buy_portfolio += tax_savings;

                    // Reduce cumulative costs by tax savings (they get money back)
                    buy_cumulative_cost -= tax_savings;

                    // Update the monthly data for this month to reflect tax benefits
                    results.monthly_buy_investment_portfolio[month_index] = running_buy_portfolio;
                    results.monthly_buy_cumulative_costs[month_index] = buy_cumulative_cost;
                    results.monthly_buy_total_wealth[month_index] =
                        equity + running_buy_portfolio - selling_costs;
                }

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
                    mortgage_calc,
                );
            }
        }
    }

    fn calculate_monthly_ownership_costs(
        &self,
        current_home_value: f64,
        year: usize,
        monthly_mortgage: f64,
    ) -> f64 {
        // Get inflation multiplier for this year
        let inflation_mult = self.investment_params.inflation_multiplier(year);

        let monthly_property_tax = ((current_home_value * self.property_data.property_tax_rate)
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
        } else if rental_costs < ownership_costs {
            // Renting is cheaper this month - renter invests the savings
            *rent_portfolio += ownership_costs - rental_costs;
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
        mortgage_calc: &MortgageCalculator,
    ) {
        let year_index = year - 1;

        // Calculate equity
        let remaining_balance = mortgage_calc.calculate_remaining_balance(month);
        let equity = current_home_value - remaining_balance;

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

    /// Calculate total wealth for both scenarios
    fn calculate_total_wealth(&self, results: &mut FinancialResults) {
        let years = self.investment_params.analysis_years;

        for year in 0..years {
            // Buyer wealth = equity + investment portfolio - selling costs
            let buy_equity = results.home_equity[year];
            let buy_investments = results.buy_investment_portfolio[year];

            // Subtract closing costs for selling
            let home_value = results.home_value[year];
            let selling_costs =
                home_value * (self.property_data.closing_costs_percent_sale / 100.0);

            results.buy_total_wealth[year] = buy_equity + buy_investments - selling_costs;

            // Renter wealth = investment portfolio only
            results.rent_total_wealth[year] = results.rent_investment_portfolio[year];
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
