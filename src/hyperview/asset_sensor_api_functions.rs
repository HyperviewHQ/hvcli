use log::{debug, trace};
use reqwest::{Client, header::AUTHORIZATION};
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;

use super::{
    api_constants::SENSOR_API_PREFIX,
    asset_sensor_api_data::{AssetSensorDto, AssetSensorUpdateDto},
    cli_data::AppConfig,
};

pub async fn bulk_update_asset_sensor_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    filename: &String,
) -> color_eyre::Result<()> {
    let mut asset_sensors_map: HashMap<String, HashMap<String, AssetSensorDto>> = HashMap::new();
    let mut reader = csv::Reader::from_path(filename)?;

    while let Some(Ok(mut record)) = reader.deserialize::<AssetSensorUpdateDto>().next() {
        debug!("updating sensor_id {}", record.sensor_id);

        // if the asset is not mapped, lookup and cache sensor mapping
        if !asset_sensors_map.contains_key(&record.asset_id.to_string())
            && let Ok(sensors) =
                get_asset_sensor_list_async(config, req, auth_header, record.asset_id).await
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
                record.access_policy_id = Some(Uuid::from_str(&sensor.access_policy_id)?);
            } else if let Some(record_access_policy) = record.access_policy_id
                && record_access_policy.is_nil()
            {
                debug!("Nil UUID detected. Resetting to parent access policy");
                record.access_policy_id = None;
            }
        }

        trace!("Sensor record: {}", serde_json::to_string(&record)?);

        update_asset_sensor_async(config, req, auth_header, record).await?;
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
    sensor: AssetSensorUpdateDto,
) -> color_eyre::Result<()> {
    let target_url = format!("{}{}", config.instance_url, SENSOR_API_PREFIX);
    debug!("Request URL: {target_url}");

    let _resp = req
        .put(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(&sensor)
        .send()
        .await?;

    Ok(())
}

pub async fn get_asset_sensor_list_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    id: Uuid,
) -> color_eyre::Result<Vec<AssetSensorDto>> {
    let target_url = format!("{}{}/{}", config.instance_url, SENSOR_API_PREFIX, id);
    debug!("Request URL: {target_url:?}");

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .json::<Vec<AssetSensorDto>>()
        .await?;

    Ok(resp)
}
