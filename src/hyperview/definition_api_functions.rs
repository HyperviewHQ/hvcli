use log::debug;
use reqwest::{Client, header::AUTHORIZATION};
use uuid::Uuid;

use super::{
    api_constants::SENSOR_DEFINITION_TYPE_API_PREFIX,
    cli_data::{AppConfig, AssetTypes, SensorValueClass, UpdateDefinitionArgs},
    definition_api_data::{Definition, SensorType},
};

pub async fn list_definitions_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    api_prefix: &str,
) -> color_eyre::Result<Vec<Definition>> {
    let target_url = format!("{}{}", config.instance_url, api_prefix);
    debug!("Request URL: {target_url}");

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<Definition>>()
        .await?;

    Ok(resp)
}

pub async fn add_definition_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    api_prefix: &str,
    name: String,
    asset_type: AssetTypes,
    description: Option<String>,
) -> color_eyre::Result<Uuid> {
    let target_url = format!("{}{}", config.instance_url, api_prefix);
    debug!("Request URL: {target_url}");

    let definition = Definition {
        id: None,
        name,
        asset_type: asset_type.to_string(),
        description,
        associated_assets: 0,
    };

    let id = req
        .post(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(&definition)
        .send()
        .await?
        .error_for_status()?
        .json::<Uuid>()
        .await?;

    Ok(id)
}

pub async fn list_sensor_definition_types_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    asset_type: AssetTypes,
    sensor_class: SensorValueClass,
) -> color_eyre::Result<Vec<SensorType>> {
    let target_url = format!(
        "{}{}",
        config.instance_url, SENSOR_DEFINITION_TYPE_API_PREFIX
    );
    debug!("Request URL: {target_url}");

    let sensor_type_value_type = match sensor_class {
        SensorValueClass::Numeric => "numeric",
        SensorValueClass::Enum => "enum",
    };

    let query = [
        ("assetTypeId", asset_type.to_string()),
        ("sensorTypeValueType", sensor_type_value_type.to_string()),
    ];

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .query(&query)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<SensorType>>()
        .await?;

    Ok(resp)
}

pub async fn get_definition_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    api_prefix: &str,
    definition_id: Uuid,
) -> color_eyre::Result<Definition> {
    let target_url = format!("{}{}/{}", config.instance_url, api_prefix, definition_id);
    debug!("Request URL: {target_url}");

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .error_for_status()?
        .json::<Definition>()
        .await?;

    Ok(resp)
}

pub async fn update_definition_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    api_prefix: &str,
    options: &UpdateDefinitionArgs,
) -> color_eyre::Result<()> {
    let target_url = format!(
        "{}{}/{}",
        config.instance_url, api_prefix, options.definition_id
    );
    debug!("Request URL: {target_url}");

    let definition = Definition {
        id: Some(options.definition_id),
        name: options.name.clone(),
        asset_type: options.asset_type.to_string(),
        description: options.description.clone(),
        associated_assets: 0,
    };

    req.put(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(&definition)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn delete_definition_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    api_prefix: &str,
    definition_id: Uuid,
) -> color_eyre::Result<()> {
    let target_url = format!("{}{}/{}", config.instance_url, api_prefix, definition_id);
    debug!("Request URL: {target_url}");

    req.delete(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

/// Deletes a single sensor from a definition. `sub_resource` is the protocol/class path segment,
/// e.g. `"bacnetIpNumericSensors"` or `"modbusTcpNonNumericSensors"`.
pub async fn delete_sensor_definition_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    api_prefix: &str,
    sub_resource: &str,
    definition_id: Uuid,
    sensor_id: Uuid,
) -> color_eyre::Result<()> {
    let target_url = format!(
        "{}{}/{}/{}/{}",
        config.instance_url, api_prefix, sub_resource, definition_id, sensor_id
    );
    debug!("Request URL: {target_url}");

    req.delete(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_list_definitions_async_returns_definitions() {
        let definition_id = Uuid::new_v4();
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path("/api/setting/bacnetIpDefinitions");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([{
                    "id": definition_id.to_string(),
                    "name": "Test Definition",
                    "assetType": "crah",
                    "associatedAssets": 3
                }]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let resp = list_definitions_async(
            &config,
            &client,
            &auth_header,
            "/api/setting/bacnetIpDefinitions",
        )
        .await
        .unwrap();

        m.assert();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].id, Some(definition_id));
        assert_eq!(resp[0].name, "Test Definition");
        assert_eq!(resp[0].asset_type, "crah");
        assert_eq!(resp[0].associated_assets, 3);
    }

    #[tokio::test]
    async fn test_list_definitions_async_tolerates_unknown_asset_type() {
        // A definition whose asset type is not one this CLI enumerates must not fail the whole
        // list; asset_type is a plain String so the row still deserializes.
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path("/api/setting/bacnetIpDefinitions");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([
                    {"id": null, "name": "Known", "assetType": "crah", "associatedAssets": 0},
                    {"id": null, "name": "Unknown", "assetType": "SomeFutureAssetType", "associatedAssets": 0}
                ]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let resp = list_definitions_async(
            &config,
            &client,
            &auth_header,
            "/api/setting/bacnetIpDefinitions",
        )
        .await
        .unwrap();

        m.assert();
        assert_eq!(resp.len(), 2);
        assert_eq!(resp[1].asset_type, "SomeFutureAssetType");
    }

    #[tokio::test]
    async fn test_add_definition_async_sends_expected_body_and_returns_id() {
        let new_id = Uuid::new_v4();
        let server = MockServer::start();
        // description is omitted from the body when None (skip_serializing_if).
        let m = server.mock(|when, then| {
            when.method(POST)
                .path("/api/setting/modbusTcpDefinitions")
                .json_body(json!({
                    "id": null,
                    "name": "New Definition",
                    "assetType": "ups",
                    "associatedAssets": 0
                }));
            then.status(201)
                .header("Content-Type", "application/json")
                .json_body(json!(new_id.to_string()));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let id = add_definition_async(
            &config,
            &client,
            &auth_header,
            "/api/setting/modbusTcpDefinitions",
            "New Definition".to_string(),
            AssetTypes::Ups,
            None,
        )
        .await
        .unwrap();

        m.assert();
        assert_eq!(id, new_id);
    }

    #[tokio::test]
    async fn test_add_definition_async_includes_description_when_set() {
        let new_id = Uuid::new_v4();
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(POST)
                .path("/api/setting/bacnetIpDefinitions")
                .json_body(json!({
                    "id": null,
                    "name": "Described",
                    "assetType": "crah",
                    "description": "A test definition",
                    "associatedAssets": 0
                }));
            then.status(201)
                .header("Content-Type", "application/json")
                .json_body(json!(new_id.to_string()));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let id = add_definition_async(
            &config,
            &client,
            &auth_header,
            "/api/setting/bacnetIpDefinitions",
            "Described".to_string(),
            AssetTypes::Crah,
            Some("A test definition".to_string()),
        )
        .await
        .unwrap();

        m.assert();
        assert_eq!(id, new_id);
    }

    #[tokio::test]
    async fn test_list_sensor_definition_types_async_sends_expected_query() {
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET)
                .path("/api/setting/sensorTypeAssetType")
                .query_param("assetTypeId", "crah")
                .query_param("sensorTypeValueType", "numeric");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([{
                    "sensorTypeId": "type-1",
                    "sensorDescription": "Temperature",
                    "unitId": "unit-1",
                    "unitDescription": "Celsius"
                }]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let resp = list_sensor_definition_types_async(
            &config,
            &client,
            &auth_header,
            AssetTypes::Crah,
            SensorValueClass::Numeric,
        )
        .await
        .unwrap();

        m.assert();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].sensor_type_id, "type-1");
        assert_eq!(resp[0].unit_description, "Celsius");
    }

    #[tokio::test]
    async fn test_list_sensor_definition_types_async_sends_enum_class() {
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET)
                .path("/api/setting/sensorTypeAssetType")
                .query_param("assetTypeId", "rack")
                .query_param("sensorTypeValueType", "enum");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let resp = list_sensor_definition_types_async(
            &config,
            &client,
            &auth_header,
            AssetTypes::Rack,
            SensorValueClass::Enum,
        )
        .await
        .unwrap();

        m.assert();
        assert!(resp.is_empty());
    }

    #[tokio::test]
    async fn test_list_definitions_async_propagates_http_error() {
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path("/api/setting/bacnetIpDefinitions");
            then.status(500);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let result = list_definitions_async(
            &config,
            &client,
            &auth_header,
            "/api/setting/bacnetIpDefinitions",
        )
        .await;

        m.assert();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_definition_async_returns_single_definition() {
        let definition_id = Uuid::new_v4();
        let path = format!("/api/setting/modbusTcpDefinitions/{definition_id}");

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path(path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "id": definition_id.to_string(),
                    "name": "Def A",
                    "assetType": "ups",
                    "description": "hi",
                    "associatedAssets": 7
                }));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let resp = get_definition_async(
            &config,
            &client,
            &auth_header,
            "/api/setting/modbusTcpDefinitions",
            definition_id,
        )
        .await
        .unwrap();

        m.assert();
        assert_eq!(resp.id, Some(definition_id));
        assert_eq!(resp.description.as_deref(), Some("hi"));
        assert_eq!(resp.associated_assets, 7);
    }

    #[tokio::test]
    async fn test_update_definition_async_puts_body_with_id() {
        let definition_id = Uuid::new_v4();
        let path = format!("/api/setting/bacnetIpDefinitions/{definition_id}");

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(PUT).path(path).json_body(json!({
                "id": definition_id.to_string(),
                "name": "Renamed",
                "assetType": "crah",
                "description": "d",
                "associatedAssets": 0
            }));
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let options = UpdateDefinitionArgs {
            definition_id,
            name: "Renamed".to_string(),
            asset_type: AssetTypes::Crah,
            description: Some("d".to_string()),
        };

        update_definition_async(
            &config,
            &client,
            &auth_header,
            "/api/setting/bacnetIpDefinitions",
            &options,
        )
        .await
        .unwrap();

        m.assert();
    }

    #[tokio::test]
    async fn test_delete_definition_async_hits_item_path() {
        let definition_id = Uuid::new_v4();
        let path = format!("/api/setting/modbusTcpDefinitions/{definition_id}");

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(DELETE).path(path);
            then.status(204);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        delete_definition_async(
            &config,
            &client,
            &auth_header,
            "/api/setting/modbusTcpDefinitions",
            definition_id,
        )
        .await
        .unwrap();

        m.assert();
    }

    #[tokio::test]
    async fn test_delete_sensor_definition_async_builds_nested_path() {
        let definition_id = Uuid::new_v4();
        let sensor_id = Uuid::new_v4();
        let path = format!(
            "/api/setting/bacnetIpDefinitions/bacnetIpNumericSensors/{definition_id}/{sensor_id}"
        );

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(DELETE).path(path);
            then.status(204);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        delete_sensor_definition_async(
            &config,
            &client,
            &auth_header,
            "/api/setting/bacnetIpDefinitions",
            "bacnetIpNumericSensors",
            definition_id,
            sensor_id,
        )
        .await
        .unwrap();

        m.assert();
    }

    #[tokio::test]
    async fn test_delete_sensor_definition_async_propagates_http_error() {
        let definition_id = Uuid::new_v4();
        let sensor_id = Uuid::new_v4();
        let path = format!(
            "/api/setting/modbusTcpDefinitions/modbusTcpNonNumericSensors/{definition_id}/{sensor_id}"
        );

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(DELETE).path(path);
            then.status(404);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let result = delete_sensor_definition_async(
            &config,
            &client,
            &auth_header,
            "/api/setting/modbusTcpDefinitions",
            "modbusTcpNonNumericSensors",
            definition_id,
            sensor_id,
        )
        .await;

        m.assert();
        assert!(result.is_err());
    }
}
