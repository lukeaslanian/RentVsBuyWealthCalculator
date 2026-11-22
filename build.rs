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
    println!("cargo:rerun-if-env-changed=FRED_API_KEY");

    // The key comes only from the environment (GitHub Actions secret in CI, or an
    // exported variable locally). Without it, the build uses the fallback rate.
    let api_key = env::var("FRED_API_KEY").ok().filter(|k| !k.trim().is_empty())?;

    let url = format!(
        "https://api.stlouisfed.org/fred/series/observations?series_id=MORTGAGE30US&api_key={}&file_type=json&limit=1&sort_order=desc",
        api_key
    );

    let response = reqwest::blocking::get(&url).ok()?;
    let fred_data: FredResponse = response.json().ok()?;

    fred_data.observations.first().and_then(|obs| {
        obs.value
            .parse::<f64>()
            .ok()
            .map(|rate| (rate, obs.date.clone()))
    })
}

fn build_tailwind_css() {
    // Rerun if any of these change
    println!("cargo:rerun-if-changed=assets/input.css");
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=src");

    // Use the locally-installed tailwindcss binary (requires `npm install` to have been run)
    let binary = if cfg!(windows) {
        "node_modules\\.bin\\tailwindcss.cmd"
    } else {
        "node_modules/.bin/tailwindcss"
    };

    let status = std::process::Command::new(binary)
        .args([
            "-i", "./assets/input.css",
            "-o", "./assets/tailwind.css",
            "--minify",
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:warning=Tailwind CSS built successfully");
        }
        Ok(s) => {
            println!("cargo:warning=tailwindcss exited with status {s} — run `npm install` first");
        }
        Err(e) => {
            println!("cargo:warning=Could not run tailwindcss: {e} — run `npm install` first");
        }
    }
}

fn main() {
    build_tailwind_css();

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
