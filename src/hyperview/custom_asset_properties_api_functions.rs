use log::{debug, trace};
use reqwest::{Client, header::AUTHORIZATION};
use serde_json::Value;
use uuid::Uuid;

use crate::hyperview::{
    app_errors::AppError, custom_asset_properties_api_data::CustomAssetPropertyUpdateDto,
};

use super::{
    api_constants::CUSTOM_ASSET_PROPERTIES_API_PREFIX, cli_data::AppConfig,
    custom_asset_properties_api_data::CustomAssetPropertyDto,
};

pub async fn get_custom_asset_property_list_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    id: Uuid,
) -> color_eyre::Result<Vec<CustomAssetPropertyDto>> {
    let target_url = format!(
        "{}{}/{}",
        config.instance_url, CUSTOM_ASSET_PROPERTIES_API_PREFIX, id
    );
    debug!("Request URL: {:?}", target_url);

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .json::<Vec<CustomAssetPropertyDto>>()
        .await?;

    Ok(resp)
}

pub async fn update_custom_property_by_name_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    id: Uuid,
    custom_asset_property_name: String,
    new_custom_property_value: String,
) -> color_eyre::Result<()> {
    let custom_asset_property_list =
        get_custom_asset_property_list_async(config, req, auth_header, id).await?;

    let custom_asset_property = custom_asset_property_list
        .into_iter()
        .filter(|p| p.name == custom_asset_property_name)
        .collect::<Vec<CustomAssetPropertyDto>>();

    if custom_asset_property.is_empty() {
        return Err(AppError::AssetDoesNotHavePropertyName(custom_asset_property_name).into());
    }

    let custom_property = custom_asset_property
        .first()
        .expect("Asset should have a custom property that can be updated");
    debug!("Custom property to update: {custom_property:#?}");

    let target_url = format!(
        "{}{}/{}",
        config.instance_url,
        CUSTOM_ASSET_PROPERTIES_API_PREFIX,
        custom_property.id.clone()
    );
    debug!("Request URL: {:?}", target_url);

    let update_dto = CustomAssetPropertyUpdateDto {
        custom_asset_property_key_id: custom_property.custom_asset_property_key_id.clone(),
        data_type: custom_property.data_type.clone(),
        group_name: custom_property.group_name.clone(),
        id: custom_property.id.clone(),
        value: new_custom_property_value,
    };
    trace!(
        "New custom property update DTO: {}",
        serde_json::to_string_pretty(&update_dto).expect("Could not serialize update_dto to JSON")
    );

    let resp = req
        .put(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(&update_dto)
        .send()
        .await?
        .json::<Value>()
        .await?;

    debug!(
        "Update custom asset property response: {}",
        serde_json::to_string_pretty(&resp).expect("Could not serialize response to JSON")
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_custom_asset_property_list_async() {
        // Arrange
        let asset_id = Uuid::from_str("3a6c3022-6140-4e85-a64f-bf868766c4c8").unwrap();
        let url_path = format!("{}/{}", CUSTOM_ASSET_PROPERTIES_API_PREFIX, asset_id);

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path(url_path);

            then.status(200)
                .header("Content-Type", "application/json")
                .body(
                    json!(
                        [
                          {
                            "id": "043036ac-3ad1-453a-a45b-fed4fe6954e4",
                            "customAssetPropertyKeyId": "4b4154eb-975f-4cdf-9ccd-e1fad3230e83",
                            "customAssetPropertyGroupId": "269040f8-4c7e-4621-9b55-0a747b7df48b",
                            "value": null,
                            "dataType": "string",
                            "name": "RFCode Asset Tag",
                            "groupName": "SS Testing",
                            "dataSource": "user",
                            "updatedDateTime": "2023-08-04T17:33:45.462475+00:00",
                            "unit": ""
                          },
                          {
                            "id": "09ce99a0-21b9-4faf-ad10-620cce43f018",
                            "customAssetPropertyKeyId": "d5221878-b9d6-44f1-aee4-7ffa5121cc1c",
                            "customAssetPropertyGroupId": "269040f8-4c7e-4621-9b55-0a747b7df48b",
                            "value": null,
                            "dataType": "string",
                            "name": "CST-1204",
                            "groupName": "SS Testing",
                            "dataSource": "user",
                            "updatedDateTime": "2023-08-04T17:33:45.462475+00:00",
                            "unit": ""
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
        let result =
            get_custom_asset_property_list_async(&config, &client, &auth_header, asset_id).await;

        // Assert
        m.assert();
        let property_list = result.unwrap();
        assert_eq!(property_list.len(), 2);
        assert_eq!(property_list[0].name, "RFCode Asset Tag".to_string());
        assert_eq!(property_list[1].name, "CST-1204".to_string());
    }
}
