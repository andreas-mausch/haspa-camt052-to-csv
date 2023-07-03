This is a small command line tool to convert camt.052 from the Hamburger Sparkasse (Haspa) into csv.

# Requirements

Rust >= 1.7.0

# Run

## CSV

You can either run the script with a .zip containing xml files:

```shell-session
$ cargo run -- ./testdata/camtPacket_1xxxxxxxxx_2018-01-01-2018-01-31.zip
Date;Valuta;Amount;Currency;Creditor;Debtor;Type;Description
2018-01-15;2018-01-15;-55.6;EUR;YYYYYYYY;Andreas Mausch Mausch;Lastschrift SEPA B2C;Blah blah blah
2018-01-15;2018-01-15;40;EUR;Andreas Mausch;Andreas Mausch;SEPA Gutschrift;Überweisung
2018-01-15;2018-01-15;-200;EUR;;Andreas Mausch;Barauszahlung GA;GA-NR00000000 BLZ20050550 1 15.01/13.37UHR Haspa0000/>H
```

..or with a single xml file:

```bash
cargo run -- ./testdata/camt_1xxxxxxxxx_15.01.2018.xml
```

## ODS

You can also output the transactions as a .ods file for LibreCalc / Excel:

```bash
cargo run -- ./testdata/camt_1xxxxxxxxx_15.01.2018.xml --format ods --output testdata/output.ods
```

### Debugging .ods

The ods library I use [spreadsheet_ods](https://docs.rs/spreadsheet-ods/latest/spreadsheet_ods/)
doesn't support negative red numbers, see
[this issue](https://github.com/thscharler/spreadsheet-ods/issues/34).

Commands to analyze the raw XML output of a .ods file:

```bash
unzip -p output.ods content.xml | xmllint --format -
unzip -p output.ods styles.xml | xmllint --format -
```

# Tests

```bash
cargo test
```

# camt052 / camt053 File format specification

I am not exactly sure what this file format is, but I think it is used in Europe only.

See this [German Wikipedia article](https://de.wikipedia.org/wiki/Camt-Format).

I found some pdfs describing the file formats:

- [ebics.de](https://www.ebics.de/de/datenformate)
- [ing.nl](https://www.ing.nl/media/ING%20Format%20Description%20CAMT052%20CAMT053%20-%20NL%20v4.0_tcm162-110479.pdf)
