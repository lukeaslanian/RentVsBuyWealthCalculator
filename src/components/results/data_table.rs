use crate::models::FinancialResults;
use crate::utils::CurrencyFormatter;
use dioxus::prelude::*;

#[component]
pub fn YearByYearTable(
    results: FinancialResults,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    let years = results.buy_total_wealth.len();

    rsx! {
        div { class: if dark_mode { "bg-gray-800 rounded-lg shadow overflow-hidden" } else { "bg-white rounded-lg shadow overflow-hidden" },
            div { class: if dark_mode { "px-6 py-4 border-b border-gray-700" } else { "px-6 py-4 border-b border-gray-200" },
                h3 { class: if dark_mode { "text-lg font-semibold text-white" } else { "text-lg font-semibold text-gray-900" },
                    "Year-by-Year Breakdown"
                }
                p { class: if dark_mode { "text-sm text-gray-400 mt-1" } else { "text-sm text-gray-600 mt-1" },
                    "Detailed financial projections over {years} years"
                }
            }

            div { class: "overflow-x-auto",
                table { class: if dark_mode { "min-w-full divide-y divide-gray-700" } else { "min-w-full divide-y divide-gray-200" },
                    thead { class: if dark_mode { "bg-gray-700" } else { "bg-gray-50" },
                        tr {
                            th { class: if dark_mode { "px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider" } else { "px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" }, "Year" }
                            th { class: if dark_mode { "px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider" } else { "px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" }, "Home Value" }
                            th { class: if dark_mode { "px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider" } else { "px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" }, "Equity" }
                            th { class: if dark_mode { "px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider" } else { "px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" }, "Buy Portfolio" }
                            th { class: if dark_mode { "px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider" } else { "px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" }, "Rent Portfolio" }
                            th { class: if dark_mode { "px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider" } else { "px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" }, "Buy Wealth" }
                            th { class: if dark_mode { "px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider" } else { "px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" }, "Rent Wealth" }
                            th { class: if dark_mode { "px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider" } else { "px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider" }, "Advantage" }
                        }
                    }
                    tbody { class: if dark_mode { "bg-gray-800 divide-y divide-gray-700" } else { "bg-white divide-y divide-gray-200" },
                        for year in 0..years {
                            {
                                let buy_wealth = results.buy_total_wealth[year];
                                let rent_wealth = results.rent_total_wealth[year];
                                let advantage = buy_wealth - rent_wealth;
                                let advantage_class = if advantage > 0.0 {
                                    "text-green-600 font-medium"
                                } else if advantage < 0.0 {
                                    "text-red-600 font-medium"
                                } else {
                                    if dark_mode { "text-gray-300" } else { "text-gray-900" }
                                };
                                let text_class = if dark_mode { "text-gray-300" } else { "text-gray-900" };

                                rsx! {
                                    tr { key: "{year}",
                                        td { class: "px-4 py-3 whitespace-nowrap text-sm {text_class}", "{year + 1}" }
                                        td { class: "px-4 py-3 whitespace-nowrap text-sm {text_class}",
                                            "{CurrencyFormatter::format_currency(results.home_value[year])}"
                                        }
                                        td { class: "px-4 py-3 whitespace-nowrap text-sm {text_class}",
                                            "{CurrencyFormatter::format_currency(results.home_equity[year])}"
                                        }
                                        td { class: "px-4 py-3 whitespace-nowrap text-sm {text_class}",
                                            "{CurrencyFormatter::format_currency(results.buy_investment_portfolio[year])}"
                                        }
                                        td { class: "px-4 py-3 whitespace-nowrap text-sm {text_class}",
                                            "{CurrencyFormatter::format_currency(results.rent_investment_portfolio[year])}"
                                        }
                                        td { class: "px-4 py-3 whitespace-nowrap text-sm {text_class} font-medium",
                                            "{CurrencyFormatter::format_currency(buy_wealth)}"
                                        }
                                        td { class: "px-4 py-3 whitespace-nowrap text-sm {text_class} font-medium",
                                            "{CurrencyFormatter::format_currency(rent_wealth)}"
                                        }
                                        td { class: "px-4 py-3 whitespace-nowrap text-sm {advantage_class}",
                                            "{CurrencyFormatter::format_currency(advantage)}"
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
}
