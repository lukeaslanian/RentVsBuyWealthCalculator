use gloo_net::http::Request;
use serde::Deserialize;

// NOTE: FRED API key should be set at compile time via environment variable for production
// For client-side fetching, the key will be visible in the compiled WASM
// This is acceptable for FRED API as it's free, read-only, and public
// Get your own free API key at: https://fred.stlouisfed.org/docs/api/api_key.html
const FRED_BASE_URL: &str = "https://api.stlouisfed.org/fred/series/observations";
const FALLBACK_RATE: f64 = 5.99;
// Development fallback API key (FRED API is free and read-only)
const FRED_API_KEY: &str = "ea90fe3709c9c7eae3dbe45bce2a2788";

fn get_fred_api_key() -> &'static str {
    FRED_API_KEY
}

/// Percent-encode all non-unreserved characters for use as a query parameter value
fn percent_encode(s: &str) -> String {
    let mut encoded = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{:02X}", byte)),
        }
    }
    encoded
}

#[derive(Debug, Deserialize)]
struct FredResponse {
    observations: Vec<FredObservation>,
}

#[derive(Debug, Deserialize)]
struct FredObservation {
    date: String,
    value: String,
}

/// Result of fetching the current mortgage rate
#[derive(Debug, Clone)]
pub struct MortgageRateResult {
    pub rate: f64,
    pub date: String,
    pub is_live: bool,
    pub error_msg: Option<String>,
}

/// Fetch the current 30-year fixed mortgage rate from FRED
/// Falls back to 5.99% if the API call fails
pub async fn fetch_current_mortgage_rate() -> MortgageRateResult {
    web_sys::console::log_1(&"FRED API: Starting fetch...".into());

    match fetch_rate_from_fred().await {
        Ok(result) => {
            web_sys::console::log_1(
                &format!(
                    "FRED API: Success! Rate: {}% from {}",
                    result.rate, result.date
                )
                .into(),
            );
            result
        }
        Err(e) => {
            web_sys::console::log_1(&format!("FRED API: Error - {}, using fallback", e).into());
            MortgageRateResult {
                rate: FALLBACK_RATE,
                date: "fallback".to_string(),
                is_live: false,
                error_msg: Some(e),
            }
        }
    }
}

async fn try_fetch_text(url: &str) -> Result<String, String> {
    let response = Request::get(url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    web_sys::console::log_1(&format!("FRED API: Response status: {}", response.status()).into());

    if !response.ok() {
        return Err(format!("HTTP error: status {}", response.status()));
    }

    response
        .text()
        .await
        .map_err(|e| format!("Text error: {}", e))
}

fn parse_fred_response(text: &str) -> Result<MortgageRateResult, String> {
    web_sys::console::log_1(
        &format!(
            "FRED API: Response body: {}",
            &text[..text.len().min(500)]
        )
        .into(),
    );

    let fred_response: FredResponse = serde_json::from_str(text).map_err(|e| {
        format!(
            "Parse error: {}. Body: {}",
            e,
            &text[..text.len().min(200)]
        )
    })?;

    let observation = fred_response
        .observations
        .first()
        .ok_or_else(|| "No observations in response".to_string())?;

    web_sys::console::log_1(
        &format!(
            "FRED API: Got observation - date: {}, value: {}",
            observation.date, observation.value
        )
        .into(),
    );

    // FRED sometimes returns "." for missing values
    let rate = observation
        .value
        .parse::<f64>()
        .map_err(|_| format!("Invalid rate value: '{}'", observation.value))?;

    Ok(MortgageRateResult {
        rate,
        date: observation.date.clone(),
        is_live: true,
        error_msg: None,
    })
}

async fn fetch_rate_from_fred() -> Result<MortgageRateResult, String> {
    let api_key = get_fred_api_key();
    if api_key.is_empty() {
        return Err(
            "No FRED API key available. Set FRED_API_KEY environment variable at build time."
                .to_string(),
        );
    }

    let fred_url = format!(
        "{}?series_id=MORTGAGE30US&api_key={}&file_type=json&sort_order=desc&limit=1",
        FRED_BASE_URL, api_key
    );

    // Fully percent-encode the target URL so it can be safely passed as a query param
    let encoded_url = percent_encode(&fred_url);

    // Try multiple CORS proxies in order
    let proxy_urls = [
        format!("https://api.allorigins.win/raw?url={}", encoded_url),
        format!("https://corsproxy.io/?url={}", encoded_url),
        format!("https://corsproxy.io/?{}", encoded_url),
    ];

    let mut last_error = String::from("No proxies attempted");

    for proxy_url in &proxy_urls {
        web_sys::console::log_1(&format!("FRED API: Trying: {}", proxy_url).into());

        match try_fetch_text(proxy_url).await {
            Ok(text) => match parse_fred_response(&text) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    web_sys::console::log_1(
                        &format!("FRED API: Parse failed for this proxy: {}", e).into(),
                    );
                    last_error = e;
                }
            },
            Err(e) => {
                web_sys::console::log_1(
                    &format!("FRED API: Fetch failed for this proxy: {}", e).into(),
                );
                last_error = e;
            }
        }
    }

    Err(last_error)
}
