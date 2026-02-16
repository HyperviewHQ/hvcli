use log::debug;
use reqwest::{Client, header::AUTHORIZATION};
use uuid::Uuid;

use super::{
    api_constants::{POWER_ASSOCIATION_API_PREFIX, RACK_PDU_OUTLETS_API_PREFIX},
    asset_power_api_data::{PowerAssociationCreateDto, RackPduOutletDto},
    cli_data::AppConfig,
};

pub async fn get_rack_pdu_outlets_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    id: Uuid,
) -> color_eyre::Result<Vec<RackPduOutletDto>> {
    let target_url = format!(
        "{}{}/{}",
        config.instance_url, RACK_PDU_OUTLETS_API_PREFIX, id
    );
    debug!("Request URL: {target_url}");

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .json::<Vec<RackPduOutletDto>>()
        .await?;

    Ok(resp)
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
