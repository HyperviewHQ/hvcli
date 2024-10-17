use color_eyre::Result;
use log::debug;
use reqwest::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde_json::{Map, Value};

use crate::hyperview::{api_constants::ASSET_ALARM_EVENT_LIST_API_PREFIX, cli_data::AppConfig};

use super::asset_alarm_events_data::AlarmListResponse;

pub async fn list_alarm_events_async(
    config: &AppConfig,
    req: Client,
    auth_header: String,
    skip: u32,
    limit: u32,
) -> Result<AlarmListResponse> {
    let target_url = format!(
        "{}{}",
        config.instance_url, ASSET_ALARM_EVENT_LIST_API_PREFIX
    );
    debug!("Request URL: {}", target_url);

    let mut query_params = Map::new();

    query_params.insert("skip".to_string(), Value::Number(skip.into()));
    query_params.insert("take".to_string(), Value::Number(limit.into()));
    query_params.insert(
        "filter".to_string(),
        Value::String("[\"acknowledgementState\", \"=\", \"unacknowledged\"]".to_string()),
    );

    debug!(
        "Query parameters: {}",
        serde_json::to_string(&query_params).unwrap()
    );

    let resp = req
        .get(target_url)
        .query(&query_params)
        .header(AUTHORIZATION, auth_header.clone())
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .json::<AlarmListResponse>()
        .await?;

    Ok(resp)
}
