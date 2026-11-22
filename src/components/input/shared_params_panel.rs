use crate::models::InvestmentParameters;
use crate::utils::AppConfig;
use dioxus::prelude::*;

#[component]
pub fn SharedParamsPanel(
    investment_params: Signal<InvestmentParameters>,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    let return_rate = investment_params.read().annual_return_rate;
    let years = investment_params.read().analysis_years;
    let inflation_rate = investment_params.read().inflation_rate;
    let capital_gains_rate = investment_params.read().capital_gains_tax_rate;

    let mut return_rate_focused = use_signal(|| false);
    let mut return_rate_text = use_signal(String::new);
    let mut inflation_focused = use_signal(|| false);
    let mut inflation_text = use_signal(String::new);
    let mut cg_focused = use_signal(|| false);
    let mut cg_text = use_signal(String::new);

    // Validate return rate (reasonable range: -50% to 50%)
    let return_rate_valid = (-50.0..50.0).contains(&return_rate);

    // Validate years (1-30)
    let years_valid =
        (AppConfig::MIN_ANALYSIS_YEARS..=AppConfig::MAX_ANALYSIS_YEARS).contains(&years);

    // Validate inflation rate (reasonable range: -10% to 20%)
    let inflation_valid = (-10.0..=20.0).contains(&inflation_rate);

    // Validate capital gains tax rate (0% to 50%)
    let cg_valid = (0.0..=50.0).contains(&capital_gains_rate);

    rsx! {
        div { class: if dark_mode { "bg-monokai-bgLight rounded-lg shadow p-6" } else { "bg-monokaiLight-bgLight rounded-lg shadow p-6" },
            h2 { class: if dark_mode { "text-xl font-semibold text-monokai-fg mb-4" } else { "text-xl font-semibold text-monokaiLight-fg mb-4" },
                "Shared Parameters"
            }
            p { class: if dark_mode { "text-sm text-monokai-fgMuted mb-6" } else { "text-sm text-monokaiLight-fgMuted mb-6" },
                "Parameters that affect both buying and renting scenarios"
            }

            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4",
                // Investment Return Rate
                div {
                    label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-1" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-1" },
                        "Investment Return Rate (%/year)"
                    }
                    input {
                        r#type: "text",
                        class: if dark_mode {
                            if return_rate_valid {
                                "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                            } else {
                                "w-full px-3 py-2 border border-monokai-red bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-red focus:border-monokai-red"
                            }
                        } else {
                            if return_rate_valid {
                                "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                            } else {
                                "w-full px-3 py-2 border border-monokaiLight-red bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-red focus:border-monokaiLight-red"
                            }
                        },
                        value: if return_rate_focused() {
                            return_rate_text()
                        } else {
                            format!("{:.2}%", return_rate)
                        },
                        inputmode: "decimal",
                        onfocus: move |_| {
                            return_rate_focused.set(true);
                            return_rate_text.set(return_rate.to_string());
                        },
                        onblur: move |_| {
                            return_rate_focused.set(false);
                            if let Some(v) = crate::utils::CurrencyFormatter::parse_input(&return_rate_text()) {
                                let mut params = investment_params.write();
                                params.annual_return_rate = v;
                            }
                        },
                        oninput: move |evt| {
                            return_rate_text.set(evt.value());
                        },
                        title: "Expected annual return on investments (e.g., 7% for stock market)"
                    }
                    if !return_rate_valid {
                        p { class: if dark_mode { "text-xs text-monokai-red mt-1" } else { "text-xs text-monokaiLight-red mt-1" },
                            "Valid range: >= -50 and < 50"
                        }
                    } else {
                        p { class: if dark_mode { "text-xs text-monokai-fgDim mt-1" } else { "text-xs text-monokaiLight-fgDim mt-1" },
                            "Valid range: >= -50 and < 50"
                        }
                    }
                }

                // Analysis Years
                div {
                    label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-1" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-1" },
                        "Analysis Period (years)"
                    }
                    input {
                        r#type: "number",
                        class: if dark_mode {
                            if years_valid {
                                "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                            } else {
                                "w-full px-3 py-2 border border-monokai-red bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-red focus:border-monokai-red"
                            }
                        } else {
                            if years_valid {
                                "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                            } else {
                                "w-full px-3 py-2 border border-monokaiLight-red bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-red focus:border-monokaiLight-red"
                            }
                        },
                        value: "{years}",
                        min: "1",
                        max: "30",
                        oninput: move |evt| {
                            if let Ok(v) = evt.value().parse::<usize>() {
                                let mut params = investment_params.write();
                                params.analysis_years = v.clamp(AppConfig::MIN_ANALYSIS_YEARS, AppConfig::MAX_ANALYSIS_YEARS);
                            }
                        },
                        title: "Number of years to analyze (1-30)"
                    }
                    if !years_valid {
                        p { class: if dark_mode { "text-xs text-monokai-red mt-1" } else { "text-xs text-monokaiLight-red mt-1" },
                            "Valid range: >= 1 and <= 30"
                        }
                    } else {
                        p { class: if dark_mode { "text-xs text-monokai-fgDim mt-1" } else { "text-xs text-monokaiLight-fgDim mt-1" },
                            "Valid range: >= 1 and <= 30"
                        }
                    }
                }

                // Inflation Rate
                div {
                    label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-1" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-1" },
                        "Inflation Rate (%/year)"
                    }
                    input {
                        r#type: "text",
                        class: if dark_mode {
                            if inflation_valid {
                                "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                            } else {
                                "w-full px-3 py-2 border border-monokai-red bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-red focus:border-monokai-red"
                            }
                        } else {
                            if inflation_valid {
                                "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                            } else {
                                "w-full px-3 py-2 border border-monokaiLight-red bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-red focus:border-monokaiLight-red"
                            }
                        },
                        value: if inflation_focused() {
                            inflation_text()
                        } else {
                            format!("{:.2}%", inflation_rate)
                        },
                        inputmode: "decimal",
                        onfocus: move |_| {
                            inflation_focused.set(true);
                            inflation_text.set(inflation_rate.to_string());
                        },
                        onblur: move |_| {
                            inflation_focused.set(false);
                            if let Some(v) = crate::utils::CurrencyFormatter::parse_input(&inflation_text()) {
                                let mut params = investment_params.write();
                                params.inflation_rate = v;
                            }
                        },
                        oninput: move |evt| {
                            inflation_text.set(evt.value());
                        },
                        title: "Expected annual inflation rate (3% is 10-year US average)"
                    }
                    if !inflation_valid {
                        p { class: if dark_mode { "text-xs text-monokai-red mt-1" } else { "text-xs text-monokaiLight-red mt-1" },
                            "Valid range: >= -10 and <= 20"
                        }
                    } else {
                        p { class: if dark_mode { "text-xs text-monokai-fgDim mt-1" } else { "text-xs text-monokaiLight-fgDim mt-1" },
                            "Default 3% (10-year US avg)"
                        }
                    }
                }

                // Capital Gains Tax Rate
                div {
                    label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-1" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-1" },
                        "Capital Gains Tax Rate (%)"
                    }
                    input {
                        r#type: "text",
                        class: if dark_mode {
                            if cg_valid {
                                "w-full px-3 py-2 border border-monokai-border bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-blue focus:border-monokai-blue"
                            } else {
                                "w-full px-3 py-2 border border-monokai-red bg-monokai-bgLighter text-monokai-fg rounded-md shadow-sm focus:ring-monokai-red focus:border-monokai-red"
                            }
                        } else {
                            if cg_valid {
                                "w-full px-3 py-2 border border-monokaiLight-border bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-blue focus:border-monokaiLight-blue"
                            } else {
                                "w-full px-3 py-2 border border-monokaiLight-red bg-monokaiLight-bg text-monokaiLight-fg rounded-md shadow-sm focus:ring-monokaiLight-red focus:border-monokaiLight-red"
                            }
                        },
                        value: if cg_focused() {
                            cg_text()
                        } else {
                            format!("{:.2}%", capital_gains_rate)
                        },
                        inputmode: "decimal",
                        onfocus: move |_| {
                            cg_focused.set(true);
                            cg_text.set(capital_gains_rate.to_string());
                        },
                        onblur: move |_| {
                            cg_focused.set(false);
                            if let Some(v) = crate::utils::CurrencyFormatter::parse_input(&cg_text()) {
                                let mut params = investment_params.write();
                                params.capital_gains_tax_rate = v;
                            }
                        },
                        oninput: move |evt| {
                            cg_text.set(evt.value());
                        },
                        title: "Combined long-term capital gains rate (federal 0/15/20%, + 3.8% NIIT for high earners, + state; DC taxes gains as income at ~6-10%). Applied to investment gains when cashed out, and to home-sale gains above the $250k/$500k exclusion. Set 0 to ignore."
                    }
                    if !cg_valid {
                        p { class: if dark_mode { "text-xs text-monokai-red mt-1" } else { "text-xs text-monokaiLight-red mt-1" },
                            "Valid range: >= 0 and <= 50"
                        }
                    } else {
                        p { class: if dark_mode { "text-xs text-monokai-fgDim mt-1" } else { "text-xs text-monokaiLight-fgDim mt-1" },
                            "Default 15% federal; add your state rate"
                        }
                    }
                }
            }
        }
    }
}
