use std::error::Error;
use std::io::Write;

use color::Rgb;
use icu_locid::{locale, Locale};
use indexmap::indexmap;
use Length::Mm;
use rusty_money::iso;
use spreadsheet_ods::{CellStyle, CellStyleRef, Length, Sheet, ValueFormatCurrency, ValueFormatText, WorkBook};
use spreadsheet_ods::condition::ValueCondition;
use spreadsheet_ods::format::{create_date_dmy_format, ValueFormatTrait, ValueStyleMap};

use crate::transaction::Transaction;
use crate::writers::Writer;

pub struct Ods;

impl Ods {
    fn create_heading_style(workbook: &mut WorkBook) -> CellStyleRef {
        let heading_style_format = ValueFormatText::new_named("heading");
        let heading_style_format = workbook.add_text_format(heading_style_format);

        let mut heading_style = CellStyle::new("heading", &heading_style_format);
        heading_style.set_font_bold();
        workbook.add_cellstyle(heading_style)
    }

    fn create_date_style(workbook: &mut WorkBook) -> CellStyleRef {
        let date_format = create_date_dmy_format("date_format");
        let date_format = workbook.add_datetime_format(date_format);

        let date_style = CellStyle::new("iso_date_style", &date_format);
        workbook.add_cellstyle(date_style)
    }

    fn create_currency_style(workbook: &mut WorkBook, locale: Locale) -> CellStyleRef {
        let mut currency_format = ValueFormatCurrency::new_localized("currency_format", locale.clone());
        currency_format.part_number()
            .min_integer_digits(1)
            .decimal_places(2)
            .min_decimal_places(2)
            .grouping()
            .build();
        currency_format.part_text(" ").build();
        currency_format.part_currency()
            .locale(locale.clone())
            .symbol(iso::EUR.symbol)
            .build();
        let currency_format = workbook.add_currency_format(currency_format);

        let mut currency_format_negative = ValueFormatCurrency::new_localized("currency_format_negative", locale.clone());
        currency_format_negative.part_text("-").build();
        currency_format_negative.part_number()
            .min_integer_digits(1)
            .decimal_places(2)
            .min_decimal_places(2)
            .grouping()
            .build();
        currency_format_negative.part_text(" ").build();
        currency_format_negative.part_currency()
            .locale(locale)
            .symbol(iso::EUR.symbol)
            .build();
        currency_format_negative.set_color(Rgb::new(255, 0, 0));
        currency_format_negative.push_stylemap(ValueStyleMap::new(ValueCondition::value_ge(0), currency_format));
        let currency_format_negative = workbook.add_currency_format(currency_format_negative);

        let currency_style = CellStyle::new("eur_currency_style", &currency_format_negative);
        workbook.add_cellstyle(currency_style)
    }
}

impl Writer for Ods {
    fn write<W: Write>(transactions: &[Transaction], mut write: W) -> Result<(), Box<dyn Error>> {
        let locale = locale!("de_DE");
        let mut workbook = WorkBook::new(locale.clone());
        let mut sheet = Sheet::new("Sheet");

        let heading_style = Self::create_heading_style(&mut workbook);
        let date_style = Self::create_date_style(&mut workbook);
        let currency_style = Self::create_currency_style(&mut workbook, locale);

        let headings = indexmap! {
            "Date" => 22.0,
            "Valuta" => 22.0,
            "Amount" => 25.0,
            "Currency" => 17.0,
            "Creditor" => 55.0,
            "Creditor IBAN" => 55.0,
            "Debtor" => 55.0,
            "Debtor IBAN" => 55.0,
            "Type" => 55.0,
            "Description" => 100.0
        };

        sheet.set_row_cellstyle(0, &heading_style);
        headings.iter().enumerate().for_each(|(index, (&name, &width))| {
            let indexu32 = index.try_into().unwrap();
            sheet.set_styled_value(0, indexu32, name, &heading_style);
            sheet.set_col_width(indexu32, Mm(width));
        });

        transactions.iter().enumerate().for_each(|(index, transaction)| {
            let serialized = serde_json::to_value(transaction).unwrap();
            let indexu32: u32 = index.try_into().unwrap();
            sheet.set_styled_value(indexu32 + 1, 0, transaction.date, &date_style);
            sheet.set_styled_value(indexu32 + 1, 1, transaction.valuta, &date_style);
            sheet.set_styled_value(indexu32 + 1, 2, *transaction.amount.0.amount(), &currency_style);
            sheet.set_value(indexu32 + 1, 3, serialized.get("amount").unwrap().as_array().unwrap().get(1).unwrap().as_str().unwrap());
            sheet.set_value(indexu32 + 1, 4, serialized.get("creditor").and_then(|creditor| creditor.get("name")).and_then(|value| value.as_str()).unwrap_or(""));
            sheet.set_value(indexu32 + 1, 5, serialized.get("creditor").and_then(|creditor| creditor.get("iban")).and_then(|value| value.as_str()).unwrap_or(""));
            sheet.set_value(indexu32 + 1, 6, serialized.get("debtor").and_then(|debtor| debtor.get("name")).and_then(|value| value.as_str()).unwrap_or(""));
            sheet.set_value(indexu32 + 1, 7, serialized.get("debtor").and_then(|debtor| debtor.get("iban")).and_then(|value| value.as_str()).unwrap_or(""));
            sheet.set_value(indexu32 + 1, 8, serialized.get("transaction_type").unwrap().as_str().unwrap());
            sheet.set_value(indexu32 + 1, 9, serialized.get("description").unwrap().as_str().unwrap());
        });

        workbook.push_sheet(sheet);

        let output_vector = vec![];
        spreadsheet_ods::write_ods_buf(&mut workbook, output_vector)
            .map_err(|e| e.into())
            .and_then(|vector| write.write_all(&vector)
                .map_err(|e| e.into()))
    }
}
