use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A Modbus TCP component listed under a definition (`ModbusTcpComponentDetailDto`). Components
/// group the sensors of a Modbus definition; a sensor references its component via `componentId`.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModbusComponentDto {
    pub id: Option<Uuid>,
    pub name: String,
    pub numeric_sensor_count: i32,
    pub non_numeric_sensor_count: i32,
}

impl fmt::Display for ModbusComponentDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let id = self.id.map(|id| id.to_string()).unwrap_or_default();

        write!(
            f,
            "id: {}\nname: {}\nnumeric sensor count: {}\nnon-numeric sensor count: {}",
            id, self.name, self.numeric_sensor_count, self.non_numeric_sensor_count
        )
    }
}

/// Request body for creating (`id: None`) or updating (`id: Some(_)`) a component
/// (`MonitorOnlyDefinitionComponentDto`).
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModbusComponentCreateDto {
    pub id: Option<Uuid>,
    pub name: String,
}
