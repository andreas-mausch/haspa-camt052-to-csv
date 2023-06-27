use chrono::{NaiveDate, ParseResult};

pub fn parse_iso_date(string: &str) -> ParseResult<NaiveDate> {
    NaiveDate::parse_from_str(string, "%Y-%m-%d")
}
