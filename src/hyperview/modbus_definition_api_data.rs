use serde::{Deserialize, Serialize, ser::SerializeStruct};
use serde_with::{DefaultOnError, serde_as};
use std::fmt;
use uuid::Uuid;

use super::definition_api_data::{ValueMapping, format_value_mapping, parse_value_mapping};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModbusNumericSensorDefinitionDto {
    pub id: Uuid,
    #[serde(alias = "componentId", default)]
    pub component_id: Option<Uuid>,
    #[serde(alias = "componentName", default)]
    pub component_name: Option<String>,
    pub name: String,
    pub multiplier: f64,
    pub offset: f64,
    #[serde(alias = "orderOfOperations")]
    pub order_of_operations: String,
    pub address: usize,
    #[serde(alias = "registerType")]
    pub register_type: String,
    #[serde(alias = "dataSetting")]
    pub data_setting: String,
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

impl fmt::Display for ModbusNumericSensorDefinitionDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let component_id = self
            .component_id
            .map(|id| id.to_string())
            .unwrap_or_default();
        let component_name = self.component_name.clone().unwrap_or_default();
        let unit = self.unit.clone().unwrap_or_default();
        let unit_id = self.unit_id.clone().unwrap_or_default();

        write!(
            f,
            "id: {}\ncomponent id: {}\ncomponent name: {}\nname: {}\nmultiplier: {}\noffset: {}\norder of operations: {}\naddress: {}\nregister type: {}\ndata setting: {}\nsensor type: {}\nsensor type id: {}\nunit: {}\nunit id: {}",
            self.id,
            component_id,
            component_name,
            self.name,
            self.multiplier,
            self.offset,
            self.order_of_operations,
            self.address,
            self.register_type,
            self.data_setting,
            self.sensor_type,
            self.sensor_type_id,
            unit,
            unit_id
        )
    }
}

/// CSV-import (`snake_case`) and outgoing request-body (`camelCase`) shape for a numeric sensor
/// definition. `id: None` means create a new sensor; `id: Some(_)` means update that sensor.
/// `component_id`, `offset`, and `order_of_operations` are optional columns; when absent they
/// are omitted from the request so the API applies its own defaults.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ModbusNumericSensorDefinitionImportDto {
    pub id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_id: Option<Uuid>,
    pub name: String,
    pub multiplier: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_of_operations: Option<String>,
    pub address: usize,
    pub register_type: String,
    pub data_setting: String,
    pub sensor_type: String,
    pub sensor_type_id: String,
    pub unit: Option<String>,
    pub unit_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModbusNonNumericSensorDefinitionDto {
    pub id: Uuid,
    #[serde(alias = "componentId", default)]
    pub component_id: Option<Uuid>,
    #[serde(alias = "componentName", default)]
    pub component_name: Option<String>,
    pub name: String,
    pub address: usize,
    #[serde(alias = "dataType")]
    pub data_type: String,
    #[serde(alias = "registerType")]
    pub register_type: String,
    #[serde(alias = "startBit")]
    pub start_bit: usize,
    #[serde(alias = "endBit")]
    pub end_bit: usize,
    #[serde(alias = "sensorType")]
    pub sensor_type: String,
    #[serde(alias = "sensorTypeId")]
    pub sensor_type_id: String,
    #[serde(alias = "valueMapping")]
    pub value_mapping: Vec<ValueMapping>,
}

impl fmt::Display for ModbusNonNumericSensorDefinitionDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let component_id = self
            .component_id
            .map(|id| id.to_string())
            .unwrap_or_default();
        let component_name = self.component_name.clone().unwrap_or_default();
        let mappings = self
            .value_mapping
            .iter()
            .fold(String::new(), |acc, m| acc + "\n" + &m.to_string());

        write!(
            f,
            "id: {}\ncomponent id: {}\ncomponent name: {}\nname: {}\naddress: {}\ndata type: {}\nregister type: {}\nstart bit: {}\nend bit: {}\nsensor type: {}\nsensor type id: {}{}",
            self.id,
            component_id,
            component_name,
            self.name,
            self.address,
            self.data_type,
            self.register_type,
            self.start_bit,
            self.end_bit,
            self.sensor_type,
            self.sensor_type_id,
            mappings
        )
    }
}

/// Serialize-only wrapper that flattens `value_mapping` to a single `"text:value,text:value"`
/// column so non-numeric sensor definitions can round-trip through csv-file/json output.
pub struct ModbusNonNumericSensorDefinitionExportWrapper(pub ModbusNonNumericSensorDefinitionDto);

impl fmt::Display for ModbusNonNumericSensorDefinitionExportWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for ModbusNonNumericSensorDefinitionExportWrapper {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state =
            serializer.serialize_struct("ModbusNonNumericSensorDefinitionExportWrapper", 12)?;

        state.serialize_field("id", &self.0.id)?;
        state.serialize_field("component_id", &self.0.component_id)?;
        state.serialize_field("component_name", &self.0.component_name)?;
        state.serialize_field("name", &self.0.name)?;
        state.serialize_field("address", &self.0.address)?;
        state.serialize_field("data_type", &self.0.data_type)?;
        state.serialize_field("register_type", &self.0.register_type)?;
        state.serialize_field("start_bit", &self.0.start_bit)?;
        state.serialize_field("end_bit", &self.0.end_bit)?;
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
pub struct ModbusNonNumericSensorDefinitionImportCsv {
    pub id: Option<Uuid>,
    #[serde(default)]
    pub component_id: Option<Uuid>,
    pub name: String,
    pub address: usize,
    pub data_type: String,
    pub register_type: String,
    pub start_bit: usize,
    pub end_bit: usize,
    pub sensor_type: String,
    pub sensor_type_id: String,
    pub value_mapping: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ModbusNonNumericSensorDefinitionImportDto {
    pub id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_id: Option<Uuid>,
    pub name: String,
    pub address: usize,
    pub data_type: String,
    pub register_type: String,
    pub start_bit: usize,
    pub end_bit: usize,
    pub sensor_type: String,
    pub sensor_type_id: String,
    pub value_mapping: Vec<ValueMapping>,
}

impl TryFrom<&ModbusNonNumericSensorDefinitionImportCsv>
    for ModbusNonNumericSensorDefinitionImportDto
{
    type Error = color_eyre::Report;

    fn try_from(source: &ModbusNonNumericSensorDefinitionImportCsv) -> color_eyre::Result<Self> {
        Ok(Self {
            id: source.id,
            component_id: source.component_id,
            name: source.name.clone(),
            address: source.address,
            data_type: source.data_type.clone(),
            register_type: source.register_type.clone(),
            start_bit: source.start_bit,
            end_bit: source.end_bit,
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
        let dto = ModbusNonNumericSensorDefinitionDto {
            id: Uuid::nil(),
            component_id: None,
            component_name: None,
            name: "Status".to_string(),
            address: 100,
            data_type: "boolean".to_string(),
            register_type: "coil".to_string(),
            start_bit: 0,
            end_bit: 0,
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
            .serialize(ModbusNonNumericSensorDefinitionExportWrapper(dto))
            .unwrap();
        let output = String::from_utf8(writer.into_inner().unwrap()).unwrap();

        assert_eq!(
            output,
            "id,component_id,component_name,name,address,data_type,register_type,start_bit,end_bit,sensor_type,sensor_type_id,value_mapping\n00000000-0000-0000-0000-000000000000,,,Status,100,boolean,coil,0,0,Status,type-1,\"Inactive:0,Active:1\"\n"
        );
    }

    #[test]
    fn test_export_wrapper_includes_component_id_when_set() {
        let component_id = Uuid::new_v4();
        let dto = ModbusNonNumericSensorDefinitionDto {
            id: Uuid::nil(),
            component_id: Some(component_id),
            component_name: Some("PDU 1".to_string()),
            name: "Status".to_string(),
            address: 100,
            data_type: "boolean".to_string(),
            register_type: "coil".to_string(),
            start_bit: 0,
            end_bit: 0,
            sensor_type: "Status".to_string(),
            sensor_type_id: "type-1".to_string(),
            value_mapping: vec![ValueMapping {
                text: "Inactive".to_string(),
                value: 0,
            }],
        };

        let mut writer = csv::Writer::from_writer(vec![]);
        writer
            .serialize(ModbusNonNumericSensorDefinitionExportWrapper(dto))
            .unwrap();
        let output = String::from_utf8(writer.into_inner().unwrap()).unwrap();

        // The exported component_id column round-trips back into the import CSV.
        assert!(output.contains(&component_id.to_string()));
        assert!(output.contains("PDU 1"));
    }

    #[test]
    fn test_try_from_csv_parses_value_mapping() {
        let component_id = Uuid::new_v4();
        let csv = ModbusNonNumericSensorDefinitionImportCsv {
            id: None,
            component_id: Some(component_id),
            name: "Status".to_string(),
            address: 100,
            data_type: "boolean".to_string(),
            register_type: "coil".to_string(),
            start_bit: 0,
            end_bit: 0,
            sensor_type: "Status".to_string(),
            sensor_type_id: "type-1".to_string(),
            value_mapping: "Inactive:0,Active:1".to_string(),
        };

        let dto = ModbusNonNumericSensorDefinitionImportDto::try_from(&csv).unwrap();

        assert_eq!(dto.component_id, Some(component_id));
        assert_eq!(dto.value_mapping.len(), 2);
        assert_eq!(dto.value_mapping[0].text, "Inactive");
        assert_eq!(dto.value_mapping[1].value, 1);
    }

    #[test]
    fn test_try_from_csv_rejects_malformed_value_mapping() {
        let csv = ModbusNonNumericSensorDefinitionImportCsv {
            id: None,
            component_id: None,
            name: "Status".to_string(),
            address: 100,
            data_type: "boolean".to_string(),
            register_type: "coil".to_string(),
            start_bit: 0,
            end_bit: 0,
            sensor_type: "Status".to_string(),
            sensor_type_id: "type-1".to_string(),
            value_mapping: "Inactive:not-a-number".to_string(),
        };

        let result = ModbusNonNumericSensorDefinitionImportDto::try_from(&csv);

        assert!(result.is_err());
    }
}
