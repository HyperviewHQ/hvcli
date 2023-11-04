use color_eyre::eyre::Result;
use log::debug;
use reqwest::{header::AUTHORIZATION, Client};

use crate::hyperview::api_constants::ASSET_CUSTOM_PROPERTIES_API_PREFIX;

use super::{asset_custom_properties_api_data::AssetCustomPropertyDto, cli_data::AppConfig};

pub async fn get_asset_custom_property_list_async(
    config: &AppConfig,
    req: Client,
    auth_header: String,
    id: String,
) -> Result<Vec<AssetCustomPropertyDto>> {
    // format the target URL
    let target_url = format!(
        "{}{}/{}",
        config.instance_url, ASSET_CUSTOM_PROPERTIES_API_PREFIX, id
    );
    debug!("Request URL: {:?}", target_url);

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .json::<Vec<AssetCustomPropertyDto>>()
        .await?;

    Ok(resp)
}
