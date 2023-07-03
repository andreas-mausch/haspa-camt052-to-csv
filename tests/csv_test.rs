use std::io::Write;
use std::str::from_utf8;

use haspa_camt052_to_csv::Format::Csv;
use haspa_camt052_to_csv::process;

#[test]
fn test_add() {
    let mut buf = Vec::new();
    {
        let mut x: Box<dyn Write> = Box::new(&mut buf);
        process(vec!["./tests/camt052.xml".to_string()], Csv, &mut x).unwrap();
    }

    let expected_output = std::fs::read_to_string("./tests/camt052.csv").unwrap();
    assert_eq!(from_utf8(&buf).unwrap(), expected_output, "CSV Output");
}
