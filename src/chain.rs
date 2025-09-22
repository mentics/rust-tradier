use crate::http::tradier_get;
use crate::types::{OptionChainResponse, OptionData};
use anyhow::Result;
use serde_json;
use tracing::error;

/// Get an option chain for the specified parameters
///
/// # Arguments
/// * `symbol` - The underlying symbol (e.g., "SPY", "AAPL")
/// * `expiration` - Expiration date in YYYY-MM-DD format
/// * `greeks` - Whether to include Greeks data
///
/// # Returns
/// A result containing the option chain response or an error
///
/// # Example
/// ```rust
/// use rust_tradier::chain::get_option_chain;
///
/// match get_option_chain("SPY", "2024-12-20", true, None).await {
///     Ok(response) => {
///         println!("Found {} options", response.options.option.len());
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub async fn get_option_chain(
    symbol: &str,
    expiration: &str,
    greeks: bool
) -> Result<OptionChainResponse> {
    // Validate required parameters
    if symbol.is_empty() {
        anyhow::bail!("Symbol is required");
    }

    if expiration.is_empty() {
        anyhow::bail!("Expiration is required");
    }

    // Validate required parameters
    if symbol.is_empty() {
        anyhow::bail!("Symbol is required");
    }

    if expiration.is_empty() {
        anyhow::bail!("Expiration is required");
    }

    // Build query parameters
    let mut query_params = Vec::new();
    query_params.push(format!("symbol={}", symbol));
    query_params.push(format!("expiration={}", expiration));
    query_params.push(format!("greeks={}", if greeks { "true" } else { "false" }));

    // Build the URI
    let uri = format!("/markets/options/chains?{}", query_params.join("&"));

    // Make the API call
    let response_text = tradier_get(&uri).await?;

    // Parse the JSON response
    let option_chain = match serde_json::from_str(&response_text) {
        Ok(option_chain) => option_chain,
        Err(e) => {
            error!("Failed to parse option chain response: {}", e);
            error!("Response text: {}", response_text);
            anyhow::bail!("Failed to parse option chain response");
        }
    };

    Ok(option_chain)
}

/// Get options sorted by strike price
///
/// # Arguments
/// * `chain` - The option chain response
/// * `ascending` - Whether to sort in ascending order (true) or descending order (false)
///
/// # Returns
/// A vector of options sorted by strike price
pub fn get_options_sorted_by_strike(
    chain: &OptionChainResponse,
    ascending: bool
) -> Vec<&OptionData> {
    let mut options = chain.options.option.iter().collect::<Vec<_>>();
    options.sort_by(|a, b| {
        if ascending {
            a.strike.partial_cmp(&b.strike).unwrap()
        } else {
            b.strike.partial_cmp(&a.strike).unwrap()
        }
    });
    options
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    /// Test to verify that single option objects are now handled correctly
    ///
    /// Previously this would fail with: "invalid type: map, expected a sequence at line 1 column 19"
    /// Now it should succeed thanks to the custom deserializer
    #[test]
    fn test_option_chain_parsing_with_single_option_as_map() {
        // Simulate the JSON response where "option" is a single object (not an array)
        let json_response = r#"{
            "options": {
                "option": {
                    "symbol": "SPY251010C00500000",
                    "description": "SPDR S&P 500 ETF Trust",
                    "exch": "Z",
                    "type": "option",
                    "last": 12.45,
                    "change": 0.15,
                    "volume": 1250,
                    "open": 12.30,
                    "high": 12.60,
                    "low": 12.20,
                    "close": 12.30,
                    "bid": 12.40,
                    "ask": 12.50,
                    "underlying": "SPY",
                    "strike": 500.0,
                    "change_percentage": 1.22,
                    "average_volume": 5000,
                    "last_volume": 100,
                    "trade_date": 1728518400000,
                    "prevclose": 12.30,
                    "week52_high": 15.20,
                    "week52_low": 8.90,
                    "bidsize": 10,
                    "bidexch": "Z",
                    "bid_date": 1728518400000,
                    "asksize": 15,
                    "askexch": "Z",
                    "ask_date": 1728518400000,
                    "open_interest": 2500,
                    "contract_size": 100,
                    "expiration_date": "2025-10-10",
                    "expiration_type": "monthly",
                    "option_type": "call",
                    "root_symbol": "SPY",
                    "greeks": {
                        "delta": 0.65,
                        "gamma": 0.02,
                        "theta": -0.15,
                        "vega": 0.25,
                        "rho": 0.08,
                        "phi": 0.0,
                        "bid_iv": 0.18,
                        "mid_iv": 0.185,
                        "ask_iv": 0.19,
                        "smv_vol": 0.182,
                        "updated_at": "2024-10-10T16:00:00Z"
                    }
                }
            }
        }"#;

        // This should now succeed with the custom deserializer
        let result: OptionChainResponse = serde_json::from_str(json_response).unwrap();
        assert_eq!(result.options.option.len(), 1);
        assert_eq!(result.options.option[0].symbol, "SPY251010C00500000");
        assert_eq!(result.options.option[0].right, "call");
    }

    /// Test to verify normal parsing works with an array of options
    #[test]
    fn test_option_chain_parsing_with_array() {
        let json_response = r#"{
            "options": {
                "option": [
                    {
                        "symbol": "SPY251010C00500000",
                        "description": "SPDR S&P 500 ETF Trust",
                        "exch": "Z",
                        "type": "option",
                        "underlying": "SPY",
                        "strike": 500.0,
                        "expiration_date": "2025-10-10",
                        "expiration_type": "monthly",
                        "option_type": "call",
                        "root_symbol": "SPY"
                    }
                ]
            }
        }"#;

        let result: OptionChainResponse = serde_json::from_str(json_response).unwrap();
        assert_eq!(result.options.option.len(), 1);
        assert_eq!(result.options.option[0].symbol, "SPY251010C00500000");
        assert_eq!(result.options.option[0].right, "call");
    }
}
