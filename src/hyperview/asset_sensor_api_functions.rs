use log::debug;
use reqwest::{Client, header::AUTHORIZATION};
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
    let mut reader = csv::Reader::from_path(filename)?;
    while let Some(Ok(record)) = reader.deserialize::<AssetSensorUpdateDto>().next() {
        debug!(
            "updating sensor_id {}",
            serde_json::to_string_pretty(&record).unwrap()
        );
    }

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
