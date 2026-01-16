use serde::{Deserialize, Serialize};
use std::fmt;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum MultiTypeValue {
    StringValue(String),
    FloatValue(f64),
    IntegerValue(i64),
    #[default]
    NullValue,
}

impl fmt::Display for MultiTypeValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MultiTypeValue::StringValue(s) => {
                write!(f, "\"{s}\"")
            }
            MultiTypeValue::FloatValue(n) => {
                write!(f, "{n}")
            }
            MultiTypeValue::IntegerValue(n) => {
                write!(f, "{n}")
            }
            MultiTypeValue::NullValue => {
                write!(f, "null")
            }
        }
    }
}
