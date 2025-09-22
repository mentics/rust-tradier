use crate::http::tradier_get;
use crate::types::{HistoricalDataPoint, HistoryResponse, Interval};
use anyhow::Result;
use cached::proc_macro::cached;
use serde_json;

/// Get historical price data for a symbol
///
/// # Arguments
/// * `symbol` - The symbol to get historical data for (e.g., "AAPL", "SPY")
/// * `interval` - The interval for the data (Daily, Weekly, or Monthly)
/// * `start` - The start date as a string in YYYY-MM-DD format
/// * `end` - The end date as a string in YYYY-MM-DD format
///
/// # Returns
/// A result containing a vector of historical data points or an error
///
/// # Example
/// ```rust
/// use rust_tradier::{history::get_history, types::Interval};
///
/// match get_history("AAPL", Interval::Daily, "2023-01-01", "2023-12-31").await {
///     Ok(data) => {
///         for point in data {
///             if let (Some(close), Some(volume)) = (point.close, point.volume) {
///                 println!("{}: Close: ${:.2}, Volume: {}", point.date, close, volume);
///             }
///         }
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
///
/// Get historical price data for a symbol
///
/// # Arguments
/// * `symbol` - The symbol to get historical data for (e.g., "AAPL", "SPY")
/// * `interval` - The interval for the data (Daily, Weekly, or Monthly)
/// * `start` - The start date as a string in YYYY-MM-DD format
/// * `end` - The end date as a string in YYYY-MM-DD format
///
/// # Returns
/// A result containing a vector of historical data points or an error
///
/// # Example
/// ```rust
/// use rust_tradier::{history::get_history, types::Interval};
///
/// match get_history("AAPL", Interval::Daily, "2023-01-01", "2023-12-31").await {
///     Ok(data) => {
///         for point in data {
///             if let (Some(close), Some(volume)) = (point.close, point.volume) {
///                 println!("{}: Close: ${:.2}, Volume: {}", point.date, close, volume);
///             }
///         }
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
///
/// # Note
/// This function requires a valid TRADIER_API_KEY environment variable to be set.
/// Results are cached for 5 minutes to reduce API calls.
#[cached(
    ty = "cached::TimedCache<String, Vec<HistoricalDataPoint>>",
    create = r#"{ cached::TimedCache::with_lifespan(60*60*8) }"#,
    key = "String",
    convert = r#"{ format!("{}:{}:{}:{}", symbol, interval, start, end) }"#,
    result = true
)]
pub async fn get_history(
    symbol: &str,
    interval: Interval,
    start: &str,
    end: &str
) -> Result<Vec<HistoricalDataPoint>> {
    // Validate required parameters
    if symbol.is_empty() {
        anyhow::bail!("Symbol is required");
    }
    let query_params = format!(
        "symbol={}&interval={}&start={}&end={}",
        symbol, interval, start, end
    );

    // Build the URI
    let uri = format!("/markets/history?{}", query_params);

    // Make the API call
    let response_text = tradier_get(&uri).await?;

    // Parse the JSON response
    let history_response: HistoryResponse = serde_json::from_str(&response_text)?;

    Ok(history_response.history.day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_history_with_empty_symbol() {
        // This should fail with an empty symbol
        let result = get_history("", Interval::Daily, "2023-01-01", "2023-12-31").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Symbol is required");
    }

    #[tokio::test]
    async fn test_get_history_with_invalid_date_range() {
        // This should fail with start date after end date
        let result = get_history("AAPL", Interval::Daily, "2023-12-31", "2023-01-01").await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Start date must be before or equal to end date"
        );
    }

    #[tokio::test]
    async fn test_get_history_with_valid_intervals() {
        // Test that all valid intervals work
        // Note: This will still fail with API key error, but that's expected
        let intervals = vec![Interval::Daily, Interval::Weekly, Interval::Monthly];

        for interval in intervals {
            let result = get_history("AAPL", interval, "2023-01-01", "2023-12-31").await;

            // The API call will fail due to missing API key, but the validation should pass
            assert!(result.is_err());
            // Should not be our validation error
            assert!(!result.unwrap_err().to_string().contains("Interval must be"));
        }
    }
}
