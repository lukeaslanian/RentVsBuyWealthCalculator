use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
struct FredResponse {
    observations: Vec<FredObservation>,
}

#[derive(Debug, Deserialize)]
struct FredObservation {
    date: String,
    value: String,
}

fn fetch_current_mortgage_rate() -> Option<(f64, String)> {
    // FRED API key can be set via environment variable
    // Get a free API key at https://fred.stlouisfed.org/docs/api/api_key.html
    let api_key = env::var("FRED_API_KEY").ok()?;

    let url = format!(
        "https://api.stlouisfed.org/fred/series/observations?series_id=MORTGAGE30US&api_key={}&file_type=json&limit=1&sort_order=desc",
        api_key
    );

    println!("cargo:rerun-if-env-changed=FRED_API_KEY");

    let response = reqwest::blocking::get(&url).ok()?;
    let fred_data: FredResponse = response.json().ok()?;

    fred_data.observations.first().and_then(|obs| {
        obs.value
            .parse::<f64>()
            .ok()
            .map(|rate| (rate, obs.date.clone()))
    })
}

fn main() {
    // Try to fetch current mortgage rate from FRED
    let (default_rate, rate_date) = match fetch_current_mortgage_rate() {
        Some((rate, date)) => {
            println!("cargo:warning=Successfully fetched current 30-year mortgage rate from FRED: {:.2}% (as of {})", rate, date);
            println!("cargo:rustc-env=DEFAULT_MORTGAGE_RATE_DATE={}", date);
            (rate, Some(date))
        }
        None => {
            let fallback = 5.99;
            println!("cargo:warning=Could not fetch mortgage rate from FRED (missing FRED_API_KEY or network error)");
            println!(
                "cargo:warning=Using fallback mortgage rate: {:.2}%",
                fallback
            );
            println!("cargo:warning=To use live rates, set FRED_API_KEY environment variable");
            println!("cargo:warning=Get a free API key at: https://fred.stlouisfed.org/docs/api/api_key.html");
            (fallback, None)
        }
    };

    // Write the rate to an environment variable for compile-time inclusion
    println!("cargo:rustc-env=DEFAULT_MORTGAGE_RATE={}", default_rate);

    // Mark if we successfully fetched from FRED
    if rate_date.is_some() {
        println!("cargo:rustc-env=MORTGAGE_RATE_FROM_FRED=true");
    }
}
