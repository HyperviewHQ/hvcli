use core::fmt;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnNull};

use crate::hyperview::common_types::MultiTypeValue;

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetCustomPropertyDto {
    pub id: String,
    #[serde(alias = "customAssetPropertyKeyId")]
    pub custom_asset_property_key_id: String,
    #[serde(alias = "customAssetPropertyGroupId")]
    pub custom_asset_property_group_id: String,
    pub value: MultiTypeValue,
    #[serde(alias = "dataType")]
    pub data_type: String,
    pub name: String,
    #[serde(alias = "groupName")]
    pub group_name: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    #[serde(alias = "dataSource")]
    pub data_source: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    #[serde(alias = "updatedDateTime")]
    pub updated_date_time: String,
    pub unit: String,
}

impl fmt::Display for AssetCustomPropertyDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "id: {}\ncustom_asset_property_key_id: {}\ncustom_asset_property_group_id: {}\nvalue: {}\ndata_type: {}\nname: {}\ngroup_name: {}\ndata_source: {}\nupdated_date_time: {}\nunit: {}",
            self.id,
            self.custom_asset_property_key_id,
            self.custom_asset_property_group_id,
            self.value,
            self.data_type,
            self.name,
            self.group_name,
            self.data_source,
            self.updated_date_time,
            self.unit
        )
    }
}
