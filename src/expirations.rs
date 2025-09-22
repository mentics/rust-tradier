use crate::http::tradier_get;
use crate::types::ExpirationsResponse;
use anyhow::Result;
use cached::proc_macro::cached;
use serde_json;

/// Get option expirations for a symbol
///
/// This function caches results for each symbol to avoid unnecessary API calls
/// since option expirations rarely change.
///
/// # Arguments
/// * `symbol` - The underlying symbol (e.g., "SPY", "AAPL")
///
/// # Returns
/// A result containing a vector of expiration dates or an error
///
/// # Example
/// ```rust
/// use rust_tradier::expirations::get_expirations_cached;
///
/// match get_expirations_cached("SPY").await {
///     Ok(dates) => {
///         println!("Found {} expiration dates", dates.len());
///         for date in &dates {
///             println!("Expiration: {}", date);
///         }
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
///
/// # Note
/// This function requires a valid TRADIER_API_KEY environment variable to be set.
/// Results are cached for 8 hours to reduce API calls.
#[cached(
    ty = "cached::TimedCache<String, Vec<String>>",
    create = r#"{ cached::TimedCache::with_lifespan(60*60*8) }"#,
    key = "String",
    convert = r#"{ symbol.to_string() }"#,
    result = true
)]
pub async fn get_expirations(symbol: &str) -> Result<Vec<String>> {
    // Validate required parameters
    if symbol.is_empty() {
        anyhow::bail!("Symbol is required");
    }

    // Fetch from API
    let query_params = format!("symbol={}", symbol);
    let uri = format!("/markets/options/expirations?{}", query_params);
    let response_text = tradier_get(&uri).await?;
    let expirations_response: ExpirationsResponse = serde_json::from_str(&response_text)?;
    let dates = expirations_response.expirations.date;

    Ok(dates)
}
