use log::debug;
use reqwest::{
    Client,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use serde_json::Map;
use uuid::Uuid;

use crate::hyperview::api_constants::PDU_RPP_BREAKERS_API_PREFIX;

use super::{
    api_constants::POWER_ASSOCIATION_API_PREFIX,
    asset_power_api_data::{
        BulkPowerAssociationCreateDto, PowerAssociationCreateDto, PowerProviderComponentDto,
    },
    cli_data::AppConfig,
};

pub async fn get_power_provider_components_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    api_path: &str,
    id: Uuid,
) -> color_eyre::Result<Vec<PowerProviderComponentDto>> {
    let mut query_params = Map::new();

    let target_url = if api_path == PDU_RPP_BREAKERS_API_PREFIX {
        query_params.insert(
            "assetId".to_string(),
            serde_json::Value::String(id.to_string()),
        );
        format!("{}{}", config.instance_url, api_path)
    } else {
        format!("{}{}/{}", config.instance_url, api_path, id)
    };

    debug!("Request URL: {target_url}");

    let resp = req
        .get(target_url)
        .query(&query_params)
        .header(AUTHORIZATION, auth_header)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .json::<Vec<PowerProviderComponentDto>>()
        .await?;

    Ok(resp)
}

pub async fn bulk_add_power_association_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    filename: &String,
) -> color_eyre::Result<()> {
    let mut reader = csv::Reader::from_path(filename)?;

    while let Some(Ok(record)) = reader.deserialize::<BulkPowerAssociationCreateDto>().next() {
        debug!("updating asset id {}", record.asset_id);
    }

    Ok(())
}

pub async fn add_power_association_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    power_consuming_asset_id: Uuid,
    power_providing_asset_id: Uuid,
) -> color_eyre::Result<()> {
    let target_url = format!("{}{}", config.instance_url, POWER_ASSOCIATION_API_PREFIX);
    debug!("Request URL: {target_url}");

    let association_data = PowerAssociationCreateDto {
        consuming_destination_asset_id: power_consuming_asset_id,
        providing_source_asset_id: power_providing_asset_id,
    };

    let _resp = req
        .post(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(&association_data)
        .send()
        .await?;

    Ok(())
}
