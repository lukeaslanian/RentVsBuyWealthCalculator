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
    fn test_interest_outside_loan_term_is_zero() {
        let calc = MortgageCalculator::new(200000.0, 6.0, 30);
        assert_eq!(calc.calculate_interest_paid_in_year(0), 0.0);
        assert_eq!(calc.calculate_interest_paid_in_year(31), 0.0);
        // Interest falls over time as the balance is paid down
        assert!(calc.calculate_interest_paid_in_year(1) > calc.calculate_interest_paid_in_year(30));
    }
}
