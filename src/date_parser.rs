use chrono::{NaiveDate, ParseError};

#[derive(Debug)]
pub struct IsoDate(pub NaiveDate);

impl TryFrom<&str> for IsoDate {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map(|date| IsoDate(date))
    }
}
