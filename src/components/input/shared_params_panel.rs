use crate::models::InvestmentParameters;
use dioxus::prelude::*;

#[component]
pub fn SharedParamsPanel(
    investment_params: Signal<InvestmentParameters>,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    let return_rate = investment_params.read().annual_return_rate;
    let years = investment_params.read().analysis_years;
    let inflation_rate = investment_params.read().inflation_rate;

    // Validate return rate (reasonable range: -50% to 50%)
    let return_rate_valid = return_rate >= -50.0 && return_rate < 50.0;

    // Validate years (1-30)
    let years_valid = years >= 1 && years <= 30;

    // Validate inflation rate (reasonable range: -10% to 20%)
    let inflation_valid = inflation_rate >= -10.0 && inflation_rate <= 20.0;

    rsx! {
        div { class: if dark_mode { "bg-monokai-bgLight rounded-lg shadow p-6" } else { "bg-monokaiLight-bgLight rounded-lg shadow p-6" },
            h2 { class: if dark_mode { "text-xl font-semibold text-monokai-fg mb-4" } else { "text-xl font-semibold text-monokaiLight-fg mb-4" },
                "Shared Parameters"
            }
            p { class: if dark_mode { "text-sm text-monokai-fgMuted mb-6" } else { "text-sm text-monokaiLight-fgMuted mb-6" },
                "Parameters that affect both buying and renting scenarios"
            }

            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                // Investment Return Rate
                div {
                    label { class: if dark_mode { "block text-sm font-medium text-monokai-fgMuted mb-1" } else { "block text-sm font-medium text-monokaiLight-fgMuted mb-1" },
                        "Investment Return Rate (%/year)"
                    }
                    input {
                        r#type: "number",
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
                        value: "{return_rate}",
                        step: "0.1",
                        oninput: move |evt| {
                            if let Ok(v) = evt.value().parse::<f64>() {
                                let mut params = investment_params.write();
                                params.annual_return_rate = v;
                            }
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
                                params.analysis_years = v.clamp(1, 30);
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
                        r#type: "number",
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
                        value: "{inflation_rate}",
                        step: "0.1",
                        oninput: move |evt| {
                            if let Ok(v) = evt.value().parse::<f64>() {
                                let mut params = investment_params.write();
                                params.inflation_rate = v;
                            }
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
            }
        }
    }
}
