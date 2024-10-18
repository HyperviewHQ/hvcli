use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnNull};
use std::fmt;

use super::common_types::MultiTypeValue;

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetPropertyDto {
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub id: String,
    #[serde(alias = "type")]
    pub property_type: String,
    pub value: MultiTypeValue,
    #[serde(alias = "dataType")]
    pub data_type: String,
    #[serde(alias = "dataSource")]
    pub data_source: String,
    #[serde(alias = "assetPropertyDisplayCategory")]
    pub asset_property_display_category: String,
    #[serde(alias = "isEditable")]
    pub is_editable: bool,
    #[serde(alias = "isInherited")]
    pub is_inherited: bool,
    #[serde(alias = "createdDateTime")]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub created_date_time: String,
    #[serde(alias = "updatedDateTime")]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub updated_date_time: String,
    #[serde(alias = "minimumValue")]
    pub minimum_value: MultiTypeValue,
}

impl fmt::Display for AssetPropertyDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let id = if !self.id.is_empty() || (self.value != MultiTypeValue::NullValue) {
            &self.id
        } else {
            "---- UNSET ----"
        };

        write!(
            f,
            "id: {}\ntype: {}\nvalue: {}\ndata_type: {}\ndata_source: {}\ndisplay_category: {}\nis_editable: {}\nis_inherited: {}\ncreated_date: {}\nupdated_date: {}\nminimum_value: {}",
            id,
            self.property_type,
            self.value,
            self.data_type,
            self.data_source,
            self.asset_property_display_category,
            self.is_editable,
            self.is_inherited,
            self.created_date_time,
            self.updated_date_time,
            self.minimum_value
        )
    }
}
