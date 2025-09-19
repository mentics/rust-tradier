use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Convert a chrono NaiveDate to Tradier's date format (YYYY-MM-DD)
///
/// # Arguments
/// * `date` - The NaiveDate to convert
///
/// # Returns
/// A string in YYYY-MM-DD format
///
/// # Example
/// ```rust
/// use chrono::NaiveDate;
/// use rust_tradier::util::date_to_tradier_format;
///
/// let date = NaiveDate::from_ymd_opt(2023, 12, 25).unwrap();
/// assert_eq!(date_to_tradier_format(date), "2023-12-25");
/// ```
pub fn date_to_tradier(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Convert a Tradier date string (YYYY-MM-DD) to a chrono NaiveDate
///
/// # Arguments
/// * `date_str` - The date string in YYYY-MM-DD format
///
/// # Returns
/// A Result containing the NaiveDate or an error if parsing fails
///
/// # Example
/// ```rust
/// use chrono::NaiveDate;
/// use rust_tradier::util::tradier_date_to_naive;
///
/// match tradier_date_to_naive("2023-12-25") {
///     Ok(date) => {
///         assert_eq!(date.year(), 2023);
///         assert_eq!(date.month(), 12);
///         assert_eq!(date.day(), 25);
///     }
///     Err(e) => eprintln!("Error parsing date: {}", e),
/// }
/// ```
pub fn tradier_to_date(date_str: &str) -> Result<NaiveDate, chrono::ParseError> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
}

/// Serialize a Vec<NaiveDate> to Vec<String> using Tradier date format
pub fn serialize_naive_dates<S>(dates: &Vec<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer
{
    let strings: Vec<String> = dates.iter().map(|date| date_to_tradier(*date)).collect();
    strings.serialize(serializer)
}

/// Deserialize a Vec<String> to Vec<NaiveDate> using Tradier date format
pub fn deserialize_naive_dates<'de, D>(deserializer: D) -> Result<Vec<NaiveDate>, D::Error>
where
    D: Deserializer<'de>
{
    let strings: Vec<String> = Vec::deserialize(deserializer)?;
    strings
        .into_iter()
        .map(|s| tradier_to_date(&s).map_err(serde::de::Error::custom))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, NaiveDate};

    #[test]
    fn test_date_to_tradier_format() {
        let date = NaiveDate::from_ymd_opt(2023, 12, 25).unwrap();
        assert_eq!(date_to_tradier(date), "2023-12-25");

        let date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        assert_eq!(date_to_tradier(date), "2020-01-01");
    }

    #[test]
    fn test_tradier_date_to_naive() {
        let date_str = "2023-12-25";
        let date = tradier_to_date(date_str).unwrap();
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 25);

        let date_str = "2020-01-01";
        let date = tradier_to_date(date_str).unwrap();
        assert_eq!(date.year(), 2020);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 1);
    }

    #[test]
    fn test_tradier_date_to_naive_invalid() {
        let invalid_date = "2023-13-45"; // Invalid month and day
        assert!(tradier_to_date(invalid_date).is_err());

        let invalid_format = "2023/12/25"; // Wrong format
        assert!(tradier_to_date(invalid_format).is_err());
    }

    #[test]
    fn test_serde_naive_dates() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        struct TestWrapper(
            #[serde(
                serialize_with = "serialize_naive_dates",
                deserialize_with = "deserialize_naive_dates"
            )]
            Vec<NaiveDate>
        );

        let dates = vec![
            NaiveDate::from_ymd_opt(2023, 12, 25).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        ];

        let wrapper = TestWrapper(dates.clone());

        // Test serialization
        let serialized = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(serialized, r#"["2023-12-25","2024-01-01"]"#);

        // Test deserialization
        let deserialized: TestWrapper = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.0, dates);
    }
}
