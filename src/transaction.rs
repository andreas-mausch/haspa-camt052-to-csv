use std::error::Error;

use chrono::NaiveDate;
use iban::Iban;
use log::warn;
use roxmltree::Node;
use rust_decimal::Decimal;
use rusty_money::{iso, Money};
use rusty_money::iso::Currency;
use serde::Serialize;

use crate::iso_date::IsoDate;
use crate::my_money::MyMoney;
use crate::xml_document_finder::XmlDocumentFinder;

#[derive(Debug, Serialize)]
struct Party {
    name: String,
    iban: Option<Iban>,
}

#[derive(Debug, Serialize)]
pub struct Transaction<'a> {
    date: NaiveDate,
    valuta: NaiveDate,
    amount: MyMoney<'a, Currency>,
    creditor: Party,
    debtor: Party,
    transaction_type: String,
    description: String,
}

impl TryFrom<&Node<'_, '_>> for Transaction<'_> {
    type Error = Box<dyn Error>;

    fn try_from(value: &Node) -> Result<Self, Self::Error> {
        let date = value.get_into::<IsoDate>("BookgDt/Dt")?.0;
        let valuta = value.get_into::<IsoDate>("ValDt/Dt")?.0;
        let debit = value.get_into::<String>("CdtDbtInd")? == "DBIT";
        let amount = value.get_into::<String>("Amt")?;
        let currency = value.find("Amt")
            .and_then(|it| it.attribute("Ccy"))
            .ok_or::<Box<dyn Error>>("No text in 'Amt[Ccy]' attribute".into())?;
        let creditor = value.find_into::<String>("NtryDtls/TxDtls/RltdPties/Cdtr/Nm")?
            .or(value.find_into::<String>("NtryDtls/TxDtls/RltdPties/Cdtr/Pty/Nm")?)
            .unwrap_or_else(|| {
                warn!("No creditor found: Date {}, Amount {}", date, amount);
                "".to_string()
            }).trim().to_string();
        let creditor_iban = value.find_into::<Iban>("NtryDtls/TxDtls/RltdPties/CdtrAcct/Id/IBAN")?;
        let debtor = value.find_into::<String>("NtryDtls/TxDtls/RltdPties/Dbtr/Nm")?
            .or(value.find_into::<String>("NtryDtls/TxDtls/RltdPties/Dbtr/Pty/Nm")?)
            .unwrap_or_else(|| {
                warn!("No debtor found: Date {}, Amount {}", date, amount);
                "".to_string()
            }).trim().to_string();
        let debtor_iban = value.find_into::<Iban>("NtryDtls/TxDtls/RltdPties/DbtrAcct/Id/IBAN")?;
        let transaction_type = value.get_into::<String>("AddtlNtryInf")?.trim().to_string();
        let description = value.filter("NtryDtls/TxDtls/RmtInf/Ustrd")
            .iter().map(|node| node.text().unwrap_or(""))
            .map(|node| node.trim())
            .collect::<Vec<_>>().join("; ");

        // rusty_money sets the locale on the currency EUR
        // and expects it to be formatted like
        // 1.000,00 and not like 1,000.00
        // https://github.com/varunsrin/rusty_money/issues/61
        // That's why we need to convert the String to a Decimal first, and then call rusty_money.
        // Otherwise, we could use Money::from_str() directly.
        let money_decimal = amount.parse::<Decimal>()
            .map(|amount| if debit { -amount } else { amount })?;
        let money = MyMoney(Money::from_decimal(money_decimal,
                                                iso::find(currency).ok_or("Currency not found")?));

        Ok(Transaction {
            date,
            valuta,
            amount: money,
            creditor: Party { name: creditor.to_string(), iban: creditor_iban },
            debtor: Party { name: debtor.to_string(), iban: debtor_iban },
            transaction_type: transaction_type.to_string(),
            description,
        })
    }
}
