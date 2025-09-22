use crate::http::tradier_get_versioned;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;

/// Represents a dividend payment from the API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CashDividend {
    /// The share class ID
    pub share_class_id: Option<String>,
    /// The dividend type
    pub dividend_type: Option<String>,
    /// The ex-dividend date
    pub ex_date: Option<String>,
    /// The dividend amount
    pub cash_amount: Option<f64>,
    /// The currency ID
    pub currency_id: Option<String>,
    /// The declaration date
    pub declaration_date: Option<String>,
    /// The dividend frequency
    pub frequency: Option<i32>,
    /// The payment date
    pub pay_date: Option<String>,
    /// The record date
    pub record_date: Option<String>
}

/// Represents a dividend payment (simplified version for external use)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Dividend {
    /// The ex-dividend date
    pub ex_dividend_date: Option<String>,
    /// The payment date
    pub payment_date: Option<String>,
    /// The dividend amount
    pub amount: Option<f64>,
    /// The dividend frequency
    pub frequency: Option<i32>,
    /// The dividend type
    pub dividend_type: Option<String>,
    /// The currency
    pub currency: Option<String>
}

/// Tables containing dividend data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DividendTables {
    pub cash_dividends: Option<Vec<CashDividend>>
}

/// Individual result entry
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DividendResult {
    /// The type of result (e.g., "Stock")
    #[serde(rename = "type")]
    pub result_type: Option<String>,
    /// The ID of the result
    pub id: Option<String>,
    /// The tables containing dividend data
    pub tables: Option<DividendTables>
}

/// Response structure for individual symbol request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DividendSymbolResponse {
    /// The requested symbol
    pub request: Option<String>,
    /// The type of request
    #[serde(rename = "type")]
    pub response_type: Option<String>,
    /// The results for this symbol
    pub results: Option<Vec<DividendResult>>
}

/// Response structure for dividend API (beta endpoint)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DividendResponse {
    /// Array of responses for each requested symbol
    #[serde(rename = "")]
    pub responses: Vec<DividendSymbolResponse>
}

/// Get dividend information for a symbol
///
/// # Arguments
/// * `symbol` - The symbol to get dividend information for (e.g., "AAPL", "MSFT")
///
/// # Returns
/// A result containing a vector of dividend data or an error
///
/// # Example
/// ```rust
/// use rust_tradier::fundamental::get_dividends;
///
/// match get_dividends("AAPL").await {
///     Ok(dividends) => {
///         for dividend in dividends {
///             if let (Some(date), Some(amount)) = (dividend.ex_dividend_date, dividend.amount) {
///                 println!("Ex-dividend date: {}, Amount: ${:.2}", date, amount);
///             }
///         }
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
///
/// # Note
/// This function requires a valid TRADIER_API_KEY environment variable to be set.
pub async fn get_dividends(symbol: &str) -> Result<Vec<Dividend>> {
    // Validate required parameters
    if symbol.is_empty() {
        anyhow::bail!("Symbol is required");
    }

    // Build query parameters
    let query_params = format!("symbols={}", symbol);

    // Build the URI
    let uri = format!("/markets/fundamentals/dividends?{}", query_params);

    println!("URI: {}", uri);
    // Make the API call
    let response_text = tradier_get_versioned(&uri, "beta").await?;
    // println!("Response text: {}", response_text);

    // Check if we got HTML instead of JSON (indicates API error)
    if response_text.trim().starts_with("<!") || response_text.trim().starts_with("<html") {
        anyhow::bail!(
            "API returned HTML instead of JSON - likely authentication or endpoint error"
        );
    }

    // Parse the JSON response as an array of symbol responses
    let symbol_responses: Vec<DividendSymbolResponse> = serde_json::from_str(&response_text)?;

    // Extract dividends from all results
    let mut dividends = Vec::new();
    for symbol_response in symbol_responses {
        if let Some(results) = symbol_response.results {
            for result in results {
                if let Some(tables) = result.tables {
                    if let Some(cash_dividends) = tables.cash_dividends {
                        for cash_dividend in cash_dividends {
                            dividends.push(Dividend {
                                ex_dividend_date: cash_dividend.ex_date,
                                payment_date: cash_dividend.pay_date,
                                amount: cash_dividend.cash_amount,
                                frequency: cash_dividend.frequency,
                                dividend_type: cash_dividend.dividend_type,
                                currency: cash_dividend.currency_id
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(dividends)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires TRADIER_API_KEY environment variable
    async fn test_get_dividends() {
        // Test getting dividends for SPY
        let result = get_dividends("SPY").await;

        match result {
            Ok(dividends) => {
                println!("Successfully retrieved {} dividends", dividends.len());
                assert!(!dividends.is_empty(), "Should have at least one dividend");

                // Check the first dividend has required fields
                if let Some(first_dividend) = dividends.first() {
                    assert!(
                        first_dividend.ex_dividend_date.is_some(),
                        "Should have ex-dividend date"
                    );
                    assert!(first_dividend.amount.is_some(), "Should have amount");
                    assert!(
                        first_dividend.amount.unwrap() > 0.0,
                        "Amount should be positive"
                    );

                    println!(
                        "First dividend: ex-date={}, amount={:?}, type={:?}",
                        first_dividend.ex_dividend_date.as_deref().unwrap_or("None"),
                        first_dividend.amount,
                        first_dividend.dividend_type
                    );
                }
            }
            Err(e) => {
                panic!("Failed to get dividends: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_get_dividends_with_empty_symbol() {
        // This should fail with an empty symbol
        let result = get_dividends("").await;
        assert!(result.is_err(), "Empty symbol should result in error");
    }
}
