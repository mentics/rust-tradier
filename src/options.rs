use chrono::NaiveDate;
use market_types::option::OptionRight;
use serde::{Deserialize, Serialize};

/// Parsed OCC option specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionSpec {
    pub underlying_symbol: String,
    pub expiration: NaiveDate,
    pub strike: f64,
    pub right: OptionRight
}

/// Parse an OCC (Options Clearing Corporation) option symbol
///
/// OCC format: SYMBOL + YYMMDD + C|P + STRIKE
/// Examples:
/// - "QQQ241010P00450000" -> QQQ, 2024-10-10, Put, 450.00
/// - "AAPL231215C00150000" -> AAPL, 2023-12-15, Call, 150.00
///
/// # Arguments
/// * `symbol` - The OCC option symbol string
///
/// # Returns
/// * `Some(OptionSpec)` if parsing succeeds
/// * `None` if parsing fails
pub fn parse_occ_option_symbol(symbol: &str) -> Option<OptionSpec> {
    // Must be at least 15 characters: symbol(1+) + date(6) + type(1) + strike(7+)
    if symbol.len() < 15 {
        return None;
    }

    // Find the date part - it starts after the underlying symbol
    // We need to find where the 6-digit date starts
    let bytes = symbol.as_bytes();
    let mut date_start = None;

    // Look for a 6-digit sequence that could be YYMMDD
    for i in 0..=(symbol.len() - 6) {
        if bytes[i].is_ascii_digit()
            && bytes[i + 1].is_ascii_digit()
            && bytes[i + 2].is_ascii_digit()
            && bytes[i + 3].is_ascii_digit()
            && bytes[i + 4].is_ascii_digit()
            && bytes[i + 5].is_ascii_digit()
        {
            // Validate that this looks like a date (YY >= 20, MM 01-12, DD 01-31)
            let yy = (bytes[i] - b'0') * 10 + (bytes[i + 1] - b'0');
            let mm = (bytes[i + 2] - b'0') * 10 + (bytes[i + 3] - b'0');
            let dd = (bytes[i + 4] - b'0') * 10 + (bytes[i + 5] - b'0');

            if yy >= 20 && mm >= 1 && mm <= 12 && dd >= 1 && dd <= 31 {
                date_start = Some(i);
                break;
            }
        }
    }

    let date_start = date_start?;
    let underlying_symbol = symbol[0..date_start].to_string();

    // Date part (YYMMDD)
    let date_str = &symbol[date_start..date_start + 6];
    let yy = &date_str[0..2];
    let mm = &date_str[2..4];
    let dd = &date_str[4..6];

    // Parse date - assume 2000+ for YY
    let year = 2000 + yy.parse::<i32>().ok()?;
    let month = mm.parse::<u32>().ok()?;
    let day = dd.parse::<u32>().ok()?;

    let expiration = NaiveDate::from_ymd_opt(year, month, day)?;

    // Option type (C or P)
    let type_pos = date_start + 6;
    if type_pos >= symbol.len() {
        return None;
    }

    let option_type = match symbol.as_bytes()[type_pos] {
        b'C' | b'c' => OptionRight::Call,
        b'P' | b'p' => OptionRight::Put,
        _ => return None
    };

    // Strike price - remaining digits, divide by 1000
    let strike_str = &symbol[type_pos + 1..];
    if strike_str.is_empty() || !strike_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let strike_value = strike_str.parse::<f64>().ok()? / 1000.0;

    Some(OptionSpec {
        underlying_symbol,
        expiration,
        strike: strike_value,
        right: option_type
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_parse_occ_option_symbol_put() {
        let symbol = "QQQ241010P00450000";
        let parsed = parse_occ_option_symbol(symbol).unwrap();

        assert_eq!(parsed.underlying_symbol, "QQQ");
        assert_eq!(
            parsed.expiration,
            NaiveDate::from_ymd_opt(2024, 10, 10).unwrap()
        );
        assert_eq!(parsed.strike, 450.0);
        assert_eq!(parsed.right, OptionRight::Put);
    }

    #[test]
    fn test_parse_occ_option_symbol_call() {
        let symbol = "AAPL231215C00150000";
        let parsed = parse_occ_option_symbol(symbol).unwrap();

        assert_eq!(parsed.underlying_symbol, "AAPL");
        assert_eq!(
            parsed.expiration,
            NaiveDate::from_ymd_opt(2023, 12, 15).unwrap()
        );
        assert_eq!(parsed.strike, 150.0);
        assert_eq!(parsed.right, OptionRight::Call);
    }

    #[test]
    fn test_parse_occ_option_symbol_case_insensitive() {
        let symbol = "qqq241010p00450000";
        let parsed = parse_occ_option_symbol(symbol).unwrap();

        assert_eq!(parsed.underlying_symbol, "qqq");
        assert_eq!(
            parsed.expiration,
            NaiveDate::from_ymd_opt(2024, 10, 10).unwrap()
        );
        assert_eq!(parsed.strike, 450.0);
        assert_eq!(parsed.right, OptionRight::Put);
    }

    #[test]
    fn test_parse_occ_option_symbol_invalid() {
        assert!(parse_occ_option_symbol("").is_none());
        assert!(parse_occ_option_symbol("QQQ").is_none());
        assert!(parse_occ_option_symbol("QQQ241010").is_none());
        assert!(parse_occ_option_symbol("QQQ241010X00450000").is_none()); // Invalid option type
        assert!(parse_occ_option_symbol("QQQ241010P").is_none()); // Missing strike
    }
}
