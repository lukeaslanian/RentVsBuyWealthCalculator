/// Calculates mortgage payment calculations and loan amortization logic.
/// Implements Algorithm 1 from the proposal (Section 4.1).
pub struct MortgageCalculator {
    principal: f64,
    annual_interest_rate: f64,
    loan_term_years: usize,
}

impl MortgageCalculator {
    /// Create a new MortgageCalculator
    ///
    /// # Arguments
    /// * `principal` - Loan principal amount
    /// * `annual_interest_rate` - Annual interest rate as percentage
    /// * `loan_term_years` - Loan term in years
    pub fn new(principal: f64, annual_interest_rate: f64, loan_term_years: usize) -> Self {
        Self {
            principal,
            annual_interest_rate,
            loan_term_years,
        }
    }

    /// Calculate monthly mortgage payment using amortization formula
    ///
    /// Formula: payment = principal × [r(1+r)^n] / [(1+r)^n - 1]
    /// where r = monthly rate, n = total number of payments
    pub fn calculate_monthly_payment(&self) -> f64 {
        // Convert annual rate to monthly rate (as decimal)
        let monthly_rate = self.annual_interest_rate / 100.0 / 12.0;

        // Calculate number of payments
        let n = (self.loan_term_years * 12) as i32;

        // Handle edge case: 0% interest
        if monthly_rate == 0.0 {
            return self.principal / n as f64;
        }

        // Apply amortization formula
        let pow_term = (1.0 + monthly_rate).powi(n);
        let numerator = monthly_rate * pow_term;
        let denominator = pow_term - 1.0;

        self.principal * (numerator / denominator)
    }

    /// Calculate total interest paid over life of loan
    pub fn calculate_total_interest(&self) -> f64 {
        let monthly_payment = self.calculate_monthly_payment();
        let num_payments = self.loan_term_years * 12;
        (monthly_payment * num_payments as f64) - self.principal
    }

    /// Calculate remaining balance at a specific month
    ///
    /// Uses the remaining balance formula:
    /// B = P × [(1+r)^n - (1+r)^p] / [(1+r)^n - 1]
    /// where B = balance, P = principal, r = monthly rate,
    /// n = total payments, p = payments made
    ///
    /// # Arguments
    /// * `month` - Month number (1-360 for 30 year loan)
    pub fn calculate_remaining_balance(&self, month: usize) -> f64 {
        if month == 0 {
            return self.principal;
        }

        let monthly_rate = self.annual_interest_rate / 100.0 / 12.0;
        let total_payments = self.loan_term_years * 12;

        if month >= total_payments {
            return 0.0;
        }

        // Handle edge case: 0% interest
        if monthly_rate == 0.0 {
            return self.principal * (1.0 - month as f64 / total_payments as f64);
        }

        // Remaining balance formula
        let pow_total = (1.0 + monthly_rate).powi(total_payments as i32);
        let pow_made = (1.0 + monthly_rate).powi(month as i32);

        (self.principal * (pow_total - pow_made)) / (pow_total - 1.0)
    }

    /// Calculate equity at a specific month
    /// Equity = currentHomeValue - remainingBalance
    ///
    /// # Arguments
    /// * `month` - Month number
    /// * `current_home_value` - Current value of home (with appreciation)
    pub fn calculate_equity_at_month(&self, month: usize, current_home_value: f64) -> f64 {
        let remaining_balance = self.calculate_remaining_balance(month);
        current_home_value - remaining_balance
    }

    /// Generate full amortization schedule
    ///
    /// Returns array of remaining balances for each month (360 months for 30 years)
    pub fn generate_amortization_schedule(&self) -> Vec<f64> {
        let total_months = self.loan_term_years * 12;
        let mut schedule = Vec::with_capacity(total_months);

        for month in 1..=total_months {
            schedule.push(self.calculate_remaining_balance(month));
        }

        schedule
    }

    /// Calculate total interest paid in a specific year
    ///
    /// # Arguments
    /// * `year` - Year number (1-30 for a 30-year loan)
    ///
    /// Returns the total interest paid during that year
    pub fn calculate_interest_paid_in_year(&self, year: usize) -> f64 {
        if year == 0 || year > self.loan_term_years {
            return 0.0;
        }

        let monthly_rate = self.annual_interest_rate / 100.0 / 12.0;
        let monthly_payment = self.calculate_monthly_payment();

        let start_month = (year - 1) * 12;
        let end_month = year * 12;

        let mut total_interest = 0.0;

        for month in start_month..end_month {
            let balance_at_start = self.calculate_remaining_balance(month);
            let interest_this_month = balance_at_start * monthly_rate;
            total_interest += interest_this_month;
        }

        // Cap interest at monthly payment * 12 (can't pay more interest than total payments)
        total_interest.min(monthly_payment * 12.0)
    }

    // Getters
    pub fn principal(&self) -> f64 {
        self.principal
    }

    pub fn annual_interest_rate(&self) -> f64 {
        self.annual_interest_rate
    }

    pub fn loan_term_years(&self) -> usize {
        self.loan_term_years
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monthly_payment() {
        // Example: $290,999 loan at 5.99% for 30 years
        let calc = MortgageCalculator::new(290999.0, 5.99, 30);
        let payment = calc.calculate_monthly_payment();

        // Expected payment around $1,743
        assert!(
            payment > 1740.0 && payment < 1750.0,
            "Payment was {}",
            payment
        );
    }

    #[test]
    fn test_zero_interest() {
        let calc = MortgageCalculator::new(120000.0, 0.0, 10);
        let payment = calc.calculate_monthly_payment();

        // With 0% interest, payment should be principal / months
        let expected = 120000.0 / (10.0 * 12.0);
        assert!((payment - expected).abs() < 0.01);
    }

    #[test]
    fn test_remaining_balance() {
        let calc = MortgageCalculator::new(200000.0, 6.0, 30);

        // At month 0, balance should equal principal
        assert_eq!(calc.calculate_remaining_balance(0), 200000.0);

        // At month 360 (end of loan), balance should be ~0
        assert!(calc.calculate_remaining_balance(360) < 1.0);

        // Halfway through, balance should be less than principal but > 0
        let mid_balance = calc.calculate_remaining_balance(180);
        assert!(mid_balance > 0.0 && mid_balance < 200000.0);
    }

    #[test]
    fn test_total_interest() {
        let calc = MortgageCalculator::new(100000.0, 5.0, 30);
        let total_interest = calc.calculate_total_interest();

        // Total interest should be positive
        assert!(total_interest > 0.0);

        // For a 30-year loan at 5%, total interest should be roughly equal to principal
        assert!(total_interest > 50000.0 && total_interest < 150000.0);
    }

    #[test]
    fn test_equity_calculation() {
        let calc = MortgageCalculator::new(200000.0, 5.0, 30);
        let home_value = 220000.0;

        // At month 0, equity = home_value - principal
        let initial_equity = calc.calculate_equity_at_month(0, home_value);
        assert_eq!(initial_equity, 20000.0);

        // At month 360, equity = home_value (loan paid off)
        let final_equity = calc.calculate_equity_at_month(360, home_value);
        assert!((final_equity - home_value).abs() < 1.0);
    }

    #[test]
    fn test_amortization_schedule() {
        let calc = MortgageCalculator::new(100000.0, 4.0, 30);
        let schedule = calc.generate_amortization_schedule();

        assert_eq!(schedule.len(), 360);

        // First month balance should be close to principal
        assert!(schedule[0] > 99000.0);

        // Last month balance should be close to 0
        assert!(schedule[359] < 100.0);

        // Balance should decrease monotonically
        for i in 1..schedule.len() {
            assert!(schedule[i] < schedule[i - 1]);
        }
    }
}
