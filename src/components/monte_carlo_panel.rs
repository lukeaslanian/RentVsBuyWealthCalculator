use crate::calculators::{MonteCarloSimulator, SimulationResults};
use crate::models::{InvestmentParameters, PropertyData, RentalData};
use crate::utils::CurrencyFormatter;
use charming::{
    component::{Axis, Grid, Legend, Title},
    element::{
        AxisLabel, AxisPointer, AxisPointerType, AxisType, ItemStyle, LineStyle, SplitLine, Symbol,
        TextStyle, Tooltip, Trigger,
    },
    series::Bar,
    Chart, WasmRenderer,
};
use dioxus::prelude::*;

/// Get the width for Monte Carlo chart (single full-width chart)
fn get_chart_width() -> u32 {
    web_sys::window()
        .and_then(|w| w.inner_width().ok())
        .and_then(|w| w.as_f64())
        .map(|w| {
            let window_width = w as u32;
            if window_width <= 320 {
                window_width.saturating_sub(32).max(200)
            } else if window_width < 480 {
                window_width.saturating_sub(40).max(240)
            } else if window_width < 640 {
                window_width.saturating_sub(56).max(280)
            } else if window_width < 1024 {
                window_width.saturating_sub(80).max(350)
            } else {
                // Single chart - use more width but cap at reasonable max
                window_width.saturating_sub(120).min(900).max(400)
            }
        })
        .unwrap_or(300)
}

#[component]
pub fn MonteCarloPanel(
    property_data: PropertyData,
    rental_data: RentalData,
    investment_params: InvestmentParameters,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    let mut selected_count = use_signal(|| 1000usize);
    let mut simulation_results = use_signal(|| None);
    let mut is_running = use_signal(|| false);
    let mut progress = use_signal(|| 0.0f64);

    let run_simulation = move |_| {
        if is_running() {
            return;
        }

        is_running.set(true);
        simulation_results.set(None);
        progress.set(0.0);

        // Clone data for the async task
        let prop_data = property_data.clone();
        let rent_data = rental_data.clone();
        let inv_params = investment_params.clone();
        let count = selected_count();

        // Spawn async task to run simulation with progress updates
        spawn(async move {
            let chunk_size = 100.min(count / 10).max(1); // Process in chunks of 100 or 10% of total
            let mut completed = 0usize;
            let mut all_buy_wealth = Vec::new();
            let mut all_rent_wealth = Vec::new();
            let mut buy_wins = 0usize;
            let mut rent_wins = 0usize;

            while completed < count {
                let batch_size = chunk_size.min(count - completed);

                let simulator = MonteCarloSimulator::new(
                    prop_data.clone(),
                    rent_data.clone(),
                    inv_params.clone(),
                );

                let batch_results = simulator.run_simulation(batch_size);

                buy_wins += batch_results.buy_wins;
                rent_wins += batch_results.rent_wins;
                all_buy_wealth.extend(batch_results.buy_final_wealth);
                all_rent_wealth.extend(batch_results.rent_final_wealth);

                completed += batch_size;
                progress.set((completed as f64 / count as f64) * 100.0);

                // Yield to allow UI updates
                gloo_timers::future::TimeoutFuture::new(10).await;
            }

            let final_results = SimulationResults {
                num_simulations: count,
                buy_wins,
                rent_wins,
                buy_final_wealth: all_buy_wealth,
                rent_final_wealth: all_rent_wealth,
            };

            simulation_results.set(Some(final_results));
            is_running.set(false);
            progress.set(100.0);
        });
    };

    rsx! {
        div { class: "space-y-6",
            // Control Panel
            div { class: if dark_mode { "bg-monokai-bgLight rounded-lg shadow-md p-6" } else { "bg-monokaiLight-bgLight rounded-lg shadow-md p-6" },
                h2 { class: if dark_mode { "text-xl font-bold text-monokai-fg mb-2" } else { "text-xl font-bold text-monokaiLight-fg mb-2" },
                    "Monte Carlo Simulation Settings"
                }

                p { class: if dark_mode { "text-sm text-monokai-fgMuted mb-4" } else { "text-sm text-monokaiLight-fgMuted mb-4" },
                    "Run multiple simulations with randomized parameters to assess outcome distributions."
                }

                div { class: if dark_mode { "bg-monokai-bgLighter border border-monokai-bgLighter rounded-lg p-4 mb-4" } else { "bg-monokaiLight-bg border border-monokaiLight-border rounded-lg p-4 mb-4" },
                    p { class: if dark_mode { "text-xs text-monokai-fgMuted font-medium mb-2" } else { "text-xs text-monokaiLight-fgMuted font-medium mb-2" },
                        "Future uncertain rates varied (known values like home price & interest rate stay fixed):"
                    }
                    ul { class: if dark_mode { "text-xs text-monokai-fgMuted space-y-1 ml-4 list-disc" } else { "text-xs text-monokaiLight-fgMuted space-y-1 ml-4 list-disc" },
                        li { "Home appreciation rate: ±4% (absolute)" }
                        li { "HOA/Condo fee increase rate: ±3% (absolute)" }
                        li { "Maintenance costs: ±1.5% (absolute)" }
                        li { "Rent increase rate: ±3% (absolute)" }
                        li { "Investment return rate: ±5% (absolute)" }
                        li { "Home insurance: ±30%" }
                    }
                }

                // Simulation count selection
                div { class: "space-y-3 mb-6",
                    h3 { class: if dark_mode { "text-sm font-semibold text-monokai-fgMuted" } else { "text-sm font-semibold text-monokaiLight-fgMuted" },
                        "Number of Simulations:"
                    }

                    div { class: "space-y-2",
                        SimulationOption {
                            value: 100,
                            label: "100 simulations (fast)",
                            selected: selected_count() == 100,
                            onselect: move |v| selected_count.set(v),
                            dark_mode: dark_mode,
                        }
                        SimulationOption {
                            value: 1000,
                            label: "1,000 simulations (recommended)",
                            selected: selected_count() == 1000,
                            onselect: move |v| selected_count.set(v),
                            dark_mode: dark_mode,
                        }
                        SimulationOption {
                            value: 10000,
                            label: "10,000 simulations (thorough)",
                            selected: selected_count() == 10000,
                            onselect: move |v| selected_count.set(v),
                            dark_mode: dark_mode,
                        }
                        SimulationOption {
                            value: 100000,
                            label: "100,000 simulations (comprehensive; may be slow)",
                            selected: selected_count() == 100000,
                            onselect: move |v| selected_count.set(v),
                            dark_mode: dark_mode,
                        }
                    }
                }

                // Run button
                div { class: "flex items-center gap-4",
                    button {
                        class: if is_running() {
                            "px-6 py-3 bg-monokai-fgMuted text-monokai-bg font-bold rounded-lg cursor-not-allowed"
                        } else {
                            "px-6 py-3 bg-monokai-green text-monokai-bg font-bold rounded-lg hover:opacity-90 transition shadow-lg"
                        },
                        onclick: run_simulation,
                        disabled: is_running(),
                        if is_running() {
                            "Running Simulation..."
                        } else {
                            "Run Simulation"
                        }
                    }

                    if is_running() {
                        div { class: "flex-1 ml-4",
                            div { class: "flex items-center gap-3 mb-2",
                                div { class: "animate-spin rounded-full h-5 w-5 border-b-2 border-monokai-green" }
                                span { class: if dark_mode { "text-sm text-monokai-fgMuted font-medium" } else { "text-sm text-monokaiLight-fgMuted font-medium" },
                                    "Running simulation... {progress():.1}%"
                                }
                            }
                            // Progress bar
                            div { class: if dark_mode { "w-full bg-monokai-bgLighter rounded-full h-3 overflow-hidden" } else { "w-full bg-monokaiLight-border rounded-full h-3 overflow-hidden" },
                                div {
                                    class: "bg-gradient-to-r from-monokai-purple to-monokai-blue h-3 rounded-full transition-all duration-300",
                                    style: "width: {progress()}%",
                                }
                            }
                            p { class: if dark_mode { "text-xs text-monokai-fgMuted mt-1" } else { "text-xs text-monokaiLight-fgMuted mt-1" },
                                "Processing in chunks to keep UI responsive..."
                            }
                        }
                    }
                }
            }

            // Results Panel
            if let Some(results) = simulation_results() {
                SimulationResultsDisplay { results: results, dark_mode: dark_mode }
            } else {
                div { class: if dark_mode { "bg-monokai-bgLight rounded-lg border-2 border-dashed border-monokai-bgLighter p-12 text-center" } else { "bg-monokaiLight-bg rounded-lg border-2 border-dashed border-monokaiLight-border p-12 text-center" },
                    p { class: if dark_mode { "text-monokai-fgMuted text-lg" } else { "text-monokaiLight-fgMuted text-lg" },
                        "Run a simulation to see results"
                    }
                    p { class: if dark_mode { "text-monokai-fgMuted text-sm mt-2 opacity-70" } else { "text-monokaiLight-fgMuted text-sm mt-2 opacity-70" },
                        "Results will appear here after the simulation completes"
                    }
                }
            }
        }
    }
}

#[component]
fn SimulationOption(
    value: usize,
    label: String,
    selected: bool,
    onselect: EventHandler<usize>,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    rsx! {
        label {
            class: if dark_mode {
                if selected {
                    "flex items-center gap-3 p-3 border-2 rounded-lg cursor-pointer hover:bg-monokai-bgLighter transition border-monokai-purple bg-monokai-purple bg-opacity-20"
                } else {
                    "flex items-center gap-3 p-3 border-2 rounded-lg cursor-pointer hover:bg-monokai-bgLighter transition border-monokai-bgLighter"
                }
            } else {
                if selected {
                    "flex items-center gap-3 p-3 border-2 rounded-lg cursor-pointer hover:bg-monokaiLight-bg transition border-monokaiLight-purple bg-monokaiLight-purple bg-opacity-10"
                } else {
                    "flex items-center gap-3 p-3 border-2 rounded-lg cursor-pointer hover:bg-monokaiLight-bg transition border-monokaiLight-border"
                }
            },
            input {
                r#type: "radio",
                name: "simulation_count",
                checked: selected,
                onchange: move |_| onselect.call(value),
                class: "w-4 h-4 text-monokai-purple cursor-pointer",
            }
            span { class: if dark_mode { "text-sm text-monokai-fg" } else { "text-sm text-monokaiLight-fg" },
                "{label}"
            }
        }
    }
}

#[component]
fn SimulationResultsDisplay(
    results: SimulationResults,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    let buy_win_pct = results.buy_win_percentage();
    let rent_win_pct = 100.0 - buy_win_pct;
    let avg_buy = results.avg_buy_wealth();
    let avg_rent = results.avg_rent_wealth();

    // Calculate statistics
    let buy_stats = calculate_stats(&results.buy_final_wealth);
    let rent_stats = calculate_stats(&results.rent_final_wealth);

    let wealth_diffs: Vec<f64> = results
        .buy_final_wealth
        .iter()
        .zip(results.rent_final_wealth.iter())
        .map(|(b, r)| b - r)
        .collect();
    let diff_stats = calculate_stats(&wealth_diffs);

    // Determine interpretation
    let interpretation = if buy_win_pct > 60.0 {
        if dark_mode {
            (
                "Buying tends to be better in most scenarios.",
                "text-monokai-green bg-monokai-green bg-opacity-20 border-monokai-green",
            )
        } else {
            (
                "Buying tends to be better in most scenarios.",
                "text-monokaiLight-green bg-monokaiLight-green bg-opacity-10 border-monokaiLight-green",
            )
        }
    } else if rent_win_pct > 60.0 {
        if dark_mode {
            (
                "Renting tends to be better in most scenarios.",
                "text-monokai-red bg-monokai-red bg-opacity-20 border-monokai-red",
            )
        } else {
            (
                "Renting tends to be better in most scenarios.",
                "text-monokaiLight-red bg-monokaiLight-red bg-opacity-10 border-monokaiLight-red",
            )
        }
    } else {
        if dark_mode {
            ("Outcome is highly sensitive to assumptions. Consider your risk tolerance carefully.", "text-monokai-yellow bg-monokai-yellow bg-opacity-20 border-monokai-yellow")
        } else {
            ("Outcome is highly sensitive to assumptions. Consider your risk tolerance carefully.", "text-monokaiLight-yellow bg-monokaiLight-yellow bg-opacity-10 border-monokaiLight-yellow")
        }
    };

    // Format numbers with thousand separators
    let num_sims_formatted = format_with_commas(results.num_simulations);
    let buy_wins_formatted = format_with_commas(results.buy_wins);
    let rent_wins_formatted = format_with_commas(results.rent_wins);

    rsx! {
        div { class: "space-y-6",
            // Summary Cards
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                // Win percentage card
                div { class: if dark_mode { "bg-monokai-bgLight rounded-lg shadow-md p-6" } else { "bg-monokaiLight-bgLight rounded-lg shadow-md p-6" },
                    h3 { class: if dark_mode { "text-lg font-semibold text-monokai-fg mb-4" } else { "text-lg font-semibold text-monokaiLight-fg mb-4" },
                        "Outcome Summary"
                    }
                    p { class: if dark_mode { "text-sm text-monokai-fgMuted mb-2" } else { "text-sm text-monokaiLight-fgMuted mb-2" },
                        "{num_sims_formatted} simulations completed"
                    }

                    div { class: "space-y-3 mt-4",
                        // Win percentage bars
                        div {
                            div { class: "flex justify-between items-center mb-1",
                                span { class: "text-sm font-medium text-monokai-green",
                                    "Buying Wins:"
                                }
                                span { class: "text-sm font-bold text-monokai-green",
                                    "{buy_wins_formatted} ({buy_win_pct:.1}%)"
                                }
                            }
                            div { class: if dark_mode { "w-full bg-monokai-bgLighter rounded-full h-4" } else { "w-full bg-monokaiLight-border rounded-full h-4" },
                                div {
                                    class: "bg-monokai-green h-4 rounded-full transition-all",
                                    style: "width: {buy_win_pct}%",
                                }
                            }
                        }

                        div {
                            div { class: "flex justify-between items-center mb-1",
                                span { class: "text-sm font-medium text-monokai-red",
                                    "Renting Wins:"
                                }
                                span { class: "text-sm font-bold text-monokai-red",
                                    "{rent_wins_formatted} ({rent_win_pct:.1}%)"
                                }
                            }
                            div { class: if dark_mode { "w-full bg-monokai-bgLighter rounded-full h-4" } else { "w-full bg-monokaiLight-border rounded-full h-4" },
                                div {
                                    class: "bg-monokai-red h-4 rounded-full transition-all",
                                    style: "width: {rent_win_pct}%",
                                }
                            }
                        }
                    }
                }

                // Interpretation card
                div { class: if dark_mode { "bg-monokai-bgLight rounded-lg shadow-md p-6" } else { "bg-monokaiLight-bgLight rounded-lg shadow-md p-6" },
                    h3 { class: if dark_mode { "text-lg font-semibold text-monokai-fg mb-4" } else { "text-lg font-semibold text-monokaiLight-fg mb-4" },
                        "Interpretation"
                    }

                    div { class: "border-2 rounded-lg p-4 {interpretation.1}",
                        p { class: "text-sm font-medium",
                            "{interpretation.0}"
                        }
                    }

                    div { class: "mt-4 space-y-2",
                        div { class: "flex justify-between",
                            span { class: if dark_mode { "text-sm text-monokai-fgMuted" } else { "text-sm text-monokaiLight-fgMuted" }, "Avg. Buy Wealth:" }
                            span { class: if dark_mode { "text-sm font-semibold text-monokai-fg" } else { "text-sm font-semibold text-monokaiLight-fg" },
                                "{CurrencyFormatter::format_currency(avg_buy)}"
                            }
                        }
                        div { class: "flex justify-between",
                            span { class: if dark_mode { "text-sm text-monokai-fgMuted" } else { "text-sm text-monokaiLight-fgMuted" }, "Avg. Rent Wealth:" }
                            span { class: if dark_mode { "text-sm font-semibold text-monokai-fg" } else { "text-sm font-semibold text-monokaiLight-fg" },
                                "{CurrencyFormatter::format_currency(avg_rent)}"
                            }
                        }
                        div { class: if dark_mode { "flex justify-between border-t border-monokai-bgLighter pt-2" } else { "flex justify-between border-t border-monokaiLight-border pt-2" },
                            span { class: if dark_mode { "text-sm font-medium text-monokai-fgMuted" } else { "text-sm font-medium text-monokaiLight-fgMuted" }, "Avg. Difference:" }
                            span {
                                class: if avg_buy > avg_rent { "text-sm font-bold text-monokai-green" } else { "text-sm font-bold text-monokai-red" },
                                "{CurrencyFormatter::format_currency(avg_buy - avg_rent)}"
                            }
                        }
                    }
                }
            }

            // Detailed Statistics
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                StatisticsCard {
                    title: "Buying Wealth Statistics",
                    stats: buy_stats,
                    color: "green",
                    dark_mode: dark_mode,
                }
                StatisticsCard {
                    title: "Renting Wealth Statistics",
                    stats: rent_stats,
                    color: "blue",
                    dark_mode: dark_mode,
                }
                StatisticsCard {
                    title: "Wealth Difference (Buy - Rent)",
                    stats: diff_stats,
                    color: "purple",
                    dark_mode: dark_mode,
                }
            }

            // Simple histogram
            HistogramChart { data: wealth_diffs, dark_mode: dark_mode }
        }
    }
}

#[component]
fn StatisticsCard(
    title: String,
    stats: Stats,
    color: String,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    let border_color = match color.as_str() {
        "green" => "border-monokai-green",
        "blue" => "border-monokai-blue",
        "purple" => "border-monokai-purple",
        _ => {
            if dark_mode {
                "border-monokai-bgLighter"
            } else {
                "border-monokaiLight-border"
            }
        }
    };

    rsx! {
        div { class: if dark_mode { "bg-monokai-bgLight rounded-lg shadow-md border-t-4 {border_color} p-6" } else { "bg-monokaiLight-bgLight rounded-lg shadow-md border-t-4 {border_color} p-6" },
            h3 { class: if dark_mode { "text-sm font-semibold text-monokai-fg mb-4" } else { "text-sm font-semibold text-monokaiLight-fg mb-4" },
                "{title}"
            }

            div { class: "space-y-2 text-xs",
                StatRow { label: "Minimum:", value: stats.min, dark_mode: dark_mode }
                StatRow { label: "25th Percentile:", value: stats.q1, dark_mode: dark_mode }
                StatRow { label: "Median:", value: stats.median, dark_mode: dark_mode }
                StatRow { label: "Mean:", value: stats.mean, dark_mode: dark_mode }
                StatRow { label: "75th Percentile:", value: stats.q3, dark_mode: dark_mode }
                StatRow { label: "Maximum:", value: stats.max, dark_mode: dark_mode }
                StatRow { label: "Std Dev:", value: stats.std_dev, dark_mode: dark_mode }
            }
        }
    }
}

#[component]
fn StatRow(label: String, value: f64, #[props(default = false)] dark_mode: bool) -> Element {
    rsx! {
        div { class: "flex justify-between",
            span { class: if dark_mode { "text-monokai-fgMuted" } else { "text-monokaiLight-fgMuted" }, "{label}" }
            span { class: if dark_mode { "font-semibold text-monokai-fg" } else { "font-semibold text-monokaiLight-fg" },
                "{CurrencyFormatter::format_currency(value)}"
            }
        }
    }
}

#[component]
fn HistogramChart(data: Vec<f64>, #[props(default = false)] dark_mode: bool) -> Element {
    use_effect(move || {
        let data = data.clone();
        draw_histogram(&data, dark_mode);
    });

    rsx! {
        div { class: if dark_mode { "bg-monokai-bgLight rounded-lg shadow-md p-6" } else { "bg-monokaiLight-bgLight rounded-lg shadow-md p-6" },
            h3 { class: if dark_mode { "text-lg font-semibold text-monokai-fg mb-4" } else { "text-lg font-semibold text-monokaiLight-fg mb-4" },
                "Wealth Difference Distribution"
            }

            p { class: if dark_mode { "text-sm text-monokai-fgMuted mb-4" } else { "text-sm text-monokaiLight-fgMuted mb-4" },
                "Histogram showing the distribution of wealth differences (Buy - Rent) across all simulations"
            }

            div {
                id: "monte-carlo-histogram",
                style: "width: 100%; min-height: 300px; max-height: 500px;",
                class: if dark_mode { "rounded" } else { "border border-monokaiLight-border rounded" },
            }
        }
    }
}

// Helper structures and functions

#[derive(Clone, PartialEq)]
struct Stats {
    min: f64,
    max: f64,
    mean: f64,
    median: f64,
    q1: f64,
    q3: f64,
    std_dev: f64,
}

fn calculate_stats(data: &[f64]) -> Stats {
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let len = sorted.len();
    let min = sorted[0];
    let max = sorted[len - 1];
    let mean = sorted.iter().sum::<f64>() / len as f64;
    let median = if len % 2 == 0 {
        (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
    } else {
        sorted[len / 2]
    };

    let q1 = sorted[len / 4];
    let q3 = sorted[len * 3 / 4];

    let variance = sorted.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / len as f64;
    let std_dev = variance.sqrt();

    Stats {
        min,
        max,
        mean,
        median,
        q1,
        q3,
        std_dev,
    }
}

fn draw_histogram(data: &[f64], dark_mode: bool) {
    // Theme colors
    const MONOKAI_BG: &str = "#2d2a2e";
    const MONOKAI_FG: &str = "#fcfcfa";
    const MONOKAI_GRID: &str = "#524f53";
    const MONOKAI_GREEN: &str = "#a9dc76";
    const MONOKAI_RED: &str = "#ff6188";

    const LIGHT_BG: &str = "#ffffff";
    const LIGHT_FG: &str = "#2d2a2e";
    const LIGHT_GRID: &str = "#c8c8c8";
    const LIGHT_GREEN: &str = "#50a14f";
    const LIGHT_RED: &str = "#e45649";

    let (bg_color, text_color, grid_color, green_color, red_color) = if dark_mode {
        (
            MONOKAI_BG,
            MONOKAI_FG,
            MONOKAI_GRID,
            MONOKAI_GREEN,
            MONOKAI_RED,
        )
    } else {
        (LIGHT_BG, LIGHT_FG, LIGHT_GRID, LIGHT_GREEN, LIGHT_RED)
    };

    if data.is_empty() {
        return;
    }

    let num_bins = 50;
    let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let bin_width = (max - min) / num_bins as f64;

    // Create histogram bins
    let mut bins = vec![0i32; num_bins];
    for &value in data {
        let bin_index = ((value - min) / bin_width).floor() as usize;
        let bin_index = bin_index.min(num_bins - 1);
        bins[bin_index] += 1;
    }

    // Create bin labels (center of each bin)
    let bin_labels: Vec<String> = (0..num_bins)
        .map(|i| {
            let center = min + (i as f64 + 0.5) * bin_width;
            format_compact(center)
        })
        .collect();

    // Create separate series for positive and negative values
    let positive_data: Vec<i32> = bins
        .iter()
        .enumerate()
        .map(|(i, &count)| {
            let center = min + (i as f64 + 0.5) * bin_width;
            if center >= 0.0 {
                count
            } else {
                0
            }
        })
        .collect();

    let negative_data: Vec<i32> = bins
        .iter()
        .enumerate()
        .map(|(i, &count)| {
            let center = min + (i as f64 + 0.5) * bin_width;
            if center < 0.0 {
                count
            } else {
                0
            }
        })
        .collect();

    let positive_bar = Bar::new()
        .name("Buy Wins")
        .data(positive_data)
        .item_style(ItemStyle::new().color(green_color));

    let negative_bar = Bar::new()
        .name("Rent Wins")
        .data(negative_data)
        .item_style(ItemStyle::new().color(red_color));

    let chart = Chart::new()
        .background_color(bg_color)
        .title(
            Title::new()
                .text("Wealth Difference Distribution")
                .left("center")
                .text_style(TextStyle::new().color(text_color).font_size(18)),
        )
        .tooltip(
            Tooltip::new()
                .trigger(Trigger::Axis)
                .axis_pointer(AxisPointer::new().type_(AxisPointerType::Shadow)),
        )
        .legend(
            Legend::new()
                .top("top")
                .right("5%")
                .text_style(TextStyle::new().color(text_color)),
        )
        .grid(Grid::new().left("8%").right("4%").top("20%").bottom("18%"))
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .data(bin_labels)
                .name("Wealth Difference (Buy - Rent)")
                .name_location(charming::element::NameLocation::Middle)
                .name_gap(35)
                .name_text_style(TextStyle::new().color(text_color).font_size(14))
                .axis_label(
                    AxisLabel::new()
                        .color(text_color)
                        .interval(9)
                        .rotate(45)
                        .font_size(11),
                ),
        )
        .y_axis(
            Axis::new()
                .type_(AxisType::Value)
                .name("Frequency")
                .name_text_style(TextStyle::new().color(text_color).font_size(14))
                .axis_label(AxisLabel::new().color(text_color).font_size(12))
                .split_line(SplitLine::new().line_style(LineStyle::new().color(grid_color))),
        )
        .series(negative_bar)
        .series(positive_bar);

    let width = get_chart_width();
    let height = if width < 300 {
        (width as f32 * 0.8).min(240.0) as u32
    } else if width < 400 {
        (width as f32 * 0.7).min(280.0) as u32
    } else if width < 600 {
        (width as f32 * 0.6).min(350.0) as u32
    } else {
        // Better height for larger screens - match other charts
        (width as f32 * 0.55).min(450.0).max(350.0) as u32
    };

    let renderer = WasmRenderer::new(width, height);
    let _ = renderer.render("monte-carlo-histogram", &chart);
}

fn format_compact(value: f64) -> String {
    if value.abs() >= 1_000_000.0 {
        format!("${:.1}M", value / 1_000_000.0)
    } else if value.abs() >= 1_000.0 {
        format!("${:.0}K", value / 1_000.0)
    } else {
        format!("${:.0}", value)
    }
}

fn format_with_commas(n: usize) -> String {
    let s = n.to_string();
    let bytes: Vec<_> = s.bytes().rev().collect();
    let chunks: Vec<String> = bytes
        .chunks(3)
        .map(|chunk| chunk.iter().rev().map(|&b| b as char).collect::<String>())
        .collect();
    let reversed: Vec<&str> = chunks.iter().map(|s| s.as_str()).rev().collect();
    reversed.join(",")
}
