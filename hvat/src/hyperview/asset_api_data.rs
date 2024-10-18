use serde::{Deserialize, Serialize};
use std::fmt;

use super::cli_data::{RackPosition, RackSide};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetDto {
    pub id: String,
    pub name: String,
    #[serde(alias = "assetLifecycleState")]
    pub asset_lifecycle_state: String,
    #[serde(alias = "assetTypeId")]
    pub asset_type_id: String,
    #[serde(alias = "manufacturerId")]
    pub manufacturer_id: String,
    #[serde(alias = "manufacturerName")]
    pub manufacturer_name: String,
    #[serde(alias = "monitoringState")]
    pub monitoring_state: String,
    #[serde(alias = "parentId")]
    pub parent_id: String,
    #[serde(alias = "parentName")]
    pub parent_name: String,
    #[serde(alias = "productId")]
    pub product_id: String,
    #[serde(alias = "productName")]
    pub product_name: String,
    pub status: String,
    pub path: String,
    #[serde(alias = "serialNumber")]
    pub serial_number: String,
    pub property: Option<String>,
}

impl fmt::Display for AssetDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let asset_record = format!(
            r#"
id                    : {}
name                  : {}
asset_lifecycle_state : {}
asset_type_id         : {}
manufacturer_id       : {}
manufacturer_name     : {}
monitoring_state      : {}
parent_id             : {}
parent_name           : {}
product_id            : {}
product_name          : {}
status                : {}
path                  : {}
serial_number         : {}
property              : {}
"#,
            self.id,
            self.name,
            self.asset_lifecycle_state,
            self.asset_type_id,
            self.manufacturer_id,
            self.manufacturer_name,
            self.monitoring_state,
            self.parent_id,
            self.parent_name,
            self.product_id,
            self.product_name,
            self.status,
            self.path,
            self.serial_number,
            self.property.clone().unwrap_or_default()
        );

        write!(f, "{}", asset_record)
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssetNameRecord {
    pub asset_id: String,
    pub new_name: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetLocationDTO {
    pub parent_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rack_position: Option<RackPosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rack_side: Option<RackSide>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rack_u_location: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssetLocationRecord {
    pub asset_id: String,
    pub new_location_id: String,
    pub rack_position: Option<RackPosition>,
    pub rack_side: Option<RackSide>,
    pub rack_u_location: Option<usize>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetPortDto {
    pub id: String,
    pub name: String,
    #[serde(alias = "parentId")]
    pub parent_id: String,
    #[serde(alias = "portNumber")]
    pub port_number: i64,
    #[serde(alias = "portSide")]
    pub port_side: Option<String>,
    #[serde(alias = "portSideValueId")]
    pub port_side_value_id: Option<String>,
    #[serde(alias = "connectorTypeValueId")]
    pub connector_type_value_id: Option<String>,
    #[serde(alias = "portTypeValueId")]
    pub port_type_value_id: Option<String>,
}

impl fmt::Display for AssetPortDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let asset_port_record = format!(
            r#"
id: {}
name: {}
parent_id: {}
port_number: {}
port_side: {}
port_side_value_id: {}
connector_type_value_id: {}
port_type_value_id: {}
"#,
            self.id,
            self.name,
            self.parent_id,
            self.port_number,
            self.port_side.clone().unwrap_or_default(),
            self.port_side_value_id.clone().unwrap_or_default(),
            self.connector_type_value_id.clone().unwrap_or_default(),
            self.port_type_value_id.clone().unwrap_or_default()
        );

        write!(f, "{}", asset_port_record)
    }
}
