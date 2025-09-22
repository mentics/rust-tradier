use rust_tradier::{chain::get_option_chain, history::get_history, types::Interval};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize)]
struct OptionHistoricalData {
    option_symbol: String,
    underlying: String,
    strike: f64,
    expiration_date: String,
    option_type: String,
    historical_data: Vec<rust_tradier::HistoricalDataPoint>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Fetching historical data for SPY $664 call option expiring 2025-09-26...");

    // The near-the-money option based on delta ~0.5
    let option_symbol = "SPY250926C00664000";

    println!("Getting historical data for: {}", option_symbol);

    // Try to get as much historical data as possible
    // Start from when the option was likely issued (roughly 2-3 months ago)
    let start_date = "2024-01-01"; // Very early to get maximum history
    let end_date = "2025-09-26";

    println!("Fetching data from {} to {}...", start_date, end_date);

    let historical_data = match get_history(&option_symbol, Interval::Daily, start_date, end_date)
        .await
    {
        Ok(data) => {
            println!(
                "Successfully retrieved {} historical data points",
                data.len()
            );
            println!("First data point: {:#?}", data);
            data
        }
        Err(e) => {
            println!("Failed to get historical data: {}", e);
            // Try with a more recent start date in case the option is newer
            println!("Trying with more recent start date...");
            let recent_start = "2025-01-01";

            match get_history(&option_symbol, Interval::Daily, recent_start, end_date).await {
                Ok(data) => {
                    println!(
                        "Successfully retrieved {} historical data points with shorter range",
                        data.len()
                    );
                    data
                }
                Err(e2) => {
                    println!("Still failed with shorter range: {}", e2);
                    // Try weekly data if daily fails
                    println!("Trying weekly interval...");
                    match get_history(&option_symbol, Interval::Weekly, start_date, end_date).await
                    {
                        Ok(data) => {
                            println!("Successfully retrieved {} weekly data points", data.len());
                            data
                        }
                        Err(e3) => {
                            println!("Failed to get any historical data: {}", e3);
                            return Err(e3.into());
                        }
                    }
                }
            }
        }
    };

    // Get current option details for context
    let chain_result = get_option_chain("SPY", "2025-09-26", true).await;

    let option_details = if let Ok(chain) = chain_result {
        chain
            .options
            .option
            .into_iter()
            .find(|opt| opt.symbol == option_symbol)
    } else {
        None
    };

    let historical_option_data = OptionHistoricalData {
        option_symbol: option_symbol.to_string(),
        underlying: "SPY".to_string(),
        strike: 664.0,
        expiration_date: "2025-09-26".to_string(),
        option_type: "call".to_string(),
        historical_data
    };

    // Write to file
    let json_output = serde_json::to_string_pretty(&historical_option_data)?;
    std::fs::write("spy_664_call_history.json", json_output)?;

    println!("Historical data written to spy_664_call_history.json");

    // Print summary
    println!("\nSummary:");
    println!(
        "Option: {} (SPY ${:.0} Call expiring 2025-09-26)",
        option_symbol, 664.0
    );
    println!(
        "Historical data points: {}",
        historical_option_data.historical_data.len()
    );

    if let Some(details) = option_details {
        println!(
            "Current price: ${:.2} (bid) / ${:.2} (ask)",
            details.bid.unwrap_or(0.0),
            details.ask.unwrap_or(0.0)
        );
        println!(
            "Current delta: {:.4}",
            details.greeks.as_ref().and_then(|g| g.delta).unwrap_or(0.0)
        );
        println!("Open interest: {}", details.open_interest.unwrap_or(0));
        println!("Volume: {}", details.volume.unwrap_or(0));
    }

    if !historical_option_data.historical_data.is_empty() {
        let total_volume: u64 = historical_option_data
            .historical_data
            .iter()
            .filter_map(|point| point.volume)
            .sum();

        println!("Total historical volume: {}", total_volume);

        // Show first and last data points
        if let Some(first_point) = historical_option_data.historical_data.first() {
            println!(
                "First data point: {} - Close: ${:.2}, Volume: {}",
                first_point.date,
                first_point.close.unwrap_or(0.0),
                first_point.volume.unwrap_or(0)
            );
        }

        if let Some(last_point) = historical_option_data.historical_data.last() {
            println!(
                "Last data point: {} - Close: ${:.2}, Volume: {}",
                last_point.date,
                last_point.close.unwrap_or(0.0),
                last_point.volume.unwrap_or(0)
            );
        }

        // Show some price statistics
        let valid_closes: Vec<f64> = historical_option_data
            .historical_data
            .iter()
            .filter_map(|point| point.close)
            .collect();

        if !valid_closes.is_empty() {
            let min_price = valid_closes.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_price = valid_closes
                .iter()
                .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let avg_price = valid_closes.iter().sum::<f64>() / valid_closes.len() as f64;

            println!("Price range: ${:.2} - ${:.2}", min_price, max_price);
            println!("Average price: ${:.2}", avg_price);
        }
    }

    Ok(())
}
