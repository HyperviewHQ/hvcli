use log::{debug, error};
use reqwest::{Client, header::AUTHORIZATION};
use uuid::Uuid;

use crate::retry_on_unauthorized_async;

use super::{
    api_constants::BACNET_DEFINITION_API_PREFIX,
    auth::AuthToken,
    bacnet_definition_api_data::{
        BacnetNonNumericSensorDefinitionDto, BacnetNonNumericSensorDefinitionImportCsv,
        BacnetNonNumericSensorDefinitionImportDto, BacnetNumericSensorDefinitionDto,
        BacnetNumericSensorDefinitionImportDto,
    },
    cli_data::AppConfig,
};

pub async fn list_bacnet_numeric_sensor_definitions_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    definition_id: Uuid,
) -> color_eyre::Result<Vec<BacnetNumericSensorDefinitionDto>> {
    let target_url = format!(
        "{}{}/bacnetIpNumericSensors/{}",
        config.instance_url, BACNET_DEFINITION_API_PREFIX, definition_id
    );
    debug!("Request URL: {target_url}");

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<BacnetNumericSensorDefinitionDto>>()
        .await?;

    Ok(resp)
}

pub async fn list_bacnet_non_numeric_sensor_definitions_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    definition_id: Uuid,
) -> color_eyre::Result<Vec<BacnetNonNumericSensorDefinitionDto>> {
    let target_url = format!(
        "{}{}/bacnetIpNonNumericSensors/{}",
        config.instance_url, BACNET_DEFINITION_API_PREFIX, definition_id
    );
    debug!("Request URL: {target_url}");

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<BacnetNonNumericSensorDefinitionDto>>()
        .await?;

    Ok(resp)
}

async fn create_bacnet_numeric_sensor_definition_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    definition_id: Uuid,
    sensor: &BacnetNumericSensorDefinitionImportDto,
) -> color_eyre::Result<()> {
    let target_url = format!(
        "{}{}/bacnetIpNumericSensors/{}",
        config.instance_url, BACNET_DEFINITION_API_PREFIX, definition_id
    );
    debug!("Request URL: {target_url}");

    req.post(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(sensor)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

async fn update_bacnet_numeric_sensor_definition_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    definition_id: Uuid,
    sensor_id: Uuid,
    sensor: &BacnetNumericSensorDefinitionImportDto,
) -> color_eyre::Result<()> {
    let target_url = format!(
        "{}{}/bacnetIpNumericSensors/{}/{}",
        config.instance_url, BACNET_DEFINITION_API_PREFIX, definition_id, sensor_id
    );
    debug!("Request URL: {target_url}");

    req.put(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(sensor)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn bulk_import_bacnet_numeric_sensor_definitions_async(
    config: &AppConfig,
    req: &Client,
    auth_token: &mut AuthToken,
    filename: &String,
    definition_id: Uuid,
    create_as_new: bool,
) -> color_eyre::Result<()> {
    let mut reader = csv::Reader::from_path(filename)?;
    let mut total: usize = 0;
    let mut failed: usize = 0;

    for result in reader.deserialize::<BacnetNumericSensorDefinitionImportDto>() {
        total += 1;

        let mut sensor = match result {
            Ok(sensor) => sensor,
            Err(e) => {
                error!("Failed to parse CSV row: {e}");
                failed += 1;
                continue;
            }
        };

        // Force every row to be created (ignore any id) so an export can be cloned into a
        // different definition.
        if create_as_new {
            sensor.id = None;
        }

        auth_token.refresh_if_needed_async(config).await?;

        debug!("Processing sensor definition row: {sensor:?}");

        let result = match sensor.id {
            Some(id) => {
                retry_on_unauthorized_async!(
                    config,
                    auth_token,
                    update_bacnet_numeric_sensor_definition_async(
                        config,
                        req,
                        &auth_token.header,
                        definition_id,
                        id,
                        &sensor,
                    )
                    .await
                )
            }
            None => {
                retry_on_unauthorized_async!(
                    config,
                    auth_token,
                    create_bacnet_numeric_sensor_definition_async(
                        config,
                        req,
                        &auth_token.header,
                        definition_id,
                        &sensor,
                    )
                    .await
                )
            }
        };

        if let Err(e) = result {
            error!("Failed to import sensor definition '{}': {e}", sensor.name);
            failed += 1;
        }
    }

    if failed > 0 {
        return Err(super::app_errors::AppError::BulkOperationFailures { failed, total }.into());
    }

    Ok(())
}

async fn create_bacnet_non_numeric_sensor_definition_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    definition_id: Uuid,
    sensor: &BacnetNonNumericSensorDefinitionImportDto,
) -> color_eyre::Result<()> {
    let target_url = format!(
        "{}{}/bacnetIpNonNumericSensors/{}",
        config.instance_url, BACNET_DEFINITION_API_PREFIX, definition_id
    );
    debug!("Request URL: {target_url}");

    req.post(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(sensor)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

async fn update_bacnet_non_numeric_sensor_definition_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    definition_id: Uuid,
    sensor_id: Uuid,
    sensor: &BacnetNonNumericSensorDefinitionImportDto,
) -> color_eyre::Result<()> {
    let target_url = format!(
        "{}{}/bacnetIpNonNumericSensors/{}/{}",
        config.instance_url, BACNET_DEFINITION_API_PREFIX, definition_id, sensor_id
    );
    debug!("Request URL: {target_url}");

    req.put(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(sensor)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn bulk_import_bacnet_non_numeric_sensor_definitions_async(
    config: &AppConfig,
    req: &Client,
    auth_token: &mut AuthToken,
    filename: &String,
    definition_id: Uuid,
    create_as_new: bool,
) -> color_eyre::Result<()> {
    let mut reader = csv::Reader::from_path(filename)?;
    let mut total: usize = 0;
    let mut failed: usize = 0;

    for result in reader.deserialize::<BacnetNonNumericSensorDefinitionImportCsv>() {
        total += 1;

        let row = match result {
            Ok(row) => row,
            Err(e) => {
                error!("Failed to parse CSV row: {e}");
                failed += 1;
                continue;
            }
        };

        auth_token.refresh_if_needed_async(config).await?;

        let mut sensor = match BacnetNonNumericSensorDefinitionImportDto::try_from(&row) {
            Ok(sensor) => sensor,
            Err(e) => {
                error!(
                    "Failed to parse value mapping for sensor '{}': {e}",
                    row.name
                );
                failed += 1;
                continue;
            }
        };

        // Force every row to be created (ignore any id) so an export can be cloned into a
        // different definition.
        if create_as_new {
            sensor.id = None;
        }

        debug!("Processing sensor definition row: {sensor:?}");

        let result = match sensor.id {
            Some(id) => {
                retry_on_unauthorized_async!(
                    config,
                    auth_token,
                    update_bacnet_non_numeric_sensor_definition_async(
                        config,
                        req,
                        &auth_token.header,
                        definition_id,
                        id,
                        &sensor,
                    )
                    .await
                )
            }
            None => {
                retry_on_unauthorized_async!(
                    config,
                    auth_token,
                    create_bacnet_non_numeric_sensor_definition_async(
                        config,
                        req,
                        &auth_token.header,
                        definition_id,
                        &sensor,
                    )
                    .await
                )
            }
        };

        if let Err(e) = result {
            error!("Failed to import sensor definition '{}': {e}", sensor.name);
            failed += 1;
        }
    }

    if failed > 0 {
        return Err(super::app_errors::AppError::BulkOperationFailures { failed, total }.into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;
    use std::io::Write;
    use std::time::Duration;

    fn auth_token() -> AuthToken {
        AuthToken::for_test("Bearer test_token", Duration::from_hours(1))
    }

    #[tokio::test]
    async fn test_list_bacnet_numeric_sensor_definitions_async_returns_dtos() {
        let definition_id = Uuid::new_v4();
        let sensor_id = Uuid::new_v4();
        let list_path =
            format!("/api/setting/bacnetIpDefinitions/bacnetIpNumericSensors/{definition_id}");

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path(list_path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([{
                    "id": sensor_id.to_string(),
                    "name": "Room Temp",
                    "multiplier": 1.0,
                    "offset": 0.0,
                    "orderOfOperations": "scaleThenOffset",
                    "objectInstance": 5,
                    "objectType": "analogInput",
                    "sensorType": "Temperature",
                    "sensorTypeId": "type-1",
                    "unit": "C",
                    "unitId": "unit-1"
                }]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let resp = list_bacnet_numeric_sensor_definitions_async(
            &config,
            &client,
            &auth_header,
            definition_id,
        )
        .await
        .unwrap();

        m.assert();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].id, sensor_id);
        assert_eq!(resp[0].object_instance, 5);
        assert_eq!(resp[0].offset, 0.0);
        assert_eq!(resp[0].order_of_operations, "scaleThenOffset");
        assert_eq!(resp[0].unit.as_deref(), Some("C"));
    }

    #[tokio::test]
    async fn test_list_bacnet_non_numeric_sensor_definitions_async_returns_dtos() {
        let definition_id = Uuid::new_v4();
        let sensor_id = Uuid::new_v4();
        let list_path =
            format!("/api/setting/bacnetIpDefinitions/bacnetIpNonNumericSensors/{definition_id}");

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path(list_path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([{
                    "id": sensor_id.to_string(),
                    "name": "Status",
                    "objectInstance": 2,
                    "objectType": "binaryInput",
                    "sensorType": "Status",
                    "sensorTypeId": "type-2",
                    "valueMapping": [
                        {"text": "Inactive", "value": 0},
                        {"text": "Active", "value": 1}
                    ]
                }]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let resp = list_bacnet_non_numeric_sensor_definitions_async(
            &config,
            &client,
            &auth_header,
            definition_id,
        )
        .await
        .unwrap();

        m.assert();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].value_mapping.len(), 2);
        assert_eq!(resp[0].value_mapping[1].text, "Active");
    }

    fn write_numeric_csv(rows: &[(&str, &str)]) -> tempfile::NamedTempFile {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            tmp,
            "id,name,multiplier,object_instance,object_type,sensor_type,sensor_type_id,unit,unit_id"
        )
        .unwrap();
        for (id, name) in rows {
            writeln!(
                tmp,
                "{id},{name},1.0,5,analogInput,Temperature,type-1,C,unit-1"
            )
            .unwrap();
        }
        tmp.flush().unwrap();
        tmp
    }

    #[tokio::test]
    async fn test_bulk_import_bacnet_numeric_sensor_definitions_async_creates_new_row() {
        let definition_id = Uuid::new_v4();
        let create_path =
            format!("/api/setting/bacnetIpDefinitions/bacnetIpNumericSensors/{definition_id}");

        let server = MockServer::start();
        let create_mock = server.mock(|when, then| {
            when.method(POST).path(create_path);
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        let csv = write_numeric_csv(&[("", "New Sensor")]);

        bulk_import_bacnet_numeric_sensor_definitions_async(
            &config,
            &client,
            &mut token,
            &csv.path().to_string_lossy().to_string(),
            definition_id,
            false,
        )
        .await
        .unwrap();

        create_mock.assert();
    }

    #[tokio::test]
    async fn test_bulk_import_bacnet_numeric_sensor_definitions_async_updates_existing_row() {
        let definition_id = Uuid::new_v4();
        let sensor_id = Uuid::new_v4();
        let update_path = format!(
            "/api/setting/bacnetIpDefinitions/bacnetIpNumericSensors/{definition_id}/{sensor_id}"
        );

        let server = MockServer::start();
        let update_mock = server.mock(|when, then| {
            when.method(PUT).path(update_path);
            then.status(200);
        });
        let create_should_not_fire = server.mock(|when, then| {
            when.method(POST).path(format!(
                "/api/setting/bacnetIpDefinitions/bacnetIpNumericSensors/{definition_id}"
            ));
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        let csv = write_numeric_csv(&[(&sensor_id.to_string(), "Existing Sensor")]);

        bulk_import_bacnet_numeric_sensor_definitions_async(
            &config,
            &client,
            &mut token,
            &csv.path().to_string_lossy().to_string(),
            definition_id,
            false,
        )
        .await
        .unwrap();

        update_mock.assert();
        create_should_not_fire.assert_calls(0);
    }

    #[tokio::test]
    async fn test_bulk_import_bacnet_numeric_sensor_definitions_async_create_as_new_ignores_id() {
        // A row with a real UUID id, imported with create_as_new = true, must be CREATED (POST),
        // not updated (PUT) — enabling an export from one definition to be cloned into another.
        let definition_id = Uuid::new_v4();
        let sensor_id = Uuid::new_v4();
        let create_path =
            format!("/api/setting/bacnetIpDefinitions/bacnetIpNumericSensors/{definition_id}");

        let server = MockServer::start();
        let create_mock = server.mock(|when, then| {
            when.method(POST).path(create_path);
            then.status(200);
        });
        let update_should_not_fire = server.mock(|when, then| {
            when.method(PUT).path(format!(
                "/api/setting/bacnetIpDefinitions/bacnetIpNumericSensors/{definition_id}/{sensor_id}"
            ));
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        let csv = write_numeric_csv(&[(&sensor_id.to_string(), "Cloned Sensor")]);

        bulk_import_bacnet_numeric_sensor_definitions_async(
            &config,
            &client,
            &mut token,
            &csv.path().to_string_lossy().to_string(),
            definition_id,
            true,
        )
        .await
        .unwrap();

        create_mock.assert();
        update_should_not_fire.assert_calls(0);
    }

    #[tokio::test]
    async fn test_bulk_import_bacnet_numeric_sensor_definitions_async_continues_after_error() {
        let definition_id = Uuid::new_v4();
        let create_path =
            format!("/api/setting/bacnetIpDefinitions/bacnetIpNumericSensors/{definition_id}");

        let server = MockServer::start();
        let fail_mock = server.mock(|when, then| {
            when.method(POST)
                .path(create_path.clone())
                .body_includes("Bad Sensor");
            then.status(500);
        });
        let ok_mock = server.mock(|when, then| {
            when.method(POST)
                .path(create_path)
                .body_includes("Good Sensor");
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        let csv = write_numeric_csv(&[("", "Bad Sensor"), ("", "Good Sensor")]);

        let result = bulk_import_bacnet_numeric_sensor_definitions_async(
            &config,
            &client,
            &mut token,
            &csv.path().to_string_lossy().to_string(),
            definition_id,
            false,
        )
        .await;

        fail_mock.assert();
        ok_mock.assert();

        let err = result.expect_err("expected BulkOperationFailures when one row failed");
        let app_err = err
            .downcast_ref::<crate::hyperview::app_errors::AppError>()
            .expect("expected AppError root cause");
        assert!(matches!(
            app_err,
            crate::hyperview::app_errors::AppError::BulkOperationFailures {
                failed: 1,
                total: 2,
            },
        ));
    }

    #[tokio::test]
    async fn test_bulk_import_bacnet_numeric_sensor_definitions_async_continues_after_malformed_row()
     {
        // A row that fails to DESERIALIZE (bad object_instance) must be counted as a failure and
        // NOT halt the loop; the valid row that follows it must still be imported.
        let definition_id = Uuid::new_v4();
        let create_path =
            format!("/api/setting/bacnetIpDefinitions/bacnetIpNumericSensors/{definition_id}");

        let server = MockServer::start();
        let ok_mock = server.mock(|when, then| {
            when.method(POST)
                .path(create_path)
                .body_includes("Good Sensor");
            then.status(200);
        });

        let mut csv = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            csv,
            "id,name,multiplier,object_instance,object_type,sensor_type,sensor_type_id,unit,unit_id"
        )
        .unwrap();
        // Bad object_instance -> csv deserialize error for this row.
        writeln!(
            csv,
            ",Bad Sensor,1.0,not-a-number,analogInput,Temp,type-1,C,unit-1"
        )
        .unwrap();
        writeln!(csv, ",Good Sensor,1.0,5,analogInput,Temp,type-1,C,unit-1").unwrap();
        csv.flush().unwrap();

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();

        let result = bulk_import_bacnet_numeric_sensor_definitions_async(
            &config,
            &client,
            &mut token,
            &csv.path().to_string_lossy().to_string(),
            definition_id,
            false,
        )
        .await;

        // The valid row after the malformed one was still processed.
        ok_mock.assert();

        let err = result.expect_err("expected BulkOperationFailures for the malformed row");
        let app_err = err
            .downcast_ref::<crate::hyperview::app_errors::AppError>()
            .expect("expected AppError root cause");
        assert!(matches!(
            app_err,
            crate::hyperview::app_errors::AppError::BulkOperationFailures {
                failed: 1,
                total: 2,
            },
        ));
    }

    fn write_non_numeric_csv(rows: &[(&str, &str, &str)]) -> tempfile::NamedTempFile {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            tmp,
            "id,name,object_instance,object_type,sensor_type,sensor_type_id,value_mapping"
        )
        .unwrap();
        for (id, name, value_mapping) in rows {
            writeln!(
                tmp,
                "{id},{name},2,binaryInput,Status,type-2,\"{value_mapping}\""
            )
            .unwrap();
        }
        tmp.flush().unwrap();
        tmp
    }

    #[tokio::test]
    async fn test_bulk_import_bacnet_non_numeric_sensor_definitions_async_creates_new_row() {
        let definition_id = Uuid::new_v4();
        let create_path =
            format!("/api/setting/bacnetIpDefinitions/bacnetIpNonNumericSensors/{definition_id}");

        let server = MockServer::start();
        let create_mock = server.mock(|when, then| {
            when.method(POST).path(create_path);
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        let csv = write_non_numeric_csv(&[("", "Status", "Inactive:0,Active:1")]);

        bulk_import_bacnet_non_numeric_sensor_definitions_async(
            &config,
            &client,
            &mut token,
            &csv.path().to_string_lossy().to_string(),
            definition_id,
            false,
        )
        .await
        .unwrap();

        create_mock.assert();
    }

    #[tokio::test]
    async fn test_bulk_import_bacnet_non_numeric_sensor_definitions_async_skips_malformed_value_mapping()
     {
        let definition_id = Uuid::new_v4();

        let server = MockServer::start();
        let create_should_not_fire = server.mock(|when, then| {
            when.method(POST).path(format!(
                "/api/setting/bacnetIpDefinitions/bacnetIpNonNumericSensors/{definition_id}"
            ));
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        let csv = write_non_numeric_csv(&[("", "Status", "Inactive:not-a-number")]);

        let result = bulk_import_bacnet_non_numeric_sensor_definitions_async(
            &config,
            &client,
            &mut token,
            &csv.path().to_string_lossy().to_string(),
            definition_id,
            false,
        )
        .await;

        create_should_not_fire.assert_calls(0);

        let err = result.expect_err("expected BulkOperationFailures for malformed value mapping");
        let app_err = err
            .downcast_ref::<crate::hyperview::app_errors::AppError>()
            .expect("expected AppError root cause");
        assert!(matches!(
            app_err,
            crate::hyperview::app_errors::AppError::BulkOperationFailures {
                failed: 1,
                total: 1,
            },
        ));
    }
}
