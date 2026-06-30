use log::{debug, error, trace};
use reqwest::{Client, header::AUTHORIZATION};
use uuid::Uuid;

use crate::retry_on_unauthorized_async;

use super::{
    api_constants::ASSET_PROPERTIES_API_PREFIX,
    app_errors::AppError,
    asset_properties_api_data::{AssetPropertyDto, AssetPropertyImportDto},
    auth::AuthToken,
    cli_data::AppConfig,
    common_types::MultiTypeValue,
};

pub async fn bulk_update_asset_property_async(
    config: &AppConfig,
    req: &Client,
    auth_token: &mut AuthToken,
    filename: String,
    asset_property_type: String,
) -> color_eyre::Result<()> {
    let mut reader = csv::Reader::from_path(filename)?;

    while let Some(Ok(record)) = reader.deserialize::<AssetPropertyImportDto>().next() {
        auth_token.refresh_if_needed_async(config).await?;

        if let Err(e) = retry_on_unauthorized_async!(
            config,
            auth_token,
            update_asset_property_async(
                config,
                req,
                &auth_token.header,
                record.asset_id,
                record.new_value.clone(),
                asset_property_type.clone(),
            )
            .await
        ) {
            error!(
                "Failed to update {asset_property_type} for asset id {}: {e}",
                record.asset_id
            );
        }
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
        let parsed_value = if current_value.data_type == "decimal" {
            let decimal_value = new_value.parse::<f64>()?;
            MultiTypeValue::FloatValue(decimal_value)
        } else {
            MultiTypeValue::StringValue(new_value)
        };

        let payload = AssetPropertyDto {
            id: current_value.id,
            property_type: current_value.property_type.clone(),
            value: parsed_value,
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
                .error_for_status()?
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
                .error_for_status()?
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
    debug!("Request URL: {target_url}");

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .error_for_status()?
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
    use std::io::Write;
    use std::str::FromStr;
    use std::time::Duration;

    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;

    fn auth_token() -> AuthToken {
        AuthToken::for_test("Bearer test_token", Duration::from_hours(1))
    }

    fn property_json(
        id: &str,
        property_type: &str,
        value: &serde_json::Value,
        data_type: &str,
    ) -> serde_json::Value {
        json!({
            "dataSource": "user",
            "assetPropertyDisplayCategory": "general",
            "isDeletable": false,
            "isEditable": true,
            "isInherited": false,
            "createdDateTime": "2023-08-04T17:33:45.462475+00:00",
            "updatedDateTime": "2023-08-04T17:33:45.462475+00:00",
            "minimumValue": null,
            "id": id,
            "type": property_type,
            "value": value,
            "dataType": data_type
        })
    }

    fn write_property_csv(rows: &[(Uuid, &str)]) -> tempfile::NamedTempFile {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "asset_id,new_value").unwrap();
        for (asset_id, new_value) in rows {
            writeln!(tmp, "{asset_id},{new_value}").unwrap();
        }
        tmp.flush().unwrap();
        tmp
    }

    #[tokio::test]
    async fn test_get_asset_property_list_async() {
        // Arrange
        let asset_id = Uuid::from_str("3a6c3022-6140-4e85-a64f-bf868766c4c8").unwrap();
        let url_path = format!("{ASSET_PROPERTIES_API_PREFIX}/{asset_id}");

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

    #[tokio::test]
    async fn test_get_named_asset_property_async_filters_to_matching_type() {
        let asset_id = Uuid::new_v4();
        let url_path = format!("{ASSET_PROPERTIES_API_PREFIX}/{asset_id}");

        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path(url_path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([
                    property_json("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa", "assetTag", &json!("tag-1"), "string"),
                    property_json("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb", "serialNumber", &json!("SN-1"), "string"),
                ]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = reqwest::Client::new();
        let auth = "Bearer t".to_string();

        let result = get_named_asset_property_async(
            &config,
            &client,
            &auth,
            asset_id,
            "assetTag".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property_type, "assetTag");
    }

    #[tokio::test]
    async fn test_update_asset_property_async_string_uses_put_when_id_present() {
        let asset_id = Uuid::new_v4();
        let property_uuid = "cccccccc-cccc-cccc-cccc-cccccccccccc";
        let get_path = format!("{ASSET_PROPERTIES_API_PREFIX}/{asset_id}");
        let put_path = format!("{ASSET_PROPERTIES_API_PREFIX}/{property_uuid}");

        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path(get_path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([property_json(
                    property_uuid,
                    "assetTag",
                    &json!("old"),
                    "string"
                )]));
        });
        let put_mock = server.mock(|when, then| {
            when.method(PUT)
                .path(put_path)
                .body_includes("new-tag")
                .body_includes("\"type\":\"assetTag\"");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({"ok": true}));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = reqwest::Client::new();

        update_asset_property_async(
            &config,
            &client,
            &"Bearer t".to_string(),
            asset_id,
            "new-tag".to_string(),
            "assetTag".to_string(),
        )
        .await
        .unwrap();

        put_mock.assert();
    }

    #[tokio::test]
    async fn test_update_asset_property_async_decimal_serializes_as_number() {
        let asset_id = Uuid::new_v4();
        let property_uuid = "dddddddd-dddd-dddd-dddd-dddddddddddd";
        let get_path = format!("{ASSET_PROPERTIES_API_PREFIX}/{asset_id}");
        let put_path = format!("{ASSET_PROPERTIES_API_PREFIX}/{property_uuid}");

        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path(get_path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([property_json(
                    property_uuid,
                    "designValue",
                    &json!(0.0),
                    "decimal"
                )]));
        });
        let put_mock = server.mock(|when, then| {
            when.method(PUT)
                .path(put_path)
                // numeric (not stringified) value in payload
                .body_includes("\"value\":12.5");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({"ok": true}));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = reqwest::Client::new();

        update_asset_property_async(
            &config,
            &client,
            &"Bearer t".to_string(),
            asset_id,
            "12.5".to_string(),
            "designValue".to_string(),
        )
        .await
        .unwrap();

        put_mock.assert();
    }

    #[tokio::test]
    async fn test_update_asset_property_async_decimal_rejects_non_numeric_input() {
        let asset_id = Uuid::new_v4();
        let property_uuid = "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee";
        let get_path = format!("{ASSET_PROPERTIES_API_PREFIX}/{asset_id}");

        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path(get_path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([property_json(
                    property_uuid,
                    "designValue",
                    &json!(0.0),
                    "decimal"
                )]));
        });
        // A PUT firing means we accepted bad input.
        let put_should_not_fire = server.mock(|when, then| {
            when.method(PUT);
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = reqwest::Client::new();

        let result = update_asset_property_async(
            &config,
            &client,
            &"Bearer t".to_string(),
            asset_id,
            "not-a-number".to_string(),
            "designValue".to_string(),
        )
        .await;

        assert!(result.is_err());
        put_should_not_fire.assert_calls(0);
    }

    #[tokio::test]
    async fn test_update_asset_property_async_errors_when_multiple_values_returned() {
        let asset_id = Uuid::new_v4();
        let get_path = format!("{ASSET_PROPERTIES_API_PREFIX}/{asset_id}");

        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path(get_path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([
                    property_json("11111111-1111-1111-1111-111111111111", "assetTag", &json!("a"), "string"),
                    property_json("22222222-2222-2222-2222-222222222222", "assetTag", &json!("b"), "string"),
                ]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = reqwest::Client::new();

        let result = update_asset_property_async(
            &config,
            &client,
            &"Bearer t".to_string(),
            asset_id,
            "new".to_string(),
            "assetTag".to_string(),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bulk_update_asset_property_async_continues_after_row_error() {
        let asset_fail = Uuid::new_v4();
        let asset_ok = Uuid::new_v4();
        let property_uuid_ok = "ffffffff-ffff-ffff-ffff-ffffffffffff";
        let get_fail_path = format!("{ASSET_PROPERTIES_API_PREFIX}/{asset_fail}");
        let get_ok_path = format!("{ASSET_PROPERTIES_API_PREFIX}/{asset_ok}");
        let put_ok_path = format!("{ASSET_PROPERTIES_API_PREFIX}/{property_uuid_ok}");

        let server = MockServer::start();
        // First asset's GET returns 500 — that row's update errors out.
        let get_fail_mock = server.mock(|when, then| {
            when.method(GET).path(get_fail_path);
            then.status(500);
        });
        // Second asset's GET + PUT both succeed — bulk loop must reach this row.
        let get_ok_mock = server.mock(|when, then| {
            when.method(GET).path(get_ok_path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([property_json(
                    property_uuid_ok,
                    "assetTag",
                    &json!("old"),
                    "string"
                )]));
        });
        let put_ok_mock = server.mock(|when, then| {
            when.method(PUT).path(put_ok_path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({"ok": true}));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = reqwest::Client::new();
        let mut token = auth_token();
        let csv = write_property_csv(&[(asset_fail, "fail"), (asset_ok, "ok")]);

        bulk_update_asset_property_async(
            &config,
            &client,
            &mut token,
            csv.path().to_string_lossy().to_string(),
            "assetTag".to_string(),
        )
        .await
        .expect("bulk update must not abort on a per-row error");

        get_fail_mock.assert();
        get_ok_mock.assert();
        put_ok_mock.assert();
    }
}
