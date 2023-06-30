use std::error::Error;
use std::io::Write;

use chrono::NaiveDate;
use color::Rgb;
use icu_locid::locale;
use spreadsheet_ods::{CellStyle, format, formula, mm, Sheet, WorkBook};
use spreadsheet_ods::style::units::{Border, Length, TextRelief};

use crate::transaction::Transaction;
use crate::writers::Writer;

pub struct Ods {}

impl Writer for Ods {
    fn write<W: Write>(_transactions: &Vec<Transaction>, mut write: W) -> Result<(), Box<dyn Error>> {
        let mut workbook = WorkBook::new(locale!("de_DE"));
        let mut sheet = Sheet::new("Sheet");

        let date_format = format::create_date_dmy_format("date_format");
        let date_format = workbook.add_datetime_format(date_format);

        let mut date_style = CellStyle::new("nice_date_style", &date_format);
        date_style.set_font_bold();
        date_style.set_font_relief(TextRelief::Engraved);
        date_style.set_border(mm!(0.2), Border::Dashed, Rgb::new(192, 72, 72));
        let date_style_ref = workbook.add_cellstyle(date_style);

        sheet.set_value(0, 0, 21.4f32);
        sheet.set_value(0, 1, "foo");
        sheet.set_styled_value(0, 2, NaiveDate::from_ymd_opt(2020, 03, 01), &date_style_ref);
        sheet.set_formula(0, 3, format!("of:={}+1", formula::fcellref(0, 0)));

        workbook.push_sheet(sheet);

        let output_vector = vec![];
        spreadsheet_ods::write_ods_buf(&mut workbook, output_vector)
            .map_err(|e| e.into())
            .and_then(|vector| write.write_all(&vector)
                .map_err(|e| e.into()))
    }
}
