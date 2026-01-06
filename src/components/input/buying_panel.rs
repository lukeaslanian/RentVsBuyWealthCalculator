use crate::models::{FilingStatus, PropertyData};
use crate::utils::fetch_current_mortgage_rate;
use dioxus::prelude::*;

#[component]
pub fn BuyingInputPanel(
    property_data: Signal<PropertyData>,
    #[props(default = false)] dark_mode: bool,
) -> Element {
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

                // Down Payment %
                InputField {
                    label: "Down Payment (%)",
                    value: property_data.read().down_payment_percent,
                    onchange: move |v| {
                        let mut data = property_data.write();
                        data.down_payment_percent = v;
                    },
                    tooltip: "Percentage of home price paid upfront (e.g., 3% or 20%)",
                    min: Some(0.0),
                    max: Some(100.0),
                    dark_mode: dark_mode,
                }

                // Interest Rate with Live Fetch
                InterestRateField {
                    property_data: property_data,
                    dark_mode: dark_mode,
                }

                // Property Tax
                InputField {
                    label: "Property Tax (%/year)",
                    value: property_data.read().property_tax_rate,
                    onchange: move |v| {
                        let mut data = property_data.write();
                        data.property_tax_rate = v;
                    },
                    tooltip: "Annual property tax as percentage of home value",
                    min: Some(0.0),
                    max: Some(10.0),
                    max_exclusive: true,
                    dark_mode: dark_mode,
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

                // Maintenance
                InputField {
                    label: "Maintenance (%/year)",
                    value: property_data.read().maintenance_percent,
                    onchange: move |v| {
                        let mut data = property_data.write();
                        data.maintenance_percent = v;
                    },
                    tooltip: "Annual maintenance cost as percentage of home value (typically 1%)",
                    min: Some(0.0),
                    max: Some(10.0),
                    max_exclusive: true,
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

                // HOA Fee Increase
                InputField {
                    label: "HOA Fee Increase (%/yr)",
                    value: property_data.read().hoa_fee_increase_rate,
                    onchange: move |v| {
                        let mut data = property_data.write();
                        data.hoa_fee_increase_rate = v;
                    },
                    tooltip: "Expected annual increase rate for HOA fees (typically 3%)",
                    min: Some(0.0),
                    max: Some(50.0),
                    max_exclusive: true,
                    dark_mode: dark_mode,
                }

                // Monthly PMI
                InputField {
                    label: "PMI ($/mo)",
                    value: property_data.read().monthly_pmi,
                    onchange: move |v| {
                        let mut data = property_data.write();
                        data.monthly_pmi = v;
                    },
                    tooltip: "Monthly Private Mortgage Insurance (required if down payment < 20%)",
                    min: Some(0.0),
                    dark_mode: dark_mode,
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

                // Purchase Closing Costs
                InputField {
                    label: "Purchase Closing Costs (%)",
                    value: property_data.read().closing_costs_percent_purchase,
                    onchange: move |v| {
                        let mut data = property_data.write();
                        data.closing_costs_percent_purchase = v;
                    },
                    tooltip: "Closing costs at purchase as percentage of home price (typically 3-5%)",
                    min: Some(0.0),
                    max: Some(20.0),
                    dark_mode: dark_mode,
                }

                // Sale Closing Costs
                InputField {
                    label: "Sale Closing Costs (%)",
                    value: property_data.read().closing_costs_percent_sale,
                    onchange: move |v| {
                        let mut data = property_data.write();
                        data.closing_costs_percent_sale = v;
                    },
                    tooltip: "Closing costs when selling as percentage of sale price (typically 6%)",
                    min: Some(0.0),
                    max: Some(20.0),
                    dark_mode: dark_mode,
                }

                // Home Appreciation Rate
                InputField {
                    label: "Appreciation Rate (%/year)",
                    value: property_data.read().home_appreciation_rate,
                    onchange: move |v| {
                        let mut data = property_data.write();
                        data.home_appreciation_rate = v;
                    },
                    tooltip: "Expected annual home value appreciation rate",
                    min: Some(-10.0),
                    max: Some(50.0),
                    max_exclusive: true,
                    dark_mode: dark_mode,
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
                                option { value: "single", "Individual return ($15,750 std deduction)" }
                                option { value: "joint", "Joint return ($31,500 std deduction)" }
                            }
                        }

                        // Marginal Tax Rate
                        InputField {
                            label: "Marginal Tax Rate (%)",
                            value: property_data.read().marginal_tax_rate,
                            onchange: move |v| {
                                let mut data = property_data.write();
                                data.marginal_tax_rate = v;
                            },
                            tooltip: "Your marginal federal income tax rate (the rate on your last dollar of income)",
                            min: Some(0.0),
                            max: Some(50.0),
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

                        // HOA/Common Fees Deduction
                        InputField {
                            label: "HOA Fees Deduction (%)",
                            value: property_data.read().hoa_deduction_percent,
                            onchange: move |v| {
                                let mut data = property_data.write();
                                data.hoa_deduction_percent = v;
                            },
                            tooltip: "Percentage of HOA fees that are tax deductible (usually 0% for primary residence, may be higher for home office)",
                            min: Some(0.0),
                            max: Some(100.0),
                            dark_mode: dark_mode,
                        }

                        // Info about SALT cap
                        p { class: if dark_mode { "text-xs text-monokai-fgDim italic" } else { "text-xs text-monokaiLight-fgDim italic" },
                            "Note: Property tax deduction is capped at $10,000 (SALT limit). Mortgage interest is fully deductible up to $750k loan."
                        }
                    }
                }
            }
        }
    }
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
    #[props(default = false)] dark_mode: bool,
) -> Element {
    // Validate the value
    let is_valid = {
        let mut valid = true;
        if let Some(min_val) = min {
            if min_exclusive {
                if value <= min_val {
                    valid = false;
                }
            } else {
                if value < min_val {
                    valid = false;
                }
            }
        }
        if let Some(max_val) = max {
            if max_exclusive {
                if value >= max_val {
                    valid = false;
                }
            } else {
                if value > max_val {
                    valid = false;
                }
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
                r#type: "number",
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
                value: "{value}",
                step: "0.01",
                oninput: move |evt| {
                    if let Ok(v) = evt.value().parse::<f64>() {
                        onchange.call(v);
                    }
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
    let mut is_fetching = use_signal(|| false);
    let mut rate_info = use_signal(|| None::<String>);

    let fetch_rate = move |_| {
        spawn(async move {
            is_fetching.set(true);
            rate_info.set(None);

            let result = fetch_current_mortgage_rate().await;

            let mut data = property_data.write();
            data.interest_rate = result.rate;

            if result.is_live {
                rate_info.set(Some(format!(
                    "Updated to {}% (as of {})",
                    result.rate, result.date
                )));
            } else {
                rate_info.set(Some(format!("Using fallback rate: {}%", result.rate)));
            }

            is_fetching.set(false);
        });
    };

    let value = property_data.read().interest_rate;
    let is_valid = value > 0.0 && value < 20.0;

    rsx! {
        div {
            label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-1" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-1" },
                "Interest Rate (%)"
            }
            div { class: "flex gap-2",
                input {
                    r#type: "number",
                    class: if dark_mode {
                        if is_valid {
                            "flex-1 px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                        } else {
                            "flex-1 px-3 py-2 border border-monokai-red bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-red focus:border-monokai-red"
                        }
                    } else {
                        if is_valid {
                            "flex-1 px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                        } else {
                            "flex-1 px-3 py-2 border border-monokaiLight-red bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-red focus:border-monokaiLight-red"
                        }
                    },
                    value: "{value}",
                    step: "0.01",
                    oninput: move |evt| {
                        if let Ok(v) = evt.value().parse::<f64>() {
                            let mut data = property_data.write();
                            data.interest_rate = v;
                        }
                    },
                    title: "Annual mortgage interest rate"
                }
                button {
                    r#type: "button",
                    class: if dark_mode {
                        "px-3 py-2 bg-monokai-blue text-monokai-bg text-sm font-medium rounded-md hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap"
                    } else {
                        "px-3 py-2 bg-monokaiLight-blue text-white text-sm font-medium rounded-md hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap"
                    },
                    disabled: is_fetching(),
                    onclick: fetch_rate,
                    title: "Fetch current 30-year fixed rate from FRED",
                    if is_fetching() {
                        "Fetching..."
                    } else {
                        "Live Rate"
                    }
                }
            }
            if let Some(info) = rate_info.read().as_ref() {
                p { class: if dark_mode { "text-xs text-monokai-green mt-1" } else { "text-xs text-monokaiLight-green mt-1" },
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
