// rusty_money has no default support for serde, so we have to write it ourselves
// https://github.com/varunsrin/rusty_money/issues/33

use rusty_money::{FormattableCurrency, Money};
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub struct MyMoney<'a, T: FormattableCurrency>(pub Money<'a, T>);

impl<T: FormattableCurrency> Serialize for MyMoney<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let as_string = self.0.to_string();
        serializer.serialize_str(&as_string)
    }
}
