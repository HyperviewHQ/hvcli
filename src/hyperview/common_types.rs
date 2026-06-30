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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_display_string_value_is_quoted() {
        let v = MultiTypeValue::StringValue("hello".to_string());
        assert_eq!(format!("{v}"), "\"hello\"");
    }

    #[test]
    fn test_display_float_value() {
        let v = MultiTypeValue::FloatValue(3.5);
        assert_eq!(format!("{v}"), "3.5");
    }

    #[test]
    fn test_display_integer_value() {
        let v = MultiTypeValue::IntegerValue(-7);
        assert_eq!(format!("{v}"), "-7");
    }

    #[test]
    fn test_display_null_value() {
        let v = MultiTypeValue::NullValue;
        assert_eq!(format!("{v}"), "null");
    }

    #[test]
    fn test_default_is_null() {
        let v = MultiTypeValue::default();
        assert_eq!(v, MultiTypeValue::NullValue);
    }

    #[test]
    fn test_deserialize_string() {
        let v: MultiTypeValue = serde_json::from_value(json!("abc")).unwrap();
        assert_eq!(v, MultiTypeValue::StringValue("abc".to_string()));
    }

    #[test]
    fn test_deserialize_null() {
        let v: MultiTypeValue = serde_json::from_value(json!(null)).unwrap();
        assert_eq!(v, MultiTypeValue::NullValue);
    }

    // Documents the untagged-enum disambiguation: FloatValue is listed before IntegerValue,
    // so any JSON number — including whole numbers like `42` — deserializes as FloatValue.
    // Callers that need to distinguish integer vs float should not rely on this enum.
    #[test]
    fn test_deserialize_whole_number_becomes_float() {
        let v: MultiTypeValue = serde_json::from_value(json!(42)).unwrap();
        assert_eq!(v, MultiTypeValue::FloatValue(42.0));
    }

    #[test]
    fn test_deserialize_decimal_number_is_float() {
        let v: MultiTypeValue = serde_json::from_value(json!(1.5)).unwrap();
        assert_eq!(v, MultiTypeValue::FloatValue(1.5));
    }

    #[test]
    fn test_serialize_round_trip_string() {
        let v = MultiTypeValue::StringValue("xyz".to_string());
        let s = serde_json::to_string(&v).unwrap();
        let back: MultiTypeValue = serde_json::from_str(&s).unwrap();
        assert_eq!(v, back);
    }
}
