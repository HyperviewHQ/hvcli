use serde::{Deserialize, Serialize, ser::SerializeStruct};
use serde_with::{DefaultOnError, serde_as};
use std::fmt;
use uuid::Uuid;

use super::definition_api_data::{ValueMapping, format_value_mapping, parse_value_mapping};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BacnetNumericSensorDefinitionDto {
    pub id: Uuid,
    pub name: String,
    pub multiplier: f64,
    pub offset: f64,
    #[serde(alias = "orderOfOperations")]
    pub order_of_operations: String,
    #[serde(alias = "objectInstance")]
    pub object_instance: usize,
    #[serde(alias = "objectType")]
    pub object_type: String,
    #[serde(alias = "sensorType")]
    pub sensor_type: String,
    #[serde(alias = "sensorTypeId")]
    pub sensor_type_id: String,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub unit: Option<String>,
    #[serde(alias = "unitId")]
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub unit_id: Option<String>,
}

impl fmt::Display for BacnetNumericSensorDefinitionDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let unit = self.unit.clone().unwrap_or_default();
        let unit_id = self.unit_id.clone().unwrap_or_default();

        write!(
            f,
            "id: {}\nname: {}\nmultiplier: {}\noffset: {}\norder of operations: {}\nobject instance: {}\nobject type: {}\nsensor type: {}\nsensor type id: {}\nunit: {}\nunit id: {}",
            self.id,
            self.name,
            self.multiplier,
            self.offset,
            self.order_of_operations,
            self.object_instance,
            self.object_type,
            self.sensor_type,
            self.sensor_type_id,
            unit,
            unit_id
        )
    }
}

/// CSV-import (`snake_case`) and outgoing request-body (`camelCase`) shape for a numeric sensor
/// definition. `id: None` means create a new sensor; `id: Some(_)` means update that sensor.
/// `offset`/`order_of_operations` are optional columns; when absent they are omitted from the
/// request so the API applies its own defaults.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct BacnetNumericSensorDefinitionImportDto {
    pub id: Option<Uuid>,
    pub name: String,
    pub multiplier: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_of_operations: Option<String>,
    pub object_instance: usize,
    pub object_type: String,
    pub sensor_type: String,
    pub sensor_type_id: String,
    pub unit: Option<String>,
    pub unit_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BacnetNonNumericSensorDefinitionDto {
    pub id: Uuid,
    pub name: String,
    #[serde(alias = "objectInstance")]
    pub object_instance: usize,
    #[serde(alias = "objectType")]
    pub object_type: String,
    #[serde(alias = "sensorType")]
    pub sensor_type: String,
    #[serde(alias = "sensorTypeId")]
    pub sensor_type_id: String,
    #[serde(alias = "valueMapping")]
    pub value_mapping: Vec<ValueMapping>,
}

impl fmt::Display for BacnetNonNumericSensorDefinitionDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mappings = self
            .value_mapping
            .iter()
            .fold(String::new(), |acc, m| acc + "\n" + &m.to_string());

        write!(
            f,
            "id: {}\nname: {}\nobject instance: {}\nobject type: {}\nsensor type: {}\nsensor type id: {}{}",
            self.id,
            self.name,
            self.object_instance,
            self.object_type,
            self.sensor_type,
            self.sensor_type_id,
            mappings
        )
    }
}

/// Serialize-only wrapper that flattens `value_mapping` to a single `"text:value,text:value"`
/// column so non-numeric sensor definitions can round-trip through csv-file/json output.
pub struct BacnetNonNumericSensorDefinitionExportWrapper(pub BacnetNonNumericSensorDefinitionDto);

impl fmt::Display for BacnetNonNumericSensorDefinitionExportWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for BacnetNonNumericSensorDefinitionExportWrapper {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state =
            serializer.serialize_struct("BacnetNonNumericSensorDefinitionExportWrapper", 7)?;

        state.serialize_field("id", &self.0.id)?;
        state.serialize_field("name", &self.0.name)?;
        state.serialize_field("object_instance", &self.0.object_instance)?;
        state.serialize_field("object_type", &self.0.object_type)?;
        state.serialize_field("sensor_type", &self.0.sensor_type)?;
        state.serialize_field("sensor_type_id", &self.0.sensor_type_id)?;
        state.serialize_field(
            "value_mapping",
            &format_value_mapping(&self.0.value_mapping),
        )?;

        state.end()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct BacnetNonNumericSensorDefinitionImportCsv {
    pub id: Option<Uuid>,
    pub name: String,
    pub object_instance: usize,
    pub object_type: String,
    pub sensor_type: String,
    pub sensor_type_id: String,
    pub value_mapping: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct BacnetNonNumericSensorDefinitionImportDto {
    pub id: Option<Uuid>,
    pub name: String,
    pub object_instance: usize,
    pub object_type: String,
    pub sensor_type: String,
    pub sensor_type_id: String,
    pub value_mapping: Vec<ValueMapping>,
}

impl TryFrom<&BacnetNonNumericSensorDefinitionImportCsv>
    for BacnetNonNumericSensorDefinitionImportDto
{
    type Error = color_eyre::Report;

    fn try_from(source: &BacnetNonNumericSensorDefinitionImportCsv) -> color_eyre::Result<Self> {
        Ok(Self {
            id: source.id,
            name: source.name.clone(),
            object_instance: source.object_instance,
            object_type: source.object_type.clone(),
            sensor_type: source.sensor_type.clone(),
            sensor_type_id: source.sensor_type_id.clone(),
            value_mapping: parse_value_mapping(&source.value_mapping)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_wrapper_flattens_value_mapping_to_csv() {
        let dto = BacnetNonNumericSensorDefinitionDto {
            id: Uuid::nil(),
            name: "Status".to_string(),
            object_instance: 1,
            object_type: "binaryInput".to_string(),
            sensor_type: "Status".to_string(),
            sensor_type_id: "type-1".to_string(),
            value_mapping: vec![
                ValueMapping {
                    text: "Inactive".to_string(),
                    value: 0,
                },
                ValueMapping {
                    text: "Active".to_string(),
                    value: 1,
                },
            ],
        };

        let mut writer = csv::Writer::from_writer(vec![]);
        writer
            .serialize(BacnetNonNumericSensorDefinitionExportWrapper(dto))
            .unwrap();
        let output = String::from_utf8(writer.into_inner().unwrap()).unwrap();

        assert_eq!(
            output,
            "id,name,object_instance,object_type,sensor_type,sensor_type_id,value_mapping\n00000000-0000-0000-0000-000000000000,Status,1,binaryInput,Status,type-1,\"Inactive:0,Active:1\"\n"
        );
    }

    #[test]
    fn test_try_from_csv_parses_value_mapping() {
        let csv = BacnetNonNumericSensorDefinitionImportCsv {
            id: None,
            name: "Status".to_string(),
            object_instance: 1,
            object_type: "binaryInput".to_string(),
            sensor_type: "Status".to_string(),
            sensor_type_id: "type-1".to_string(),
            value_mapping: "Inactive:0,Active:1".to_string(),
        };

        let dto = BacnetNonNumericSensorDefinitionImportDto::try_from(&csv).unwrap();

        assert_eq!(dto.value_mapping.len(), 2);
        assert_eq!(dto.value_mapping[0].text, "Inactive");
        assert_eq!(dto.value_mapping[0].value, 0);
        assert_eq!(dto.value_mapping[1].text, "Active");
        assert_eq!(dto.value_mapping[1].value, 1);
    }

    #[test]
    fn test_try_from_csv_rejects_malformed_value_mapping() {
        let csv = BacnetNonNumericSensorDefinitionImportCsv {
            id: None,
            name: "Status".to_string(),
            object_instance: 1,
            object_type: "binaryInput".to_string(),
            sensor_type: "Status".to_string(),
            sensor_type_id: "type-1".to_string(),
            value_mapping: "Inactive:not-a-number".to_string(),
        };

        let result = BacnetNonNumericSensorDefinitionImportDto::try_from(&csv);

        assert!(result.is_err());
    }
}
