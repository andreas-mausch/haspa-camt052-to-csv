use std::str::from_utf8;

use pretty_assertions::assert_eq;

use haspa_camt052_to_csv::Format::Csv;
use haspa_camt052_to_csv::process;

#[test]
fn test_csv() {
    let mut output = Vec::new();
    process(vec!["./tests/camt052.xml".to_string()], Csv, &mut output).unwrap();

    let expected_output = std::fs::read_to_string("./tests/camt052.csv").unwrap();
    assert_eq!(from_utf8(&output).unwrap(), expected_output, "CSV Output");
}
