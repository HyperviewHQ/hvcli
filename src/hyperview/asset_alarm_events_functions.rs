use log::{debug, error};
use reqwest::{
    Client,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use serde_json::{Map, Value, json};

use crate::retry_on_unauthorized_async;

use super::{
    api_constants::{
        ASSET_ALARM_EVENT_BULK_ACKNOWLEDGE_API_PREFIX, ASSET_ALARM_EVENT_BULK_CLOSE_API_PREFIX,
        ASSET_ALARM_EVENT_LIST_API_PREFIX, BULK_ACTION_BATCH_SIZE,
    },
    asset_alarm_events_data::{AlarmEventDto, AlarmListResponse},
    auth::AuthToken,
    cli_data::{AlarmEventFilterOptions, AppConfig, ManageActionOptions},
};

pub async fn list_alarm_events_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    skip: u32,
    limit: u32,
    alarm_filter_option: AlarmEventFilterOptions,
) -> color_eyre::Result<AlarmListResponse> {
    let target_url = format!(
        "{}{}",
        config.instance_url, ASSET_ALARM_EVENT_LIST_API_PREFIX
    );
    debug!("Request URL: {target_url}");

    let mut query_params = Map::new();

    query_params.insert("skip".to_string(), Value::Number(skip.into()));
    query_params.insert("take".to_string(), Value::Number(limit.into()));

    match alarm_filter_option {
        AlarmEventFilterOptions::Unacknowledged => {
            query_params.insert(
                "filter".to_string(),
                Value::String("[\"acknowledgementState\", \"=\", \"unacknowledged\"]".to_string()),
            );
        }

        AlarmEventFilterOptions::Active => {
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
        .header(AUTHORIZATION, auth_header)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .error_for_status()?
        .json::<AlarmListResponse>()
        .await?;

    Ok(resp)
}

async fn close_alarm_batch_async(
    req: &Client,
    auth_header: &String,
    target_url: &str,
    batch: &[String],
) -> color_eyre::Result<()> {
    req.put(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(batch)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

async fn acknowledge_alarm_batch_async(
    req: &Client,
    auth_header: &String,
    target_url: &str,
    batch: &[String],
) -> color_eyre::Result<()> {
    let payload = json!({
        "alarmEventIds": batch,
        "acknowledgementState": "acknowledged"
    });

    req.put(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn manage_asset_alarm_events_async(
    config: &AppConfig,
    req: &Client,
    auth_token: &mut AuthToken,
    filename: String,
    manage_action_options: ManageActionOptions,
) -> color_eyre::Result<()> {
    let mut reader = csv::Reader::from_path(filename)?;
    let mut work = Vec::new();

    while let Some(Ok(record)) = reader.deserialize::<AlarmEventDto>().next() {
        work.push(record.id);
    }

    let mut work_batches: Vec<Vec<String>> = Vec::new();
    work_batches.push(Vec::new());
    let mut work_queue_index = 0;

    work.into_iter().enumerate().for_each(|(e, id)| {
        if e > 0 && (e % BULK_ACTION_BATCH_SIZE) == 0 {
            work_batches.push(Vec::new());
            work_queue_index += 1;
        }
        work_batches[work_queue_index].push(id);
    });

    match manage_action_options {
        ManageActionOptions::Close => {
            let target_url = format!(
                "{}{}",
                config.instance_url, ASSET_ALARM_EVENT_BULK_CLOSE_API_PREFIX
            );
            debug!("Request URL: {target_url}");

            for batch in work_batches {
                auth_token.refresh_if_needed_async(config).await?;

                if let Err(e) = retry_on_unauthorized_async!(
                    config,
                    auth_token,
                    close_alarm_batch_async(req, &auth_token.header, &target_url, &batch).await
                ) {
                    error!("Failed to close alarm event batch {batch:?}: {e}");
                }
            }
        }

        ManageActionOptions::Acknowledge => {
            let target_url = format!(
                "{}{}",
                config.instance_url, ASSET_ALARM_EVENT_BULK_ACKNOWLEDGE_API_PREFIX
            );
            debug!("Request URL: {target_url}");

            for batch in work_batches {
                auth_token.refresh_if_needed_async(config).await?;

                if let Err(e) = retry_on_unauthorized_async!(
                    config,
                    auth_token,
                    acknowledge_alarm_batch_async(req, &auth_token.header, &target_url, &batch)
                        .await
                ) {
                    error!("Failed to acknowledge alarm event batch {batch:?}: {e}");
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;
    use std::io::Write;
    use std::time::Duration;

    fn auth_token() -> AuthToken {
        AuthToken::for_test("Bearer test_token", Duration::from_hours(1))
    }

    fn write_alarm_csv(ids: &[&str]) -> tempfile::NamedTempFile {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        // AlarmEventDto deserialization keys (camelCase via serde rename_all).
        writeln!(
            tmp,
            "id,severity,assetName,assetLocationPath,alarmEventSettingId,assetId,startTimestamp,endTimestamp,acknowledgementState,acknowledgedBy,acknowledgedTimestamp,closedBy,alarmEventCategory,isActive,propertyValues,textTemplate"
        )
        .unwrap();
        for id in ids {
            writeln!(
                tmp,
                "{id},critical,assetname,path,setting,assetid,2026-01-01,,unacknowledged,,,,cat,true,{{}},template"
            )
            .unwrap();
        }
        tmp.flush().unwrap();
        tmp
    }

    #[tokio::test]
    async fn test_list_alarm_events_sets_unacknowledged_filter_query() {
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET)
                .path(ASSET_ALARM_EVENT_LIST_API_PREFIX)
                .query_param("skip", "0")
                .query_param("take", "10")
                .query_param(
                    "filter",
                    "[\"acknowledgementState\", \"=\", \"unacknowledged\"]",
                );
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "data": [],
                    "groupCount": 0,
                    "totalCount": 0
                }));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth = "Bearer t".to_string();

        let result = list_alarm_events_async(
            &config,
            &client,
            &auth,
            0,
            10,
            AlarmEventFilterOptions::Unacknowledged,
        )
        .await
        .unwrap();

        m.assert();
        assert!(result.data.is_empty());
    }

    #[tokio::test]
    async fn test_list_alarm_events_sets_active_filter_query() {
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET)
                .path(ASSET_ALARM_EVENT_LIST_API_PREFIX)
                .query_param("filter", "[\"isActive\", \"=\", true]");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({ "data": [], "groupCount": 0, "totalCount": 0 }));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        list_alarm_events_async(
            &config,
            &client,
            &"Bearer t".to_string(),
            0,
            10,
            AlarmEventFilterOptions::Active,
        )
        .await
        .unwrap();

        m.assert();
    }

    #[tokio::test]
    async fn test_manage_close_sends_single_batch_when_under_batch_size() {
        let server = MockServer::start();
        let close_mock = server.mock(|when, then| {
            when.method(PUT)
                .path(ASSET_ALARM_EVENT_BULK_CLOSE_API_PREFIX)
                .body_includes("alarm-1")
                .body_includes("alarm-2");
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        let csv = write_alarm_csv(&["alarm-1", "alarm-2"]);

        manage_asset_alarm_events_async(
            &config,
            &client,
            &mut token,
            csv.path().to_string_lossy().to_string(),
            ManageActionOptions::Close,
        )
        .await
        .unwrap();

        close_mock.assert_calls(1);
    }

    #[tokio::test]
    async fn test_manage_acknowledge_uses_alarm_event_ids_payload() {
        let server = MockServer::start();
        let ack_mock = server.mock(|when, then| {
            when.method(PUT)
                .path(ASSET_ALARM_EVENT_BULK_ACKNOWLEDGE_API_PREFIX)
                .body_includes("alarmEventIds")
                .body_includes("acknowledged")
                .body_includes("alarm-x");
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        let csv = write_alarm_csv(&["alarm-x"]);

        manage_asset_alarm_events_async(
            &config,
            &client,
            &mut token,
            csv.path().to_string_lossy().to_string(),
            ManageActionOptions::Acknowledge,
        )
        .await
        .unwrap();

        ack_mock.assert();
    }

    #[tokio::test]
    async fn test_manage_close_splits_into_multiple_batches_at_batch_size_boundary() {
        // BULK_ACTION_BATCH_SIZE rows in one batch + 1 leftover triggers a second batch.
        let mut ids: Vec<String> = (0..=BULK_ACTION_BATCH_SIZE)
            .map(|i| format!("alarm-{i}"))
            .collect();
        let id_refs: Vec<&str> = ids.iter_mut().map(|s| s.as_str()).collect();

        let server = MockServer::start();
        let close_mock = server.mock(|when, then| {
            when.method(PUT)
                .path(ASSET_ALARM_EVENT_BULK_CLOSE_API_PREFIX);
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        let csv = write_alarm_csv(&id_refs);

        manage_asset_alarm_events_async(
            &config,
            &client,
            &mut token,
            csv.path().to_string_lossy().to_string(),
            ManageActionOptions::Close,
        )
        .await
        .unwrap();

        close_mock.assert_calls(2);
    }

    #[tokio::test]
    async fn test_manage_close_continues_after_batch_error() {
        // Two batches: first 500s, second 200. The whole call must still complete Ok(()).
        let mut ids: Vec<String> = (0..=BULK_ACTION_BATCH_SIZE)
            .map(|i| format!("alarm-{i}"))
            .collect();
        let id_refs: Vec<&str> = ids.iter_mut().map(|s| s.as_str()).collect();

        let server = MockServer::start();
        // First batch includes alarm-0, second batch only contains alarm-100.
        let fail_mock = server.mock(|when, then| {
            when.method(PUT)
                .path(ASSET_ALARM_EVENT_BULK_CLOSE_API_PREFIX)
                .body_includes("alarm-0");
            then.status(500);
        });
        let ok_mock = server.mock(|when, then| {
            when.method(PUT)
                .path(ASSET_ALARM_EVENT_BULK_CLOSE_API_PREFIX)
                .body_includes(format!("alarm-{BULK_ACTION_BATCH_SIZE}"));
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        let csv = write_alarm_csv(&id_refs);

        manage_asset_alarm_events_async(
            &config,
            &client,
            &mut token,
            csv.path().to_string_lossy().to_string(),
            ManageActionOptions::Close,
        )
        .await
        .expect("manage alarm events must not abort on a per-batch 500");

        fail_mock.assert();
        ok_mock.assert();
    }
}
