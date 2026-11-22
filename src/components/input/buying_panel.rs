use crate::calculators::{MortgageCalculator, RefinancePlan};
use crate::components::input::DualInputField;
use crate::models::{
    hpap_income_limit, salt_cap, FilingStatus, HpapAtRefinance, HpapRepayment, PropertyData,
    RefinanceCostPayment, RefinanceTerm, HPAP_MAX_CLOSING_COST_ASSISTANCE, HPAP_MAX_GAP_FINANCING,
    HPAP_MAX_HOUSEHOLD_SIZE, TAX_BASE_YEAR,
};
use dioxus::prelude::*;

#[component]
pub fn BuyingInputPanel(
    property_data: Signal<PropertyData>,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    // HPAP eligibility, amounts, and repayment for the current inputs
    let hpap_max_amount = property_data
        .read()
        .hpap_tier()
        .map(|tier| tier.max_gap_financing)
        .unwrap_or(0.0);
    let hpap_amount_label = format!(
        "HPAP Amount to Use (max {})",
        format_currency_input(hpap_max_amount)
    );
    let (hpap_eligible, hpap_tier_text, hpap_summary, hpap_room_text, hpap_full_percent) = {
        let data = property_data.read();
        match data.hpap_tier() {
            Some(tier) => {
                let tier_label = if data.hpap_enter_own_max {
                    "Your maximum".to_string()
                } else {
                    format!("Your tier ({})", tier.mfi_band)
                };
                let tier_text = format!(
                    "{}: up to {} gap financing + up to {} for closing costs; {}.",
                    tier_label,
                    format_currency_input(tier.max_gap_financing),
                    format_currency_input(HPAP_MAX_CLOSING_COST_ASSISTANCE),
                    tier.repayment.display_name(),
                );
                let mut summary = format!(
                    "HPAP loan: {} ({} gap + {} closing) · Required contribution: {} · Your cash at closing: {} · First mortgage: {} ({:.1}% LTV)",
                    format_currency_input(data.hpap_total_loan()),
                    format_currency_input(data.hpap_gap_financing()),
                    format_currency_input(data.hpap_closing_costs_applied()),
                    format_currency_input(data.hpap_required_contribution()),
                    format_currency_input(data.buyer_cash_at_closing()),
                    format_currency_input(data.loan_amount()),
                    data.initial_ltv_percent(),
                );
                if tier.repayment == HpapRepayment::FiveYearDeferral40Year {
                    summary.push_str(&format!(
                        " · HPAP payment from year 6: {}/mo",
                        format_currency_input(data.hpap_scheduled_payment())
                    ));
                }
                let unused = tier.max_gap_financing - data.hpap_gap_financing();
                let room_text = if unused > 1.0 {
                    format!(
                        "You're using {} of your {} HPAP limit. Using more borrows more from HPAP (0%) and less from the bank.",
                        format_currency_input(data.hpap_gap_financing()),
                        format_currency_input(tier.max_gap_financing),
                    )
                } else {
                    String::new()
                };
                (
                    true,
                    tier_text,
                    summary,
                    room_text,
                    data.hpap_full_use_down_payment_percent(),
                )
            }
            None => {
                let tier_text = if data.hpap_enter_own_max {
                    "HPAP is on, but you haven't entered your maximum, so no HPAP is applied. Enter your HPAP maximum above, or choose \"Calculate it for me\".".to_string()
                } else {
                    format!(
                        "Not eligible: household income of {} is above the 110% MFI limit of {} for a household of {}.",
                        format_currency_input(data.household_income),
                        format_currency_input(hpap_income_limit(data.hpap_household_size)),
                        data.hpap_household_size,
                    )
                };
                (false, tier_text, String::new(), String::new(), None)
            }
        }
    };
    let down_payment_label = if property_data.read().enable_hpap {
        "Down Payment (incl. HPAP)"
    } else {
        "Down Payment"
    };
    let down_payment_tooltip = if property_data.read().enable_hpap {
        "Total down payment, including HPAP gap financing. This sets your first mortgage; HPAP covers what your required contribution doesn't, up to your tier limit."
    } else {
        "Percentage of home price paid upfront (e.g., 3% or 20%)"
    };

    // Refinance summary for the current inputs
    let refinance_summary = {
        let data = property_data.read();
        RefinancePlan::from_property_data(&data).map(|plan| {
            let mut parts = vec![format!(
                "After year {}: {} balance",
                plan.after_month / 12,
                format_currency_input(plan.old_balance)
            )];
            if plan.hpap_rolled_in > 0.0 {
                parts.push(format!(
                    "+ {} HPAP",
                    format_currency_input(plan.hpap_rolled_in)
                ));
            }
            if plan.costs_rolled_in > 0.0 {
                parts.push(format!(
                    "+ {} closing costs",
                    format_currency_input(plan.costs_rolled_in)
                ));
            }
            let mut text = format!(
                "{} → new loan {} at {:.2}% for {} years. Payment {} → {}/mo.",
                parts.join(" "),
                format_currency_input(plan.new_principal),
                plan.new_rate,
                plan.new_term_years,
                format_currency_input(plan.old_payment),
                format_currency_input(plan.new_payment),
            );
            if plan.cash_due() > 0.0 {
                text.push_str(&format!(
                    " Cash due at refinance: {}.",
                    format_currency_input(plan.cash_due())
                ));
            }
            text
        })
    };
    let keep_remaining_years = 30usize.saturating_sub(property_data.read().refinance_after_years);

    // SALT cap note for the first analysis year (depends on income)
    let salt_note = {
        let data = property_data.read();
        format!(
            "Note: State/local income tax + property tax share one SALT cap: {} in {} for your income (rises 1%/yr through 2029, then reverts to $10,000 in 2030). Mortgage interest is fully deductible up to a $750k loan.",
            format_currency_input(salt_cap(TAX_BASE_YEAR, data.household_income)),
            TAX_BASE_YEAR,
        )
    };

    rsx! {
        div { class: if dark_mode { "bg-monokai-bgLight rounded-lg shadow p-6" } else { "bg-monokaiLight-bgLight rounded-lg shadow p-6" },
            h2 { class: if dark_mode { "text-xl font-semibold text-monokai-fg mb-4" } else { "text-xl font-semibold text-monokaiLight-fg mb-4" },
                "Buying Scenario"
            }
            p { class: if dark_mode { "text-sm text-monokai-fgMuted mb-6 italic" } else { "text-sm text-monokaiLight-fgMuted mb-6 italic" },
                "Buyer builds equity through mortgage paydown and appreciation. Any monthly savings vs renting are invested."
            }

            div { class: "space-y-4",
                // Home Price
                InputField {
                    label: "Home Price ($)",
                    value: property_data.read().home_price,
                    onchange: move |v| {
                        let mut data = property_data.write();
                        data.home_price = v;
                    },
                    tooltip: "The purchase price of the property",
                    min: Some(0.0),
                    max: Some(10_000_000.0),
                    min_exclusive: true,
                    max_exclusive: true,
                    dark_mode: dark_mode,
                }

                // Down Payment % and $
                DualInputField {
                    label: down_payment_label,
                    percent_value: property_data.read().down_payment_percent,
                    dollar_value: {
                        let data = property_data.read();
                        data.home_price * data.down_payment_percent / 100.0
                    },
                    on_percent_change: move |v| {
                        let mut data = property_data.write();
                        data.down_payment_percent = v;
                    },
                    on_dollar_change: move |dollars| {
                        let data_read = property_data.read();
                        let home_price = data_read.home_price;
                        if home_price > 0.01 {
                            let percent = (dollars / home_price) * 100.0;
                            drop(data_read);
                            let mut data = property_data.write();
                            data.down_payment_percent = percent;
                        }
                    },
                    tooltip: down_payment_tooltip,
                    show_first_year_label: false,
                    percent_min: Some(0.0),
                    percent_max: Some(100.0),
                    dollar_min: Some(0.0),
                    dollar_max: Some(property_data.read().home_price),
                    dark_mode: dark_mode,
                }

                // Interest Rate with Live Fetch
                InterestRateField {
                    property_data: property_data,
                    dark_mode: dark_mode,
                }

                // Property Tax % and $
                DualInputField {
                    label: "Property Tax",
                    percent_value: property_data.read().property_tax_rate,
                    dollar_value: {
                        let data = property_data.read();
                        data.home_price * data.property_tax_rate / 100.0
                    },
                    on_percent_change: move |v| {
                        let mut data = property_data.write();
                        data.property_tax_rate = v;
                    },
                    on_dollar_change: move |dollars| {
                        let data_read = property_data.read();
                        let home_price = data_read.home_price;
                        if home_price > 0.01 {
                            let percent = (dollars / home_price) * 100.0;
                            drop(data_read);
                            let mut data = property_data.write();
                            data.property_tax_rate = percent;
                        }
                    },
                    tooltip: "Annual property tax as percentage of home value",
                    show_first_year_label: true,
                    percent_min: Some(0.0),
                    percent_max: Some(10.0),
                    percent_max_exclusive: true,
                    dollar_min: Some(0.0),
                    dollar_max: Some(property_data.read().home_price * 0.10),
                    dollar_max_exclusive: true,
                    dark_mode: dark_mode,
                }

                // Assessment growth cap (SF Prop 13, DC homestead)
                div { class: "space-y-2",
                    div { class: "flex items-center justify-between",
                        div {
                            label { class: if dark_mode { "text-sm font-medium text-monokai-fgMuted" } else { "text-sm font-medium text-monokaiLight-fgMuted" },
                                "Assessment Growth Cap"
                            }
                            p { class: if dark_mode { "text-xs text-monokai-fgDim" } else { "text-xs text-monokaiLight-fgDim" },
                                "Limits how fast taxable value rises (SF: 2%/yr, DC owner-occupied: 10%/yr)"
                            }
                        }
                        button {
                            r#type: "button",
                            class: if property_data.read().enable_assessment_cap {
                                "relative inline-flex h-6 w-11 flex-shrink-0 items-center rounded-full bg-monokai-purple transition-colors"
                            } else if dark_mode {
                                "relative inline-flex h-6 w-11 flex-shrink-0 items-center rounded-full bg-monokai-bgLighter transition-colors"
                            } else {
                                "relative inline-flex h-6 w-11 flex-shrink-0 items-center rounded-full bg-monokaiLight-border transition-colors"
                            },
                            onclick: move |_| {
                                let mut data = property_data.write();
                                data.enable_assessment_cap = !data.enable_assessment_cap;
                            },
                            span {
                                class: if property_data.read().enable_assessment_cap {
                                    "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-6"
                                } else {
                                    "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-1"
                                },
                            }
                        }
                    }
                    if property_data.read().enable_assessment_cap {
                        InputField {
                            label: "Max Assessment Increase (%/year)",
                            percent: true,
                            value: property_data.read().assessment_cap_percent,
                            onchange: move |v| {
                                let mut data = property_data.write();
                                data.assessment_cap_percent = v;
                            },
                            tooltip: "Taxable value starts at the purchase price and rises at most this much per year (never above market value). Property tax is charged on the taxable value.",
                            min: Some(0.0),
                            max: Some(100.0),
                            dark_mode: dark_mode,
                        }
                    }
                }

                // Home Insurance
                InputField {
                    label: "Home Insurance ($/year)",
                    value: property_data.read().home_insurance_annual,
                    onchange: move |v| {
                        let mut data = property_data.write();
                        data.home_insurance_annual = v;
                    },
                    tooltip: "Annual cost of homeowners insurance",
                    min: Some(0.0),
                    dark_mode: dark_mode,
                }

                // Maintenance % and $
                DualInputField {
                    label: "Maintenance",
                    percent_value: property_data.read().maintenance_percent,
                    dollar_value: {
                        let data = property_data.read();
                        data.home_price * data.maintenance_percent / 100.0
                    },
                    on_percent_change: move |v| {
                        let mut data = property_data.write();
                        data.maintenance_percent = v;
                    },
                    on_dollar_change: move |dollars| {
                        let data_read = property_data.read();
                        let home_price = data_read.home_price;
                        if home_price > 0.01 {
                            let percent = (dollars / home_price) * 100.0;
                            drop(data_read);
                            let mut data = property_data.write();
                            data.maintenance_percent = percent;
                        }
                    },
                    tooltip: "Annual maintenance cost as percentage of home value (typically 1%)",
                    show_first_year_label: true,
                    percent_min: Some(0.0),
                    percent_max: Some(10.0),
                    percent_max_exclusive: true,
                    dollar_min: Some(0.0),
                    dollar_max: Some(property_data.read().home_price * 0.10),
                    dollar_max_exclusive: true,
                    dark_mode: dark_mode,
                }

                // HOA Fee
                InputField {
                    label: "HOA/Condo Fee ($/mo)",
                    value: property_data.read().hoa_fee,
                    onchange: move |v| {
                        let mut data = property_data.write();
                        data.hoa_fee = v;
                    },
                    tooltip: "Monthly HOA or condo association fee",
                    min: Some(0.0),
                    dark_mode: dark_mode,
                }

                // HOA Fee Increase % and $
                DualInputField {
                    label: "HOA Fee Increase",
                    percent_value: property_data.read().hoa_fee_increase_rate,
                    dollar_value: {
                        let data = property_data.read();
                        data.hoa_fee * data.hoa_fee_increase_rate / 100.0
                    },
                    on_percent_change: move |v| {
                        let mut data = property_data.write();
                        data.hoa_fee_increase_rate = v;
                    },
                    on_dollar_change: move |dollars| {
                        let data_read = property_data.read();
                        let hoa_fee = data_read.hoa_fee;
                        if hoa_fee > 0.01 {
                            let percent = (dollars / hoa_fee) * 100.0;
                            drop(data_read);
                            let mut data = property_data.write();
                            data.hoa_fee_increase_rate = percent;
                        }
                    },
                    tooltip: "Expected annual increase rate for HOA fees (typically 3%)",
                    show_first_year_label: true,
                    percent_min: Some(0.0),
                    percent_max: Some(50.0),
                    percent_max_exclusive: true,
                    dollar_min: Some(0.0),
                    dollar_max: Some(property_data.read().hoa_fee * 0.50),
                    dollar_max_exclusive: true,
                    dark_mode: dark_mode,
                }

                // PMI: annual % of the loan, shown with the resulting $/mo
                DualInputField {
                    label: "PMI (annual % of loan)",
                    percent_value: property_data.read().pmi_rate_percent,
                    dollar_value: property_data.read().monthly_pmi(),
                    on_percent_change: move |v| {
                        let mut data = property_data.write();
                        data.pmi_rate_percent = v;
                    },
                    on_dollar_change: move |dollars_per_month| {
                        let data_read = property_data.read();
                        let loan = data_read.loan_amount();
                        if loan > 0.01 {
                            let percent = dollars_per_month * 12.0 / loan * 100.0;
                            drop(data_read);
                            let mut data = property_data.write();
                            data.pmi_rate_percent = percent;
                        }
                    },
                    tooltip: "Private mortgage insurance as a yearly % of the first mortgage (typically 0.3-1.0%). It updates automatically with price, down payment and HPAP, and is $0 when the loan starts at or below 80% of the price.",
                    dollar_label: Some("Monthly PMI ($/mo)".to_string()),
                    show_first_year_label: false,
                    percent_min: Some(0.0),
                    percent_max: Some(5.0),
                    dollar_min: Some(0.0),
                    dollar_max: Some(property_data.read().loan_amount().max(0.0) * 0.05 / 12.0),
                    dark_mode: dark_mode,
                }

                // PMI drop-off LTV threshold (only shown when PMI applies)
                if property_data.read().monthly_pmi() > 0.0 {
                    InputField {
                        label: "PMI Drop-off LTV (%)",
                        percent: true,
                        value: property_data.read().pmi_drop_off_ltv,
                        onchange: move |v| {
                            let mut data = property_data.write();
                            data.pmi_drop_off_ltv = v;
                        },
                        tooltip: "Loan-to-value ratio at which PMI is cancelled. Lenders must automatically cancel at 78% of original purchase price (per federal law); you can request cancellation at 80%.",
                        min: Some(0.0),
                        max: Some(100.0),
                        dark_mode: dark_mode,
                    }
                }

                // Non-included Utilities
                InputField {
                    label: "Non-Included Utilities ($/mo)",
                    value: property_data.read().non_included_utilities,
                    onchange: move |v| {
                        let mut data = property_data.write();
                        data.non_included_utilities = v;
                    },
                    tooltip: "Monthly cost of utilities you pay separately",
                    min: Some(0.0),
                    dark_mode: dark_mode,
                }

                // Purchase Closing Costs % and $
                DualInputField {
                    label: "Purchase Closing Costs",
                    percent_value: property_data.read().closing_costs_percent_purchase,
                    dollar_value: {
                        let data = property_data.read();
                        data.home_price * data.closing_costs_percent_purchase / 100.0
                    },
                    on_percent_change: move |v| {
                        let mut data = property_data.write();
                        data.closing_costs_percent_purchase = v;
                    },
                    on_dollar_change: move |dollars| {
                        let data_read = property_data.read();
                        let home_price = data_read.home_price;
                        if home_price > 0.01 {
                            let percent = (dollars / home_price) * 100.0;
                            drop(data_read);
                            let mut data = property_data.write();
                            data.closing_costs_percent_purchase = percent;
                        }
                    },
                    tooltip: "Closing costs at purchase as percentage of home price (typically 3-5%)",
                    show_first_year_label: false,
                    percent_min: Some(0.0),
                    percent_max: Some(20.0),
                    dollar_min: Some(0.0),
                    dollar_max: Some(property_data.read().home_price * 0.20),
                    dark_mode: dark_mode,
                }

                // Sale Closing Costs % and $
                DualInputField {
                    label: "Sale Closing Costs",
                    percent_value: property_data.read().closing_costs_percent_sale,
                    dollar_value: {
                        let data = property_data.read();
                        data.home_price * data.closing_costs_percent_sale / 100.0
                    },
                    on_percent_change: move |v| {
                        let mut data = property_data.write();
                        data.closing_costs_percent_sale = v;
                    },
                    on_dollar_change: move |dollars| {
                        let data_read = property_data.read();
                        let home_price = data_read.home_price;
                        if home_price > 0.01 {
                            let percent = (dollars / home_price) * 100.0;
                            drop(data_read);
                            let mut data = property_data.write();
                            data.closing_costs_percent_sale = percent;
                        }
                    },
                    tooltip: "Closing costs when selling as percentage of sale price (typically 6%)",
                    show_first_year_label: false,
                    percent_min: Some(0.0),
                    percent_max: Some(20.0),
                    dollar_min: Some(0.0),
                    dollar_max: Some(property_data.read().home_price * 0.20),
                    dark_mode: dark_mode,
                }

                // Home Appreciation Rate % and $
                DualInputField {
                    label: "Home Appreciation Rate",
                    percent_value: property_data.read().home_appreciation_rate,
                    dollar_value: {
                        let data = property_data.read();
                        data.home_price * data.home_appreciation_rate / 100.0
                    },
                    on_percent_change: move |v| {
                        let mut data = property_data.write();
                        data.home_appreciation_rate = v;
                    },
                    on_dollar_change: move |dollars| {
                        let data_read = property_data.read();
                        let home_price = data_read.home_price;
                        if home_price > 0.01 {
                            let percent = (dollars / home_price) * 100.0;
                            drop(data_read);
                            let mut data = property_data.write();
                            data.home_appreciation_rate = percent;
                        }
                    },
                    tooltip: "Expected annual home value appreciation rate",
                    show_first_year_label: true,
                    percent_min: Some(-10.0),
                    percent_max: Some(50.0),
                    percent_max_exclusive: true,
                    dollar_min: Some(-property_data.read().home_price * 0.10),
                    dollar_max: Some(property_data.read().home_price * 0.50),
                    dollar_max_exclusive: true,
                    dark_mode: dark_mode,
                }

                // DC HPAP Section
                div { class: if dark_mode { "border-t border-monokai-bgLighter pt-4 mt-4" } else { "border-t border-monokaiLight-border pt-4 mt-4" },
                    // HPAP Toggle
                    div { class: "flex items-center justify-between mb-3",
                        div {
                            label { class: if dark_mode { "text-sm font-medium text-monokai-fg" } else { "text-sm font-medium text-monokaiLight-fg" },
                                "DC HPAP"
                            }
                            p { class: if dark_mode { "text-xs text-monokai-fgMuted" } else { "text-xs text-monokaiLight-fgMuted" },
                                "DC Home Purchase Assistance Program: 0% deferred loan for eligible first-time DC buyers"
                            }
                        }
                        button {
                            r#type: "button",
                            class: if property_data.read().enable_hpap {
                                "relative inline-flex h-6 w-11 flex-shrink-0 items-center rounded-full bg-monokai-purple transition-colors"
                            } else {
                                if dark_mode {
                                    "relative inline-flex h-6 w-11 flex-shrink-0 items-center rounded-full bg-monokai-bgLighter transition-colors"
                                } else {
                                    "relative inline-flex h-6 w-11 flex-shrink-0 items-center rounded-full bg-monokaiLight-border transition-colors"
                                }
                            },
                            onclick: move |_| {
                                let mut data = property_data.write();
                                data.enable_hpap = !data.enable_hpap;
                            },
                            span {
                                class: if property_data.read().enable_hpap {
                                    "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-6"
                                } else {
                                    "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-1"
                                },
                            }
                        }
                    }

                    // HPAP details (only shown when enabled)
                    if property_data.read().enable_hpap {
                        div { class: if dark_mode { "bg-monokai-bgLighter rounded-lg p-4 space-y-3" } else { "bg-monokaiLight-bg rounded-lg p-4 space-y-3" },
                            // How the HPAP maximum is determined
                            div {
                                label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-1" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-1" },
                                    "HPAP Maximum"
                                }
                                select {
                                    class: if dark_mode {
                                        "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                                    } else {
                                        "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                                    },
                                    value: if property_data.read().hpap_enter_own_max { "own" } else { "table" },
                                    onchange: move |evt| {
                                        let mut data = property_data.write();
                                        data.hpap_enter_own_max = evt.value() == "own";
                                    },
                                    option { value: "own", "I know my maximum" }
                                    option { value: "table", "Calculate it for me (household size + income)" }
                                }
                            }

                            if property_data.read().hpap_enter_own_max {
                                InputField {
                                    label: "Your HPAP Maximum ($)",
                                    value: property_data.read().hpap_own_max,
                                    onchange: move |v| {
                                        let mut data = property_data.write();
                                        data.hpap_own_max = v;
                                    },
                                    tooltip: "Your maximum HPAP gap financing (e.g., from your DHCD eligibility letter), not counting closing cost assistance. The program cap is $202,000.",
                                    min: Some(0.0),
                                    max: Some(HPAP_MAX_GAP_FINANCING),
                                    dark_mode: dark_mode,
                                }

                                div {
                                    label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-1" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-1" },
                                        "Your Repayment Terms"
                                    }
                                    select {
                                        class: if dark_mode {
                                            "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                                        } else {
                                            "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                                        },
                                        value: if property_data.read().hpap_own_repayment == HpapRepayment::FiveYearDeferral40Year { "payments" } else { "deferred" },
                                        onchange: move |evt| {
                                            let mut data = property_data.write();
                                            data.hpap_own_repayment = if evt.value() == "payments" {
                                                HpapRepayment::FiveYearDeferral40Year
                                            } else {
                                                HpapRepayment::DeferredUntilSale
                                            };
                                        },
                                        option { value: "deferred", "No monthly payments; due on sale (up to 80% MFI)" }
                                        option { value: "payments", "No payments for 5 years, then 40-year payments (81-110% MFI)" }
                                    }
                                }

                                InputField {
                                    label: "Your Required Contribution ($)",
                                    value: property_data.read().hpap_own_contribution,
                                    onchange: move |v| {
                                        let mut data = property_data.write();
                                        data.hpap_own_contribution = v;
                                    },
                                    tooltip: "Cash you must put in yourself, usually stated in your DHCD paperwork (at least $500, or 50% of liquid assets above $3,000). HPAP covers the rest of the down payment up to your maximum.",
                                    min: Some(0.0),
                                    dark_mode: dark_mode,
                                }
                            } else {
                                // Household size (DHCD table column)
                                div {
                                    label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-1" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-1" },
                                        "Household Size"
                                    }
                                    select {
                                        class: if dark_mode {
                                            "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                                        } else {
                                            "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                                        },
                                        value: "{property_data.read().hpap_household_size}",
                                        onchange: move |evt| {
                                            if let Ok(n) = evt.value().parse::<usize>() {
                                                let mut data = property_data.write();
                                                data.hpap_household_size = n.clamp(1, HPAP_MAX_HOUSEHOLD_SIZE);
                                            }
                                        },
                                        for n in 1..=HPAP_MAX_HOUSEHOLD_SIZE {
                                            option { value: "{n}", selected: n == property_data.read().hpap_household_size, "{n}" }
                                        }
                                    }
                                }

                                // Household income (same value as the Tax Benefits section)
                                InputField {
                                    label: "Household Income ($/year)",
                                    value: property_data.read().household_income,
                                    onchange: move |v| {
                                        let mut data = property_data.write();
                                        data.household_income = v;
                                    },
                                    tooltip: "Total annual income of everyone in the household. With household size, this sets your HPAP tier from the DHCD FY26 table (loans closed after Oct 1, 2026). Also used for taxes.",
                                    min: Some(0.0),
                                    max: Some(100_000_000.0),
                                    max_exclusive: true,
                                    dark_mode: dark_mode,
                                }

                                InputField {
                                    label: "Savings / Liquid Assets ($)",
                                    value: property_data.read().hpap_liquid_assets,
                                    onchange: move |v| {
                                        let mut data = property_data.write();
                                        data.hpap_liquid_assets = v;
                                    },
                                    tooltip: "Cash and liquid assets before buying. HPAP requires you to contribute $500 or 50% of liquid assets above $3,000, whichever is greater.",
                                    min: Some(0.0),
                                    dark_mode: dark_mode,
                                }
                            }

                            // Tier / eligibility: the maximum, shown before choosing how much to use
                            p { class: if hpap_eligible {
                                    if dark_mode { "text-xs text-monokai-fg" } else { "text-xs text-monokaiLight-fg" }
                                } else {
                                    if dark_mode { "text-xs text-monokai-red" } else { "text-xs text-monokaiLight-red" }
                                },
                                "{hpap_tier_text}"
                            }

                            // How much of the maximum to use (sets the total down payment)
                            if hpap_eligible {
                                // Amount field with the "Use full HPAP" button beside it
                                div { class: "flex items-start gap-2",
                                    div { class: "flex-1",
                                        InputField {
                                            label: hpap_amount_label.clone(),
                                            value: property_data.read().hpap_gap_financing(),
                                            onchange: move |v| {
                                                let percent = property_data.read().hpap_down_payment_percent_for(v);
                                                if let Some(percent) = percent {
                                                    let mut data = property_data.write();
                                                    data.down_payment_percent = percent;
                                                }
                                            },
                                            tooltip: "How much of your HPAP maximum to borrow (0% interest). Your total down payment is set to this amount plus your own contribution. Using more HPAP means a smaller bank mortgage.",
                                            min: Some(0.0),
                                            max: Some(hpap_max_amount),
                                            dark_mode: dark_mode,
                                        }
                                    }
                                    if let Some(percent) = hpap_full_percent.filter(|_| !hpap_room_text.is_empty()) {
                                        button {
                                            r#type: "button",
                                            // Same box as the input: label offset (mt-6), padding, text size and border
                                            class: "mt-6 px-3 py-2 border border-transparent text-base font-medium rounded-md shadow-sm bg-monokai-purple text-white hover:opacity-90 transition-opacity whitespace-nowrap",
                                            onclick: move |_| {
                                                let mut data = property_data.write();
                                                data.down_payment_percent = percent;
                                            },
                                            "Use full HPAP"
                                        }
                                    }
                                }
                            }

                            InputField {
                                label: "HPAP Closing Cost Assistance ($)",
                                value: property_data.read().hpap_closing_cost_assistance,
                                onchange: move |v| {
                                    let mut data = property_data.write();
                                    data.hpap_closing_cost_assistance = v;
                                },
                                tooltip: "HPAP assistance toward closing costs (up to $4,000), added to the HPAP loan balance",
                                min: Some(0.0),
                                max: Some(HPAP_MAX_CLOSING_COST_ASSISTANCE),
                                dark_mode: dark_mode,
                            }

                            if hpap_eligible {
                                p { class: if dark_mode { "text-xs text-monokai-fgMuted" } else { "text-xs text-monokaiLight-fgMuted" },
                                    "{hpap_summary}"
                                }
                            }

                            if !hpap_room_text.is_empty() {
                                p { class: if dark_mode { "text-xs text-monokai-fgMuted" } else { "text-xs text-monokaiLight-fgMuted" },
                                    "{hpap_room_text}"
                                }
                            }

                            p { class: if dark_mode { "text-xs text-monokai-fgDim italic" } else { "text-xs text-monokaiLight-fgDim italic" },
                                "Note: HPAP is a 0% loan, due in full when you sell, refinance, or stop living there (the model assumes you stay and don't refinance). You must be a first-time buyer (no home ownership in the last 3 years), live in the DC home as your primary residence, and have good credit. Funding is limited each year; confirm with DC DHCD."
                            }
                        }
                    }
                }

                // Refinance Section
                div { class: if dark_mode { "border-t border-monokai-bgLighter pt-4 mt-4" } else { "border-t border-monokaiLight-border pt-4 mt-4" },
                    div { class: "flex items-center justify-between mb-3",
                        div {
                            label { class: if dark_mode { "text-sm font-medium text-monokai-fg" } else { "text-sm font-medium text-monokaiLight-fg" },
                                "Refinance"
                            }
                            p { class: if dark_mode { "text-xs text-monokai-fgMuted" } else { "text-xs text-monokaiLight-fgMuted" },
                                "Switch to a new rate and term later (no cash-out)"
                            }
                        }
                        button {
                            r#type: "button",
                            class: if property_data.read().enable_refinance {
                                "relative inline-flex h-6 w-11 flex-shrink-0 items-center rounded-full bg-monokai-purple transition-colors"
                            } else if dark_mode {
                                "relative inline-flex h-6 w-11 flex-shrink-0 items-center rounded-full bg-monokai-bgLighter transition-colors"
                            } else {
                                "relative inline-flex h-6 w-11 flex-shrink-0 items-center rounded-full bg-monokaiLight-border transition-colors"
                            },
                            onclick: move |_| {
                                let mut data = property_data.write();
                                data.enable_refinance = !data.enable_refinance;
                            },
                            span {
                                class: if property_data.read().enable_refinance {
                                    "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-6"
                                } else {
                                    "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-1"
                                },
                            }
                        }
                    }

                    if property_data.read().enable_refinance {
                        div { class: if dark_mode { "bg-monokai-bgLighter rounded-lg p-4 space-y-3" } else { "bg-monokaiLight-bg rounded-lg p-4 space-y-3" },
                            div {
                                label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-1" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-1" },
                                    "Refinance After Year"
                                }
                                select {
                                    class: if dark_mode {
                                        "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                                    } else {
                                        "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                                    },
                                    value: "{property_data.read().refinance_after_years}",
                                    onchange: move |evt| {
                                        if let Ok(n) = evt.value().parse::<usize>() {
                                            let mut data = property_data.write();
                                            data.refinance_after_years = n.clamp(1, 29);
                                        }
                                    },
                                    for n in 1..=29usize {
                                        option { value: "{n}", selected: n == property_data.read().refinance_after_years, "{n}" }
                                    }
                                }
                            }

                            InputField {
                                label: "New Interest Rate (%)",
                                percent: true,
                                value: property_data.read().refinance_rate,
                                onchange: move |v| {
                                    let mut data = property_data.write();
                                    data.refinance_rate = v;
                                },
                                tooltip: "Interest rate on the new loan",
                                min: Some(0.0),
                                max: Some(20.0),
                                min_exclusive: true,
                                max_exclusive: true,
                                dark_mode: dark_mode,
                            }

                            div {
                                label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-1" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-1" },
                                    "New Loan Term"
                                }
                                select {
                                    class: if dark_mode {
                                        "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                                    } else {
                                        "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                                    },
                                    value: match property_data.read().refinance_term {
                                        RefinanceTerm::KeepRemaining => "keep",
                                        RefinanceTerm::Years30 => "30",
                                        RefinanceTerm::Years20 => "20",
                                        RefinanceTerm::Years15 => "15",
                                    },
                                    onchange: move |evt| {
                                        let mut data = property_data.write();
                                        data.refinance_term = match evt.value().as_str() {
                                            "keep" => RefinanceTerm::KeepRemaining,
                                            "20" => RefinanceTerm::Years20,
                                            "15" => RefinanceTerm::Years15,
                                            _ => RefinanceTerm::Years30,
                                        };
                                    },
                                    option { value: "30", "New 30-year loan" }
                                    option { value: "20", "New 20-year loan" }
                                    option { value: "15", "New 15-year loan" }
                                    option { value: "keep", "Keep remaining term ({keep_remaining_years} years)" }
                                }
                            }

                            InputField {
                                label: "Refinance Closing Costs (% of loan)",
                                percent: true,
                                value: property_data.read().refinance_closing_cost_percent,
                                onchange: move |v| {
                                    let mut data = property_data.write();
                                    data.refinance_closing_cost_percent = v;
                                },
                                tooltip: "Lender, title and appraisal fees, as a % of the new loan (typically 2-3%)",
                                min: Some(0.0),
                                max: Some(10.0),
                                dark_mode: dark_mode,
                            }

                            div {
                                label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-1" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-1" },
                                    "Pay Closing Costs"
                                }
                                select {
                                    class: if dark_mode {
                                        "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                                    } else {
                                        "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                                    },
                                    value: if property_data.read().refinance_cost_payment == RefinanceCostPayment::PayCash { "cash" } else { "roll" },
                                    onchange: move |evt| {
                                        let mut data = property_data.write();
                                        data.refinance_cost_payment = if evt.value() == "cash" {
                                            RefinanceCostPayment::PayCash
                                        } else {
                                            RefinanceCostPayment::RollIntoLoan
                                        };
                                    },
                                    option { value: "roll", "Roll into the new loan" }
                                    option { value: "cash", "Pay in cash" }
                                }
                            }

                            if property_data.read().enable_hpap {
                                div {
                                    label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-1" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-1" },
                                        "HPAP at Refinance"
                                    }
                                    select {
                                        class: if dark_mode {
                                        "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                                    } else {
                                        "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                                    },
                                        value: match property_data.read().hpap_at_refinance {
                                            HpapAtRefinance::RollIntoLoan => "roll",
                                            HpapAtRefinance::PayCash => "cash",
                                            HpapAtRefinance::Keep => "keep",
                                        },
                                        onchange: move |evt| {
                                            let mut data = property_data.write();
                                            data.hpap_at_refinance = match evt.value().as_str() {
                                                "cash" => HpapAtRefinance::PayCash,
                                                "keep" => HpapAtRefinance::Keep,
                                                _ => HpapAtRefinance::RollIntoLoan,
                                            };
                                        },
                                        option { value: "roll", "Roll into the new loan (starts costing interest)" }
                                        option { value: "cash", "Pay it off in cash" }
                                        option { value: "keep", "Keep it (DHCD approves)" }
                                    }
                                    p { class: if dark_mode { "text-xs text-monokai-fgDim italic mt-1" } else { "text-xs text-monokaiLight-fgDim italic mt-1" },
                                        "DHCD makes HPAP due on refinance unless it approves keeping it in place."
                                    }
                                }
                            }

                            if let Some(summary) = refinance_summary.clone() {
                                p { class: if dark_mode { "text-xs text-monokai-fgMuted" } else { "text-xs text-monokaiLight-fgMuted" },
                                    "{summary}"
                                }
                            }
                        }
                    }
                }
            }

            // Tax Benefits Section
            div { class: if dark_mode { "mt-6 pt-6 border-t border-monokai-border" } else { "mt-6 pt-6 border-t border-monokaiLight-border" },
                h3 { class: if dark_mode { "text-lg font-medium text-monokai-fg mb-2" } else { "text-lg font-medium text-monokaiLight-fg mb-2" },
                    "Tax Benefits (US)"
                }
                p { class: if dark_mode { "text-sm text-monokai-fgMuted mb-4" } else { "text-sm text-monokaiLight-fgMuted mb-4" },
                    "Property taxes and mortgage interest are deductible. You only benefit if itemized deductions exceed the standard deduction."
                }

                div { class: "space-y-4",
                    // Enable Tax Benefits Toggle
                    div { class: "flex items-center justify-between",
                        div {
                            label { class: if dark_mode { "text-sm font-medium text-monokai-fgMuted" } else { "text-sm font-medium text-monokaiLight-fgMuted" },
                                "Enable Tax Benefits"
                            }
                            p { class: if dark_mode { "text-xs text-monokai-fgDim" } else { "text-xs text-monokaiLight-fgDim" },
                                "Calculate tax savings from homeownership deductions"
                            }
                        }
                        button {
                            r#type: "button",
                            class: if property_data.read().enable_tax_benefits {
                                if dark_mode {
                                    "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-monokai-green transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-monokai-green focus:ring-offset-2"
                                } else {
                                    "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-monokaiLight-green transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-monokaiLight-green focus:ring-offset-2"
                                }
                            } else {
                                if dark_mode {
                                    "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-monokai-bgHighlight transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-monokai-green focus:ring-offset-2"
                                } else {
                                    "relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-monokaiLight-bgHighlight transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-monokaiLight-green focus:ring-offset-2"
                                }
                            },
                            onclick: move |_| {
                                let mut data = property_data.write();
                                data.enable_tax_benefits = !data.enable_tax_benefits;
                            },
                            span {
                                class: if property_data.read().enable_tax_benefits {
                                    "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out translate-x-5"
                                } else {
                                    "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out translate-x-0"
                                }
                            }
                        }
                    }

                    // Only show tax fields if enabled
                    if property_data.read().enable_tax_benefits {
                        // Filing Status
                        div {
                            label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-1" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-1" },
                                "Filing Status"
                            }
                            select {
                                class: if dark_mode {
                                    "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                                } else {
                                    "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                                },
                                value: if matches!(property_data.read().filing_status, FilingStatus::Single) { "single" } else { "joint" },
                                onchange: move |evt| {
                                    let mut data = property_data.write();
                                    data.filing_status = if evt.value() == "joint" {
                                        FilingStatus::MarriedFilingJointly
                                    } else {
                                        FilingStatus::Single
                                    };
                                },
                                option { value: "single", "Individual return ($16,100 std deduction)" }
                                option { value: "joint", "Joint return ($32,200 std deduction)" }
                            }
                        }

                        // Marginal Tax Rate % and estimated tax savings $
                        DualInputField {
                            label: "Marginal Tax Rate",
                            percent_value: property_data.read().marginal_tax_rate,
                            dollar_value: {
                                let data = property_data.read();
                                // Calculate first year mortgage interest
                                let loan_amount = data.loan_amount();
                                let annual_interest_rate = data.effective_interest_rate();
                                let calculator = MortgageCalculator::new(loan_amount, annual_interest_rate, 30);
                                let first_year_interest = calculator.calculate_interest_paid_in_year(1);

                                // Calculate tax savings
                                data.calculate_annual_tax_savings(
                                    1,
                                    first_year_interest,
                                    data.home_price,
                                    data.hoa_fee * 12.0,
                                )
                            },
                            on_percent_change: move |v| {
                                let mut data = property_data.write();
                                data.marginal_tax_rate = v;
                            },
                            on_dollar_change: move |_| {
                                // Dollar input is read-only, no action needed
                            },
                            tooltip: "Your marginal federal income tax rate (the rate on your last dollar of income)",
                            dollar_label: Some("Estimated first year tax savings".to_string()),
                            percent_min: Some(0.0),
                            percent_max: Some(50.0),
                            dollar_min: Some(0.0),
                            dollar_max: Some(50000.0),
                            disable_dollar_input: true,
                            dark_mode: dark_mode,
                        }

                        // Household Income (drives state income tax and the SALT cap phase-down)
                        InputField {
                            label: "Household Income ($/year)",
                            value: property_data.read().household_income,
                            onchange: move |v| {
                                let mut data = property_data.write();
                                data.household_income = v;
                            },
                            tooltip: "Approximate annual household income (MAGI). Used to estimate state income tax and the SALT cap, which phases down above ~$505,000.",
                            min: Some(0.0),
                            max: Some(100_000_000.0),
                            max_exclusive: true,
                            dark_mode: dark_mode,
                        }

                        // State/Local Income Tax % of income and $
                        DualInputField {
                            label: "State/Local Income Tax",
                            percent_value: property_data.read().state_income_tax_rate,
                            dollar_value: property_data.read().annual_state_income_tax(),
                            on_percent_change: move |v| {
                                let mut data = property_data.write();
                                data.state_income_tax_rate = v;
                            },
                            on_dollar_change: move |dollars| {
                                let data_read = property_data.read();
                                let income = data_read.household_income;
                                if income > 0.01 {
                                    let percent = (dollars / income) * 100.0;
                                    drop(data_read);
                                    let mut data = property_data.write();
                                    data.state_income_tax_rate = percent;
                                }
                            },
                            tooltip: "Effective (average) state + local income tax as a % of income. Shares the SALT cap with property tax, and counts for renters too. DC is roughly 6-7% for middle incomes; enter 0 for states without income tax.",
                            show_first_year_label: false,
                            percent_min: Some(0.0),
                            percent_max: Some(20.0),
                            dollar_min: Some(0.0),
                            dollar_max: Some(property_data.read().household_income * 0.20),
                            dark_mode: dark_mode,
                        }

                        // Other Itemized Deductions
                        InputField {
                            label: "Other Itemized Deductions ($)",
                            value: property_data.read().other_itemized_deductions,
                            onchange: move |v| {
                                let mut data = property_data.write();
                                data.other_itemized_deductions = v;
                            },
                            tooltip: "Other annual itemized deductions (charitable contributions, medical expenses, etc.)",
                            min: Some(0.0),
                            dark_mode: dark_mode,
                        }

                        // HOA/Common Fees Deduction % and $
                        div { class: "space-y-1",
                            DualInputField {
                                label: "HOA Fees Deduction",
                                percent_value: property_data.read().hoa_deduction_percent,
                                dollar_value: {
                                    let data = property_data.read();
                                    data.hoa_fee * 12.0 * data.hoa_deduction_percent / 100.0
                                },
                                on_percent_change: move |v| {
                                    let mut data = property_data.write();
                                    data.hoa_deduction_percent = v;
                                },
                                on_dollar_change: move |dollars| {
                                    let data_read = property_data.read();
                                    let annual_hoa_fees = data_read.hoa_fee * 12.0;
                                    if annual_hoa_fees > 0.01 {
                                        let percent = (dollars / annual_hoa_fees) * 100.0;
                                        drop(data_read);
                                        let mut data = property_data.write();
                                        data.hoa_deduction_percent = percent;
                                    }
                                },
                                tooltip: "Percentage of HOA fees that are tax deductible",
                                show_first_year_label: true,
                                percent_min: Some(0.0),
                                percent_max: Some(100.0),
                                dollar_min: Some(0.0),
                                dollar_max: Some(property_data.read().hoa_fee * 12.0),
                                dark_mode: dark_mode,
                            }
                            p { class: if dark_mode { "text-xs text-monokai-fgDim italic mt-1" } else { "text-xs text-monokaiLight-fgDim italic mt-1" },
                                "Note: HOA fees are typically NOT deductible for primary residences. Only adjust if you have a home office."
                            }
                        }

                        // Info about SALT cap
                        p { class: if dark_mode { "text-xs text-monokai-fgDim italic" } else { "text-xs text-monokaiLight-fgDim italic" },
                            "{salt_note}"
                        }
                    }
                }
            }
        }
    }
}

/// Format a number as currency with commas and 2 decimal places ("$1,234.56").
/// Delegates to the shared formatter, which rounds to cents before splitting dollars and
/// cents (so $7,999.9999 shows as $8,000.00, not "$7,999.100").
fn format_currency_input(value: f64) -> String {
    crate::utils::CurrencyFormatter::format_currency(value)
}

#[component]
fn InputField(
    label: String,
    value: f64,
    onchange: EventHandler<f64>,
    tooltip: String,
    #[props(default = None)] min: Option<f64>,
    #[props(default = None)] max: Option<f64>,
    #[props(default = false)] min_exclusive: bool,
    #[props(default = false)] max_exclusive: bool,
    /// Show and parse the value as a percentage ("10.00%") instead of currency ("$10.00")
    #[props(default = false)]
    percent: bool,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    let mut is_focused = use_signal(|| false);
    let mut input_text = use_signal(String::new);

    // Validate the value
    let is_valid = {
        let mut valid = true;
        if let Some(min_val) = min {
            if min_exclusive {
                if value <= min_val {
                    valid = false;
                }
            } else if value < min_val {
                valid = false;
            }
        }
        if let Some(max_val) = max {
            if max_exclusive {
                if value >= max_val {
                    valid = false;
                }
            } else if value > max_val {
                valid = false;
            }
        }
        valid
    };

    // Build range text
    let range_text = match (min, max, min_exclusive, max_exclusive) {
        (Some(min_val), Some(max_val), min_ex, max_ex) => {
            let min_op = if min_ex { ">" } else { ">=" };
            let max_op = if max_ex { "<" } else { "<=" };
            format!(
                "Valid range: {} {} and {} {}",
                min_op, min_val, max_op, max_val
            )
        }
        (Some(min_val), None, min_ex, _) => {
            let min_op = if min_ex { ">" } else { ">=" };
            format!("Must be {} {}", min_op, min_val)
        }
        (None, Some(max_val), _, max_ex) => {
            let max_op = if max_ex { "<" } else { "<=" };
            format!("Must be {} {}", max_op, max_val)
        }
        (None, None, _, _) => String::new(),
    };

    rsx! {
        div {
            label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-1" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-1" },
                "{label}"
            }
            input {
                r#type: "text",
                class: if dark_mode {
                    if is_valid {
                        "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                    } else {
                        "w-full px-3 py-2 border border-monokai-red bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-red focus:border-monokai-red"
                    }
                } else {
                    if is_valid {
                        "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                    } else {
                        "w-full px-3 py-2 border border-monokaiLight-red bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-red focus:border-monokaiLight-red"
                    }
                },
                value: if is_focused() {
                    input_text()
                } else {
                    if percent {
                        format!("{:.2}%", value)
                    } else {
                        format_currency_input(value)
                    }
                },
                inputmode: "decimal",
                onfocus: move |_| {
                    is_focused.set(true);
                    input_text.set(value.to_string());
                },
                onblur: move |_| {
                    is_focused.set(false);
                    // Parse and update on blur
                    if let Some(v) = crate::utils::CurrencyFormatter::parse_input(&input_text()) {
                        onchange.call(v);
                    }
                },
                oninput: move |evt| {
                    // Just track what user is typing, don't update value yet
                    input_text.set(evt.value());
                },
                title: "{tooltip}"
            }
            if !is_valid && !range_text.is_empty() {
                p { class: if dark_mode { "text-xs text-monokai-red mt-1" } else { "text-xs text-monokaiLight-red mt-1" },
                    "{range_text}"
                }
            } else if is_valid && !range_text.is_empty() {
                p { class: if dark_mode { "text-xs text-monokai-fgDim mt-1" } else { "text-xs text-monokaiLight-fgDim mt-1" },
                    "{range_text}"
                }
            }
        }
    }
}

#[component]
fn InterestRateField(
    property_data: Signal<PropertyData>,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    let mut is_focused = use_signal(|| false);
    let mut input_text = use_signal(String::new);

    let value = property_data.read().interest_rate;
    let is_valid = value > 0.0 && value < 20.0;

    // Build rate info label from compile-time FRED data
    let rate_info: Option<String> = if crate::models::is_default_rate_from_fred() {
        crate::models::default_interest_rate_date().map(|date| {
            format!(
                "Defaults to the FRED 30-yr fixed rate as of {} (updated weekly)",
                date
            )
        })
    } else {
        None
    };

    rsx! {
        div {
            label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-1" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-1" },
                "Interest Rate (%)"
            }
            input {
                r#type: "text",
                class: if dark_mode {
                    if is_valid {
                        "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                    } else {
                        "w-full px-3 py-2 border border-monokai-red bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-red focus:border-monokai-red"
                    }
                } else {
                    if is_valid {
                        "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                    } else {
                        "w-full px-3 py-2 border border-monokaiLight-red bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-red focus:border-monokaiLight-red"
                    }
                },
                value: if is_focused() {
                    input_text()
                } else {
                    format!("{:.2}%", value)
                },
                inputmode: "decimal",
                onfocus: move |_| {
                    is_focused.set(true);
                    input_text.set(value.to_string());
                },
                onblur: move |_| {
                    is_focused.set(false);
                    if let Some(v) = crate::utils::CurrencyFormatter::parse_input(&input_text()) {
                        let mut data = property_data.write();
                        data.interest_rate = v;
                    }
                },
                oninput: move |evt| {
                    input_text.set(evt.value());
                },
                title: "Annual mortgage interest rate"
            }
            if let Some(info) = rate_info {
                p { class: if dark_mode { "text-xs text-monokai-fgDim mt-1" } else { "text-xs text-monokaiLight-fgDim mt-1" },
                    "{info}"
                }
            }
            if !is_valid {
                p { class: if dark_mode { "text-xs text-monokai-red mt-1" } else { "text-xs text-monokaiLight-red mt-1" },
                    "Valid range: > 0 and < 20"
                }
            }
        }
    }
}
