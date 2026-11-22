//! Property-based tests: each test states a rule that must hold for ANY valid input, and
//! proptest tries hundreds of random inputs (shrinking any failure to a minimal example).

use proptest::prelude::*;
use rent_vs_buy_calculator::calculators::{LoanSchedule, WealthAnalysisEngine};
use rent_vs_buy_calculator::models::*;
use rent_vs_buy_calculator::utils::CurrencyFormatter;

/// Random but valid home-purchase inputs, including HPAP, refinance and assessment caps
fn property_data() -> impl Strategy<Value = PropertyData> {
    let purchase = (
        50_000.0..5_000_000.0f64, // home price
        0.0..=100.0f64,           // down payment %
        0.5..15.0f64,             // interest rate
        0.0..3.0f64,              // mortgage points
        0.0..3.0f64,              // property tax rate
        -5.0..10.0f64,            // appreciation
        0.0..2_000.0f64,          // HOA
        0.0..2.0f64,              // PMI rate
        0.0..8.0f64,              // purchase closing costs %
        any::<bool>(),            // tax benefits
        0.0..1_000_000.0f64,      // household income
        0.0..12.0f64,             // state income tax rate
    );
    let hpap = (
        any::<bool>(),      // HPAP on
        any::<bool>(),      // own maximum vs DHCD table
        0.0..=202_000.0f64, // own maximum
        0.0..60_000.0f64,   // own contribution
        any::<bool>(),      // own repayment: payments vs deferred
        1usize..=8,         // household size
        0.0..400_000.0f64,  // liquid assets
        0.0..=4_000.0f64,   // closing cost assistance
    );
    let refinance_and_cap = (
        any::<bool>(), // refinance on
        1usize..=29,   // refinance after years
        0.5..15.0f64,  // refinance rate
        0u8..4,        // term choice
        any::<bool>(), // closing costs in cash
        0u8..3,        // HPAP at refinance
        0.0..5.0f64,   // refinance closing costs %
        any::<bool>(), // assessment cap on
        0.0..20.0f64,  // assessment cap %
    );
    (purchase, hpap, refinance_and_cap).prop_map(|(a, h, r)| {
        let mut p = PropertyData::new();
        (
            p.home_price,
            p.down_payment_percent,
            p.interest_rate,
            p.mortgage_points,
            p.property_tax_rate,
            p.home_appreciation_rate,
            p.hoa_fee,
            p.pmi_rate_percent,
            p.closing_costs_percent_purchase,
            p.enable_tax_benefits,
            p.household_income,
            p.state_income_tax_rate,
        ) = a;
        p.enable_hpap = h.0;
        p.hpap_enter_own_max = h.1;
        p.hpap_own_max = h.2;
        p.hpap_own_contribution = h.3;
        p.hpap_own_repayment = if h.4 {
            HpapRepayment::FiveYearDeferral40Year
        } else {
            HpapRepayment::DeferredUntilSale
        };
        p.hpap_household_size = h.5;
        p.hpap_liquid_assets = h.6;
        p.hpap_closing_cost_assistance = h.7;
        p.enable_refinance = r.0;
        p.refinance_after_years = r.1;
        p.refinance_rate = r.2;
        p.refinance_term = match r.3 {
            0 => RefinanceTerm::KeepRemaining,
            1 => RefinanceTerm::Years30,
            2 => RefinanceTerm::Years20,
            _ => RefinanceTerm::Years15,
        };
        p.refinance_cost_payment = if r.4 {
            RefinanceCostPayment::PayCash
        } else {
            RefinanceCostPayment::RollIntoLoan
        };
        p.hpap_at_refinance = match r.5 {
            0 => HpapAtRefinance::RollIntoLoan,
            1 => HpapAtRefinance::PayCash,
            _ => HpapAtRefinance::Keep,
        };
        p.refinance_closing_cost_percent = r.6;
        p.enable_assessment_cap = r.7;
        p.assessment_cap_percent = r.8;
        p
    })
}

fn rental_data() -> impl Strategy<Value = RentalData> {
    (300.0..20_000.0f64, 0.0..10.0f64, any::<bool>()).prop_map(|(rent, increase, broker)| {
        let mut r = RentalData::new();
        r.monthly_rent = rent;
        r.security_deposit = rent;
        r.rent_increase_rate = increase;
        r.enable_broker_fee = broker;
        r
    })
}

fn investment_params() -> impl Strategy<Value = InvestmentParameters> {
    (-2.0..15.0f64, 1usize..=30, 0.0..6.0f64, 0.0..40.0f64).prop_map(|(ret, years, infl, cg)| {
        let mut i = InvestmentParameters::new();
        i.annual_return_rate = ret;
        i.analysis_years = years;
        i.inflation_rate = infl;
        i.capital_gains_tax_rate = cg;
        i
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    /// The engine never produces NaN/infinity, and arrays have the right lengths
    #[test]
    fn engine_results_are_finite(p in property_data(), r in rental_data(), i in investment_params()) {
        prop_assume!(p.is_valid() && r.is_valid());
        let years = i.analysis_years;
        let mut engine = WealthAnalysisEngine::new(p, r, i);
        engine.run_analysis();
        let res = engine.results().expect("results");
        prop_assert_eq!(res.buy_total_wealth.len(), years);
        prop_assert_eq!(res.monthly_buy_total_wealth.len(), years * 12);
        for v in res.buy_total_wealth.iter()
            .chain(&res.rent_total_wealth)
            .chain(&res.home_equity)
            .chain(&res.mortgage_balance)
            .chain(&res.monthly_buy_costs)
            .chain(&res.monthly_rent_costs)
            .chain(&res.monthly_buy_investment_portfolio)
            .chain(&res.monthly_rent_investment_portfolio)
        {
            prop_assert!(v.is_finite(), "non-finite value");
        }
        prop_assert!(res.monthly_mortgage_payment.is_finite());
        prop_assert!(!res.break_even_description.is_empty());
    }

    /// HPAP never lends more than allowed, and never more than the down payment gap
    #[test]
    fn hpap_respects_limits(p in property_data()) {
        prop_assume!(p.is_valid());
        let gap = p.hpap_gap_financing();
        let closing_help = p.hpap_closing_costs_applied();
        prop_assert!(gap >= 0.0 && closing_help >= 0.0);
        prop_assert!(gap <= p.down_payment_amount().max(0.0) + 1e-6);
        prop_assert!(gap <= HPAP_MAX_GAP_FINANCING + 1e-6);
        prop_assert!(closing_help <= HPAP_MAX_CLOSING_COST_ASSISTANCE + 1e-6);
        prop_assert!(closing_help <= p.closing_costs_purchase() + 1e-6);
        if let Some(tier) = p.hpap_tier() {
            prop_assert!(gap <= tier.max_gap_financing + 1e-6);
        } else {
            prop_assert_eq!(p.hpap_total_loan(), 0.0);
        }
        // The buyer always pays the required contribution, unless upfront costs are smaller
        let upfront = p.down_payment_amount() + p.closing_costs_purchase() + p.mortgage_points_cost();
        if p.hpap_tier().is_some() {
            let floor = p.hpap_required_contribution().min(upfront);
            prop_assert!(p.buyer_cash_at_closing() >= floor - 1e-6);
        }
    }

    /// "Amount to use" lands on the requested HPAP amount (capped at the maximum)
    #[test]
    fn hpap_amount_to_use_round_trips(p in property_data(), amount in 0.0..250_000.0f64) {
        prop_assume!(p.is_valid());
        if let Some(percent) = p.hpap_down_payment_percent_for(amount) {
            let mut q = p.clone();
            q.down_payment_percent = percent;
            let target = amount.min(q.hpap_tier().unwrap().max_gap_financing);
            if percent < 100.0 {
                prop_assert!((q.hpap_gap_financing() - target).abs() < 1.0,
                    "asked for {target}, got {}", q.hpap_gap_financing());
            }
        }
    }

    /// Loan balances never go negative, payments and interest are never negative,
    /// and balances only rise at a refinance (rolled-in costs/HPAP)
    #[test]
    fn loan_schedule_is_sane(p in property_data()) {
        prop_assume!(p.is_valid());
        let schedule = LoanSchedule::from_property_data(&p);
        let refi_month = schedule.refinance_plan().map(|plan| plan.after_month);
        let mut previous = schedule.remaining_balance(0);
        for month in 1..=360 {
            let balance = schedule.remaining_balance(month);
            prop_assert!(balance >= -1e-6, "negative balance at month {month}");
            prop_assert!(schedule.monthly_payment(month) >= 0.0);
            if Some(month) != refi_month {
                prop_assert!(balance <= previous + 1e-6, "balance rose at month {month}");
            }
            previous = balance;
        }
        for year in 1..=30 {
            prop_assert!(schedule.interest_paid_in_year(year) >= -1e-9);
        }
    }

    /// Tax savings are never negative and never exceed the marginal rate times the
    /// housing deductions themselves
    #[test]
    fn tax_savings_are_bounded(p in property_data(), year in 1usize..=30, interest in 0.0..200_000.0f64) {
        prop_assume!(p.is_valid());
        let savings = p.calculate_annual_tax_savings(year, interest, p.home_price, p.hoa_fee * 12.0);
        prop_assert!(savings >= 0.0);
        let property_tax = p.home_price * p.property_tax_rate / 100.0;
        let housing = interest + property_tax + p.hoa_fee * 12.0 * p.hoa_deduction_percent / 100.0;
        prop_assert!(savings <= housing * p.marginal_tax_rate / 100.0 + 1e-6);
    }

    /// The SALT cap always stays between $10,000 and the 2029 peak
    #[test]
    fn salt_cap_in_range(year in 2000i32..2100, income in 0.0..10_000_000.0f64) {
        let cap = salt_cap(year, income);
        prop_assert!((10_000.0..=41_624.0).contains(&cap));
    }

    /// Assessed value never exceeds market value or grows faster than the cap
    #[test]
    fn assessment_cap_bounds(p in property_data(), previous in 1.0..5_000_000.0f64, market in 1.0..10_000_000.0f64) {
        let next = p.next_assessed_value(previous, market);
        prop_assert!(next <= market + 1e-6);
        if p.enable_assessment_cap {
            prop_assert!(next <= previous * (1.0 + p.assessment_cap_percent / 100.0) + 1e-6);
        } else {
            prop_assert!((next - market).abs() < 1e-6);
        }
    }

    /// Formatting then parsing a dollar amount gives back the amount rounded to cents,
    /// and the cents are always exactly two digits (no "$7,999.100")
    #[test]
    fn currency_format_round_trips(x in -1_000_000_000.0..1_000_000_000.0f64) {
        let text = CurrencyFormatter::format_currency(x);
        let cents = text.rsplit('.').next().unwrap();
        prop_assert_eq!(cents.len(), 2, "bad cents in {}", text);
        let parsed = CurrencyFormatter::parse_input(&text).expect("parses");
        prop_assert!((parsed - (x * 100.0).round() / 100.0).abs() < 0.0051, "{} -> {}", x, text);
    }

    /// The input parser never returns NaN or infinity, whatever is typed
    #[test]
    fn parse_input_never_non_finite(text in ".{0,20}") {
        if let Some(v) = CurrencyFormatter::parse_input(&text) {
            prop_assert!(v.is_finite());
        }
    }
}
