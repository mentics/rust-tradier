// #![feature(asm)]

pub mod account;
pub mod chain;
pub mod data;
mod error;
pub mod expirations;
pub mod fundamental;
pub mod history;
pub mod http;
pub mod quotes;
pub mod subscription_manager;
pub mod types;

// Re-export commonly used types
pub use types::{
    ExpirationsResponse, ExpirationsWrapper, Greeks, HistoricalDataPoint, HistoryResponse,
    HistoryWrapper, Interval, OptionChainResponse, OptionData, OptionsWrapper, QuoteResponse,
    QuoteWrapper, TradierQuote
};

// Re-export fundamental types
pub use fundamental::{Dividend, DividendResponse};

// Re-export chain functions
pub use chain::{get_option_chain, get_options_sorted_by_strike};

// Re-export quotes functions
pub use quotes::{get_quote, get_quotes};

// Re-export history functions
pub use history::get_history;

// Re-export fundamental functions
pub use fundamental::get_dividends;

// Re-export expirations functions
pub use expirations::get_expirations;

// Re-export subscription manager
pub use subscription_manager::{LiveDataSubscriptionManager, MarketData, PrintHandler};
