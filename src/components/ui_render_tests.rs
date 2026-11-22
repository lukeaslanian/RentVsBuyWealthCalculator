//! UI rendering tests: render real components to HTML (server-side rendering, no browser)
//! and check what a user would see for given inputs. Clicks and typing can't be simulated
//! this way, but every conditional section, label, warning and formatted value can.

use crate::calculators::WealthAnalysisEngine;
use crate::components::input::{
    BuyingInputPanel, CitySelectionPanel, RentalInputPanel, SharedParamsPanel,
};
use crate::components::results::{ResultsPanel, YearByYearTable};
use crate::models::{CityPreset, FinancialResults, InvestmentParameters, PropertyData, RentalData};
use dioxus::prelude::*;

fn render(mut dom: VirtualDom) -> String {
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// Render the Buying panel with the given inputs and return its HTML
fn render_buying_panel(data: PropertyData) -> String {
    #[component]
    fn Root(data: PropertyData) -> Element {
        let property_data = use_signal(|| data.clone());
        rsx! { BuyingInputPanel { property_data } }
    }
    render(VirtualDom::new_with_props(Root, RootProps { data }))
}

fn render_rental_panel(data: RentalData) -> String {
    #[component]
    fn Root(data: RentalData) -> Element {
        let rental_data = use_signal(|| data.clone());
        rsx! { RentalInputPanel { rental_data } }
    }
    render(VirtualDom::new_with_props(Root, RootProps { data }))
}

fn render_shared_panel(params: InvestmentParameters) -> String {
    #[component]
    fn Root(params: InvestmentParameters) -> Element {
        let investment_params = use_signal(|| params.clone());
        rsx! { SharedParamsPanel { investment_params } }
    }
    render(VirtualDom::new_with_props(Root, RootProps { params }))
}

fn render_city_panel() -> String {
    #[component]
    fn Root() -> Element {
        let property_data = use_signal(PropertyData::new);
        let rental_data = use_signal(RentalData::new);
        let investment_params = use_signal(InvestmentParameters::new);
        rsx! { CitySelectionPanel { property_data, rental_data, investment_params } }
    }
    render(VirtualDom::new(Root))
}

fn render_table(results: FinancialResults) -> String {
    #[component]
    fn Root(results: FinancialResults) -> Element {
        rsx! { YearByYearTable { results } }
    }
    render(VirtualDom::new_with_props(Root, RootProps { results }))
}

fn render_results_panel(results: FinancialResults) -> String {
    #[component]
    fn Root(results: FinancialResults) -> Element {
        rsx! { ResultsPanel { results } }
    }
    render(VirtualDom::new_with_props(Root, RootProps { results }))
}

fn run_engine(p: PropertyData) -> FinancialResults {
    let mut engine = WealthAnalysisEngine::new(p, RentalData::new(), InvestmentParameters::new());
    engine.run_analysis();
    engine.results().expect("results").clone()
}

/// Strip HTML tags so assertions read like what's on screen
fn visible_text(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                text.push(' ');
            }
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    text.replace("&#39;", "'")
        .replace("&#62;", ">")
        .replace("&#60;", "<")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Position of `needle` in `haystack`, failing with context if it's missing
fn position(haystack: &str, needle: &str) -> usize {
    haystack
        .find(needle)
        .unwrap_or_else(|| panic!("missing {needle:?} in: {haystack}"))
}

fn hpap_on() -> PropertyData {
    let mut data = PropertyData::new();
    data.enable_hpap = true;
    data
}

// ---------------- Buying panel: defaults ----------------

#[test]
fn buying_panel_defaults() {
    let html = render_buying_panel(PropertyData::new());
    let text = visible_text(&html);
    assert!(text.contains("Buying Scenario"));
    // Optional sections are collapsed until turned on
    assert!(!text.contains("HPAP Maximum"));
    assert!(!text.contains("Refinance After Year"));
    // Tax benefits are on by default, so their fields show
    assert!(text.contains("Filing Status"));
    assert!(text.contains("Household Income ($/year)"));
    // Dollar and percent fields are formatted for display
    assert!(
        html.contains("value=\"$575,000.00\""),
        "home price formatting"
    );
    assert!(
        html.contains("value=\"10.00%\""),
        "DC assessment cap shows as a percent"
    );
    // Plain down payment label when HPAP is off
    assert!(text.contains("Down Payment") && !text.contains("Down Payment (incl. HPAP)"));
}

#[test]
fn tax_fields_hidden_when_tax_benefits_off() {
    let mut data = PropertyData::new();
    data.enable_tax_benefits = false;
    let text = visible_text(&render_buying_panel(data));
    assert!(!text.contains("Filing Status"));
}

#[test]
fn assessment_cap_input_hidden_when_cap_off() {
    let mut data = PropertyData::new();
    data.enable_assessment_cap = false;
    let text = visible_text(&render_buying_panel(data));
    assert!(text.contains("Assessment Growth Cap"));
    assert!(!text.contains("Max Assessment Increase"));
}

#[test]
fn pmi_field_shows_monthly_amount_and_dropoff_only_when_pmi_applies() {
    // 3% down: PMI applies ($557,750 x 0.5% / 12 = $232.40/mo)
    let html = render_buying_panel(PropertyData::new());
    assert!(html.contains("$232.40"), "monthly PMI shown");
    assert!(visible_text(&html).contains("PMI Drop-off LTV"));
    // 20% down: no PMI, so no drop-off field
    let mut twenty = PropertyData::new();
    twenty.down_payment_percent = 20.0;
    assert!(!visible_text(&render_buying_panel(twenty)).contains("PMI Drop-off LTV"));
}

// ---------------- Buying panel: HPAP ----------------

#[test]
fn hpap_own_max_mode_without_max_shows_red_warning() {
    // Default when HPAP is turned on: "I know my maximum", with $0 entered
    let html = render_buying_panel(hpap_on());
    let text = visible_text(&html);
    assert!(text.contains("Down Payment (incl. HPAP)"));
    assert!(text.contains("you haven't entered your maximum, so no HPAP is applied"));
    assert!(
        html.contains("text-monokaiLight-red"),
        "warning is styled red"
    );
    assert!(!text.contains("HPAP Amount to Use"), "nothing to use yet");
    // "I know my maximum" is the first option
    assert!(position(&text, "I know my maximum") < position(&text, "Calculate it for me"));
}

#[test]
fn hpap_own_max_mode_with_max_shows_fields_in_order() {
    let mut data = hpap_on();
    data.hpap_own_max = 101000.0;
    let text = visible_text(&render_buying_panel(data));
    // Own-max inputs, then the maximum, then how much to use
    let max_field = position(&text, "Your HPAP Maximum ($)");
    let contribution = position(&text, "Your Required Contribution ($)");
    let tier = position(&text, "Your maximum: up to $101,000.00 gap financing");
    let amount = position(&text, "HPAP Amount to Use (max $101,000.00)");
    let button = position(&text, "Use full HPAP");
    let closing = position(&text, "HPAP Closing Cost Assistance ($)");
    assert!(max_field < contribution && contribution < tier && tier < amount && amount < closing);
    // The button sits right next to the amount field
    assert!(amount < button && button < closing);
    // Table-mode inputs are hidden
    assert!(!text.contains("Household Size"));
    assert!(!text.contains("Savings / Liquid Assets"));
    // Usage summary and button
    // The $500 contribution is already covered by the closing costs the buyer pays,
    // so HPAP covers the whole 3% down payment
    assert!(text.contains("You're using $17,250.00 of your $101,000.00 HPAP limit"));
    assert!(text.contains("Use full HPAP"));
}

#[test]
fn hpap_calculate_mode_shows_max_before_amount_to_use() {
    let mut data = hpap_on();
    data.hpap_enter_own_max = false; // "Calculate it for me"
    data.household_income = 70000.0;
    data.hpap_household_size = 1;
    let text = visible_text(&render_buying_panel(data));
    let household = position(&text, "Household Size");
    let savings = position(&text, "Savings / Liquid Assets ($)");
    let tier = position(
        &text,
        "Your tier (51-80% MFI): up to $161,600.00 gap financing",
    );
    let amount = position(&text, "HPAP Amount to Use (max $161,600.00)");
    assert!(household < savings && savings < tier && tier < amount);
    assert!(text.contains("Required contribution: $3,500.00"));
    assert!(!text.contains("Your HPAP Maximum ($)"));
}

#[test]
fn hpap_calculate_mode_ineligible_income() {
    let mut data = hpap_on();
    data.hpap_enter_own_max = false;
    data.household_income = 150000.0;
    let text = visible_text(&render_buying_panel(data));
    assert!(text.contains(
        "Not eligible: household income of $150,000.00 is above the 110% MFI limit of $127,900.00"
    ));
    assert!(!text.contains("HPAP Amount to Use"));
    assert!(!text.contains("Use full HPAP"));
}

#[test]
fn hpap_payments_tier_shows_year_six_payment() {
    let mut data = hpap_on();
    data.hpap_own_max = 96000.0;
    data.hpap_own_repayment = crate::models::HpapRepayment::FiveYearDeferral40Year;
    data.down_payment_percent = data.hpap_full_use_down_payment_percent().unwrap();
    let text = visible_text(&render_buying_panel(data));
    assert!(text.contains("HPAP payment from year 6:"));
    // Full use: nothing left to borrow, so no "you're using X of Y" nudge
    assert!(!text.contains("You're using"));
}

// ---------------- Buying panel: refinance ----------------

#[test]
fn refinance_section_shows_summary() {
    let mut data = PropertyData::new();
    data.enable_refinance = true;
    let text = visible_text(&render_buying_panel(data));
    assert!(text.contains("Refinance After Year"));
    assert!(text.contains("New Loan Term"));
    assert!(text.contains("Keep remaining term (25 years)"));
    assert!(text.contains("After year 5:"));
    assert!(text.contains("→ new loan"));
    // HPAP choice only appears when HPAP is on
    assert!(!text.contains("HPAP at Refinance"));
}

#[test]
fn refinance_with_hpap_offers_hpap_choice_and_cash_due() {
    let mut data = hpap_on();
    data.hpap_own_max = 50000.0;
    data.enable_refinance = true;
    data.hpap_at_refinance = crate::models::HpapAtRefinance::PayCash;
    let text = visible_text(&render_buying_panel(data));
    assert!(text.contains("HPAP at Refinance"));
    assert!(text.contains("DHCD makes HPAP due on refinance"));
    assert!(text.contains("Cash due at refinance:"));
}

// ---------------- Other input panels ----------------

#[test]
fn rental_panel_broker_fee_toggle() {
    let text = visible_text(&render_rental_panel(RentalData::new()));
    assert!(text.contains("Broker's Fee"));
    assert!(!text.contains("Broker Fee %") && !text.contains("of annual rent"));

    let mut with_fee = RentalData::new();
    with_fee.enable_broker_fee = true;
    let html = render_rental_panel(with_fee);
    // The broker fee input appears, with its dollar amount (15% of $2,900 x 12)
    assert!(html.contains("$5,220.00"), "broker fee dollar amount");
}

#[test]
fn shared_panel_shows_capital_gains_as_percent() {
    let html = render_shared_panel(InvestmentParameters::new());
    assert!(visible_text(&html).contains("Capital Gains Tax Rate (%)"));
    assert!(html.contains("value=\"15.00%\""));
    assert!(html.contains("value=\"7.00%\""), "return rate");
}

#[test]
fn city_panel_lists_all_cities() {
    let text = visible_text(&render_city_panel());
    for city in ["Washington DC", "Boston", "New York City", "San Francisco"] {
        assert!(text.contains(city), "missing {city}");
    }
}

// ---------------- Results ----------------

#[test]
fn year_by_year_table_has_a_row_per_year() {
    let results = run_engine(CityPreset::dc_two_bedroom().to_property_data());
    let html = render_table(results.clone());
    assert_eq!(
        html.matches("<tr").count(),
        results.buy_total_wealth.len() + 1
    );
}

#[test]
fn results_panel_shows_refinanced_payment() {
    let mut data = PropertyData::new();
    data.enable_refinance = true;
    let results = run_engine(data);
    let text = visible_text(&render_results_panel(results));
    assert!(text.contains("Monthly Mortgage (refi after year 5)"));
    assert!(text.contains("→"));
}
