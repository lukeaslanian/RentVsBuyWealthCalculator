use crate::models::FinancialResults;
use charming::{
    component::{Axis, Grid, Legend, Title},
    element::{
        AreaStyle, AxisLabel, AxisPointer, AxisPointerType, AxisType, Emphasis, EmphasisFocus,
        ItemStyle, LineStyle, MarkLine, MarkLineData, MarkLineVariant, NameLocation, SplitLine,
        Symbol, TextStyle, Tooltip, Trigger,
    },
    series::Line,
    Chart, WasmRenderer,
};
use dioxus::prelude::*;

/// Get the width for charts based on window size, optimized for mobile
fn get_chart_width() -> u32 {
    web_sys::window()
        .and_then(|w| w.inner_width().ok())
        .and_then(|w| w.as_f64())
        .map(|w| {
            let window_width = w as u32;
            if window_width <= 320 {
                // Very small mobile: minimal padding
                window_width.saturating_sub(32).max(200)
            } else if window_width < 480 {
                // Small mobile: tight padding
                window_width.saturating_sub(40).max(240)
            } else if window_width < 640 {
                // Mobile: standard padding
                window_width.saturating_sub(56).max(280)
            } else if window_width < 1024 {
                // Tablet: account for container padding
                window_width.saturating_sub(80).max(350)
            } else {
                // Desktop: 2-column grid, so roughly half width minus gaps
                ((window_width - 120) / 2).min(550).max(350)
            }
        })
        .unwrap_or(300)
}

// Monokai theme colors
const MONOKAI_CYAN: &str = "#66d9ef";
const MONOKAI_RED: &str = "#ff6188";
const MONOKAI_GREEN: &str = "#a9dc76";
const MONOKAI_PURPLE: &str = "#ab9df2";
const MONOKAI_YELLOW: &str = "#ffd866";
const MONOKAI_BG: &str = "#2d2a2e";
const MONOKAI_FG: &str = "#fcfcfa";

// Light theme colors
const LIGHT_BG: &str = "#ffffff";
const LIGHT_FG: &str = "#272822";

#[component]
pub fn ChartsGrid(results: FinancialResults, #[props(default = false)] dark_mode: bool) -> Element {
    rsx! {
        div { class: "space-y-8",
            h2 { class: if dark_mode { "text-2xl font-bold text-monokai-fg mb-6" } else { "text-2xl font-bold text-monokaiLight-fg mb-6" },
                "Visual Analysis"
            }

            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                // Chart 1: Cost Comparison
                CharmingLineChart {
                    chart_id: "cost-comparison",
                    title: "Chart 1: Cost Comparison",
                    description: "Total money spent on housing over time (mortgage, taxes, insurance, maintenance vs rent)",
                    dataset1: results.monthly_buy_cumulative_costs.clone(),
                    dataset2: results.monthly_rent_cumulative_costs.clone(),
                    label1: "Buy Costs",
                    label2: "Rent Costs",
                    color1: MONOKAI_CYAN.to_string(),
                    color2: MONOKAI_RED.to_string(),
                    dark_mode: dark_mode,
                }

                // Chart 2: Total Wealth Over Time
                CharmingLineChart {
                    chart_id: "total-wealth",
                    title: "Chart 2: Total Wealth Over Time",
                    description: "Net wealth comparison: buyer (equity + investments) vs renter (investments only). Higher line wins.",
                    dataset1: results.monthly_buy_total_wealth.clone(),
                    dataset2: results.monthly_rent_total_wealth.clone(),
                    label1: "Buy Total Wealth",
                    label2: "Rent Total Wealth",
                    color1: MONOKAI_GREEN.to_string(),
                    color2: MONOKAI_RED.to_string(),
                    dark_mode: dark_mode,
                }

                // Chart 3: Buyer Wealth Components (stacked)
                CharmingStackedAreaChart {
                    chart_id: "buyer-wealth-components",
                    title: "Chart 3: Buyer Wealth Components",
                    description: "Buyer's wealth breakdown: home equity from mortgage paydown and appreciation plus investment portfolio",
                    dataset1: results.monthly_home_equity.clone(),
                    dataset2: results.monthly_buy_investment_portfolio.clone(),
                    label1: "Home Equity",
                    label2: "Buy Investments",
                    color1: MONOKAI_CYAN.to_string(),
                    color2: MONOKAI_PURPLE.to_string(),
                    dark_mode: dark_mode,
                }

                // Chart 4: Monthly Costs Comparison
                CharmingLineChart {
                    chart_id: "monthly-costs",
                    title: "Chart 4: Monthly Costs Comparison",
                    description: "Monthly housing costs over time (buying increases with property taxes and HOA fees, renting increases with rent inflation)",
                    dataset1: results.monthly_buy_costs.clone(),
                    dataset2: results.monthly_rent_costs.clone(),
                    label1: "Buy Monthly Cost",
                    label2: "Rent Monthly Cost",
                    color1: MONOKAI_CYAN.to_string(),
                    color2: MONOKAI_RED.to_string(),
                    dark_mode: dark_mode,
                }

                // Chart 5: Investment Portfolios
                CharmingLineChart {
                    chart_id: "investment-portfolios",
                    title: "Chart 5: Investment Portfolios",
                    description: "Investment portfolio growth from monthly savings differences and down payment (renter invests saved down payment)",
                    dataset1: results.monthly_buy_investment_portfolio.clone(),
                    dataset2: results.monthly_rent_investment_portfolio.clone(),
                    label1: "Buy Investments",
                    label2: "Rent Investments",
                    color1: MONOKAI_CYAN.to_string(),
                    color2: MONOKAI_YELLOW.to_string(),
                    dark_mode: dark_mode,
                }
            }
        }
    }
}

#[component]
fn CharmingLineChart(
    chart_id: String,
    title: String,
    description: String,
    dataset1: Vec<f64>,
    dataset2: Vec<f64>,
    label1: String,
    label2: String,
    color1: String,
    color2: String,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    let chart_id_clone = chart_id.clone();

    use_effect(move || {
        let dataset1 = dataset1.clone();
        let dataset2 = dataset2.clone();
        let label1 = label1.clone();
        let label2 = label2.clone();
        let color1 = color1.clone();
        let color2 = color2.clone();
        let title = title.clone();
        let chart_id = chart_id_clone.clone();

        let (bg_color, text_color, grid_color) = if dark_mode {
            (MONOKAI_BG, MONOKAI_FG, "#524f53")
        } else {
            (LIGHT_BG, LIGHT_FG, "#dcdcdc")
        };

        let num_points = dataset1.len().min(dataset2.len());
        let num_years = (num_points + 11) / 12;

        // Create x-axis labels (years)
        let x_labels: Vec<String> = (0..=num_years).map(|y| format!("{}", y)).collect();

        // Sample data to yearly intervals for x-axis alignment
        let yearly_data1: Vec<f64> = (0..=num_years)
            .map(|y| {
                let idx = (y * 12).min(num_points.saturating_sub(1));
                dataset1.get(idx).copied().unwrap_or(0.0)
            })
            .collect();

        let yearly_data2: Vec<f64> = (0..=num_years)
            .map(|y| {
                let idx = (y * 12).min(num_points.saturating_sub(1));
                dataset2.get(idx).copied().unwrap_or(0.0)
            })
            .collect();

        // Find crossover points
        let mut crossover_year: Option<i32> = None;
        for i in 1..yearly_data1.len() {
            let diff_prev = yearly_data1[i - 1] - yearly_data2[i - 1];
            let diff_curr = yearly_data1[i] - yearly_data2[i];
            if diff_prev * diff_curr < 0.0 {
                crossover_year = Some(i as i32);
                break;
            }
        }

        let mut line1 = Line::new()
            .name(&label1)
            .data(yearly_data1)
            .symbol(Symbol::Circle)
            .symbol_size(6)
            .line_style(LineStyle::new().width(3))
            .item_style(ItemStyle::new().color(color1.as_str()))
            .emphasis(Emphasis::new().focus(EmphasisFocus::Series));

        // Add mark line for crossover if found
        if let Some(year) = crossover_year {
            line1 = line1.mark_line(
                MarkLine::new()
                    .symbol(vec![Symbol::None, Symbol::None])
                    .data(vec![MarkLineVariant::Simple(
                        MarkLineData::new()
                            .x_axis(year)
                            .name(format!("Break-even Year: {}", year)),
                    )])
                    .label(charming::element::Label::new().formatter("{b}"))
                    .line_style(
                        LineStyle::new()
                            .color(MONOKAI_YELLOW)
                            .width(2)
                            .type_(charming::element::LineStyleType::Dashed),
                    ),
            );
        }

        let line2 = Line::new()
            .name(&label2)
            .data(yearly_data2)
            .symbol(Symbol::Circle)
            .symbol_size(6)
            .line_style(LineStyle::new().width(3))
            .item_style(ItemStyle::new().color(color2.as_str()))
            .emphasis(Emphasis::new().focus(EmphasisFocus::Series));

        let chart = Chart::new()
            .background_color(bg_color)
            .title(
                Title::new()
                    .text(&title)
                    .left("center")
                    .text_style(TextStyle::new().color(text_color).font_size(16)),
            )
            .tooltip(
                Tooltip::new()
                    .trigger(Trigger::Axis)
                    .axis_pointer(AxisPointer::new().type_(AxisPointerType::Line)),
            )
            .legend(
                Legend::new()
                    .top("bottom")
                    .text_style(TextStyle::new().color(text_color)),
            )
            .grid(
                Grid::new()
                    .left("15%")
                    .right("5%")
                    .top("18%")
                    .bottom("18%")
                    .contain_label(true),
            )
            .x_axis(
                Axis::new()
                    .type_(AxisType::Category)
                    .data(x_labels)
                    .name("Year")
                    .name_location(NameLocation::Middle)
                    .name_gap(25)
                    .axis_label(AxisLabel::new().color(text_color))
                    .axis_pointer(AxisPointer::new().type_(AxisPointerType::Shadow)),
            )
            .y_axis(
                Axis::new()
                    .type_(AxisType::Value)
                    .axis_label(AxisLabel::new().color(text_color))
                    .split_line(SplitLine::new().line_style(LineStyle::new().color(grid_color))),
            )
            .series(line1)
            .series(line2);

        // Get width dynamically for responsive sizing
        let width = get_chart_width();
        let height = if width < 300 {
            (width as f32 * 0.75) as u32 // Taller ratio for very small
        } else if width < 400 {
            (width as f32 * 0.7).min(280.0) as u32
        } else {
            (width as f32 * 0.6).min(350.0) as u32
        };

        let renderer = WasmRenderer::new(width, height);
        let _ = renderer.render(&chart_id, &chart);
    });

    rsx! {
        div {
            class: if dark_mode { "bg-monokai-bgLight rounded-lg shadow-md p-2 sm:p-4 w-full overflow-hidden" } else { "bg-monokaiLight-bgLight rounded-lg shadow-md p-2 sm:p-4 w-full overflow-hidden" },
            div { class: if dark_mode { "bg-monokai-bgLighter border border-monokai-bgLighter rounded p-1.5 sm:p-2 mb-2 sm:mb-3" } else { "bg-monokaiLight-bg border border-monokaiLight-border rounded p-1.5 sm:p-2 mb-2 sm:mb-3" },
                p { class: if dark_mode { "text-xs text-monokai-fgMuted text-center leading-tight" } else { "text-xs text-monokaiLight-fgMuted text-center leading-tight" },
                    "{description}"
                }
            }
            div {
                id: "{chart_id}",
                class: if dark_mode { "rounded" } else { "border border-monokaiLight-border rounded" },
            }
        }
    }
}

#[component]
fn CharmingStackedAreaChart(
    chart_id: String,
    title: String,
    description: String,
    dataset1: Vec<f64>,
    dataset2: Vec<f64>,
    label1: String,
    label2: String,
    color1: String,
    color2: String,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    let chart_id_clone = chart_id.clone();

    use_effect(move || {
        let dataset1 = dataset1.clone();
        let dataset2 = dataset2.clone();
        let label1 = label1.clone();
        let label2 = label2.clone();
        let color1 = color1.clone();
        let color2 = color2.clone();
        let title = title.clone();
        let chart_id = chart_id_clone.clone();

        let (bg_color, text_color, grid_color) = if dark_mode {
            (MONOKAI_BG, MONOKAI_FG, "#524f53")
        } else {
            (LIGHT_BG, LIGHT_FG, "#dcdcdc")
        };

        let num_points = dataset1.len().min(dataset2.len());
        let num_years = (num_points + 11) / 12;

        // Create x-axis labels (years)
        let x_labels: Vec<String> = (0..=num_years).map(|y| format!("{}", y)).collect();

        // Sample data to yearly intervals
        let yearly_data1: Vec<f64> = (0..=num_years)
            .map(|y| {
                let idx = (y * 12).min(num_points.saturating_sub(1));
                dataset1.get(idx).copied().unwrap_or(0.0)
            })
            .collect();

        let yearly_data2: Vec<f64> = (0..=num_years)
            .map(|y| {
                let idx = (y * 12).min(num_points.saturating_sub(1));
                dataset2.get(idx).copied().unwrap_or(0.0)
            })
            .collect();

        let area1 = Line::new()
            .name(&label1)
            .data(yearly_data1)
            .stack("Total")
            .area_style(AreaStyle::new().opacity(0.7))
            .symbol(Symbol::Circle)
            .symbol_size(4)
            .line_style(LineStyle::new().width(2))
            .item_style(ItemStyle::new().color(color1.as_str()))
            .emphasis(Emphasis::new().focus(EmphasisFocus::Series));

        let area2 = Line::new()
            .name(&label2)
            .data(yearly_data2)
            .stack("Total")
            .area_style(AreaStyle::new().opacity(0.7))
            .symbol(Symbol::Circle)
            .symbol_size(4)
            .line_style(LineStyle::new().width(2))
            .item_style(ItemStyle::new().color(color2.as_str()))
            .emphasis(Emphasis::new().focus(EmphasisFocus::Series));

        let chart = Chart::new()
            .background_color(bg_color)
            .title(
                Title::new()
                    .text(&title)
                    .left("center")
                    .text_style(TextStyle::new().color(text_color).font_size(16)),
            )
            .tooltip(
                Tooltip::new()
                    .trigger(Trigger::Axis)
                    .axis_pointer(AxisPointer::new().type_(AxisPointerType::Line)),
            )
            .legend(
                Legend::new()
                    .top("bottom")
                    .text_style(TextStyle::new().color(text_color)),
            )
            .grid(
                Grid::new()
                    .left("15%")
                    .right("5%")
                    .top("18%")
                    .bottom("18%")
                    .contain_label(true),
            )
            .x_axis(
                Axis::new()
                    .type_(AxisType::Category)
                    .data(x_labels)
                    .name("Year")
                    .name_location(NameLocation::Middle)
                    .name_gap(25)
                    .boundary_gap(false)
                    .axis_label(AxisLabel::new().color(text_color))
                    .axis_pointer(AxisPointer::new().type_(AxisPointerType::Shadow)),
            )
            .y_axis(
                Axis::new()
                    .type_(AxisType::Value)
                    .axis_label(AxisLabel::new().color(text_color))
                    .split_line(SplitLine::new().line_style(LineStyle::new().color(grid_color))),
            )
            .series(area1)
            .series(area2);

        // Get width dynamically for responsive sizing
        let width = get_chart_width();
        let height = if width < 300 {
            (width as f32 * 0.75) as u32 // Taller ratio for very small
        } else if width < 400 {
            (width as f32 * 0.7).min(280.0) as u32
        } else {
            (width as f32 * 0.6).min(350.0) as u32
        };

        let renderer = WasmRenderer::new(width, height);
        let _ = renderer.render(&chart_id, &chart);
    });

    rsx! {
        div {
            class: if dark_mode { "bg-monokai-bgLight rounded-lg shadow-md p-2 sm:p-4 w-full overflow-hidden" } else { "bg-monokaiLight-bgLight rounded-lg shadow-md p-2 sm:p-4 w-full overflow-hidden" },
            div { class: if dark_mode { "bg-monokai-bgLighter border border-monokai-bgLighter rounded p-1.5 sm:p-2 mb-2 sm:mb-3" } else { "bg-monokaiLight-bg border border-monokaiLight-border rounded p-1.5 sm:p-2 mb-2 sm:mb-3" },
                p { class: if dark_mode { "text-xs text-monokai-fgMuted text-center leading-tight" } else { "text-xs text-monokaiLight-fgMuted text-center leading-tight" },
                    "{description}"
                }
            }
            div {
                id: "{chart_id}",
                class: if dark_mode { "rounded" } else { "border border-monokaiLight-border rounded" },
            }
        }
    }
}
