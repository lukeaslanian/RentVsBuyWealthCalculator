use serde::{Deserialize, Serialize};

/// Stores all calculated financial outcomes from the analysis.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FinancialResults {
    // All arrays are sized by the number of analysis years (yearly snapshots)
    pub buy_cumulative_costs: Vec<f64>,
    pub rent_cumulative_costs: Vec<f64>,
    pub home_equity: Vec<f64>,      // buyer's equity from home
    pub home_value: Vec<f64>,       // appreciated home value over time
    pub mortgage_balance: Vec<f64>, // remaining mortgage balance over time

    // Both buyer and renter track investment portfolios in tandem
    pub buy_investment_portfolio: Vec<f64>,
    pub rent_investment_portfolio: Vec<f64>,

    pub buy_total_wealth: Vec<f64>,
    pub rent_total_wealth: Vec<f64>,

    // Monthly costs (not cumulative) - yearly snapshots
    pub buy_monthly_costs: Vec<f64>,
    pub rent_monthly_costs: Vec<f64>,

    // Monthly data for charts (sized by total months)
    pub monthly_buy_cumulative_costs: Vec<f64>,
    pub monthly_rent_cumulative_costs: Vec<f64>,
    pub monthly_buy_total_wealth: Vec<f64>,
    pub monthly_rent_total_wealth: Vec<f64>,
    pub monthly_home_equity: Vec<f64>,
    pub monthly_buy_investment_portfolio: Vec<f64>,
    pub monthly_rent_investment_portfolio: Vec<f64>,
    pub monthly_buy_costs: Vec<f64>,  // per-month costs
    pub monthly_rent_costs: Vec<f64>, // per-month costs

    // Break-even analysis
    pub break_even_year: Option<usize>,
    pub all_break_even_years: Vec<usize>, // All crossover years
    pub break_even_description: String,   // Human-readable description
    pub monthly_mortgage_payment: f64,
}

impl FinancialResults {
    /// Constructor initializing arrays for specified years
    pub fn new(years: usize) -> Self {
        let total_months = years * 12;
        Self {
            buy_cumulative_costs: vec![0.0; years],
            rent_cumulative_costs: vec![0.0; years],
            home_equity: vec![0.0; years],
            home_value: vec![0.0; years],
            mortgage_balance: vec![0.0; years],
            buy_investment_portfolio: vec![0.0; years],
            rent_investment_portfolio: vec![0.0; years],
            buy_total_wealth: vec![0.0; years],
            rent_total_wealth: vec![0.0; years],
            buy_monthly_costs: vec![0.0; years],
            rent_monthly_costs: vec![0.0; years],
            // Monthly data for charts
            monthly_buy_cumulative_costs: vec![0.0; total_months],
            monthly_rent_cumulative_costs: vec![0.0; total_months],
            monthly_buy_total_wealth: vec![0.0; total_months],
            monthly_rent_total_wealth: vec![0.0; total_months],
            monthly_home_equity: vec![0.0; total_months],
            monthly_buy_investment_portfolio: vec![0.0; total_months],
            monthly_rent_investment_portfolio: vec![0.0; total_months],
            monthly_buy_costs: vec![0.0; total_months],
            monthly_rent_costs: vec![0.0; total_months],
            break_even_year: None,
            all_break_even_years: Vec::new(),
            break_even_description: String::from("Not calculated"),
            monthly_mortgage_payment: 0.0,
        }
    }

    /// Get buyer's wealth at a specific year
    pub fn buy_wealth_at_year(&self, year: usize) -> f64 {
        if year < 1 || year > self.buy_total_wealth.len() {
            return 0.0;
        }
        self.buy_total_wealth[year - 1]
    }

    /// Get renter's wealth at a specific year
    pub fn rent_wealth_at_year(&self, year: usize) -> f64 {
        if year < 1 || year > self.rent_total_wealth.len() {
            return 0.0;
        }
        self.rent_total_wealth[year - 1]
    }

    /// Get wealth difference at a specific year
    pub fn wealth_difference_at_year(&self, year: usize) -> f64 {
        self.buy_wealth_at_year(year) - self.rent_wealth_at_year(year)
    }

    /// Determine the winner
    pub fn get_winner(&self) -> String {
        let last_year = self.buy_total_wealth.len();
        if last_year == 0 {
            return String::from("No data");
        }

        let buy_wealth = self.buy_total_wealth[last_year - 1];
        let rent_wealth = self.rent_total_wealth[last_year - 1];

        if buy_wealth > rent_wealth {
            String::from("BUY IT!")
        } else if rent_wealth > buy_wealth {
            String::from("RENT IT!")
        } else {
            String::from("TIE!")
        }
    }

    /// Check if there's a break-even point
    pub fn has_break_even_point(&self) -> bool {
        self.break_even_year.is_some()
    }

    /// Get maximum wealth across all scenarios
    pub fn get_max_wealth(&self) -> f64 {
        let mut max = 0.0;

        for &wealth in &self.buy_total_wealth {
            if wealth > max {
                max = wealth;
            }
        }

        for &wealth in &self.rent_total_wealth {
            if wealth > max {
                max = wealth;
            }
        }

        max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_results() {
        let results = FinancialResults::new(30);
        assert_eq!(results.buy_total_wealth.len(), 30);
        assert_eq!(results.rent_total_wealth.len(), 30);
        assert_eq!(results.break_even_description, "Not calculated");
    }

    #[test]
    fn test_wealth_at_year() {
        let mut results = FinancialResults::new(30);
        results.buy_total_wealth[0] = 10000.0;
        results.rent_total_wealth[0] = 8000.0;

        assert_eq!(results.buy_wealth_at_year(1), 10000.0);
        assert_eq!(results.rent_wealth_at_year(1), 8000.0);
        assert_eq!(results.wealth_difference_at_year(1), 2000.0);
    }

    #[test]
    fn test_get_winner() {
        let mut results = FinancialResults::new(30);
        results.buy_total_wealth[29] = 500000.0;
        results.rent_total_wealth[29] = 400000.0;

        assert_eq!(results.get_winner(), "BUY IT!");

        results.rent_total_wealth[29] = 600000.0;
        assert_eq!(results.get_winner(), "RENT IT!");
    }

    #[test]
    fn test_max_wealth() {
        let mut results = FinancialResults::new(30);
        results.buy_total_wealth[10] = 250000.0;
        results.rent_total_wealth[15] = 300000.0;

        assert_eq!(results.get_max_wealth(), 300000.0);
    }
}
