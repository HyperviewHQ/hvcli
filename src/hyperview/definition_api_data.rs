use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, serde_as};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Definition {
    pub id: Option<Uuid>,
    pub name: String,
    // Kept as a plain String rather than the AssetTypes enum so listing never fails wholesale
    // when the API returns an asset type this CLI doesn't enumerate.
    pub asset_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // Read-only count of assets currently using this definition; there is no API to manage the
    // association, so this is display-only.
    pub associated_assets: usize,
}

impl fmt::Display for Definition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let id = self.id.map(|id| id.to_string()).unwrap_or_default();
        let description = self.description.clone().unwrap_or_default();

        write!(
            f,
            "id: {}\nname: {}\nasset type: {}\ndescription: {}\nassociated assets: {}",
            id, self.name, self.asset_type, description, self.associated_assets
        )
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SensorType {
    // Field name mirrors the API's `sensorTypeId`; renaming to drop the type prefix would
    // require a `#[serde(rename)]` for no real gain.
    #[allow(clippy::struct_field_names)]
    pub sensor_type_id: String,
    // Nullable in the API; tolerate null/absent so one catalog entry with no description doesn't
    // fail the whole list (matches the unit fields below).
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub sensor_description: String,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub unit_id: String,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub unit_description: String,
}

impl fmt::Display for SensorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "sensor type id: {}\ndescription: {}\nunit id: {}\nunit: {}",
            self.sensor_type_id, self.sensor_description, self.unit_id, self.unit_description
        )
    }
}

/// A single text/value pair used by non-numeric (enum) sensor definitions, e.g. `{ "text":
/// "Active", "value": 1 }`. Shared between the `BACnet` and Modbus non-numeric sensor DTOs.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValueMapping {
    pub text: String,
    pub value: usize,
}

impl fmt::Display for ValueMapping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "text: {}, value: {}", self.text, self.value)
    }
}

/// Parses a `"text:value,text:value"` CSV cell (e.g. `"Inactive:0,Active:1"`) into value
/// mappings. Whitespace around entries and values is trimmed, and empty segments (an empty cell,
/// or a trailing comma) are skipped — so an empty cell yields an empty list rather than an error,
/// letting a sensor with no mappings round-trip. Returns an error (instead of panicking) when a
/// non-empty entry is missing its `:value` half or the value isn't a valid integer, so a genuinely
/// malformed row can be counted and skipped rather than crashing the whole import.
///
/// Note: `text` values containing `:` or `,` cannot round-trip through this flat format.
pub fn parse_value_mapping(raw: &str) -> color_eyre::Result<Vec<ValueMapping>> {
    raw.split(',')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let (text, value) = segment.split_once(':').ok_or_else(|| {
                color_eyre::eyre::eyre!("invalid value mapping entry (missing ':'): {segment:?}")
            })?;
            let text = text.trim();
            if text.is_empty() {
                return Err(color_eyre::eyre::eyre!(
                    "invalid value mapping entry (empty text): {segment:?}"
                ));
            }
            let value = value.trim().parse::<usize>().map_err(|e| {
                color_eyre::eyre::eyre!("invalid value mapping entry {segment:?}: {e}")
            })?;

            Ok(ValueMapping {
                text: text.to_string(),
                value,
            })
        })
        .collect()
}

/// Inverse of [`parse_value_mapping`]: flattens value mappings back to a `"text:value,..."`
/// string for CSV/JSON export.
pub fn format_value_mapping(mappings: &[ValueMapping]) -> String {
    mappings
        .iter()
        .map(|vm| format!("{}:{}", vm.text, vm.value))
        .collect::<Vec<String>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_value_mapping_parses_multiple_entries() {
        let result = parse_value_mapping("Inactive:0,Active:1").unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].text, "Inactive");
        assert_eq!(result[0].value, 0);
        assert_eq!(result[1].text, "Active");
        assert_eq!(result[1].value, 1);
    }

    #[test]
    fn test_parse_value_mapping_rejects_non_numeric_value() {
        assert!(parse_value_mapping("Inactive:not-a-number").is_err());
    }

    #[test]
    fn test_parse_value_mapping_rejects_missing_value() {
        assert!(parse_value_mapping("Inactive").is_err());
    }

    #[test]
    fn test_parse_value_mapping_empty_yields_empty_list() {
        assert!(parse_value_mapping("").unwrap().is_empty());
        assert!(parse_value_mapping("   ").unwrap().is_empty());
    }

    #[test]
    fn test_parse_value_mapping_skips_empty_segments_and_trims() {
        // Trailing comma and spaces around entries/values must not fail the row.
        let result = parse_value_mapping("Inactive:0, Active:1,").unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].text, "Inactive");
        assert_eq!(result[1].text, "Active");
        assert_eq!(result[1].value, 1);
    }

    #[test]
    fn test_format_value_mapping_round_trips_parse_value_mapping() {
        let mappings = parse_value_mapping("Inactive:0,Active:1").unwrap();

        assert_eq!(format_value_mapping(&mappings), "Inactive:0,Active:1");

        // Empty round-trips too: format([]) -> "" and parse("") -> [].
        assert_eq!(format_value_mapping(&[]), "");
        assert!(
            parse_value_mapping(&format_value_mapping(&[]))
                .unwrap()
                .is_empty()
        );
    }
}
