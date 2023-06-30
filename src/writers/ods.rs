use std::error::Error;
use std::io::Write;

use icu_locid::locale;
use indexmap::indexmap;
use Length::Mm;
use spreadsheet_ods::{CellStyle, Length, Sheet, ValueFormatText, WorkBook};

use crate::transaction::Transaction;
use crate::writers::Writer;

pub struct Ods {}

impl Writer for Ods {
    fn write<W: Write>(_transactions: &Vec<Transaction>, mut write: W) -> Result<(), Box<dyn Error>> {
        let mut workbook = WorkBook::new(locale!("de_DE"));
        let mut sheet = Sheet::new("Sheet");

        let heading_style_format = ValueFormatText::new_named("heading");
        let heading_style_format = workbook.add_text_format(heading_style_format);

        let mut heading_style = CellStyle::new("heading", &heading_style_format);
        heading_style.set_font_bold();
        let heading_style_ref = workbook.add_cellstyle(heading_style);

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

        sheet.set_row_cellstyle(0, &heading_style_ref);
        headings.iter().enumerate().for_each(|(index, (&name, &width))| {
            let indexu32 = index.try_into().unwrap();
            sheet.set_styled_value(0, indexu32, name, &heading_style_ref);
            sheet.set_col_width(indexu32, Mm(width));
        });

        workbook.push_sheet(sheet);

        let output_vector = vec![];
        spreadsheet_ods::write_ods_buf(&mut workbook, output_vector)
            .map_err(|e| e.into())
            .and_then(|vector| write.write_all(&vector)
                .map_err(|e| e.into()))
    }
}
