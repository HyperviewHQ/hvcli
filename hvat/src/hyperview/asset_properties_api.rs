use color_eyre::eyre::Result;
use log::debug;
use reqwest::{header::AUTHORIZATION, Client};

use crate::hyperview::{
    api_constants::ASSET_PROPERTIES_API_PREFIX, asset_properties_api_data::AssetPropertyDto,
};

use super::cli_data::AppConfig;

pub async fn get_asset_property_list_async(
    config: &AppConfig,
    req: Client,
    auth_header: String,
    id: String,
) -> Result<Vec<AssetPropertyDto>> {
    // format the target URL
    let target_url = format!(
        "{}{}/{}",
        config.instance_url, ASSET_PROPERTIES_API_PREFIX, id
    );
    debug!("Request URL: {:?}", target_url);

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .json::<Vec<AssetPropertyDto>>()
        .await?;

    Ok(resp)
}
