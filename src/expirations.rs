use crate::http::tradier_get;
use crate::types::ExpirationsResponse;
use anyhow::Result;
use chrono::NaiveDate;
use serde_json;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

static EXPIRATIONS_CACHE: OnceLock<RwLock<HashMap<String, Vec<String>>>> = OnceLock::new();

/// Get the global expirations cache, initializing it if needed
fn get_cache() -> &'static RwLock<HashMap<String, Vec<String>>> {
    EXPIRATIONS_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Get option expirations for a symbol
///
/// This function caches results for each symbol to avoid unnecessary API calls
/// since option expirations rarely change.
///
/// # Arguments
/// * `symbol` - The underlying symbol (e.g., "SPY", "AAPL")
/// * `refresh` - If true, forces a fresh API call and updates the cache (default: false)
///
/// # Returns
/// A result containing a vector of expiration dates or an error
///
/// # Example
/// ```rust
/// use rust_tradier::expirations::get_expirations;
///
/// // Get cached expirations (or fetch if not cached)
/// match get_expirations("SPY", false).await {
///     Ok(dates) => {
///         println!("Found {} expiration dates", dates.len());
///         for date in &dates {
///             println!("Expiration: {}", date);
///         }
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
///
/// // Force refresh the cache
/// match get_expirations("SPY", true).await {
///     Ok(dates) => println!("Cache refreshed with {} expirations", dates.len()),
///     Err(e) => eprintln!("Error refreshing cache: {}", e),
/// }
/// ```
///
/// # Note
/// This function requires a valid TRADIER_API_KEY environment variable to be set.
pub async fn get_expirations(symbol: &str, refresh: bool) -> Result<Vec<String>> {
    // Validate required parameters
    if symbol.is_empty() {
        anyhow::bail!("Symbol is required");
    }

    let cache = get_cache();

    // Check cache first unless refresh is requested
    if !refresh {
        if let Ok(cache_read) = cache.read() {
            if let Some(cached_response) = cache_read.get(symbol) {
                return Ok(cached_response.clone());
            }
        }
    }

    // Fetch from API
    let query_params = format!("symbol={}", symbol);
    let uri = format!("/markets/options/expirations?{}", query_params);
    let response_text = tradier_get(&uri).await?;
    let expirations_response: ExpirationsResponse = serde_json::from_str(&response_text)?;
    let dates = expirations_response.expirations.date;

    // Store in cache
    if let Ok(mut cache_write) = cache.write() {
        cache_write.insert(symbol.to_string(), dates.clone());
    }

    Ok(dates)
}

/// Get option expirations for a symbol (convenience function with default refresh=false)
///
/// This is a convenience wrapper around `get_expirations` with `refresh` set to `false`.
///
/// # Arguments
/// * `symbol` - The underlying symbol (e.g., "SPY", "AAPL")
///
/// # Returns
/// A result containing a vector of expiration dates or an error
///
/// # Example
/// ```rust
/// use rust_tradier::get_expirations_cached;
///
/// match get_expirations_cached("SPY").await {
///     Ok(dates) => {
///         println!("Found {} expiration dates", dates.len());
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub async fn get_expirations_cached(symbol: &str) -> Result<Vec<String>> {
    get_expirations(symbol, false).await
}

/// Clear the expirations cache
///
/// This function clears all cached expiration data. Useful for maintenance
/// or when you want to force fresh data for all symbols.
///
/// # Example
/// ```rust
/// use rust_tradier::clear_expirations_cache;
///
/// clear_expirations_cache();
/// println!("Cache cleared");
/// ```
pub fn clear_expirations_cache() {
    if let Ok(mut cache_write) = get_cache().write() {
        cache_write.clear();
    }
}

/// Get cache statistics
///
/// Returns the number of symbols currently cached.
///
/// # Returns
/// The number of cached symbols
///
/// # Example
/// ```rust
/// use rust_tradier::{get_expirations_cached, get_expirations_cache_size};
///
/// // Fetch some data to cache
/// let _ = get_expirations_cached("SPY").await;
/// let _ = get_expirations_cached("AAPL").await;
///
/// println!("Cache contains {} symbols", get_expirations_cache_size());
/// ```
pub fn get_expirations_cache_size() -> usize {
    if let Ok(cache_read) = get_cache().read() {
        cache_read.len()
    } else {
        0
    }
}
