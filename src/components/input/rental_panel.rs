use crate::models::RentalData;
use dioxus::prelude::*;

#[component]
pub fn RentalInputPanel(
    rental_data: Signal<RentalData>,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    rsx! {
        div { class: if dark_mode { "bg-monokai-bgLight rounded-lg shadow p-6" } else { "bg-monokaiLight-bgLight rounded-lg shadow p-6" },
            h2 { class: if dark_mode { "text-xl font-semibold text-monokai-fg mb-4" } else { "text-xl font-semibold text-monokaiLight-fg mb-4" },
                "Renting Scenario"
            }
            p { class: if dark_mode { "text-sm text-monokai-fgMuted mb-6 italic" } else { "text-sm text-monokaiLight-fgMuted mb-6 italic" },
                "Renter invests down payment savings immediately. Any monthly savings vs owning are invested."
            }

            div { class: "space-y-4",
                // Monthly Rent
                InputField {
                    label: "Monthly Rent ($)",
                    value: rental_data.read().monthly_rent,
                    onchange: move |v| {
                        let mut data = rental_data.write();
                        data.monthly_rent = v;
                    },
                    tooltip: "Monthly rent payment",
                    min: Some(0.0),
                    max: Some(50_000.0),
                    min_exclusive: true,
                    max_exclusive: true,
                    dark_mode: dark_mode,
                }

                // Amenity Fees
                InputField {
                    label: "Amenity Fees ($/mo)",
                    value: rental_data.read().amenity_fees,
                    onchange: move |v| {
                        let mut data = rental_data.write();
                        data.amenity_fees = v;
                    },
                    tooltip: "Monthly amenity fees (parking, gym, etc.)",
                    min: Some(0.0),
                    dark_mode: dark_mode,
                }

                // Non-included Utilities
                InputField {
                    label: "Non-Included Utilities ($/mo)",
                    value: rental_data.read().rent_non_included_utilities,
                    onchange: move |v| {
                        let mut data = rental_data.write();
                        data.rent_non_included_utilities = v;
                    },
                    tooltip: "Monthly utilities you pay separately",
                    min: Some(0.0),
                    dark_mode: dark_mode,
                }

                // Renters Insurance
                InputField {
                    label: "Renters Insurance ($/mo)",
                    value: rental_data.read().renters_insurance,
                    onchange: move |v| {
                        let mut data = rental_data.write();
                        data.renters_insurance = v;
                    },
                    tooltip: "Monthly renters insurance",
                    min: Some(0.0),
                    dark_mode: dark_mode,
                }

                // Rent Increase Rate
                InputField {
                    label: "Rent Increase Rate (%/year)",
                    value: rental_data.read().rent_increase_rate,
                    onchange: move |v| {
                        let mut data = rental_data.write();
                        data.rent_increase_rate = v;
                    },
                    tooltip: "Expected annual rent increase (e.g., 4%)",
                    min: Some(0.0),
                    max: Some(50.0),
                    max_exclusive: true,
                    dark_mode: dark_mode,
                }

                // Security Deposit
                InputField {
                    label: "Security Deposit ($)",
                    value: rental_data.read().security_deposit,
                    onchange: move |v| {
                        let mut data = rental_data.write();
                        data.security_deposit = v;
                    },
                    tooltip: "Upfront security deposit (typically 1 month's rent)",
                    min: Some(0.0),
                    dark_mode: dark_mode,
                }

                // Broker Fee Section
                div { class: if dark_mode { "border-t border-monokai-bgLighter pt-4 mt-4" } else { "border-t border-monokaiLight-border pt-4 mt-4" },
                    // Broker Fee Toggle
                    div { class: "flex items-center justify-between mb-3",
                        div {
                            label { class: if dark_mode { "text-sm font-medium text-monokai-fg" } else { "text-sm font-medium text-monokaiLight-fg" },
                                "Broker's Fee"
                            }
                            p { class: if dark_mode { "text-xs text-monokai-fgMuted" } else { "text-xs text-monokaiLight-fgMuted" },
                                "Common in NYC and some other markets"
                            }
                        }
                        button {
                            class: if rental_data.read().enable_broker_fee {
                                "relative inline-flex h-6 w-11 items-center rounded-full bg-monokai-purple transition-colors"
                            } else {
                                if dark_mode {
                                    "relative inline-flex h-6 w-11 items-center rounded-full bg-monokai-bgLighter transition-colors"
                                } else {
                                    "relative inline-flex h-6 w-11 items-center rounded-full bg-monokaiLight-border transition-colors"
                                }
                            },
                            onclick: move |_| {
                                let mut data = rental_data.write();
                                data.enable_broker_fee = !data.enable_broker_fee;
                            },
                            span {
                                class: if rental_data.read().enable_broker_fee {
                                    "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-6"
                                } else {
                                    "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-1"
                                },
                            }
                        }
                    }

                    // Broker Fee Percentage (only shown when enabled)
                    if rental_data.read().enable_broker_fee {
                        div { class: if dark_mode { "bg-monokai-bgLighter rounded-lg p-4 space-y-3" } else { "bg-monokaiLight-bg rounded-lg p-4 space-y-3" },
                            InputField {
                                label: "Broker Fee (% of annual rent)",
                                value: rental_data.read().broker_fee_percent,
                                onchange: move |v| {
                                    let mut data = rental_data.write();
                                    data.broker_fee_percent = v;
                                },
                                tooltip: "Broker's fee as percentage of annual rent (e.g., 15%)",
                                min: Some(0.0),
                                max: Some(100.0),
                                dark_mode: dark_mode,
                            }

                            // Show calculated broker fee amount
                            {
                                let fee = rental_data.read().broker_fee_amount();
                                rsx! {
                                    div { class: if dark_mode { "text-xs text-monokai-fgMuted" } else { "text-xs text-monokaiLight-fgMuted" },
                                        "Broker fee amount: ${fee:.0}"
                                    }
                                }
                            }
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
