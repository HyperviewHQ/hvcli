use anyhow::Result;
use log::{debug, info};
use reqwest::{header::AUTHORIZATION, Client};
use serde_json::Value;

use super::{api_constants::ASSET_API_PREFIX, cli::AppConfig};

pub async fn get_asset_list_async(
    config: &AppConfig,
    req: Client,
    auth_header: String,
    query: Vec<(String, String)>,
) -> Result<()> {
    // format the target URL
    let target_url = format!("{}{}", config.instance_url, ASSET_API_PREFIX);
    debug!("Request URL: {:?}", target_url);

    let resp = req
        .get(target_url)
        .query(&query)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .json::<Value>()
        .await?;

    let mut total = 0;
    let mut limit = 0;

    if let Some(metadata) = &resp.get("_metadata") {
        total = metadata["total"].as_u64().unwrap();
        limit = metadata["limit"].as_u64().unwrap();
        info!("Meta Data: | Total: {} | Limit: {} |", total, limit);
    }

    let end = if limit < total {
        limit as usize
    } else {
        total as usize
    };

    debug!("End: {}", end);

    if let Some(assets) = &resp.get("data") {
        for i in 0..end {
            let id = assets[i]["id"].as_str().unwrap().to_string();
            let name = assets[i]["name"].as_str().unwrap().to_string();
            debug!("id: {}, name: {}", id, name);
        }
    };

    Ok(())
}
