use super::MortgageCalculator;
use crate::models::{HpapAtRefinance, PropertyData, RefinanceCostPayment, RefinanceTerm};
use crate::utils::AppConfig;

/// The numbers for a planned rate/term refinance (no cash-out).
#[derive(Clone, Debug, PartialEq)]
pub struct RefinancePlan {
    /// Last month on the original loan; the new loan's first payment is the next month
    pub after_month: usize,
    /// Original loan balance paid off by the refinance
    pub old_balance: f64,
    /// HPAP balance added to the new loan (HPAP "roll into loan")
    pub hpap_rolled_in: f64,
    /// HPAP balance paid off in cash in the refinance month (HPAP "pay off")
    pub hpap_paid_cash: f64,
    /// Refinance closing costs added to the new loan
    pub costs_rolled_in: f64,
    /// Refinance closing costs paid in cash in the refinance month
    pub costs_paid_cash: f64,
    /// New loan amount
    pub new_principal: f64,
    pub new_rate: f64,
    pub new_term_years: usize,
    pub old_payment: f64,
    pub new_payment: f64,
}

impl RefinancePlan {
    /// Build the refinance plan from the inputs (None if refinancing is off or the original
    /// loan is already paid off by then).
    pub fn from_property_data(data: &PropertyData) -> Option<Self> {
        if !data.enable_refinance {
            return None;
        }
        let term_years = AppConfig::DEFAULT_MORTGAGE_TERM_YEARS;
        let after_years = data.refinance_after_years.clamp(1, term_years - 1);
        let after_month = after_years * AppConfig::MONTHS_PER_YEAR;

        let original = original_loan(data);
        let old_balance = original.calculate_remaining_balance(after_month);
        if old_balance <= 0.0 {
            return None;
        }

        // HPAP is due on refinance unless DHCD approves keeping it
        let hpap_balance = data.hpap_balance_after_month(after_month);
        let (hpap_rolled_in, hpap_paid_cash) = match data.hpap_at_refinance {
            HpapAtRefinance::RollIntoLoan => (hpap_balance, 0.0),
            HpapAtRefinance::PayCash => (0.0, hpap_balance),
            HpapAtRefinance::Keep => (0.0, 0.0),
        };

        let base_loan = old_balance + hpap_rolled_in;
        let closing_costs = base_loan * data.refinance_closing_cost_percent.max(0.0) / 100.0;
        let (costs_rolled_in, costs_paid_cash) = match data.refinance_cost_payment {
            RefinanceCostPayment::RollIntoLoan => (closing_costs, 0.0),
            RefinanceCostPayment::PayCash => (0.0, closing_costs),
        };
        let new_principal = base_loan + costs_rolled_in;

        let new_term_years = match data.refinance_term {
            RefinanceTerm::KeepRemaining => term_years - after_years,
            RefinanceTerm::Years30 => 30,
            RefinanceTerm::Years20 => 20,
            RefinanceTerm::Years15 => 15,
        };
        let new_loan = MortgageCalculator::new(new_principal, data.refinance_rate, new_term_years);

        Some(Self {
            after_month,
            old_balance,
            hpap_rolled_in,
            hpap_paid_cash,
            costs_rolled_in,
            costs_paid_cash,
            new_principal,
            new_rate: data.refinance_rate,
            new_term_years,
            old_payment: original.calculate_monthly_payment(),
            new_payment: new_loan.calculate_monthly_payment(),
        })
    }

    /// Cash paid out of pocket in the refinance month (closing costs and/or HPAP payoff)
    pub fn cash_due(&self) -> f64 {
        self.costs_paid_cash + self.hpap_paid_cash
    }

    /// Whether the HPAP loan is paid off (rolled in or paid in cash) by the refinance
    pub fn pays_off_hpap(&self, data: &PropertyData) -> bool {
        data.hpap_at_refinance != HpapAtRefinance::Keep
    }
}

/// The original first mortgage, using the effective (points-adjusted) rate
fn original_loan(data: &PropertyData) -> MortgageCalculator {
    MortgageCalculator::new(
        data.loan_amount(),
        data.effective_interest_rate(),
        AppConfig::DEFAULT_MORTGAGE_TERM_YEARS,
    )
}

/// The first-mortgage payment schedule, switching to the new loan after a refinance.
/// Months are 1-indexed (month 1 = first payment); balances are after that month's payment.
pub struct LoanSchedule {
    original: MortgageCalculator,
    original_rate: f64,
    refinance: Option<(RefinancePlan, MortgageCalculator)>,
}

impl LoanSchedule {
    pub fn from_property_data(data: &PropertyData) -> Self {
        let refinance = RefinancePlan::from_property_data(data).map(|plan| {
            let loan =
                MortgageCalculator::new(plan.new_principal, plan.new_rate, plan.new_term_years);
            (plan, loan)
        });
        Self {
            original: original_loan(data),
            original_rate: data.effective_interest_rate(),
            refinance,
        }
    }

    pub fn refinance_plan(&self) -> Option<&RefinancePlan> {
        self.refinance.as_ref().map(|(plan, _)| plan)
    }

    /// Payment on the original loan (shown as "Monthly Mortgage")
    pub fn original_payment(&self) -> f64 {
        self.original.calculate_monthly_payment()
    }

    /// Mortgage payment due in `month` (0 once the active loan is paid off)
    pub fn monthly_payment(&self, month: usize) -> f64 {
        match &self.refinance {
            Some((plan, loan)) if month > plan.after_month => {
                if month - plan.after_month <= loan_months(plan.new_term_years) {
                    loan.calculate_monthly_payment()
                } else {
                    0.0
                }
            }
            _ => {
                if month <= loan_months(AppConfig::DEFAULT_MORTGAGE_TERM_YEARS) {
                    self.original.calculate_monthly_payment()
                } else {
                    0.0
                }
            }
        }
    }

    /// Balance after `month`'s payment (month 0 = the original loan amount)
    pub fn remaining_balance(&self, month: usize) -> f64 {
        match &self.refinance {
            Some((plan, loan)) if month >= plan.after_month => {
                loan.calculate_remaining_balance(month - plan.after_month)
            }
            _ => self.original.calculate_remaining_balance(month),
        }
    }

    /// Interest charged in `month`: the active loan's rate on the balance owed at the start
    /// of that month
    fn interest_in_month(&self, month: usize) -> f64 {
        match &self.refinance {
            Some((plan, loan)) if month > plan.after_month => {
                let k = month - plan.after_month;
                if k > loan_months(plan.new_term_years) {
                    return 0.0;
                }
                loan.calculate_remaining_balance(k - 1) * plan.new_rate / 100.0 / 12.0
            }
            _ => {
                if month > loan_months(AppConfig::DEFAULT_MORTGAGE_TERM_YEARS) {
                    return 0.0;
                }
                self.original.calculate_remaining_balance(month - 1) * self.original_rate
                    / 100.0
                    / 12.0
            }
        }
    }

    /// Total mortgage interest paid in analysis year `year` (1-indexed), across both loans
    pub fn interest_paid_in_year(&self, year: usize) -> f64 {
        if year == 0 {
            return 0.0;
        }
        let start = (year - 1) * AppConfig::MONTHS_PER_YEAR + 1;
        (start..start + AppConfig::MONTHS_PER_YEAR)
            .map(|month| self.interest_in_month(month))
            .sum()
    }
}

fn loan_months(years: usize) -> usize {
    years * AppConfig::MONTHS_PER_YEAR
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refi_data() -> PropertyData {
        let mut data = PropertyData::new();
        data.interest_rate = 7.0;
        data.enable_refinance = true;
        data.refinance_after_years = 5;
        data.refinance_rate = 5.0;
        data.refinance_term = RefinanceTerm::Years30;
        data.refinance_closing_cost_percent = 2.0;
        data.refinance_cost_payment = RefinanceCostPayment::RollIntoLoan;
        data
    }

    #[test]
    fn test_no_refinance_matches_mortgage_calculator() {
        let data = PropertyData::new();
        let schedule = LoanSchedule::from_property_data(&data);
        let calc = MortgageCalculator::new(data.loan_amount(), data.effective_interest_rate(), 30);
        assert!(schedule.refinance_plan().is_none());
        for month in [0, 1, 12, 60, 359, 360] {
            assert!(
                (schedule.remaining_balance(month) - calc.calculate_remaining_balance(month)).abs()
                    < 1e-6
            );
        }
        for year in [1, 5, 30] {
            assert!(
                (schedule.interest_paid_in_year(year) - calc.calculate_interest_paid_in_year(year))
                    .abs()
                    < 1e-6
            );
        }
        assert_eq!(
            schedule.monthly_payment(1),
            calc.calculate_monthly_payment()
        );
    }

    #[test]
    fn test_refinance_plan_rolls_costs_in() {
        let data = refi_data();
        let plan = RefinancePlan::from_property_data(&data).unwrap();
        assert_eq!(plan.after_month, 60);
        let expected_costs = plan.old_balance * 0.02;
        assert!((plan.costs_rolled_in - expected_costs).abs() < 1e-6);
        assert!((plan.new_principal - (plan.old_balance + expected_costs)).abs() < 1e-6);
        assert_eq!(plan.cash_due(), 0.0);
        // A lower rate lowers the payment even with costs rolled in
        assert!(plan.new_payment < plan.old_payment);
    }

    #[test]
    fn test_refinance_costs_paid_in_cash() {
        let mut data = refi_data();
        data.refinance_cost_payment = RefinanceCostPayment::PayCash;
        let plan = RefinancePlan::from_property_data(&data).unwrap();
        assert!((plan.new_principal - plan.old_balance).abs() < 1e-6);
        assert!((plan.cash_due() - plan.old_balance * 0.02).abs() < 1e-6);
    }

    #[test]
    fn test_schedule_switches_loans() {
        let data = refi_data();
        let schedule = LoanSchedule::from_property_data(&data);
        let plan = schedule.refinance_plan().unwrap().clone();
        assert_eq!(schedule.monthly_payment(60), plan.old_payment);
        assert_eq!(schedule.monthly_payment(61), plan.new_payment);
        assert!((schedule.remaining_balance(60) - plan.new_principal).abs() < 1e-6);
        assert!(schedule.remaining_balance(61) < plan.new_principal);
        // Interest in year 6 is at the new 5% rate on the new balance
        let year6_interest = schedule.interest_paid_in_year(6);
        assert!(
            year6_interest < plan.new_principal * 0.05
                && year6_interest > plan.new_principal * 0.045
        );
    }

    #[test]
    fn test_short_term_refinance_pays_off_and_payments_stop() {
        let mut data = refi_data();
        data.refinance_term = RefinanceTerm::Years15;
        let schedule = LoanSchedule::from_property_data(&data);
        // Refinanced at month 60 into a 15-year loan: paid off at month 240
        assert!(schedule.monthly_payment(240) > 0.0);
        assert_eq!(schedule.monthly_payment(241), 0.0);
        assert!(schedule.remaining_balance(240).abs() < 1e-6);
        assert_eq!(schedule.interest_paid_in_year(21), 0.0);
    }

    #[test]
    fn test_keep_remaining_term() {
        let mut data = refi_data();
        data.refinance_term = RefinanceTerm::KeepRemaining;
        let plan = RefinancePlan::from_property_data(&data).unwrap();
        assert_eq!(plan.new_term_years, 25);
    }

    #[test]
    fn test_hpap_options_at_refinance() {
        let mut data = refi_data();
        data.enable_hpap = true;
        data.hpap_enter_own_max = true;
        data.hpap_own_max = 50000.0;
        data.down_payment_percent = data.hpap_full_use_down_payment_percent().unwrap();
        let hpap_balance = data.hpap_balance_after_month(60);
        assert!(hpap_balance > 0.0);

        data.hpap_at_refinance = HpapAtRefinance::RollIntoLoan;
        let rolled = RefinancePlan::from_property_data(&data).unwrap();
        assert!((rolled.hpap_rolled_in - hpap_balance).abs() < 1e-6);
        assert!(rolled.pays_off_hpap(&data));

        data.hpap_at_refinance = HpapAtRefinance::PayCash;
        let paid = RefinancePlan::from_property_data(&data).unwrap();
        assert!((paid.hpap_paid_cash - hpap_balance).abs() < 1e-6);
        assert!(paid.new_principal < rolled.new_principal);

        data.hpap_at_refinance = HpapAtRefinance::Keep;
        let kept = RefinancePlan::from_property_data(&data).unwrap();
        assert_eq!(kept.hpap_rolled_in + kept.hpap_paid_cash, 0.0);
        assert!(!kept.pays_off_hpap(&data));
    }

    #[test]
    fn test_paid_off_loan_has_nothing_to_refinance() {
        // 100% down: no loan, so no refinance plan and no payments
        let mut data = refi_data();
        data.down_payment_percent = 100.0;
        assert!(RefinancePlan::from_property_data(&data).is_none());
        let schedule = LoanSchedule::from_property_data(&data);
        assert_eq!(schedule.remaining_balance(1), 0.0);
        assert_eq!(schedule.interest_paid_in_year(1), 0.0);
    }

    #[test]
    fn test_past_the_original_term_and_year_zero() {
        let data = PropertyData::new();
        let schedule = LoanSchedule::from_property_data(&data);
        // The original 30-year loan has no payments or interest after month 360
        assert!(schedule.monthly_payment(360) > 0.0);
        assert_eq!(schedule.monthly_payment(361), 0.0);
        assert_eq!(schedule.interest_paid_in_year(31), 0.0);
        // Year 0 isn't a real year
        assert_eq!(schedule.interest_paid_in_year(0), 0.0);
    }
}
