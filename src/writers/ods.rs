use std::error::Error;
use std::io::Write;

use icu_locid::locale;
use indexmap::indexmap;
use Length::Mm;
use spreadsheet_ods::{CellStyle, CellStyleRef, Length, Sheet, ValueFormatText, WorkBook};
use spreadsheet_ods::format::create_date_dmy_format;

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
}

impl Writer for Ods {
    fn write<W: Write>(transactions: &Vec<Transaction>, mut write: W) -> Result<(), Box<dyn Error>> {
        let mut workbook = WorkBook::new(locale!("de_DE"));
        let mut sheet = Sheet::new("Sheet");

        let heading_style = Self::create_heading_style(&mut workbook);
        let date_style = Self::create_date_style(&mut workbook);

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
            sheet.set_value(indexu32 + 1, 2, serialized.get("amount").unwrap().as_array().unwrap().get(0).unwrap().as_str().unwrap());
            sheet.set_value(indexu32 + 1, 3, serialized.get("amount").unwrap().as_array().unwrap().get(1).unwrap().as_str().unwrap());
            sheet.set_value(indexu32 + 1, 4, serialized.get("creditor").map(|creditor| creditor.get("name")).flatten().map(|value| value.as_str()).flatten().unwrap_or(""));
            sheet.set_value(indexu32 + 1, 5, serialized.get("creditor").map(|creditor| creditor.get("iban")).flatten().map(|value| value.as_str()).flatten().unwrap_or(""));
            sheet.set_value(indexu32 + 1, 6, serialized.get("debtor").map(|debtor| debtor.get("name")).flatten().map(|value| value.as_str()).flatten().unwrap_or(""));
            sheet.set_value(indexu32 + 1, 7, serialized.get("debtor").map(|debtor| debtor.get("iban")).flatten().map(|value| value.as_str()).flatten().unwrap_or(""));
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
