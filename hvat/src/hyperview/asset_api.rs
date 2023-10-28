use color_eyre::Result;
use log::{debug, info};
use reqwest::{header::AUTHORIZATION, Client};
use serde_json::Value;

use crate::hyperview::asset_api_data::AssetDto;

use super::{api_constants::ASSET_API_PREFIX, cli::AppConfig};

pub async fn get_asset_list_async(
    config: &AppConfig,
    req: Client,
    auth_header: String,
    query: Vec<(String, String)>,
) -> Result<Vec<AssetDto>> {
    // format the target URL
    let target_url = format!("{}{}", config.instance_url, ASSET_API_PREFIX);
    debug!("Request URL: {:?}", target_url);

    let resp = req
        .get(target_url)
        .query(&query)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .json::<Value>()
        .await?;

    if let Some(metadata) = &resp.get("_metadata") {
        let total = metadata["total"].as_u64().unwrap();
        let limit = metadata["limit"].as_u64().unwrap();
        info!("Meta Data: | Total: {} | Limit: {} |", total, limit);
    }

    let mut asset_list = Vec::new();

    if let Some(assets) = resp.get("data") {
        assets.as_array().unwrap().iter().for_each(|a| {
            debug!("RAW: {}", serde_json::to_string_pretty(&a).unwrap());

            let asset = AssetDto {
                id: a.get("id").unwrap().to_string(),
                name: a.get("name").unwrap().to_string(),
                asset_lifecycle_state: a.get("assetLifecycleState").unwrap().to_string(),
                asset_type_category: a.get("assetTypeCategory").unwrap().to_string(),
                asset_type_id: a.get("assetTypeId").unwrap().to_string(),
                manufacturer_id: a.get("manufacturerId").unwrap().to_string(),
                manufacturer_name: a.get("manufacturerName").unwrap().to_string(),
                monitoring_state: a.get("monitoringState").unwrap().to_string(),
                parent_id: a.get("parentId").unwrap().to_string(),
                parent_name: a.get("parentName").unwrap().to_string(),
                product_id: a.get("productId").unwrap().to_string(),
                product_name: a.get("productName").unwrap().to_string(),
                status: a.get("status").unwrap().to_string(),
                path: a
                    .get("tabDelimitedPath")
                    .unwrap()
                    .to_string()
                    .replace("\\t", "/"),
            };

            asset_list.push(asset);
        });
    };

    Ok(asset_list)
}
