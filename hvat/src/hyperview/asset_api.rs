use color_eyre::Result;
use log::{debug, info};
use reqwest::{header::AUTHORIZATION, Client};
use serde_json::Value;

use crate::hyperview::{api_constants::ASSET_API_PREFIX, asset_api_data::AssetDto, cli::AppConfig};

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

pub async fn get_asset_by_id_async(
    config: &AppConfig,
    req: Client,
    auth_header: String,
    id: String,
) -> Result<AssetDto> {
    // format the target URL
    let target_url = format!("{}{}/{}", config.instance_url, ASSET_API_PREFIX, id);
    debug!("Request URL: {:?}", target_url);

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .json::<Value>()
        .await?;

    debug!("RAW: {}", serde_json::to_string_pretty(&resp).unwrap());

    let asset = AssetDto {
        id: resp.get("id").unwrap().to_string(),
        name: resp.get("name").unwrap().to_string(),
        asset_lifecycle_state: resp.get("assetLifecycleState").unwrap().to_string(),
        asset_type_category: resp.get("assetTypeCategory").unwrap().to_string(),
        asset_type_id: resp.get("assetTypeId").unwrap().to_string(),
        manufacturer_id: resp.get("manufacturerId").unwrap().to_string(),
        manufacturer_name: resp.get("manufacturerName").unwrap().to_string(),
        monitoring_state: resp.get("monitoringState").unwrap().to_string(),
        parent_id: resp.get("parentId").unwrap().to_string(),
        parent_name: resp.get("parentName").unwrap().to_string(),
        product_id: resp.get("productId").unwrap().to_string(),
        product_name: resp.get("productName").unwrap().to_string(),
        status: resp.get("status").unwrap().to_string(),
        path: resp
            .get("tabDelimitedPath")
            .unwrap()
            .to_string()
            .replace("\\t", "/"),
    };

    Ok(asset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_asset_by_id_async() {
        // Arrange
        let asset_id = "3fa85f64-5717-4562-b3fc-2c963f66afa6".to_string();
        let url_path = format!("{}/{}", ASSET_API_PREFIX, asset_id);

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path(url_path);

            then.status(200)
                .header("Content-Type", "application/json")
                .body(
                    json!({
                        "hasChildren": false,
                        "locationData": null,
                        "baseInformationLastUpdated": "2021-10-25T18:17:09.979662+00:00",
                        "accessState": "full",
                        "tabDelimitedPath": "All\tEU\tLoc-001\tTestRack",
                        "accessPolicyId": "eea77bbe-c1fb-464e-841c-bce66ae5beb4",
                        "id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
                        "name": "TestRack",
                        "status": "normal",
                        "assetTypeId": "rack",
                        "assetTypeCategory": "rack",
                        "parentId": "a23f3ec8-89a4-4caa-95b0-0f6f0a77073f",
                        "parentName": "Loc-001",
                        "productId": "3afd7bbb-95e8-4bd0-924a-ccee26ac33bc",
                        "productName": "AR3100",
                        "manufacturerId": "e417483a-20b0-4b86-b0e0-2c2be6592892",
                        "manufacturerName": "APC",
                        "dimension": {},
                        "assetLifecycleState": "active",
                        "discoveryState": "manuallyCreated",
                        "monitoringState": "off",
                        "sensorMonitoringProfileType": "discovered"
                    })
                    .to_string(),
                );
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = reqwest::Client::new();
        let auth_header = "Bearer test_token".to_string();

        // Act
        let result = get_asset_by_id_async(&config, client, auth_header, asset_id.clone()).await;

        // Assert
        m.assert();
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.id, format!("\"{}\"", asset_id));
        assert_eq!(asset.name, "\"TestRack\"");
    }
}
