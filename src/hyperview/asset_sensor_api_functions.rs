use log::{debug, error, trace};
use reqwest::{Client, header::AUTHORIZATION};
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;

use crate::retry_on_unauthorized_async;

use super::{
    api_constants::SENSOR_API_PREFIX,
    asset_sensor_api_data::{AssetSensorDto, AssetSensorUpdateDto},
    auth::AuthToken,
    cli_data::AppConfig,
};

pub async fn bulk_update_asset_sensor_async(
    config: &AppConfig,
    req: &Client,
    auth_token: &mut AuthToken,
    filename: &String,
) -> color_eyre::Result<()> {
    let mut asset_sensors_map: HashMap<String, HashMap<String, AssetSensorDto>> = HashMap::new();
    let mut reader = csv::Reader::from_path(filename)?;

    while let Some(Ok(mut record)) = reader.deserialize::<AssetSensorUpdateDto>().next() {
        auth_token.refresh_if_needed_async(config).await?;

        debug!("updating sensor_id {}", record.sensor_id);

        // if the asset is not mapped, lookup and cache sensor mapping
        if !asset_sensors_map.contains_key(&record.asset_id.to_string())
            && let Ok(sensors) = retry_on_unauthorized_async!(
                config,
                auth_token,
                get_asset_sensor_list_async(config, req, &auth_token.header, record.asset_id).await
            )
        {
            map_asset_sensors(record.asset_id.to_string(), sensors, &mut asset_sensors_map);
        }

        // If the sensor exists update name and access policy
        if let Some(sensor) = get_sensor_record(
            &record.asset_id.to_string(),
            &record.sensor_id.to_string(),
            &asset_sensors_map,
        ) {
            // If the access policy is None and is not inherited, leave as is, do not reset to parent
            if record.access_policy_id.is_none() && !sensor.access_policy_is_inherited {
                debug!(
                    "Update record does not set access policy. Keeping original: {}",
                    &sensor.access_policy_id
                );
                match Uuid::from_str(&sensor.access_policy_id) {
                    Ok(uuid) => record.access_policy_id = Some(uuid),
                    Err(e) => {
                        // Skip this row instead of aborting the whole bulk run.
                        error!(
                            "Failed to parse access policy id {:?} for sensor {}: {e}",
                            sensor.access_policy_id, record.sensor_id
                        );
                        continue;
                    }
                }
            } else if let Some(record_access_policy) = record.access_policy_id
                && record_access_policy.is_nil()
            {
                debug!("Nil UUID detected. Resetting to parent access policy");
                record.access_policy_id = None;
            }
        }

        trace!("Sensor record: {}", serde_json::to_string(&record)?);

        if let Err(e) = retry_on_unauthorized_async!(
            config,
            auth_token,
            update_asset_sensor_async(config, req, &auth_token.header, &record).await
        ) {
            error!("Failed to update sensor id {}: {e}", record.sensor_id);
        }
    }

    Ok(())
}

fn map_asset_sensors(
    asset_id: String,
    sensors: Vec<AssetSensorDto>,
    asset_sensors_map: &mut HashMap<String, HashMap<String, AssetSensorDto>>,
) {
    let mut sensor_map = HashMap::new();

    for sensor in sensors {
        sensor_map.insert(sensor.id.clone(), sensor);
    }

    asset_sensors_map.insert(asset_id, sensor_map);
}

fn get_sensor_record(
    asset_id: &String,
    sensor_id: &String,
    asset_sensors_map: &HashMap<String, HashMap<String, AssetSensorDto>>,
) -> Option<AssetSensorDto> {
    if let Some(asset_sensors) = asset_sensors_map.get(asset_id)
        && let Some(sensor) = asset_sensors.get(sensor_id)
    {
        return Some(sensor.clone());
    }

    None
}

async fn update_asset_sensor_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    sensor: &AssetSensorUpdateDto,
) -> color_eyre::Result<()> {
    let target_url = format!("{}{}", config.instance_url, SENSOR_API_PREFIX);
    debug!("Request URL: {target_url}");

    req.put(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(sensor)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn get_asset_sensor_list_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    id: Uuid,
) -> color_eyre::Result<Vec<AssetSensorDto>> {
    let target_url = format!("{}{}/{}", config.instance_url, SENSOR_API_PREFIX, id);
    debug!("Request URL: {target_url}");

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<AssetSensorDto>>()
        .await?;

    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;
    use std::io::Write;

    fn make_sensor(id: &str, asset_id: &str, access_policy_id: &str) -> AssetSensorDto {
        AssetSensorDto {
            id: id.to_string(),
            asset_id: asset_id.to_string(),
            access_policy_id: access_policy_id.to_string(),
            ..AssetSensorDto::default()
        }
    }

    #[test]
    fn test_map_asset_sensors_indexes_by_sensor_id() {
        let asset_id = "asset-1".to_string();
        let sensors = vec![
            make_sensor("sensor-a", &asset_id, "policy-1"),
            make_sensor("sensor-b", &asset_id, "policy-2"),
        ];

        let mut map = HashMap::new();
        map_asset_sensors(asset_id.clone(), sensors, &mut map);

        let sensor_map = map.get(&asset_id).expect("asset entry present");
        assert_eq!(sensor_map.len(), 2);
        assert_eq!(sensor_map.get("sensor-a").unwrap().access_policy_id, "policy-1");
        assert_eq!(sensor_map.get("sensor-b").unwrap().access_policy_id, "policy-2");
    }

    #[test]
    fn test_get_sensor_record_returns_clone_when_found() {
        let asset_id = "asset-1".to_string();
        let mut map = HashMap::new();
        map_asset_sensors(
            asset_id.clone(),
            vec![make_sensor("sensor-a", &asset_id, "policy-1")],
            &mut map,
        );

        let result = get_sensor_record(&asset_id, &"sensor-a".to_string(), &map);
        assert!(result.is_some());
        assert_eq!(result.unwrap().access_policy_id, "policy-1");
    }

    #[test]
    fn test_get_sensor_record_returns_none_when_asset_missing() {
        let map = HashMap::new();
        let result =
            get_sensor_record(&"missing".to_string(), &"sensor-a".to_string(), &map);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_sensor_record_returns_none_when_sensor_missing() {
        let asset_id = "asset-1".to_string();
        let mut map = HashMap::new();
        map_asset_sensors(
            asset_id.clone(),
            vec![make_sensor("sensor-a", &asset_id, "policy-1")],
            &mut map,
        );

        let result = get_sensor_record(&asset_id, &"sensor-other".to_string(), &map);
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_asset_sensor_list_async_returns_dtos() {
        let asset_id = Uuid::new_v4();
        let url_path = format!("{SENSOR_API_PREFIX}/{asset_id}");

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path(url_path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([{
                    "id": "sensor-a",
                    "name": "Temp",
                    "sensorTypeId": "type-1",
                    "listIndex": null,
                    "sensorTypeDescription": "Temperature",
                    "value": 21.5,
                    "rawValue": 21.5,
                    "unitString": "C",
                    "dataSource": null,
                    "dataCollectorId": null,
                    "dataCollectorName": null,
                    "lastValueUpdate": null,
                    "sourceDeviceAssetId": asset_id.to_string(),
                    "sensorAssociationType": "direct",
                    "isNumeric": true,
                    "accessPolicyId": "00000000-0000-0000-0000-000000000001",
                    "accessPolicyName": "policy",
                    "assetAccessPolicyId": "00000000-0000-0000-0000-000000000002",
                    "accessPolicyIsInherited": false
                }]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let result = get_asset_sensor_list_async(&config, &client, &auth_header, asset_id)
            .await
            .unwrap();

        m.assert();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "sensor-a");
        assert!(!result[0].access_policy_is_inherited);
    }

    fn write_csv(rows: &[(Uuid, Uuid, &str, Option<Uuid>)]) -> tempfile::NamedTempFile {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "asset_id,sensor_id,sensor_name,access_policy_id").unwrap();
        for (asset_id, sensor_id, name, policy) in rows {
            let policy_str = policy.map(|p| p.to_string()).unwrap_or_default();
            writeln!(tmp, "{asset_id},{sensor_id},{name},{policy_str}").unwrap();
        }
        tmp.flush().unwrap();
        tmp
    }

    fn auth_token() -> AuthToken {
        AuthToken::for_test("Bearer test_token", std::time::Duration::from_hours(1))
    }

    #[tokio::test]
    async fn test_bulk_update_asset_sensor_async_succeeds_for_valid_row() {
        let asset_id = Uuid::new_v4();
        let sensor_id = Uuid::new_v4();
        let new_policy = Uuid::new_v4();
        let list_path = format!("{SENSOR_API_PREFIX}/{asset_id}");

        let server = MockServer::start();
        let list_mock = server.mock(|when, then| {
            when.method(GET).path(list_path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([{
                    "id": sensor_id.to_string(),
                    "name": "Old name",
                    "sensorTypeId": "type-1",
                    "listIndex": null,
                    "sensorTypeDescription": "Temperature",
                    "value": 21.5,
                    "rawValue": 21.5,
                    "unitString": "C",
                    "dataSource": null,
                    "dataCollectorId": null,
                    "dataCollectorName": null,
                    "lastValueUpdate": null,
                    "sourceDeviceAssetId": asset_id.to_string(),
                    "sensorAssociationType": "direct",
                    "isNumeric": true,
                    "accessPolicyId": "00000000-0000-0000-0000-000000000001",
                    "accessPolicyName": "policy",
                    "assetAccessPolicyId": "00000000-0000-0000-0000-000000000002",
                    "accessPolicyIsInherited": false
                }]));
        });
        let update_mock = server.mock(|when, then| {
            when.method(PUT).path(SENSOR_API_PREFIX);
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        let csv = write_csv(&[(asset_id, sensor_id, "New name", Some(new_policy))]);

        bulk_update_asset_sensor_async(
            &config,
            &client,
            &mut token,
            &csv.path().to_string_lossy().to_string(),
        )
        .await
        .unwrap();

        list_mock.assert();
        update_mock.assert();
    }

    #[tokio::test]
    async fn test_bulk_update_asset_sensor_async_continues_after_update_error() {
        let asset_id = Uuid::new_v4();
        let sensor_id_ok = Uuid::new_v4();
        let sensor_id_fail = Uuid::new_v4();
        let list_path = format!("{SENSOR_API_PREFIX}/{asset_id}");

        let server = MockServer::start();
        // List is hit only once because the asset id is cached after the first row.
        let list_mock = server.mock(|when, then| {
            when.method(GET).path(list_path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([
                    {
                        "id": sensor_id_ok.to_string(),
                        "name": "ok",
                        "sensorTypeId": "t",
                        "listIndex": null,
                        "sensorTypeDescription": "",
                        "value": 0,
                        "rawValue": 0,
                        "unitString": null,
                        "dataSource": null,
                        "dataCollectorId": null,
                        "dataCollectorName": null,
                        "lastValueUpdate": null,
                        "sourceDeviceAssetId": asset_id.to_string(),
                        "sensorAssociationType": "direct",
                        "isNumeric": true,
                        "accessPolicyId": "00000000-0000-0000-0000-000000000001",
                        "accessPolicyName": "policy",
                        "assetAccessPolicyId": "00000000-0000-0000-0000-000000000002",
                        "accessPolicyIsInherited": false
                    },
                    {
                        "id": sensor_id_fail.to_string(),
                        "name": "fail",
                        "sensorTypeId": "t",
                        "listIndex": null,
                        "sensorTypeDescription": "",
                        "value": 0,
                        "rawValue": 0,
                        "unitString": null,
                        "dataSource": null,
                        "dataCollectorId": null,
                        "dataCollectorName": null,
                        "lastValueUpdate": null,
                        "sourceDeviceAssetId": asset_id.to_string(),
                        "sensorAssociationType": "direct",
                        "isNumeric": true,
                        "accessPolicyId": "00000000-0000-0000-0000-000000000001",
                        "accessPolicyName": "policy",
                        "assetAccessPolicyId": "00000000-0000-0000-0000-000000000002",
                        "accessPolicyIsInherited": false
                    }
                ]));
        });
        // First PUT (for the failing row) returns 500; bulk loop must log-and-continue.
        // Second PUT (for the ok row) returns 200.
        let put_fail_mock = server.mock(|when, then| {
            when.method(PUT)
                .path(SENSOR_API_PREFIX)
                .body_includes(sensor_id_fail.to_string());
            then.status(500);
        });
        let put_ok_mock = server.mock(|when, then| {
            when.method(PUT)
                .path(SENSOR_API_PREFIX)
                .body_includes(sensor_id_ok.to_string());
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        let csv = write_csv(&[
            (asset_id, sensor_id_fail, "fail-new", Some(Uuid::new_v4())),
            (asset_id, sensor_id_ok, "ok-new", Some(Uuid::new_v4())),
        ]);

        bulk_update_asset_sensor_async(
            &config,
            &client,
            &mut token,
            &csv.path().to_string_lossy().to_string(),
        )
        .await
        .expect("bulk update should not abort on a per-row error");

        list_mock.assert();
        put_fail_mock.assert();
        put_ok_mock.assert();
    }

    #[tokio::test]
    async fn test_bulk_update_asset_sensor_async_skips_row_with_malformed_cached_policy_id() {
        let asset_id = Uuid::new_v4();
        let sensor_id = Uuid::new_v4();
        let list_path = format!("{SENSOR_API_PREFIX}/{asset_id}");

        let server = MockServer::start();
        let list_mock = server.mock(|when, then| {
            when.method(GET).path(list_path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([{
                    "id": sensor_id.to_string(),
                    "name": "ok",
                    "sensorTypeId": "t",
                    "listIndex": null,
                    "sensorTypeDescription": "",
                    "value": 0,
                    "rawValue": 0,
                    "unitString": null,
                    "dataSource": null,
                    "dataCollectorId": null,
                    "dataCollectorName": null,
                    "lastValueUpdate": null,
                    "sourceDeviceAssetId": asset_id.to_string(),
                    "sensorAssociationType": "direct",
                    "isNumeric": true,
                    "accessPolicyId": "not-a-uuid",
                    "accessPolicyName": "policy",
                    "assetAccessPolicyId": "00000000-0000-0000-0000-000000000002",
                    "accessPolicyIsInherited": false
                }]));
        });
        // A PUT happening here would mean the bad UUID didn't cause us to skip the row.
        let put_should_not_fire = server.mock(|when, then| {
            when.method(PUT).path(SENSOR_API_PREFIX);
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        // Empty access_policy_id triggers the "keep original" branch that parses the cached id.
        let csv = write_csv(&[(asset_id, sensor_id, "New name", None)]);

        // Must not abort the whole bulk run on a bad cached UUID.
        bulk_update_asset_sensor_async(
            &config,
            &client,
            &mut token,
            &csv.path().to_string_lossy().to_string(),
        )
        .await
        .expect("bulk update must not abort on a malformed cached UUID");

        list_mock.assert();
        put_should_not_fire.assert_calls(0);
    }
}
