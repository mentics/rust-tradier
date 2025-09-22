use rust_tradier::{history::get_history, types::Interval};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Calculate date range - 10 years ago to today
    let end_date = "2025-09-21";
    let start_date = "2015-09-21";

    // Get data directory and create samples/hist path
    let data_dir =
        std::env::var("DATA_DIR").unwrap_or("/home/jshellman/dev/git/perig/data".to_string());
    let mut hist_dir = PathBuf::from(data_dir);
    hist_dir.push("samples");
    hist_dir.push("hist");

    // Create directory if it doesn't exist
    fs::create_dir_all(&hist_dir)?;

    // Symbols to fetch historical data for
    let symbols = vec!["SPY", "QQQ", "TLT"];

    println!("Fetching 10 years of historical daily pricing data...");
    println!("Date range: {} to {}", start_date, end_date);
    println!("Output directory: {}", hist_dir.display());

    for symbol in symbols {
        println!("\nFetching data for {}...", symbol);

        match get_history(symbol, Interval::Daily, start_date, end_date).await {
            Ok(data) => {
                println!(
                    "Successfully retrieved {} data points for {}",
                    data.len(),
                    symbol
                );

                // Create filename: symbol_hist.json
                let filename = format!("{}_hist.json", symbol.to_lowercase());
                let filepath = hist_dir.join(filename);

                // Serialize data to JSON
                let json_output = serde_json::to_string_pretty(&data)?;

                // Write to file
                fs::write(&filepath, json_output)?;

                println!("Data saved to: {}", filepath.display());

                // Print summary
                if !data.is_empty() {
                    if let (Some(first), Some(last)) = (data.first(), data.last()) {
                        println!("Date range: {} to {}", first.date, last.date);
                        if let (Some(first_close), Some(last_close)) = (first.close, last.close) {
                            println!("Price change: ${:.2} to ${:.2}", first_close, last_close);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to get historical data for {}: {}", symbol, e);
            }
        }
    }

    println!("\nHistorical data collection complete.");
    Ok(())
}
