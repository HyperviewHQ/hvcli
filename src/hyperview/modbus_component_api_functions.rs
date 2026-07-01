use log::debug;
use reqwest::{Client, header::AUTHORIZATION};
use uuid::Uuid;

use super::{
    api_constants::MODBUS_DEFINITION_API_PREFIX,
    cli_data::AppConfig,
    modbus_component_api_data::{ModbusComponentCreateDto, ModbusComponentDto},
};

pub async fn list_modbus_components_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    definition_id: Uuid,
) -> color_eyre::Result<Vec<ModbusComponentDto>> {
    let target_url = format!(
        "{}{}/modbusTcpComponents/{}",
        config.instance_url, MODBUS_DEFINITION_API_PREFIX, definition_id
    );
    debug!("Request URL: {target_url}");

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<ModbusComponentDto>>()
        .await?;

    Ok(resp)
}

pub async fn add_modbus_component_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    definition_id: Uuid,
    name: String,
) -> color_eyre::Result<Uuid> {
    let target_url = format!(
        "{}{}/modbusTcpComponents/{}",
        config.instance_url, MODBUS_DEFINITION_API_PREFIX, definition_id
    );
    debug!("Request URL: {target_url}");

    let component = ModbusComponentCreateDto { id: None, name };

    let id = req
        .post(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(&component)
        .send()
        .await?
        .error_for_status()?
        .json::<Uuid>()
        .await?;

    Ok(id)
}

pub async fn update_modbus_component_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    definition_id: Uuid,
    component_id: Uuid,
    name: String,
) -> color_eyre::Result<()> {
    // Update is a PUT to the collection path; the component id travels in the body.
    let target_url = format!(
        "{}{}/modbusTcpComponents/{}",
        config.instance_url, MODBUS_DEFINITION_API_PREFIX, definition_id
    );
    debug!("Request URL: {target_url}");

    let component = ModbusComponentCreateDto {
        id: Some(component_id),
        name,
    };

    req.put(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(&component)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn delete_modbus_component_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    definition_id: Uuid,
    component_id: Uuid,
) -> color_eyre::Result<()> {
    let target_url = format!(
        "{}{}/modbusTcpComponents/{}/{}",
        config.instance_url, MODBUS_DEFINITION_API_PREFIX, definition_id, component_id
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

    #[tokio::test]
    async fn test_list_modbus_components_async_returns_dtos() {
        let definition_id = Uuid::new_v4();
        let component_id = Uuid::new_v4();
        let list_path =
            format!("/api/setting/modbusTcpDefinitions/modbusTcpComponents/{definition_id}");

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path(list_path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([{
                    "id": component_id.to_string(),
                    "name": "PDU 1",
                    "numericSensorCount": 4,
                    "nonNumericSensorCount": 2
                }]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let resp = list_modbus_components_async(&config, &client, &auth_header, definition_id)
            .await
            .unwrap();

        m.assert();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].id, Some(component_id));
        assert_eq!(resp[0].name, "PDU 1");
        assert_eq!(resp[0].numeric_sensor_count, 4);
        assert_eq!(resp[0].non_numeric_sensor_count, 2);
    }

    #[tokio::test]
    async fn test_add_modbus_component_async_sends_name_and_returns_id() {
        let definition_id = Uuid::new_v4();
        let new_id = Uuid::new_v4();
        let create_path =
            format!("/api/setting/modbusTcpDefinitions/modbusTcpComponents/{definition_id}");

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(POST)
                .path(create_path)
                .json_body(json!({"id": null, "name": "PDU 2"}));
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

        let id = add_modbus_component_async(
            &config,
            &client,
            &auth_header,
            definition_id,
            "PDU 2".to_string(),
        )
        .await
        .unwrap();

        m.assert();
        assert_eq!(id, new_id);
    }

    #[tokio::test]
    async fn test_update_modbus_component_async_puts_id_and_name() {
        let definition_id = Uuid::new_v4();
        let component_id = Uuid::new_v4();
        let update_path =
            format!("/api/setting/modbusTcpDefinitions/modbusTcpComponents/{definition_id}");

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(PUT)
                .path(update_path)
                .json_body(json!({"id": component_id.to_string(), "name": "Renamed"}));
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        update_modbus_component_async(
            &config,
            &client,
            &auth_header,
            definition_id,
            component_id,
            "Renamed".to_string(),
        )
        .await
        .unwrap();

        m.assert();
    }

    #[tokio::test]
    async fn test_delete_modbus_component_async_hits_item_path() {
        let definition_id = Uuid::new_v4();
        let component_id = Uuid::new_v4();
        let delete_path = format!(
            "/api/setting/modbusTcpDefinitions/modbusTcpComponents/{definition_id}/{component_id}"
        );

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(DELETE).path(delete_path);
            then.status(204);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        delete_modbus_component_async(&config, &client, &auth_header, definition_id, component_id)
            .await
            .unwrap();

        m.assert();
    }

    #[tokio::test]
    async fn test_delete_modbus_component_async_propagates_http_error() {
        let definition_id = Uuid::new_v4();
        let component_id = Uuid::new_v4();
        let delete_path = format!(
            "/api/setting/modbusTcpDefinitions/modbusTcpComponents/{definition_id}/{component_id}"
        );

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(DELETE).path(delete_path);
            then.status(404);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let result = delete_modbus_component_async(
            &config,
            &client,
            &auth_header,
            definition_id,
            component_id,
        )
        .await;

        m.assert();
        assert!(result.is_err());
    }
}
