use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnNull, serde_as};
use std::fmt;
use uuid::Uuid;

use super::common_types::MultiTypeValue;

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetPropertyDto {
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub id: Option<Uuid>,
    #[serde(alias = "type")]
    #[serde(rename(serialize = "type"))]
    pub property_type: String,
    pub value: MultiTypeValue,
    #[serde(alias = "dataType")]
    pub data_type: String,
    #[serde(alias = "dataSource")]
    pub data_source: String,
    #[serde(alias = "assetPropertyDisplayCategory")]
    pub asset_property_display_category: String,
    #[serde(alias = "isDeletable")]
    pub is_deletable: bool,
    #[serde(alias = "isEditable")]
    pub is_editable: bool,
    #[serde(alias = "isInherited")]
    pub is_inherited: bool,
    #[serde(alias = "createdDateTime")]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub created_date_time: Option<String>,
    #[serde(alias = "updatedDateTime")]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub updated_date_time: Option<String>,
    #[serde(alias = "minimumValue")]
    pub minimum_value: MultiTypeValue,
}

impl fmt::Display for AssetPropertyDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut id = "---- UNSET ----".to_string();
        if let Some(x) = self.id {
            id = x.to_string();
        }

        let created_at = if self.created_date_time.is_some() {
            &self.created_date_time.clone().unwrap()
        } else {
            "---- UNSET ----"
        };

        let updated_at = if self.updated_date_time.is_some() {
            &self.updated_date_time.clone().unwrap()
        } else {
            "---- UNSET ----"
        };

        write!(
            f,
            "id: {}\ntype: {}\nvalue: {}\ndata_type: {}\ndata_source: {}\ndisplay_category: {}\nis_deletable: {}\nis_editable: {}\nis_inherited: {}\ncreated_date: {}\nupdated_date: {}\nminimum_value: {}",
            id,
            self.property_type,
            self.value,
            self.data_type,
            self.data_source,
            self.asset_property_display_category,
            self.is_deletable,
            self.is_editable,
            self.is_inherited,
            created_at,
            updated_at,
            self.minimum_value
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetPropertyImportDto {
    pub asset_id: Uuid,
    pub new_value: String,
}
