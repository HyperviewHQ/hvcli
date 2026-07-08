use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize, Clone)]
pub struct SensorReportRow {
    pub asset_name: String,
    pub asset_id: String,
    pub custom_property: String,
    pub sensor_name: String,
    pub sensor_id: String,
    pub sensor_unit: String,
    pub timestamp: String,
    pub avg: f64,
    pub max: f64,
    pub min: f64,
    pub lst: f64,
}

impl fmt::Display for SensorReportRow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let row = format!(
            r"
asset_name      : {}
asset_id        : {}
custom_property : {}
sensor_name     : {}
sensor_id       : {}
sensor_unit     : {}
timestamp       : {}
avg             : {}
max             : {}
min             : {}
lst             : {}
        ",
            self.asset_name,
            self.asset_id,
            self.custom_property,
            self.sensor_name,
            self.sensor_id,
            self.sensor_unit,
            self.timestamp,
            self.avg,
            self.max,
            self.min,
            self.lst
        );

        write!(f, "{row}")
    }
}
