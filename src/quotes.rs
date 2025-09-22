use crate::http::tradier_get;
use crate::types::{QuoteResponse, TradierQuote};
use anyhow::Result;
use serde_json;

/// Get the current quote for a symbol
///
/// # Arguments
/// * `symbol` - The symbol to get the quote for (e.g., "AAPL", "SPY")
///
/// # Returns
/// A result containing the quote data or an error
///
/// # Example
/// ```rust
/// use rust_tradier::quotes::get_quote;
///
/// match get_quote("AAPL").await {
///     Ok(quote) => {
///         if let Some(last_price) = quote.last {
///             println!("AAPL last price: ${:.2}", last_price);
///         }
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
///
/// # Note
/// This function requires a valid TRADIER_API_KEY environment variable to be set.
pub async fn get_quote(symbol: &str) -> Result<TradierQuote> {
    // Validate required parameters
    if symbol.is_empty() {
        anyhow::bail!("Symbol is required");
    }

    // Build query parameters
    let query_params = format!("symbols={}", symbol);

    // Build the URI
    let uri = format!("/markets/quotes?{}", query_params);

    // Make the API call
    let response_text = tradier_get(&uri).await?;

    // Parse the JSON response
    let quote_response: QuoteResponse = serde_json::from_str(&response_text)?;

    // Extract the first (and typically only) quote from the response
    if quote_response.quotes.quote.is_empty() {
        anyhow::bail!("No quote data returned for symbol: {}", symbol);
    }

    Ok(quote_response.quotes.quote.into_iter().next().unwrap())
}

/// Get quotes for multiple symbols
///
/// # Arguments
/// * `symbols` - A vector of symbols to get quotes for
///
/// # Returns
/// A result containing a vector of quote data or an error
///
/// # Example
/// ```rust
/// use rust_tradier::quotes::get_quotes;
///
/// match get_quotes(vec!["AAPL", "GOOGL", "MSFT"]).await {
///     Ok(quotes) => {
///         for quote in quotes {
///             if let Some(last_price) = quote.last {
///                 println!("{}: ${:.2}", quote.symbol, last_price);
///             }
///         }
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
///
/// # Note
/// This function requires a valid TRADIER_API_KEY environment variable to be set.
pub async fn get_quotes(symbols: Vec<&str>) -> Result<Vec<TradierQuote>> {
    // Validate required parameters
    if symbols.is_empty() {
        anyhow::bail!("At least one symbol is required");
    }

    for symbol in &symbols {
        if symbol.is_empty() {
            anyhow::bail!("All symbols must be non-empty");
        }
    }

    // Build query parameters
    let symbols_param = symbols.join(",");
    let query_params = format!("symbols={}", symbols_param);

    // Build the URI
    let uri = format!("/markets/quotes?{}", query_params);

    // Make the API call
    let response_text = tradier_get(&uri).await?;

    // Parse the JSON response
    let quote_response: QuoteResponse = serde_json::from_str(&response_text)?;

    Ok(quote_response.quotes.quote)
}
