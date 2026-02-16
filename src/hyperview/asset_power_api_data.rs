use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct PowerProviderComponentDto {
    pub id: Uuid,
    pub name: String,
    #[serde(
        alias = "outletNumber",
        alias = "tapOffNumber",
        alias = "breakerNumber"
    )]
    pub number: u64,
    #[serde(alias = "panelNumber")]
    pub panel_number: Option<u64>,
}

impl fmt::Display for PowerProviderComponentDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut panel_number = "---- N/A ----".to_string();
        if let Some(x) = self.panel_number {
            panel_number = x.to_string();
        }

        write!(
            f,
            r"
id           : {}
name         : {}
number       : {}
panel_number : {}
       ",
            self.id, self.name, self.number, panel_number
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct PowerAssociationCreateDto {
    pub consuming_destination_asset_id: Uuid,
    pub providing_source_asset_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct BulkPowerAssociationCreateDto {
    pub asset_id: Uuid,
    pub provider_asset_id: Uuid,
    pub provider_component_number: Option<u64>,
    pub provider_panel_number: Option<u64>,
}
