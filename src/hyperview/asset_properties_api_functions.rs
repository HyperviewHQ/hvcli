use log::{debug, trace};
use reqwest::{Client, header::AUTHORIZATION};
use uuid::Uuid;

use crate::hyperview::common_types::MultiTypeValue;

use super::{
    api_constants::ASSET_PROPERTIES_API_PREFIX,
    app_errors::AppError,
    asset_properties_api_data::{AssetPropertyDto, AssetPropertyImportDto},
    cli_data::AppConfig,
};

pub async fn bulk_update_asset_property_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    filename: String,
    asset_property_type: String,
) -> color_eyre::Result<()> {
    let mut reader = csv::Reader::from_path(filename)?;

    while let Some(Ok(record)) = reader.deserialize::<AssetPropertyImportDto>().next() {
        update_asset_property_async(
            config,
            req,
            auth_header,
            record.asset_id,
            record.new_value,
            asset_property_type.clone(),
        )
        .await?;
    }
    Ok(())
}

pub async fn update_asset_property_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    id: Uuid,
    new_value: String,
    asset_property_type: String,
) -> color_eyre::Result<()> {
    let current_values =
        get_named_asset_property_async(config, req, auth_header, id, asset_property_type).await?;

    debug!(
        "Current property values: {}",
        serde_json::to_string_pretty(&current_values)?
    );

    if current_values.len() > 1 {
        return Err(AppError::MultipleValuesDetectedForProperty.into());
    }

    if let Some(current_value) = current_values.first() {
        let payload = AssetPropertyDto {
            id: current_value.id,
            property_type: current_value.property_type.clone(),
            value: MultiTypeValue::StringValue(new_value),
            data_type: current_value.data_type.clone(),
            data_source: current_value.data_source.clone(),
            asset_property_display_category: current_value.asset_property_display_category.clone(),
            is_deletable: current_value.is_deletable,
            is_editable: current_value.is_editable,
            is_inherited: current_value.is_inherited,
            created_date_time: current_value.created_date_time.clone(),
            updated_date_time: current_value.updated_date_time.clone(),
            minimum_value: current_value.minimum_value.clone(),
        };

        trace!("Payload: {}", serde_json::to_string_pretty(&payload)?);

        if let Some(id) = payload.id {
            // Updating an existing value
            let target_url = format!(
                "{}{}/{}",
                config.instance_url, ASSET_PROPERTIES_API_PREFIX, id
            );
            debug!("Request URL: {target_url}");

            let resp = req
                .put(target_url)
                .header(AUTHORIZATION, auth_header)
                .json(&payload)
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;

            debug!(
                "Update serial number: {}",
                serde_json::to_string_pretty(&resp)?
            );
        } else {
            // Setting serial number for the first time
            let target_url = format!(
                "{}{}/?assetId={}",
                config.instance_url, ASSET_PROPERTIES_API_PREFIX, id
            );
            debug!("Request URL: {target_url}");

            let resp = req
                .post(target_url)
                .header(AUTHORIZATION, auth_header)
                .json(&payload)
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;

            debug!(
                "Update serial number: {}",
                serde_json::to_string_pretty(&resp)?
            );
        }
    }

    Ok(())
}

pub async fn get_asset_property_list_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    id: Uuid,
) -> color_eyre::Result<Vec<AssetPropertyDto>> {
    let target_url = format!(
        "{}{}/{}",
        config.instance_url, ASSET_PROPERTIES_API_PREFIX, id
    );
    debug!("Request URL: {target_url:?}");

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
    req: &Client,
    auth_header: &String,
    id: Uuid,
    property_type: String,
) -> color_eyre::Result<Vec<AssetPropertyDto>> {
    let property_list = get_asset_property_list_async(config, req, auth_header, id)
        .await?
        .into_iter()
        .filter(|p| p.property_type == property_type)
        .collect::<Vec<AssetPropertyDto>>();

    Ok(property_list)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_asset_property_list_async() {
        // Arrange
        let asset_id = Uuid::from_str("3a6c3022-6140-4e85-a64f-bf868766c4c8").unwrap();
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
                            "isDeletable": false,
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
                            "isDeletable": false,
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
        let result = get_asset_property_list_async(&config, &client, &auth_header, asset_id).await;

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
