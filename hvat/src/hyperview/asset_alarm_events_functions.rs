use color_eyre::Result;
use log::debug;
use reqwest::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde_json::{Map, Value};

use crate::hyperview::{
    api_constants::{
        ASSET_ALARM_EVENT_BULK_CLOSE_API_PREFIX, ASSET_ALARM_EVENT_LIST_API_PREFIX,
        BULK_ACTION_BATCH_SIZE,
    },
    asset_alarm_events_data::AlarmListResponse,
    cli_data::{AlarmEventFilterOption, AppConfig},
};

use super::asset_alarm_events_data::AlarmEventDto;

pub async fn list_alarm_events_async(
    config: &AppConfig,
    req: Client,
    auth_header: String,
    skip: u32,
    limit: u32,
    alarm_filter_option: AlarmEventFilterOption,
) -> Result<AlarmListResponse> {
    let target_url = format!(
        "{}{}",
        config.instance_url, ASSET_ALARM_EVENT_LIST_API_PREFIX
    );
    debug!("Request URL: {}", target_url);

    let mut query_params = Map::new();

    query_params.insert("skip".to_string(), Value::Number(skip.into()));
    query_params.insert("take".to_string(), Value::Number(limit.into()));

    match alarm_filter_option {
        AlarmEventFilterOption::Unacknowledged => {
            query_params.insert(
                "filter".to_string(),
                Value::String("[\"acknowledgementState\", \"=\", \"unacknowledged\"]".to_string()),
            );
        }

        AlarmEventFilterOption::Active => {
            query_params.insert(
                "filter".to_string(),
                Value::String("[\"isActive\", \"=\", true]".to_string()),
            );
        }
    }

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

pub async fn manage_asset_alarm_events_async(
    config: &AppConfig,
    req: Client,
    auth_header: String,
    filename: String,
) -> Result<()> {
    let target_url = format!(
        "{}{}",
        config.instance_url, ASSET_ALARM_EVENT_BULK_CLOSE_API_PREFIX
    );
    debug!("Request URL: {}", target_url);

    let mut reader = csv::Reader::from_path(filename)?;
    let mut work = Vec::new();

    while let Some(Ok(record)) = reader.deserialize::<AlarmEventDto>().next() {
        work.push(record.id);
    }

    let mut work_batches: Vec<Vec<String>> = Vec::new();
    work_batches.push(Vec::new());
    let mut work_queue_index = 0;

    work.into_iter().enumerate().for_each(|(e, id)| {
        if e > 0 {
            if (e % BULK_ACTION_BATCH_SIZE) == 0 {
                work_batches.push(Vec::new());
                work_queue_index += 1;
            }
        }
        work_batches[work_queue_index].push(id);
    });

    for batch in work_batches {
        let _resp = &req
            .put(&target_url)
            .header(AUTHORIZATION, &auth_header)
            .json(&batch)
            .send()
            .await?;
    }

    Ok(())
}
