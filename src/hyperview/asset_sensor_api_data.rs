use std::fmt;

use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnNull, serde_as};

use crate::hyperview::common_types::MultiTypeValue;

#[serde_as]
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AssetSensorDto {
    pub id: String,

    pub name: String,

    #[serde(alias = "sensorTypeId")]
    pub sensor_type_id: String,

    #[serde(alias = "listIndex")]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub list_index: String,

    #[serde(alias = "sensorTypeDescription")]
    pub sensor_type_description: String,

    pub value: MultiTypeValue,

    #[serde(alias = "rawValue")]
    pub raw_value: MultiTypeValue,

    #[serde(alias = "unitString")]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub unit_string: String,

    #[serde(alias = "dataSource")]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub data_source: String,

    #[serde(alias = "dataCollectorId")]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub data_collector_id: String,

    #[serde(alias = "dataCollectorName")]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub data_collector_name: String,

    #[serde(alias = "lastValueUpdate")]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub last_value_update: String,

    #[serde(alias = "sourceAssetDisplayName")]
    pub source_asset_display_name: String,

    #[serde(alias = "sourceAssetId")]
    pub source_asset_id: String,

    #[serde(alias = "sourceDeviceAssetId")]
    pub source_device_asset_id: String,

    #[serde(alias = "sensorAssociationType")]
    pub sensor_association_type: String,

    #[serde(alias = "isNumeric")]
    pub is_numeric: bool,

    #[serde(alias = "accessPolicyId")]
    pub access_policy_id: String,

    #[serde(alias = "accessPolicyName")]
    pub access_policy_name: String,

    #[serde(alias = "assetAccessPolicyId")]
    pub asset_access_policy_id: String,

    #[serde(alias = "accessPolicyIsInherited")]
    pub access_policy_is_inherited: bool,
}

impl fmt::Display for AssetSensorDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let asset_sensor_dto = format!(
            r"
id                         : {}
name                       : {}
sensor_type_id             : {}
list_index                 : {}
sensor_type_description    : {}
value                      : {}
raw_value                  : {}
unit_string                : {}
data_source                : {}
data_collector_id          : {}
data_collector_name        : {}
last_value_update          : {}
source_asset_display_name  : {}
source_asset_id            : {}
source_device_asset_id     : {}
sensor_association_type    : {}
is_numeric                 : {}
access_policy_id           : {}
access_policy_name         : {}
asset_access_policy_id     : {}
access_policy_is_inherited : {}
        ",
            self.id,
            self.name,
            self.sensor_type_id,
            self.list_index,
            self.sensor_type_description,
            self.value,
            self.raw_value,
            self.unit_string,
            self.data_source,
            self.data_collector_id,
            self.data_collector_name,
            self.last_value_update,
            self.source_asset_display_name,
            self.source_asset_id,
            self.source_device_asset_id,
            self.sensor_association_type,
            self.is_numeric,
            self.access_policy_id,
            self.access_policy_name,
            self.asset_access_policy_id,
            self.access_policy_is_inherited
        );

        write!(f, "{asset_sensor_dto}")
    }
}
