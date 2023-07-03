use std::str::from_utf8;

use haspa_camt052_to_csv::Format::Csv;
use haspa_camt052_to_csv::process;

#[test]
fn test_csv() {
    let mut buf = Vec::new();
    process(vec!["./tests/camt052.xml".to_string()], Csv, &mut Box::new(&mut buf)).unwrap();

    let expected_output = std::fs::read_to_string("./tests/camt052.csv").unwrap();
    assert_eq!(from_utf8(&buf).unwrap(), expected_output, "CSV Output");
}
