use super::{ChartsGrid, YearByYearTable};
use crate::models::FinancialResults;
use crate::utils::CurrencyFormatter;
use dioxus::prelude::*;

#[component]
pub fn ResultsPanel(
    results: FinancialResults,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    let last_year_idx = results.buy_total_wealth.len() - 1;
    let buy_final_wealth = results.buy_total_wealth[last_year_idx];
    let rent_final_wealth = results.rent_total_wealth[last_year_idx];
    let winner = results.get_winner();
    let buying_wins = buy_final_wealth > rent_final_wealth;
    let wealth_diff = (buy_final_wealth - rent_final_wealth).abs();

    // Calculate progress bar percentage (how much of total wealth belongs to winner)
    let total_wealth = buy_final_wealth + rent_final_wealth;
    let buy_percentage = if total_wealth > 0.0 {
        (buy_final_wealth / total_wealth) * 100.0
    } else {
        50.0
    };

    rsx! {
        div { class: "space-y-6",
            // Summary Card - Monokai themed
            div { class: if dark_mode { "bg-monokai-bgLight text-monokai-fg rounded-lg shadow-lg p-8 animate-fade-in-up" } else { "bg-monokaiLight-green text-white rounded-lg shadow-lg p-8 animate-fade-in-up" },
                h2 { class: "text-3xl font-bold mb-4 flex items-center gap-3",
                    i { class: "fa-solid fa-chart-line" }
                    "Analysis Results"
                }

                div { class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                    // Winner - conditional coloring
                    div { class: if buying_wins {
                        if dark_mode { "bg-monokai-green bg-opacity-20 border border-monokai-green rounded-lg p-4 animate-fade-in-scale animate-delay-100" }
                        else { "bg-white bg-opacity-30 border border-white rounded-lg p-4 animate-fade-in-scale animate-delay-100" }
                    } else {
                        if dark_mode { "bg-monokai-red bg-opacity-20 border border-monokai-red rounded-lg p-4 animate-fade-in-scale animate-delay-100" }
                        else { "bg-white bg-opacity-30 border border-white rounded-lg p-4 animate-fade-in-scale animate-delay-100" }
                    },
                        p { class: "text-sm font-medium opacity-90 flex items-center gap-2",
                            i { class: "fa-solid fa-trophy" }
                            "Recommendation"
                        }
                        p { class: "text-2xl font-bold mt-1", "{winner}" }
                    }

                    // Buy Wealth
                    div { class: if dark_mode { "bg-monokai-bgLighter rounded-lg p-4 animate-fade-in-scale animate-delay-200" } else { "bg-white bg-opacity-20 rounded-lg p-4 animate-fade-in-scale animate-delay-200" },
                        p { class: "text-sm font-medium opacity-90 flex items-center gap-2",
                            i { class: "fa-solid fa-house" }
                            "Buy Total Wealth (Year {last_year_idx + 1})"
                        }
                        p { class: "text-2xl font-bold mt-1",
                            "{CurrencyFormatter::format_currency(buy_final_wealth)}"
                        }
                    }

                    // Rent Wealth
                    div { class: if dark_mode { "bg-monokai-bgLighter rounded-lg p-4 animate-fade-in-scale animate-delay-300" } else { "bg-white bg-opacity-20 rounded-lg p-4 animate-fade-in-scale animate-delay-300" },
                        p { class: "text-sm font-medium opacity-90 flex items-center gap-2",
                            i { class: "fa-solid fa-building" }
                            "Rent Total Wealth (Year {last_year_idx + 1})"
                        }
                        p { class: "text-2xl font-bold mt-1",
                            "{CurrencyFormatter::format_currency(rent_final_wealth)}"
                        }
                    }
                }

                // Progress bar showing wealth comparison
                div { class: "mt-6 animate-fade-in-scale animate-delay-400",
                    div { class: "flex justify-between text-sm mb-2",
                        span { class: "flex items-center gap-1",
                            i { class: "fa-solid fa-house text-xs" }
                            "Buy"
                        }
                        span { class: "flex items-center gap-1",
                            "Rent"
                            i { class: "fa-solid fa-building text-xs" }
                        }
                    }
                    div { class: if dark_mode { "w-full bg-monokai-bgHighlight rounded-full h-4 overflow-hidden" } else { "w-full bg-white bg-opacity-30 rounded-full h-4 overflow-hidden" },
                        div {
                            class: if buying_wins {
                                if dark_mode { "bg-monokai-green h-4 rounded-l-full transition-all duration-1000" }
                                else { "bg-white h-4 rounded-l-full transition-all duration-1000" }
                            } else {
                                if dark_mode { "bg-monokai-orange h-4 rounded-l-full transition-all duration-1000" }
                                else { "bg-white bg-opacity-60 h-4 rounded-l-full transition-all duration-1000" }
                            },
                            style: "width: {buy_percentage:.1}%",
                        }
                    }
                    div { class: "flex justify-between text-xs mt-1 opacity-80",
                        span { "{buy_percentage:.1}%" }
                        span { "{100.0 - buy_percentage:.1}%" }
                    }
                }

                // Break-even info
                div { class: if dark_mode { "mt-6 bg-monokai-bgLighter rounded-lg p-4 animate-fade-in-scale animate-delay-400" } else { "mt-6 bg-white bg-opacity-20 rounded-lg p-4 animate-fade-in-scale animate-delay-400" },
                    p { class: "text-sm font-medium opacity-90 flex items-center gap-2",
                        i { class: "fa-solid fa-scale-balanced" }
                        "Break-Even Point"
                    }
                    p { class: "text-lg font-semibold mt-1", "{results.break_even_description}" }
                }

                // Wealth advantage callout
                div { class: if buying_wins {
                    if dark_mode { "mt-4 bg-monokai-green bg-opacity-20 border border-monokai-green rounded-lg p-4 animate-fade-in-scale animate-delay-400" }
                    else { "mt-4 bg-white bg-opacity-30 border border-white rounded-lg p-4 animate-fade-in-scale animate-delay-400" }
                } else {
                    if dark_mode { "mt-4 bg-monokai-orange bg-opacity-20 border border-monokai-orange rounded-lg p-4 animate-fade-in-scale animate-delay-400" }
                    else { "mt-4 bg-white bg-opacity-30 border border-white rounded-lg p-4 animate-fade-in-scale animate-delay-400" }
                },
                    p { class: "text-sm font-medium opacity-90 flex items-center gap-2",
                        i { class: "fa-solid fa-coins" }
                        "Wealth Advantage"
                    }
                    p { class: "text-xl font-bold mt-1",
                        if buying_wins {
                            "Buying saves you {CurrencyFormatter::format_currency(wealth_diff)}"
                        } else {
                            "Renting saves you {CurrencyFormatter::format_currency(wealth_diff)}"
                        }
                    }
                }
            }

            // Key Metrics with icons
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4",
                MetricCard {
                    label: "Monthly Mortgage",
                    value: CurrencyFormatter::format_currency(results.monthly_mortgage_payment),
                    icon: "fa-solid fa-money-bill-wave",
                    dark_mode: dark_mode,
                }
                MetricCard {
                    label: "Home Equity (Year {last_year_idx + 1})",
                    value: CurrencyFormatter::format_currency(results.home_equity[last_year_idx]),
                    icon: "fa-solid fa-piggy-bank",
                    dark_mode: dark_mode,
                }
                MetricCard {
                    label: "Home Value (Year {last_year_idx + 1})",
                    value: CurrencyFormatter::format_currency(results.home_value[last_year_idx]),
                    icon: "fa-solid fa-house-chimney",
                    dark_mode: dark_mode,
                }
                MetricCard {
                    label: "Analysis Period",
                    value: format!("{} years", last_year_idx + 1),
                    icon: "fa-solid fa-calendar",
                    dark_mode: dark_mode,
                }
            }

            // Charts
            ChartsGrid { results: results.clone(), dark_mode: dark_mode }

            // Year-by-year breakdown
            YearByYearTable { results: results.clone(), dark_mode: dark_mode }

            // Back button (optional)
            div { class: "text-center pt-6",
                p { class: if dark_mode { "text-sm text-monokai-fgMuted" } else { "text-sm text-monokaiLight-fgMuted" },
                    "Modify inputs in the Input tab and recalculate to see updated results"
                }
            }
        }
    }
}

#[component]
fn MetricCard(
    label: String,
    value: String,
    icon: String,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    rsx! {
        div { class: if dark_mode { "bg-monokai-bgLight rounded-lg shadow p-4" } else { "bg-monokaiLight-bgLight rounded-lg shadow p-4" },
            p { class: if dark_mode { "text-sm text-monokai-fgMuted flex items-center gap-2" } else { "text-sm text-monokaiLight-fgMuted flex items-center gap-2" },
                i { class: "{icon}" }
                "{label}"
            }
            p { class: if dark_mode { "text-lg font-semibold text-monokai-fg mt-1" } else { "text-lg font-semibold text-monokaiLight-fg mt-1" }, "{value}" }
        }
    }
}
