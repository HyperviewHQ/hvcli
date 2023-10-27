use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::fmt;

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetDto {
    pub id: String,
    pub name: String,
    #[serde(alias = "assetLifecycleState")]
    pub asset_lifecycle_state: String,
    #[serde(alias = "assetTypeCategory")]
    pub asset_type_category: String,
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
}

impl fmt::Display for AssetDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, 
            "id: {}\nname: {}\nasset_lifecycle_state: {}\nasset_type_category: {}\nasset_type_id: {}\nmanufacturer_id: {}\nmanufacturer_name: {}\nmonitoring_state: {}\nparent_id: {}\nparent_name: {}\nproduct_id: {}\nproduct_name: {}\nstatus: {}\npath: {}", 
            self.id, 
            self.name,
            self.asset_lifecycle_state,
            self.asset_type_category,
            self.asset_type_id,
            self.manufacturer_id,
            self.manufacturer_name,
            self.monitoring_state,
            self.parent_id,
            self.parent_name,
            self.product_id,
            self.product_name,
            self.status,
            self.path
        )
    }
}
