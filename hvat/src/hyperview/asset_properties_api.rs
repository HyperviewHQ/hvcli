use color_eyre::eyre::{Ok, Result};
use log::debug;
use reqwest::{header::AUTHORIZATION, Client};
use uuid::Uuid;

use crate::hyperview::{
    api_constants::ASSET_PROPERTIES_API_PREFIX, app_errors::AppError,
    asset_properties_api_data::AssetPropertyDto, cli_data::AppConfig,
};

pub async fn get_asset_property_list_async(
    config: &AppConfig,
    req: Client,
    auth_header: String,
    id: String,
) -> Result<Vec<AssetPropertyDto>> {
    if Uuid::parse_str(&id).is_err() {
        return Err(AppError::InvalidId.into());
    }

    let target_url = format!(
        "{}{}/{}",
        config.instance_url, ASSET_PROPERTIES_API_PREFIX, id
    );
    debug!("Request URL: {:?}", target_url);

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .json::<Vec<AssetPropertyDto>>()
        .await?;

    Ok(resp)
}

pub async fn get_named_asset_property_async(
    config: &AppConfig,
    req: Client,
    auth_header: String,
    id: String,
    property_type: String,
) -> Result<Vec<AssetPropertyDto>> {
    let property_list = get_asset_property_list_async(config, req, auth_header, id)
        .await?
        .into_iter()
        .filter(|p| p.property_type == property_type)
        .collect::<Vec<AssetPropertyDto>>();

    Ok(property_list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_asset_property_list_async() {
        // Arrange
        let asset_id = "3a6c3022-6140-4e85-a64f-bf868766c4c8".to_string();
        let url_path = format!("{}/{}", ASSET_PROPERTIES_API_PREFIX, asset_id);

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path(url_path);

            then.status(200)
                .header("Content-Type", "application/json")
                .body(
                    json!(
                        [
                          {
                            "dataSource": "snmp",
                            "assetPropertyDisplayCategory": "power",
                            "isEditable": false,
                            "isInherited": true,
                            "createdDateTime": "2023-08-04T17:33:45.462475+00:00",
                            "updatedDateTime": "2023-08-04T17:33:45.462475+00:00",
                            "minimumValue": null,
                            "id": "0702c619-ee10-4af2-bda7-471772ac97c3",
                            "type": "numberOfPhases",
                            "value": 3,
                            "dataType": "integer"
                          },
                          {
                            "dataSource": "snmp",
                            "assetPropertyDisplayCategory": "general",
                            "isEditable": false,
                            "isInherited": false,
                            "createdDateTime": "2023-08-04T17:33:45.462475+00:00",
                            "updatedDateTime": "2023-08-04T17:33:45.462475+00:00",
                            "minimumValue": null,
                            "id": "0bce12c7-5b74-4c78-9eb2-fc52efbbf4b3",
                            "type": "firmwareVersion",
                            "value": "6.3.3",
                            "dataType": "string"
                          }
                        ]
                    )
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
        let result = get_asset_property_list_async(&config, client, auth_header, asset_id).await;

        // Assert
        m.assert();
        let property_list = result.unwrap();
        assert_eq!(property_list.len(), 2);
        assert_eq!(property_list[0].property_type, "numberOfPhases".to_string());
        assert_eq!(
            property_list[1].property_type,
            "firmwareVersion".to_string()
        );
    }
}
