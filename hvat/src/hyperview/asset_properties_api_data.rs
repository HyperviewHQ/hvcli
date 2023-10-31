use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnNull};
use std::fmt;

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetPropertyDto {
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub id: String,
    #[serde(alias = "type")]
    pub property_type: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub value: String,
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
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub minimum_value: String,
}

impl fmt::Display for AssetPropertyDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}
