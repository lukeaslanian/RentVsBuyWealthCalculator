use crate::models::FinancialResults;
use dioxus::prelude::*;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

// Monokai dark theme colors
const MONOKAI_BG: RGBColor = RGBColor(45, 42, 46); // #2d2a2e
const MONOKAI_FG: RGBColor = RGBColor(252, 252, 250); // #fcfcfa
const MONOKAI_GRID: RGBColor = RGBColor(82, 79, 83); // #524f53
const MONOKAI_GRID_BOLD: RGBColor = RGBColor(99, 96, 100); // #636064

// Light theme colors
const LIGHT_BG: RGBColor = RGBColor(255, 255, 255);
const LIGHT_FG: RGBColor = RGBColor(39, 40, 34); // #272822
const LIGHT_GRID: RGBColor = RGBColor(220, 220, 220);
const LIGHT_GRID_BOLD: RGBColor = RGBColor(180, 180, 180);

#[component]
pub fn ChartsGrid(results: FinancialResults, #[props(default = false)] dark_mode: bool) -> Element {
    rsx! {
        div { class: "space-y-8",
            h2 { class: if dark_mode { "text-2xl font-bold text-monokai-fg mb-6" } else { "text-2xl font-bold text-monokaiLight-fg mb-6" },
                "Visual Analysis"
            }

            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                // Chart 1: Cost Comparison (with crossover indicator)
                PlottersLineChartWithCrossover {
                    chart_id: "cost-comparison",
                    title: "Chart 1: Cost Comparison",
                    description: "Total money spent on housing over time (mortgage, taxes, insurance, maintenance vs rent)",
                    dataset1: results.monthly_buy_cumulative_costs.clone(),
                    dataset2: results.monthly_rent_cumulative_costs.clone(),
                    label1: "Buy Costs",
                    label2: "Rent Costs",
                    color1: RGBColor(102, 217, 239),  // Monokai cyan
                    color2: RGBColor(255, 97, 136),   // Monokai red
                    dark_mode: dark_mode,
                }

                // Chart 2: Total Wealth Over Time (with crossover indicator)
                PlottersLineChartWithCrossover {
                    chart_id: "total-wealth",
                    title: "Chart 2: Total Wealth Over Time",
                    description: "Net wealth comparison: buyer (equity + investments) vs renter (investments only). Higher line wins.",
                    dataset1: results.monthly_buy_total_wealth.clone(),
                    dataset2: results.monthly_rent_total_wealth.clone(),
                    label1: "Buy Total Wealth",
                    label2: "Rent Total Wealth",
                    color1: RGBColor(169, 220, 118),  // Monokai green
                    color2: RGBColor(255, 97, 136),   // Monokai red
                    dark_mode: dark_mode,
                }

                // Chart 3: Buyer Wealth Components (stacked)
                PlottersStackedAreaChart {
                    chart_id: "buyer-wealth-components",
                    title: "Chart 3: Buyer Wealth Components",
                    description: "Buyer's wealth breakdown: home equity from mortgage paydown and appreciation plus investment portfolio",
                    dataset1: results.monthly_home_equity.clone(),
                    dataset2: results.monthly_buy_investment_portfolio.clone(),
                    label1: "Home Equity",
                    label2: "Buy Investments",
                    color1: RGBColor(102, 217, 239),  // Monokai cyan
                    color2: RGBColor(171, 157, 242),  // Monokai purple
                    dark_mode: dark_mode,
                }

                // Chart 4: Monthly Costs Comparison (with crossover indicator)
                PlottersLineChartWithCrossover {
                    chart_id: "monthly-costs",
                    title: "Chart 4: Monthly Costs Comparison",
                    description: "Monthly housing costs over time (buying increases with property taxes and HOA fees, renting increases with rent inflation)",
                    dataset1: results.monthly_buy_costs.clone(),
                    dataset2: results.monthly_rent_costs.clone(),
                    label1: "Buy Monthly Cost",
                    label2: "Rent Monthly Cost",
                    color1: RGBColor(102, 217, 239),  // Monokai cyan
                    color2: RGBColor(255, 97, 136),   // Monokai red
                    dark_mode: dark_mode,
                }

                // Chart 5: Investment Portfolios (with crossover indicator)
                PlottersLineChartWithCrossover {
                    chart_id: "investment-portfolios",
                    title: "Chart 5: Investment Portfolios",
                    description: "Investment portfolio growth from monthly savings differences and down payment (renter invests saved down payment)",
                    dataset1: results.monthly_buy_investment_portfolio.clone(),
                    dataset2: results.monthly_rent_investment_portfolio.clone(),
                    label1: "Buy Investments",
                    label2: "Rent Investments",
                    color1: RGBColor(102, 217, 239),  // Monokai cyan
                    color2: RGBColor(255, 216, 102),  // Monokai yellow
                    dark_mode: dark_mode,
                }
            }
        }
    }
}

#[component]
fn PlottersLineChartWithCrossover(
    chart_id: String,
    title: String,
    description: String,
    dataset1: Vec<f64>,
    dataset2: Vec<f64>,
    label1: String,
    label2: String,
    color1: RGBColor,
    color2: RGBColor,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    let chart_id_clone = chart_id.clone();
    let title_clone = title.clone();
    let label1_clone = label1.clone();
    let label2_clone = label2.clone();

    use_effect(move || {
        let dataset1 = dataset1.clone();
        let dataset2 = dataset2.clone();

        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(canvas) = document.get_element_by_id(&chart_id_clone) {
                    if let Ok(canvas_element) = canvas.dyn_into::<HtmlCanvasElement>() {
                        draw_line_chart_with_crossover(
                            canvas_element,
                            &title_clone,
                            &dataset1,
                            &dataset2,
                            &label1_clone,
                            &label2_clone,
                            color1,
                            color2,
                            dark_mode,
                        );
                    }
                }
            }
        }
    });

    rsx! {
        div { class: if dark_mode { "bg-monokai-bgLight rounded-lg shadow-md p-4" } else { "bg-monokaiLight-bgLight rounded-lg shadow-md p-4" },
            div { class: if dark_mode { "bg-monokai-bgLighter border border-monokai-bgLighter rounded p-2 mb-3" } else { "bg-monokaiLight-bg border border-monokaiLight-border rounded p-2 mb-3" },
                p { class: if dark_mode { "text-xs text-monokai-fgMuted text-center" } else { "text-xs text-monokaiLight-fgMuted text-center" },
                    "{description}"
                }
            }
            canvas {
                id: "{chart_id}",
                width: "700",
                height: "400",
                class: if dark_mode { "w-full rounded" } else { "w-full border border-monokaiLight-border rounded" },
            }
        }
    }
}

#[component]
fn PlottersStackedAreaChart(
    chart_id: String,
    title: String,
    description: String,
    dataset1: Vec<f64>,
    dataset2: Vec<f64>,
    label1: String,
    label2: String,
    color1: RGBColor,
    color2: RGBColor,
    #[props(default = false)] dark_mode: bool,
) -> Element {
    let chart_id_clone = chart_id.clone();
    let title_clone = title.clone();
    let label1_clone = label1.clone();
    let label2_clone = label2.clone();

    use_effect(move || {
        let dataset1 = dataset1.clone();
        let dataset2 = dataset2.clone();

        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(canvas) = document.get_element_by_id(&chart_id_clone) {
                    if let Ok(canvas_element) = canvas.dyn_into::<HtmlCanvasElement>() {
                        draw_stacked_area_chart(
                            canvas_element,
                            &title_clone,
                            &dataset1,
                            &dataset2,
                            &label1_clone,
                            &label2_clone,
                            color1,
                            color2,
                            dark_mode,
                        );
                    }
                }
            }
        }
    });

    rsx! {
        div { class: if dark_mode { "bg-monokai-bgLight rounded-lg shadow-md p-4" } else { "bg-monokaiLight-bgLight rounded-lg shadow-md p-4" },
            div { class: if dark_mode { "bg-monokai-bgLighter border border-monokai-bgLighter rounded p-2 mb-3" } else { "bg-monokaiLight-bg border border-monokaiLight-border rounded p-2 mb-3" },
                p { class: if dark_mode { "text-xs text-monokai-fgMuted text-center" } else { "text-xs text-monokaiLight-fgMuted text-center" },
                    "{description}"
                }
            }
            canvas {
                id: "{chart_id}",
                width: "700",
                height: "400",
                class: if dark_mode { "w-full rounded" } else { "w-full border border-monokaiLight-border rounded" },
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_line_chart_with_crossover(
    canvas: HtmlCanvasElement,
    title: &str,
    dataset1: &[f64],
    dataset2: &[f64],
    label1: &str,
    label2: &str,
    color1: RGBColor,
    color2: RGBColor,
    dark_mode: bool,
) {
    let (bg_color, fg_color, grid_color, grid_bold_color) = if dark_mode {
        (MONOKAI_BG, MONOKAI_FG, MONOKAI_GRID, MONOKAI_GRID_BOLD)
    } else {
        (LIGHT_BG, LIGHT_FG, LIGHT_GRID, LIGHT_GRID_BOLD)
    };

    let backend =
        CanvasBackend::with_canvas_object(canvas).expect("Failed to create canvas backend");
    let root = backend.into_drawing_area();
    root.fill(&bg_color).unwrap();

    let max_value = dataset1
        .iter()
        .chain(dataset2.iter())
        .cloned()
        .fold(0.0f64, f64::max)
        .max(1.0);

    let min_value = dataset1
        .iter()
        .chain(dataset2.iter())
        .cloned()
        .fold(f64::INFINITY, f64::min)
        .min(0.0);

    let num_points = dataset1.len().min(dataset2.len());
    let num_years = (num_points + 11) / 12;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            title,
            ("sans-serif", 22)
                .into_font()
                .style(FontStyle::Bold)
                .color(&fg_color),
        )
        .margin(15)
        .x_label_area_size(45)
        .y_label_area_size(75)
        .build_cartesian_2d(0usize..num_points, min_value..max_value)
        .unwrap();

    chart
        .configure_mesh()
        .x_desc("Year")
        .y_desc("Amount")
        .x_labels(num_years + 1)
        .x_label_formatter(&|x| {
            if *x % 12 == 0 {
                format!("{}", x / 12)
            } else {
                String::new()
            }
        })
        .y_label_formatter(&|v| format_compact(*v))
        .label_style(
            ("sans-serif", 14)
                .into_font()
                .style(FontStyle::Bold)
                .color(&fg_color),
        )
        .axis_desc_style(
            ("sans-serif", 15)
                .into_font()
                .style(FontStyle::Bold)
                .color(&fg_color),
        )
        .axis_style(ShapeStyle::from(&fg_color).stroke_width(1))
        .light_line_style(grid_color)
        .bold_line_style(grid_bold_color.stroke_width(2))
        .draw()
        .unwrap();

    // Find crossover points
    let mut crossover_points: Vec<(f64, f64)> = Vec::new();
    for i in 1..num_points {
        let diff_prev = dataset1[i - 1] - dataset2[i - 1];
        let diff_curr = dataset1[i] - dataset2[i];
        if diff_prev * diff_curr < 0.0 {
            let t = diff_prev.abs() / (diff_prev.abs() + diff_curr.abs());
            let x = (i - 1) as f64 + t;
            let y = dataset1[i - 1] + t * (dataset1[i] - dataset1[i - 1]);
            crossover_points.push((x, y));
        }
    }

    // Draw first dataset
    chart
        .draw_series(LineSeries::new(
            dataset1.iter().enumerate().map(|(i, &v)| (i, v)),
            ShapeStyle {
                color: color1.to_rgba(),
                filled: true,
                stroke_width: 3,
            },
        ))
        .unwrap()
        .label(label1)
        .legend(move |(x, y)| Rectangle::new([(x, y - 6), (x + 18, y + 6)], color1.filled()));

    // Draw second dataset
    chart
        .draw_series(LineSeries::new(
            dataset2.iter().enumerate().map(|(i, &v)| (i, v)),
            ShapeStyle {
                color: color2.to_rgba(),
                filled: true,
                stroke_width: 3,
            },
        ))
        .unwrap()
        .label(label2)
        .legend(move |(x, y)| Rectangle::new([(x, y - 6), (x + 18, y + 6)], color2.filled()));

    // Draw crossover points
    let crossover_color = RGBColor(255, 216, 102); // Monokai yellow
    for (x, y) in &crossover_points {
        let x_idx = x.round() as usize;
        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(x_idx, min_value), (x_idx, max_value)],
                ShapeStyle {
                    color: crossover_color.mix(0.5).to_rgba(),
                    filled: false,
                    stroke_width: 2,
                },
            )))
            .unwrap();
        chart
            .draw_series(std::iter::once(Circle::new(
                (x_idx, *y),
                7,
                crossover_color.filled(),
            )))
            .unwrap();
    }

    // Add break-even info to legend
    if !crossover_points.is_empty() {
        let breakeven_labels: Vec<String> = crossover_points
            .iter()
            .map(|(x, _)| {
                let month = *x as usize;
                let year = month / 12;
                let remaining_months = month % 12;
                if remaining_months == 0 {
                    format!("Break-even: Year {}", year)
                } else {
                    format!("Break-even: Year {} Month {}", year, remaining_months)
                }
            })
            .collect();
        let breakeven_label = breakeven_labels.join(", ");

        chart
            .draw_series(std::iter::once(Circle::new(
                (0usize, min_value),
                0,
                crossover_color.filled(),
            )))
            .unwrap()
            .label(breakeven_label)
            .legend(move |(x, y)| Circle::new((x + 9, y), 7, crossover_color.filled()));
    }

    let legend_bg = if dark_mode {
        RGBColor(53, 50, 54).mix(0.95) // Monokai bgLight
    } else {
        WHITE.mix(0.9)
    };

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .background_style(legend_bg)
        .border_style(ShapeStyle {
            color: fg_color.to_rgba(),
            filled: false,
            stroke_width: 1,
        })
        .label_font(
            ("sans-serif", 13)
                .into_font()
                .style(FontStyle::Bold)
                .color(&fg_color),
        )
        .draw()
        .unwrap();

    root.present().unwrap();
}

#[allow(clippy::too_many_arguments)]
fn draw_stacked_area_chart(
    canvas: HtmlCanvasElement,
    title: &str,
    dataset1: &[f64],
    dataset2: &[f64],
    label1: &str,
    label2: &str,
    color1: RGBColor,
    color2: RGBColor,
    dark_mode: bool,
) {
    let (bg_color, fg_color, grid_color, grid_bold_color) = if dark_mode {
        (MONOKAI_BG, MONOKAI_FG, MONOKAI_GRID, MONOKAI_GRID_BOLD)
    } else {
        (LIGHT_BG, LIGHT_FG, LIGHT_GRID, LIGHT_GRID_BOLD)
    };

    let backend =
        CanvasBackend::with_canvas_object(canvas).expect("Failed to create canvas backend");
    let root = backend.into_drawing_area();
    root.fill(&bg_color).unwrap();

    let num_points = dataset1.len().min(dataset2.len());
    let num_years = (num_points + 11) / 12;

    if num_points == 0 {
        root.present().unwrap();
        return;
    }

    let max_value = (0..num_points)
        .map(|i| dataset1[i] + dataset2[i])
        .fold(0.0f64, f64::max)
        .max(1.0);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            title,
            ("sans-serif", 22)
                .into_font()
                .style(FontStyle::Bold)
                .color(&fg_color),
        )
        .margin(15)
        .x_label_area_size(45)
        .y_label_area_size(75)
        .build_cartesian_2d(0usize..num_points, 0.0..max_value)
        .unwrap();

    chart
        .configure_mesh()
        .x_desc("Year")
        .y_desc("Amount")
        .x_labels(num_years + 1)
        .x_label_formatter(&|x| {
            if *x % 12 == 0 {
                format!("{}", x / 12)
            } else {
                String::new()
            }
        })
        .y_label_formatter(&|v| format_compact(*v))
        .label_style(
            ("sans-serif", 14)
                .into_font()
                .style(FontStyle::Bold)
                .color(&fg_color),
        )
        .axis_desc_style(
            ("sans-serif", 15)
                .into_font()
                .style(FontStyle::Bold)
                .color(&fg_color),
        )
        .axis_style(ShapeStyle::from(&fg_color).stroke_width(1))
        .light_line_style(grid_color)
        .bold_line_style(grid_bold_color.stroke_width(2))
        .draw()
        .unwrap();

    // Draw stacked areas
    chart
        .draw_series(AreaSeries::new(
            (0..num_points).map(|x| (x, dataset1[x])),
            0.0,
            &color1.mix(0.8),
        ))
        .unwrap()
        .label(label1)
        .legend(move |(x, y)| Rectangle::new([(x, y - 6), (x + 18, y + 6)], color1.filled()));

    chart
        .draw_series(AreaSeries::new(
            (0..num_points).map(|x| (x, dataset1[x] + dataset2[x])),
            0.0,
            &color2.mix(0.8),
        ))
        .unwrap()
        .label(label2)
        .legend(move |(x, y)| Rectangle::new([(x, y - 6), (x + 18, y + 6)], color2.filled()));

    // Draw outlines
    chart
        .draw_series(LineSeries::new(
            (0..num_points).map(|x| (x, dataset1[x])),
            ShapeStyle {
                color: color1.to_rgba(),
                filled: false,
                stroke_width: 3,
            },
        ))
        .unwrap();

    chart
        .draw_series(LineSeries::new(
            (0..num_points).map(|x| (x, dataset1[x] + dataset2[x])),
            ShapeStyle {
                color: color2.to_rgba(),
                filled: false,
                stroke_width: 3,
            },
        ))
        .unwrap();

    let legend_bg = if dark_mode {
        RGBColor(53, 50, 54).mix(0.95)
    } else {
        WHITE.mix(0.9)
    };

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .background_style(legend_bg)
        .border_style(ShapeStyle {
            color: fg_color.to_rgba(),
            filled: false,
            stroke_width: 1,
        })
        .label_font(
            ("sans-serif", 13)
                .into_font()
                .style(FontStyle::Bold)
                .color(&fg_color),
        )
        .draw()
        .unwrap();

    root.present().unwrap();
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
