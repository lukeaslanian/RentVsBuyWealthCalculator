use gloo_net::http::Request;
use serde::Deserialize;

const FRED_API_KEY: &str = "ea90fe3709c9c7eae3dbe45bce2a2788";
const FRED_BASE_URL: &str = "https://api.stlouisfed.org/fred/series/observations";
// Use corsproxy.io to bypass CORS restrictions
const CORS_PROXY: &str = "https://corsproxy.io/?";
const FALLBACK_RATE: f64 = 5.99;

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

async fn fetch_rate_from_fred() -> Result<MortgageRateResult, String> {
    let fred_url = format!(
        "{}?series_id=MORTGAGE30US&api_key={}&file_type=json&sort_order=desc&limit=1",
        FRED_BASE_URL, FRED_API_KEY
    );
    // Wrap with CORS proxy - URL encode the target URL
    let encoded_url = fred_url
        .replace("%", "%25")
        .replace("&", "%26")
        .replace("?", "%3F")
        .replace("=", "%3D");
    let url = format!("{}{}", CORS_PROXY, encoded_url);

    web_sys::console::log_1(&format!("FRED API: Fetching from URL: {}", url).into());

    let response = Request::get(&url).send().await.map_err(|e| {
        let msg = format!("Network error: {}", e);
        web_sys::console::log_1(&format!("FRED API: {}", msg).into());
        msg
    })?;

    web_sys::console::log_1(&format!("FRED API: Response status: {}", response.status()).into());

    if !response.ok() {
        let msg = format!("API error: status {}", response.status());
        web_sys::console::log_1(&format!("FRED API: {}", msg).into());
        return Err(msg);
    }

    let text = response
        .text()
        .await
        .map_err(|e| format!("Text error: {}", e))?;
    web_sys::console::log_1(
        &format!("FRED API: Response body: {}", &text[..text.len().min(500)]).into(),
    );

    let fred_response: FredResponse = serde_json::from_str(&text).map_err(|e| {
        let msg = format!("Parse error: {}", e);
        web_sys::console::log_1(&format!("FRED API: {}", msg).into());
        msg
    })?;

    let observation = fred_response
        .observations
        .first()
        .ok_or_else(|| "No data available".to_string())?;

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
        .map_err(|_| "Invalid rate value".to_string())?;

    Ok(MortgageRateResult {
        rate,
        date: observation.date.clone(),
        is_live: true,
        error_msg: None,
    })
}
