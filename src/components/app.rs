use super::input::*;
use super::results::*;
use super::MonteCarloPanel;
use crate::calculators::WealthAnalysisEngine;
use crate::models::{FinancialResults, InvestmentParameters, PropertyData, RentalData};
use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    // State
    let mut property_data = use_signal(|| PropertyData::new());
    let mut rental_data = use_signal(|| RentalData::new());
    let mut investment_params = use_signal(|| InvestmentParameters::new());
    let mut results = use_signal(|| None::<FinancialResults>);
    let mut active_tab = use_signal(|| 0);
    let mut error_message = use_signal(|| None::<String>);
    let mut dark_mode = use_signal(|| true); // Dark mode enabled by default

    // Calculate handler
    let calculate = move |_| {
        let prop = property_data.read();
        let rent = rental_data.read();

        // Validate inputs
        if !prop.is_valid() {
            error_message.set(Some(
                "Invalid property data. Please check all fields.".to_string(),
            ));
            return;
        }
        if !rent.is_valid() {
            error_message.set(Some(
                "Invalid rental data. Please check all fields.".to_string(),
            ));
            return;
        }

        error_message.set(None);

        // Run analysis
        let mut engine =
            WealthAnalysisEngine::new(prop.clone(), rent.clone(), investment_params.read().clone());
        engine.run_analysis();

        if let Some(calc_results) = engine.results() {
            results.set(Some(calc_results.clone()));
            active_tab.set(1); // Switch to results tab
        }
    };

    // Reset handler
    let reset = move |_| {
        property_data.set(PropertyData::new());
        rental_data.set(RentalData::new());
        investment_params.set(InvestmentParameters::new());
        results.set(None);
        active_tab.set(0);
        error_message.set(None);
    };

    rsx! {
        div {
            class: if dark_mode() { "min-h-screen bg-monokai-bg dark" } else { "min-h-screen bg-monokaiLight-bg" },
            // Header
            div { class: if dark_mode() { "bg-monokai-bgLight shadow-sm border-b border-monokai-border" } else { "bg-monokaiLight-bgLight shadow-sm border-b border-monokaiLight-border" },
                div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 flex justify-between items-center",
                    div {
                        h1 { class: if dark_mode() { "text-3xl font-bold text-monokai-fg" } else { "text-3xl font-bold text-monokaiLight-fg" },
                            "To Buy Or Not To Buy"
                        }
                        p { class: if dark_mode() { "mt-1 text-sm text-monokai-fgMuted" } else { "mt-1 text-sm text-monokaiLight-fgMuted" },
                            "Rent vs. Buy Calculator - Compare 30-year wealth outcomes"
                        }
                    }

                    // Dark mode toggle
                    button {
                        class: if dark_mode() {
                            "p-2 rounded-lg bg-monokai-bgLighter hover:bg-monokai-bgHighlight transition"
                        } else {
                            "p-2 rounded-lg bg-monokaiLight-bgDark hover:bg-monokaiLight-bgHighlight transition"
                        },
                        onclick: move |_| dark_mode.set(!dark_mode()),
                        title: if dark_mode() { "Switch to light mode" } else { "Switch to dark mode" },
                        if dark_mode() {
                            // Sun icon for light mode
                            svg {
                                class: "w-6 h-6 text-monokai-yellow",
                                xmlns: "http://www.w3.org/2000/svg",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke: "currentColor",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
                                }
                            }
                        } else {
                            // Moon icon for dark mode
                            svg {
                                class: "w-6 h-6 text-monokaiLight-purple",
                                xmlns: "http://www.w3.org/2000/svg",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke: "currentColor",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"
                                }
                            }
                        }
                    }
                }
            }

            // Error message
            if let Some(err) = error_message.read().as_ref() {
                div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 mt-4",
                    div { class: if dark_mode() { "bg-monokai-bgLighter border-l-4 border-monokai-red p-4 rounded" } else { "bg-monokaiLight-bgDark border-l-4 border-monokaiLight-red p-4 rounded" },
                        p { class: if dark_mode() { "text-sm text-monokai-red" } else { "text-sm text-monokaiLight-red" }, "{err}" }
                    }
                }
            }

            // Main content
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8",
                // Tabs - traditional style with background
                div { class: "mb-6",
                    nav { class: "flex",
                        button {
                            class: if active_tab() == 0 {
                                if dark_mode() {
                                    "px-6 py-3 text-sm font-semibold bg-monokai-green text-monokai-bg rounded-t-lg border-t-2 border-l-2 border-r-2 border-monokai-green"
                                } else {
                                    "px-6 py-3 text-sm font-semibold bg-monokaiLight-bgLight text-monokaiLight-green rounded-t-lg border-t-2 border-l-2 border-r-2 border-monokaiLight-green -mb-px"
                                }
                            } else {
                                if dark_mode() {
                                    "px-6 py-3 text-sm font-medium bg-monokai-bgLighter text-monokai-fgMuted rounded-t-lg border border-monokai-border hover:bg-monokai-bgHighlight hover:text-monokai-fg transition"
                                } else {
                                    "px-6 py-3 text-sm font-medium bg-monokaiLight-bgDark text-monokaiLight-fgMuted rounded-t-lg border border-monokaiLight-border hover:bg-monokaiLight-bgHighlight hover:text-monokaiLight-fg transition"
                                }
                            },
                            onclick: move |_| active_tab.set(0),
                            "Input"
                        }
                        button {
                            class: if active_tab() == 1 {
                                if dark_mode() {
                                    "px-6 py-3 text-sm font-semibold bg-monokai-green text-monokai-bg rounded-t-lg border-t-2 border-l-2 border-r-2 border-monokai-green ml-1"
                                } else {
                                    "px-6 py-3 text-sm font-semibold bg-monokaiLight-bgLight text-monokaiLight-green rounded-t-lg border-t-2 border-l-2 border-r-2 border-monokaiLight-green -mb-px ml-1"
                                }
                            } else {
                                if dark_mode() {
                                    "px-6 py-3 text-sm font-medium bg-monokai-bgLighter text-monokai-fgMuted rounded-t-lg border border-monokai-border hover:bg-monokai-bgHighlight hover:text-monokai-fg transition ml-1"
                                } else {
                                    "px-6 py-3 text-sm font-medium bg-monokaiLight-bgDark text-monokaiLight-fgMuted rounded-t-lg border border-monokaiLight-border hover:bg-monokaiLight-bgHighlight hover:text-monokaiLight-fg transition ml-1"
                                }
                            },
                            onclick: move |_| {
                                // Auto-calculate if no results yet
                                if results.read().is_none() {
                                    let prop = property_data.read();
                                    let rent = rental_data.read();

                                    if prop.is_valid() && rent.is_valid() {
                                        let mut engine = WealthAnalysisEngine::new(
                                            prop.clone(),
                                            rent.clone(),
                                            investment_params.read().clone()
                                        );
                                        engine.run_analysis();
                                        if let Some(calc_results) = engine.results() {
                                            results.set(Some(calc_results.clone()));
                                        }
                                    }
                                }
                                active_tab.set(1);
                            },
                            "Results"
                        }
                        button {
                            class: if active_tab() == 2 {
                                if dark_mode() {
                                    "px-6 py-3 text-sm font-semibold bg-monokai-green text-monokai-bg rounded-t-lg border-t-2 border-l-2 border-r-2 border-monokai-green ml-1"
                                } else {
                                    "px-6 py-3 text-sm font-semibold bg-monokaiLight-bgLight text-monokaiLight-green rounded-t-lg border-t-2 border-l-2 border-r-2 border-monokaiLight-green -mb-px ml-1"
                                }
                            } else {
                                if dark_mode() {
                                    "px-6 py-3 text-sm font-medium bg-monokai-bgLighter text-monokai-fgMuted rounded-t-lg border border-monokai-border hover:bg-monokai-bgHighlight hover:text-monokai-fg transition ml-1"
                                } else {
                                    "px-6 py-3 text-sm font-medium bg-monokaiLight-bgDark text-monokaiLight-fgMuted rounded-t-lg border border-monokaiLight-border hover:bg-monokaiLight-bgHighlight hover:text-monokaiLight-fg transition ml-1"
                                }
                            },
                            onclick: move |_| active_tab.set(2),
                            "Monte Carlo"
                        }
                    }
                    // Tab content border
                    div { class: if dark_mode() { "border-b-2 border-monokai-green" } else { "border-b-2 border-monokaiLight-green" } }
                }

                // Tab content
                match active_tab() {
                    0 => rsx! {
                        div { class: "space-y-6",
                            // City Selection
                            CitySelectionPanel {
                                property_data: property_data,
                                rental_data: rental_data,
                                investment_params: investment_params,
                                dark_mode: dark_mode(),
                            }

                            // Input panels grid
                            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                                BuyingInputPanel {
                                    property_data: property_data,
                                    dark_mode: dark_mode(),
                                }
                                RentalInputPanel {
                                    rental_data: rental_data,
                                    dark_mode: dark_mode(),
                                }
                            }

                            // Shared parameters
                            SharedParamsPanel {
                                investment_params: investment_params,
                                dark_mode: dark_mode(),
                            }

                            // Action buttons
                            div { class: if dark_mode() { "flex gap-4 pt-6 border-t border-monokai-border" } else { "flex gap-4 pt-6 border-t border-monokaiLight-border" },
                                button {
                                    class: if dark_mode() {
                                        "px-6 py-3 bg-monokai-blue text-monokai-bg font-semibold rounded-lg hover:opacity-90 shadow-sm"
                                    } else {
                                        "px-6 py-3 bg-monokaiLight-blue text-white font-semibold rounded-lg hover:opacity-90 shadow-sm"
                                    },
                                    onclick: calculate,
                                    "Calculate"
                                }
                                button {
                                    class: if dark_mode() {
                                        "px-6 py-3 bg-monokai-bgLighter text-monokai-fgMuted font-semibold rounded-lg hover:bg-monokai-bgHighlight hover:text-monokai-fg"
                                    } else {
                                        "px-6 py-3 bg-monokaiLight-bgDark text-monokaiLight-fgMuted font-semibold rounded-lg hover:bg-monokaiLight-bgHighlight hover:text-monokaiLight-fg"
                                    },
                                    onclick: reset,
                                    "Reset"
                                }
                            }
                        }
                    },
                    1 => rsx! {
                        if let Some(calc_results) = results.read().as_ref() {
                            ResultsPanel {
                                results: calc_results.clone(),
                                dark_mode: dark_mode(),
                            }
                        }
                    },
                    2 => rsx! {
                        MonteCarloPanel {
                            property_data: property_data.read().clone(),
                            rental_data: rental_data.read().clone(),
                            investment_params: investment_params.read().clone(),
                            dark_mode: dark_mode(),
                        }
                    },
                    _ => rsx! { div { "Unknown tab" } }
                }
            }

            // Footer
            div { class: if dark_mode() { "mt-12 py-6 bg-monokai-bgLight border-t border-monokai-border" } else { "mt-12 py-6 bg-monokaiLight-bgDark border-t border-monokaiLight-border" },
                div { class: if dark_mode() { "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center text-sm text-monokai-fgMuted" } else { "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center text-sm text-monokaiLight-fgMuted" },
                    p { "Built with Rust, Dioxus, and WebAssembly" }
                    p { class: "mt-1", "© 2025 Luke Aslanian" }
                }
            }
        }
    }
}
