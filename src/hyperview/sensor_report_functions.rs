use chrono::NaiveDate;
use log::{debug, error};
use reqwest::Client;
use std::collections::HashMap;
use uuid::Uuid;

use crate::retry_on_unauthorized_async;

use super::{
    api_constants::BULK_ACTION_BATCH_SIZE,
    app_errors::AppError,
    asset_api_functions::search_assets_async,
    asset_sensor_api_functions::{
        get_asset_sensor_list_async, get_numeric_sensor_daily_summaries_async,
    },
    auth::AuthToken,
    cli_data::{AppConfig, GenerateSensorReportArgs, OutputOptions, SearchAssetsArgs},
    common_types::MultiTypeValue,
    custom_asset_properties_api_functions::get_custom_asset_property_list_async,
    sensor_report_data::SensorReportRow,
};

fn multi_type_value_to_plain_string(v: &MultiTypeValue) -> String {
    match v {
        MultiTypeValue::StringValue(s) => s.clone(),
        MultiTypeValue::FloatValue(n) => n.to_string(),
        MultiTypeValue::IntegerValue(n) => n.to_string(),
        MultiTypeValue::NullValue => String::new(),
    }
}

struct AssetContext {
    asset_name: String,
    asset_id: Uuid,
    custom_property: String,
    sensor_id: Uuid,
    sensor_name: String,
    sensor_unit: String,
}

#[allow(clippy::too_many_lines)]
pub async fn generate_sensor_report_async(
    config: &AppConfig,
    req: &Client,
    auth_token: &mut AuthToken,
    options: GenerateSensorReportArgs,
) -> color_eyre::Result<Vec<SensorReportRow>> {
    let (start, end) = resolve_date_range(&options)?;
    debug!("Report date range: {start} .. {end}");

    let search_args = SearchAssetsArgs {
        search_pattern: None,
        asset_type: Some(options.asset_type.clone()),
        location_path: options.location_path.clone(),
        properties: None,
        custom_properties: None,
        id: None,
        manufacturer: options.manufacturer.clone(),
        product: options.product.clone(),
        show_property: None,
        skip: options.skip,
        limit: options.limit,
        output_type: OutputOptions::Record,
        filename: None,
    };

    auth_token.refresh_if_needed_async(config).await?;
    let assets = retry_on_unauthorized_async!(
        config,
        auth_token,
        search_assets_async(config, req, &auth_token.header, search_args.clone()).await
    )?;

    let mut contexts: Vec<AssetContext> = Vec::new();

    for asset in assets {
        auth_token.refresh_if_needed_async(config).await?;

        let sensors = match retry_on_unauthorized_async!(
            config,
            auth_token,
            get_asset_sensor_list_async(config, req, &auth_token.header, asset.id).await
        ) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to fetch sensors for asset {}: {e}", asset.id);
                continue;
            }
        };

        let Some(sensor) = sensors.into_iter().find(|s| s.name == options.sensor) else {
            error!(
                "Sensor {:?} not found on asset {}. Skipping.",
                options.sensor, asset.id
            );
            continue;
        };

        if !sensor.is_numeric {
            error!(
                "Sensor {:?} on asset {} is not numeric. Skipping.",
                options.sensor, asset.id
            );
            continue;
        }

        let sensor_id = match Uuid::parse_str(&sensor.id) {
            Ok(u) => u,
            Err(e) => {
                error!(
                    "Failed to parse sensor id {:?} on asset {}: {e}. Skipping.",
                    sensor.id, asset.id
                );
                continue;
            }
        };

        let custom_property = if let Some(name) = options.custom_property.as_ref() {
            match retry_on_unauthorized_async!(
                config,
                auth_token,
                get_custom_asset_property_list_async(config, req, &auth_token.header, asset.id)
                    .await
            ) {
                Ok(props) => props
                    .into_iter()
                    .find(|p| p.name.eq_ignore_ascii_case(name))
                    .map(|p| multi_type_value_to_plain_string(&p.value))
                    .unwrap_or_default(),
                Err(e) => {
                    error!(
                        "Failed to fetch custom properties for asset {}: {e}. Continuing without enrichment.",
                        asset.id
                    );
                    String::new()
                }
            }
        } else {
            String::new()
        };

        contexts.push(AssetContext {
            asset_name: asset.name.clone(),
            asset_id: asset.id,
            custom_property,
            sensor_id,
            sensor_name: sensor.name,
            sensor_unit: sensor.unit_string,
        });
    }

    let mut data_points_by_sensor: HashMap<
        String,
        Vec<super::asset_sensor_api_data::NumericSensorDailySummaryDataPoint>,
    > = HashMap::new();

    for chunk in contexts
        .iter()
        .map(|c| c.sensor_id)
        .collect::<Vec<Uuid>>()
        .chunks(BULK_ACTION_BATCH_SIZE)
    {
        auth_token.refresh_if_needed_async(config).await?;

        match retry_on_unauthorized_async!(
            config,
            auth_token,
            get_numeric_sensor_daily_summaries_async(
                config,
                req,
                &auth_token.header,
                chunk,
                start,
                end,
            )
            .await
        ) {
            Ok(summaries) => {
                for summary in summaries {
                    data_points_by_sensor
                        .entry(summary.sensor_id)
                        .or_default()
                        .extend(summary.sensor_data_points);
                }
            }
            Err(e) => {
                error!(
                    "Failed to fetch daily summaries for a batch of {} sensors: {e}. Skipping batch.",
                    chunk.len()
                );
            }
        }
    }

    let mut rows: Vec<SensorReportRow> = Vec::new();
    for ctx in contexts {
        let Some(points) = data_points_by_sensor.get(&ctx.sensor_id.to_string()) else {
            continue;
        };
        for point in points {
            rows.push(SensorReportRow {
                asset_name: ctx.asset_name.clone(),
                asset_id: ctx.asset_id.to_string(),
                custom_property: ctx.custom_property.clone(),
                sensor_name: ctx.sensor_name.clone(),
                sensor_id: ctx.sensor_id.to_string(),
                sensor_unit: ctx.sensor_unit.clone(),
                timestamp: point.r.clone(),
                avg: point.avg,
                max: point.max,
                min: point.min,
                lst: point.lst,
            });
        }
    }

    Ok(rows)
}

fn resolve_date_range(
    options: &GenerateSensorReportArgs,
) -> color_eyre::Result<(NaiveDate, NaiveDate)> {
    let has_ym = options.year.is_some() || options.month.is_some();
    let has_range = options.start.is_some() || options.end.is_some();

    if has_ym && has_range {
        return Err(AppError::InvalidDateRangeArgs.into());
    }

    if has_range {
        let start_str = options
            .start
            .as_ref()
            .ok_or(AppError::InvalidDateRangeArgs)?;
        let end_str = options.end.as_ref().ok_or(AppError::InvalidDateRangeArgs)?;
        let start = NaiveDate::parse_from_str(start_str, "%Y-%m-%d")
            .map_err(|_| AppError::InvalidDateFormat(start_str.clone()))?;
        let end = NaiveDate::parse_from_str(end_str, "%Y-%m-%d")
            .map_err(|_| AppError::InvalidDateFormat(end_str.clone()))?;
        return Ok((start, end));
    }

    let year = options.year.ok_or(AppError::InvalidDateRangeArgs)?;
    let month = options.month.ok_or(AppError::InvalidDateRangeArgs)?;

    let start = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| AppError::InvalidDateFormat(format!("{year}-{month:02}-01")))?;
    let (ny, nm) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let end = NaiveDate::from_ymd_opt(ny, nm, 1)
        .ok_or_else(|| AppError::InvalidDateFormat(format!("{ny}-{nm:02}-01")))?;
    Ok((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hyperview::api_constants::{
        ASSET_ASSETS_API_PREFIX, ASSET_SEARCH_API_PREFIX, CUSTOM_ASSET_PROPERTIES_API_PREFIX,
        SENSOR_API_PREFIX, SENSOR_DAILY_SUMMARIES_NUMERIC_API_PREFIX,
    };
    use crate::hyperview::cli_data::AssetTypes;
    use httpmock::prelude::*;
    use httpmock::{Mock, MockServer};
    use serde_json::{Value, json};
    use std::time::Duration;

    const ALL_LOCATION_ID: &str = "11223344-5566-7788-99aa-bbccddeeff00";

    fn base_args(asset_type: AssetTypes, sensor: &str) -> GenerateSensorReportArgs {
        GenerateSensorReportArgs {
            asset_type,
            sensor: sensor.to_string(),
            year: Some(2026),
            month: Some(2),
            start: None,
            end: None,
            custom_property: None,
            location_path: None,
            manufacturer: None,
            product: None,
            skip: 0,
            limit: 100,
            output_type: OutputOptions::Record,
            filename: None,
        }
    }

    fn auth_token() -> AuthToken {
        AuthToken::for_test("Bearer test_token", Duration::from_hours(1))
    }

    fn asset_hit(asset_id: Uuid, name: &str) -> Value {
        json!({
            "id": asset_id.to_string(),
            "displayName": name,
            "assetLifecycleState": "InService",
            "assetType": "Rack",
            "manufacturerId": "m",
            "manufacturerName": "m-name",
            "monitoringState": "Ok",
            "parentId": "p",
            "parentDisplayName": "p-name",
            "productId": "pd",
            "productName": "pd-name",
            "status": "Ok",
            "delimitedPath": "All~Datacenter",
            "assetProperty_serialNumber": [],
        })
    }

    fn mock_search<'a>(server: &'a MockServer, hits: &[Value]) -> (Mock<'a>, Mock<'a>) {
        let all_location_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{ASSET_ASSETS_API_PREFIX}/{ALL_LOCATION_ID}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({"name": "All"}));
        });

        let total = hits.len();
        let hits_owned = hits.to_vec();
        let search_mock = server.mock(|when, then| {
            when.method(POST).path(ASSET_SEARCH_API_PREFIX);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "estimatedTotalHits": total,
                    "limit": 100,
                    "hits": hits_owned,
                }));
        });

        (all_location_mock, search_mock)
    }

    fn sensor_body(sensor_id: &str, asset_id: Uuid, name: &str, is_numeric: bool) -> Value {
        json!({
            "id": sensor_id,
            "name": name,
            "sensorTypeId": "t",
            "listIndex": null,
            "sensorTypeDescription": "",
            "value": 0,
            "rawValue": 0,
            "unitString": "kW",
            "dataSource": null,
            "dataCollectorId": null,
            "dataCollectorName": null,
            "lastValueUpdate": null,
            "sourceDeviceAssetId": asset_id.to_string(),
            "sensorAssociationType": "direct",
            "isNumeric": is_numeric,
            "accessPolicyId": "00000000-0000-0000-0000-000000000001",
            "accessPolicyName": "policy",
            "assetAccessPolicyId": "00000000-0000-0000-0000-000000000002",
            "accessPolicyIsInherited": false,
        })
    }

    #[test]
    fn test_resolve_date_range_year_month_happy_path() {
        let mut args = base_args(AssetTypes::Rack, "s");
        args.year = Some(2026);
        args.month = Some(2);
        let (start, end) = resolve_date_range(&args).unwrap();
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 2, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
    }

    #[test]
    fn test_resolve_date_range_december_rollover() {
        let mut args = base_args(AssetTypes::Rack, "s");
        args.year = Some(2026);
        args.month = Some(12);
        let (start, end) = resolve_date_range(&args).unwrap();
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 12, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2027, 1, 1).unwrap());
    }

    #[test]
    fn test_resolve_date_range_start_end_overrides() {
        let mut args = base_args(AssetTypes::Rack, "s");
        args.year = None;
        args.month = None;
        args.start = Some("2026-02-15".to_string());
        args.end = Some("2026-02-20".to_string());
        let (start, end) = resolve_date_range(&args).unwrap();
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 2, 15).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 2, 20).unwrap());
    }

    #[test]
    fn test_resolve_date_range_rejects_mixing_ym_and_range() {
        let mut args = base_args(AssetTypes::Rack, "s");
        args.start = Some("2026-02-15".to_string());
        args.end = Some("2026-02-20".to_string());
        // year/month still set from base_args
        assert!(resolve_date_range(&args).is_err());
    }

    #[test]
    fn test_resolve_date_range_rejects_when_nothing_supplied() {
        let mut args = base_args(AssetTypes::Rack, "s");
        args.year = None;
        args.month = None;
        assert!(resolve_date_range(&args).is_err());
    }

    #[test]
    fn test_resolve_date_range_rejects_start_without_end() {
        let mut args = base_args(AssetTypes::Rack, "s");
        args.year = None;
        args.month = None;
        args.start = Some("2026-02-15".to_string());
        assert!(resolve_date_range(&args).is_err());
    }

    #[test]
    fn test_resolve_date_range_rejects_bad_date_format() {
        let mut args = base_args(AssetTypes::Rack, "s");
        args.year = None;
        args.month = None;
        args.start = Some("not-a-date".to_string());
        args.end = Some("2026-02-20".to_string());
        assert!(resolve_date_range(&args).is_err());
    }

    #[tokio::test]
    async fn test_generate_sensor_report_happy_path_single_asset() {
        let asset_id = Uuid::new_v4();
        let sensor_id = Uuid::new_v4();

        let server = MockServer::start();
        let (all_mock, search_mock) = mock_search(&server, &[asset_hit(asset_id, "Rack-42")]);

        let sensors_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("{SENSOR_API_PREFIX}/{asset_id}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([sensor_body(
                    &sensor_id.to_string(),
                    asset_id,
                    "averageKwhByHour",
                    true
                )]));
        });

        let summaries_mock = server.mock(|when, then| {
            when.method(GET)
                .path(SENSOR_DAILY_SUMMARIES_NUMERIC_API_PREFIX)
                .query_param("startTime", "2026-02-01T00:00:00.000")
                .query_param("endTime", "2026-03-01T00:00:00.000");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([{
                    "sensorId": sensor_id.to_string(),
                    "sensorTypeDescription": "Power",
                    "sensorTypeId": "type-1",
                    "name": "averageKwhByHour",
                    "sensorDataPoints": [
                        { "r": "2026-02-01T00:00:00.000", "avg": 1.0, "max": 2.0, "min": 0.5, "lst": 1.5 },
                        { "r": "2026-02-02T00:00:00.000", "avg": 3.0, "max": 4.0, "min": 2.5, "lst": 3.5 }
                    ]
                }]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();

        let rows = generate_sensor_report_async(
            &config,
            &client,
            &mut token,
            base_args(AssetTypes::Rack, "averageKwhByHour"),
        )
        .await
        .unwrap();

        all_mock.assert();
        search_mock.assert();
        sensors_mock.assert();
        summaries_mock.assert();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].asset_id, asset_id.to_string());
        assert_eq!(rows[0].sensor_id, sensor_id.to_string());
        assert_eq!(rows[0].sensor_unit, "kW");
        assert_eq!(rows[0].custom_property, "");
        assert!((rows[0].avg - 1.0).abs() < f64::EPSILON);
        assert!((rows[1].avg - 3.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_generate_sensor_report_skips_non_numeric_sensor() {
        let asset_id = Uuid::new_v4();
        let sensor_id = Uuid::new_v4();

        let server = MockServer::start();
        let (_all, _search) = mock_search(&server, &[asset_hit(asset_id, "Rack-42")]);

        server.mock(|when, then| {
            when.method(GET)
                .path(format!("{SENSOR_API_PREFIX}/{asset_id}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([sensor_body(
                    &sensor_id.to_string(),
                    asset_id,
                    "averageKwhByHour",
                    false
                )]));
        });
        let summaries_should_not_fire = server.mock(|when, then| {
            when.method(GET)
                .path(SENSOR_DAILY_SUMMARIES_NUMERIC_API_PREFIX);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();

        let rows = generate_sensor_report_async(
            &config,
            &client,
            &mut token,
            base_args(AssetTypes::Rack, "averageKwhByHour"),
        )
        .await
        .unwrap();

        assert!(rows.is_empty());
        summaries_should_not_fire.assert_calls(0);
    }

    #[tokio::test]
    async fn test_generate_sensor_report_skips_when_sensor_name_missing() {
        let asset_id = Uuid::new_v4();
        let sensor_id = Uuid::new_v4();

        let server = MockServer::start();
        let (_all, _search) = mock_search(&server, &[asset_hit(asset_id, "Rack-42")]);

        server.mock(|when, then| {
            when.method(GET)
                .path(format!("{SENSOR_API_PREFIX}/{asset_id}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([sensor_body(
                    &sensor_id.to_string(),
                    asset_id,
                    "someOtherSensor",
                    true
                )]));
        });
        let summaries_should_not_fire = server.mock(|when, then| {
            when.method(GET)
                .path(SENSOR_DAILY_SUMMARIES_NUMERIC_API_PREFIX);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();

        let rows = generate_sensor_report_async(
            &config,
            &client,
            &mut token,
            base_args(AssetTypes::Rack, "averageKwhByHour"),
        )
        .await
        .unwrap();

        assert!(rows.is_empty());
        summaries_should_not_fire.assert_calls(0);
    }

    #[tokio::test]
    async fn test_generate_sensor_report_continues_after_sensor_list_error() {
        let asset_ok = Uuid::new_v4();
        let asset_fail = Uuid::new_v4();
        let sensor_ok = Uuid::new_v4();

        let server = MockServer::start();
        let (_all, _search) = mock_search(
            &server,
            &[
                asset_hit(asset_fail, "Rack-fail"),
                asset_hit(asset_ok, "Rack-ok"),
            ],
        );

        server.mock(|when, then| {
            when.method(GET)
                .path(format!("{SENSOR_API_PREFIX}/{asset_fail}"));
            then.status(500);
        });
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("{SENSOR_API_PREFIX}/{asset_ok}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([sensor_body(
                    &sensor_ok.to_string(),
                    asset_ok,
                    "averageKwhByHour",
                    true
                )]));
        });
        server.mock(|when, then| {
            when.method(GET).path(SENSOR_DAILY_SUMMARIES_NUMERIC_API_PREFIX);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([{
                    "sensorId": sensor_ok.to_string(),
                    "sensorTypeDescription": "",
                    "sensorTypeId": "t",
                    "name": "averageKwhByHour",
                    "sensorDataPoints": [
                        { "r": "2026-02-01T00:00:00.000", "avg": 10.0, "max": 11.0, "min": 9.0, "lst": 10.5 }
                    ]
                }]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();

        let rows = generate_sensor_report_async(
            &config,
            &client,
            &mut token,
            base_args(AssetTypes::Rack, "averageKwhByHour"),
        )
        .await
        .expect("bulk report must not abort on a per-asset error");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].asset_id, asset_ok.to_string());
    }

    #[tokio::test]
    async fn test_generate_sensor_report_custom_property_enrichment() {
        let asset_id = Uuid::new_v4();
        let sensor_id = Uuid::new_v4();

        let server = MockServer::start();
        let (_all, _search) = mock_search(&server, &[asset_hit(asset_id, "Rack-42")]);

        server.mock(|when, then| {
            when.method(GET)
                .path(format!("{SENSOR_API_PREFIX}/{asset_id}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([sensor_body(
                    &sensor_id.to_string(),
                    asset_id,
                    "averageKwhByHour",
                    true
                )]));
        });
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("{CUSTOM_ASSET_PROPERTIES_API_PREFIX}/{asset_id}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([
                    { "id": "cp1", "customAssetPropertyKeyId": "k", "customAssetPropertyGroupId": "g", "value": "Engineering", "dataType": "String", "name": "Business Unit", "groupName": "", "dataSource": "", "updatedDateTime": "", "unit": "" }
                ]));
        });
        server.mock(|when, then| {
            when.method(GET).path(SENSOR_DAILY_SUMMARIES_NUMERIC_API_PREFIX);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([{
                    "sensorId": sensor_id.to_string(),
                    "sensorTypeDescription": "",
                    "sensorTypeId": "t",
                    "name": "averageKwhByHour",
                    "sensorDataPoints": [
                        { "r": "2026-02-01T00:00:00.000", "avg": 1.0, "max": 2.0, "min": 0.5, "lst": 1.5 }
                    ]
                }]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();

        let mut args = base_args(AssetTypes::Rack, "averageKwhByHour");
        args.custom_property = Some("business unit".to_string()); // lowercase to prove case-insensitive

        let rows = generate_sensor_report_async(&config, &client, &mut token, args)
            .await
            .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].custom_property, "Engineering");
    }

    #[tokio::test]
    async fn test_generate_sensor_report_custom_property_miss_is_not_fatal() {
        let asset_id = Uuid::new_v4();
        let sensor_id = Uuid::new_v4();

        let server = MockServer::start();
        let (_all, _search) = mock_search(&server, &[asset_hit(asset_id, "Rack-42")]);

        server.mock(|when, then| {
            when.method(GET)
                .path(format!("{SENSOR_API_PREFIX}/{asset_id}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([sensor_body(
                    &sensor_id.to_string(),
                    asset_id,
                    "averageKwhByHour",
                    true
                )]));
        });
        server.mock(|when, then| {
            when.method(GET)
                .path(format!("{CUSTOM_ASSET_PROPERTIES_API_PREFIX}/{asset_id}"));
            then.status(404);
        });
        server.mock(|when, then| {
            when.method(GET).path(SENSOR_DAILY_SUMMARIES_NUMERIC_API_PREFIX);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([{
                    "sensorId": sensor_id.to_string(),
                    "sensorTypeDescription": "",
                    "sensorTypeId": "t",
                    "name": "averageKwhByHour",
                    "sensorDataPoints": [
                        { "r": "2026-02-01T00:00:00.000", "avg": 1.0, "max": 2.0, "min": 0.5, "lst": 1.5 }
                    ]
                }]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();

        let mut args = base_args(AssetTypes::Rack, "averageKwhByHour");
        args.custom_property = Some("Business Unit".to_string());

        let rows = generate_sensor_report_async(&config, &client, &mut token, args)
            .await
            .expect("custom property fetch failure must not abort the run");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].custom_property, "");
    }

    #[tokio::test]
    async fn test_generate_sensor_report_batches_at_boundary() {
        // BULK_ACTION_BATCH_SIZE + 1 assets → 2 daily-summary calls.
        let asset_count = BULK_ACTION_BATCH_SIZE + 1;
        let assets: Vec<(Uuid, Uuid)> = (0..asset_count)
            .map(|_| (Uuid::new_v4(), Uuid::new_v4()))
            .collect();
        let hits: Vec<Value> = assets
            .iter()
            .enumerate()
            .map(|(i, (aid, _))| asset_hit(*aid, &format!("Rack-{i}")))
            .collect();

        let server = MockServer::start();
        let (_all, _search) = mock_search(&server, &hits);

        for (aid, sid) in &assets {
            let asset_id = *aid;
            let sensor_id = *sid;
            server.mock(|when, then| {
                when.method(GET)
                    .path(format!("{SENSOR_API_PREFIX}/{asset_id}"));
                then.status(200)
                    .header("Content-Type", "application/json")
                    .json_body(json!([sensor_body(
                        &sensor_id.to_string(),
                        asset_id,
                        "averageKwhByHour",
                        true
                    )]));
            });
        }

        // Return an empty summary array for both batches — we only care about call counts here.
        let summaries_mock = server.mock(|when, then| {
            when.method(GET)
                .path(SENSOR_DAILY_SUMMARIES_NUMERIC_API_PREFIX);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();

        let rows = generate_sensor_report_async(
            &config,
            &client,
            &mut token,
            base_args(AssetTypes::Rack, "averageKwhByHour"),
        )
        .await
        .unwrap();

        assert!(rows.is_empty());
        summaries_mock.assert_calls(2);
    }

    #[tokio::test]
    async fn test_generate_sensor_report_continues_after_batch_error() {
        let asset_count = BULK_ACTION_BATCH_SIZE + 1;
        let assets: Vec<(Uuid, Uuid)> = (0..asset_count)
            .map(|_| (Uuid::new_v4(), Uuid::new_v4()))
            .collect();
        let hits: Vec<Value> = assets
            .iter()
            .enumerate()
            .map(|(i, (aid, _))| asset_hit(*aid, &format!("Rack-{i}")))
            .collect();

        let server = MockServer::start();
        let (_all, _search) = mock_search(&server, &hits);

        for (aid, sid) in &assets {
            let asset_id = *aid;
            let sensor_id = *sid;
            server.mock(|when, then| {
                when.method(GET)
                    .path(format!("{SENSOR_API_PREFIX}/{asset_id}"));
                then.status(200)
                    .header("Content-Type", "application/json")
                    .json_body(json!([sensor_body(
                        &sensor_id.to_string(),
                        asset_id,
                        "averageKwhByHour",
                        true
                    )]));
            });
        }

        // First batch (100 sensors) → 500; batch differentiator uses sensor id from the last asset,
        // which lands in batch 2 (the 101st entry).
        let last_sensor_id = assets.last().unwrap().1.to_string();
        let batch1_mock = server.mock(|when, then| {
            when.method(GET)
                .path(SENSOR_DAILY_SUMMARIES_NUMERIC_API_PREFIX)
                .query_param_excludes("sensorIds", last_sensor_id.clone());
            then.status(500);
        });
        let batch2_mock = server.mock(|when, then| {
            when.method(GET)
                .path(SENSOR_DAILY_SUMMARIES_NUMERIC_API_PREFIX)
                .query_param("sensorIds", assets.last().unwrap().1.to_string());
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([{
                    "sensorId": assets.last().unwrap().1.to_string(),
                    "sensorTypeDescription": "",
                    "sensorTypeId": "t",
                    "name": "averageKwhByHour",
                    "sensorDataPoints": [
                        { "r": "2026-02-01T00:00:00.000", "avg": 42.0, "max": 43.0, "min": 41.0, "lst": 42.5 }
                    ]
                }]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();

        let rows = generate_sensor_report_async(
            &config,
            &client,
            &mut token,
            base_args(AssetTypes::Rack, "averageKwhByHour"),
        )
        .await
        .expect("orchestrator must not abort on a per-batch error");

        batch1_mock.assert();
        batch2_mock.assert();
        assert_eq!(rows.len(), 1);
        assert!((rows[0].avg - 42.0).abs() < f64::EPSILON);
    }
}
