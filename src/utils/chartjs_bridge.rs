use wasm_bindgen::prelude::*;

/// Creates a line chart with two datasets using Chart.js
/// This is used for cost comparison, wealth, monthly costs, and investment charts
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    pub fn createLineChart(
        canvas_id: &str,
        labels: Vec<f64>,
        dataset1_label: &str,
        dataset1_data: Vec<f64>,
        dataset1_color: &str,
        dataset2_label: &str,
        dataset2_data: Vec<f64>,
        dataset2_color: &str,
    );
}

/// Creates a stacked area chart with two datasets using Chart.js
/// This is used for the buyer wealth components chart
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    pub fn createStackedAreaChart(
        canvas_id: &str,
        labels: Vec<f64>,
        dataset1_label: &str,
        dataset1_data: Vec<f64>,
        dataset1_color: &str,
        dataset2_label: &str,
        dataset2_data: Vec<f64>,
        dataset2_color: &str,
    );
}

/// Creates a histogram (bar chart) using Chart.js
/// This is used for Monte Carlo simulation results
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    pub fn createHistogram(
        canvas_id: &str,
        bin_labels: Vec<JsValue>,
        bin_values: Vec<f64>,
        bin_colors: Vec<JsValue>,
    );
}

/// Helper function to sample monthly data to yearly data (every 12th month)
/// Returns 30 yearly data points for cleaner visualization
pub fn sample_to_yearly(monthly_data: &[f64]) -> Vec<f64> {
    (0..30)
        .map(|year| {
            let idx = (year * 12).min(monthly_data.len() - 1);
            monthly_data[idx]
        })
        .collect()
}

/// Generates year labels from 0 to 30
pub fn generate_year_labels() -> Vec<f64> {
    (0..30).map(|y| y as f64).collect()
}
