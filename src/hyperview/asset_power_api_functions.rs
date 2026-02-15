use log::debug;
use reqwest::{Client, header::AUTHORIZATION};
use uuid::Uuid;

use super::{
    api_constants::RACK_PDU_OUTLETS_API_PREFIX, asset_power_api_data::RackPduOutletDto,
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
