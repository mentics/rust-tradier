// #![feature(asm)]

pub mod account;
pub mod chain;
pub mod custom_datetime;
pub mod data;
mod error;
pub mod expirations;
pub mod history;
pub mod http;
pub mod quotes;
pub mod types;
pub mod util;

// Re-export commonly used types
pub use types::{
    ExpirationsResponse, ExpirationsWrapper, Greeks, HistoricalDataPoint, HistoryResponse,
    HistoryWrapper, Interval, OptionChainResponse, OptionData, OptionsWrapper, QuoteResponse,
    QuoteWrapper, Underlying
};

// Re-export chain functions
pub use chain::{get_calls, get_option_chain, get_options_sorted_by_strike, get_puts};

// Re-export quotes functions
pub use quotes::{get_quote, get_quotes};

// Re-export history functions
pub use history::get_history;

// Re-export expirations functions
pub use expirations::{
    clear_expirations_cache, get_expirations, get_expirations_cache_size, get_expirations_cached
};

// Re-export utility functions
pub use util::{date_to_tradier, tradier_to_date};
