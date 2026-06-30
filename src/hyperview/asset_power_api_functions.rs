use log::{debug, error};
use reqwest::{
    Client,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use serde_json::Map;
use std::collections::HashMap;
use uuid::Uuid;

use crate::hyperview::{
    api_constants::{
        BUSWAY_TAPOFF_API_PREFIX, PDU_RPP_BREAKERS_API_PREFIX, RACK_PDU_OUTLETS_API_PREFIX,
    },
    cli_data::AssetTypes,
};
use crate::retry_on_unauthorized_async;

use super::{
    api_constants::POWER_ASSOCIATION_API_PREFIX,
    asset_power_api_data::{
        BulkPowerAssociationCreateDto, PowerAssociationCreateDto, PowerProviderComponentDto,
    },
    auth::AuthToken,
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
        .error_for_status()?
        .json::<Vec<PowerProviderComponentDto>>()
        .await?;

    Ok(resp)
}

pub async fn bulk_add_power_association_async(
    config: &AppConfig,
    req: &Client,
    auth_token: &mut AuthToken,
    filename: &String,
) -> color_eyre::Result<()> {
    // Asset ID : (Component Number, Optional Panel Number): Component Id
    let mut power_provider_component_map: HashMap<Uuid, HashMap<(u64, Option<u64>), Uuid>> =
        HashMap::new();

    let mut reader = csv::Reader::from_path(filename)?;

    while let Some(Ok(record)) = reader.deserialize::<BulkPowerAssociationCreateDto>().next() {
        auth_token.refresh_if_needed_async(config).await?;

        debug!("updating asset id {}", record.asset_id);

        if record.provider_component_number.is_none() {
            debug!("Component number is not asset, assuming direct asset to asset association");
            if let Err(e) = retry_on_unauthorized_async!(
                config,
                auth_token,
                add_power_association_async(
                    config,
                    req,
                    &auth_token.header,
                    record.asset_id,
                    record.provider_asset_id,
                )
                .await
            ) {
                error!(
                    "Failed to add power association for asset id {}: {e}",
                    record.asset_id
                );
            }

            continue;
        }

        // Cache the component mapping (e.g. Outlets) to make the work faster
        if !power_provider_component_map.contains_key(&record.provider_asset_id) {
            let api_path = match record.provider_asset_type {
                AssetTypes::PduAndRpp => Some(PDU_RPP_BREAKERS_API_PREFIX),
                AssetTypes::RackPdu => Some(RACK_PDU_OUTLETS_API_PREFIX),
                AssetTypes::Busway => Some(BUSWAY_TAPOFF_API_PREFIX),
                _ => None,
            };

            if api_path.is_none() {
                continue;
            }

            if let Err(e) = retry_on_unauthorized_async!(
                config,
                auth_token,
                get_provider_component_map_async(
                    config,
                    req,
                    &auth_token.header,
                    record.provider_asset_id,
                    api_path.expect("Expect API path variable to be set at this point"),
                    &mut power_provider_component_map,
                )
                .await
            ) {
                error!(
                    "Failed to fetch power provider components for asset id {}: {e}",
                    record.provider_asset_id
                );
                continue;
            }
        }

        // Add power association
        if let Some(component_map) = power_provider_component_map.get(&record.provider_asset_id)
            && let Some(component_id) = component_map.get(&(
                record
                    .provider_component_number
                    .expect("Expect component number to be set"),
                record.provider_panel_number,
            ))
            && let Err(e) = retry_on_unauthorized_async!(
                config,
                auth_token,
                add_power_association_async(
                    config,
                    req,
                    &auth_token.header,
                    record.asset_id,
                    *component_id,
                )
                .await
            )
        {
            error!(
                "Failed to add power association for asset id {}: {e}",
                record.asset_id
            );
        }
    }

    Ok(())
}

async fn get_provider_component_map_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    provider_asset_id: Uuid,
    api_path: &str,
    power_provider_component_map: &mut HashMap<Uuid, HashMap<(u64, Option<u64>), Uuid>>,
) -> color_eyre::Result<()> {
    let mut component_map: HashMap<(u64, Option<u64>), Uuid> = HashMap::new();

    let component_list =
        get_power_provider_components_async(config, req, auth_header, api_path, provider_asset_id)
            .await?;

    for component in component_list {
        component_map.insert((component.number, component.panel_number), component.id);
    }

    power_provider_component_map.insert(provider_asset_id, component_map);

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

    req.post(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(&association_data)
        .send()
        .await?
        .error_for_status()?;

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

    fn write_csv(rows: &[&str]) -> tempfile::NamedTempFile {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            tmp,
            "asset_id,provider_asset_id,provider_asset_type,provider_component_number,provider_panel_number"
        )
        .unwrap();
        for row in rows {
            writeln!(tmp, "{row}").unwrap();
        }
        tmp.flush().unwrap();
        tmp
    }

    #[tokio::test]
    async fn test_get_power_provider_components_uses_path_id_for_rack_pdu() {
        let provider_id = Uuid::new_v4();
        let url_path = format!("{RACK_PDU_OUTLETS_API_PREFIX}/{provider_id}");

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path(url_path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([{
                    "id": "11111111-1111-1111-1111-111111111111",
                    "name": "Outlet 1",
                    "outletNumber": 1,
                    "panelNumber": null
                }]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth = "Bearer t".to_string();

        let result = get_power_provider_components_async(
            &config,
            &client,
            &auth,
            RACK_PDU_OUTLETS_API_PREFIX,
            provider_id,
        )
        .await
        .unwrap();

        m.assert();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].number, 1);
        assert_eq!(result[0].panel_number, None);
    }

    #[tokio::test]
    async fn test_get_power_provider_components_uses_query_id_for_pdu_rpp() {
        let provider_id = Uuid::new_v4();
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET)
                .path(PDU_RPP_BREAKERS_API_PREFIX)
                .query_param("assetId", provider_id.to_string());
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([{
                    "id": "22222222-2222-2222-2222-222222222222",
                    "name": "Breaker A1",
                    "breakerNumber": 5,
                    "panelNumber": 2
                }]));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth = "Bearer t".to_string();

        let result = get_power_provider_components_async(
            &config,
            &client,
            &auth,
            PDU_RPP_BREAKERS_API_PREFIX,
            provider_id,
        )
        .await
        .unwrap();

        m.assert();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].number, 5);
        assert_eq!(result[0].panel_number, Some(2));
    }

    #[tokio::test]
    async fn test_bulk_add_power_association_direct_asset_to_asset_when_no_component_number() {
        let consumer = Uuid::new_v4();
        let provider = Uuid::new_v4();

        let server = MockServer::start();
        let assoc_mock = server.mock(|when, then| {
            when.method(POST)
                .path(POWER_ASSOCIATION_API_PREFIX)
                .body_includes(consumer.to_string())
                .body_includes(provider.to_string());
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        // No component number — direct asset-to-asset path.
        let csv = write_csv(&[&format!("{consumer},{provider},RackPdu,,")]);

        bulk_add_power_association_async(
            &config,
            &client,
            &mut token,
            &csv.path().to_string_lossy().to_string(),
        )
        .await
        .unwrap();

        assoc_mock.assert();
    }

    #[tokio::test]
    async fn test_bulk_add_power_association_skips_unsupported_provider_type() {
        let consumer = Uuid::new_v4();
        let provider = Uuid::new_v4();

        let server = MockServer::start();
        // If we hit either of these the test fails — Location is not a power provider type.
        let any_get = server.mock(|when, then| {
            when.method(GET);
            then.status(200).json_body(json!([]));
        });
        let any_post = server.mock(|when, then| {
            when.method(POST);
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        let csv = write_csv(&[&format!("{consumer},{provider},Location,1,")]);

        bulk_add_power_association_async(
            &config,
            &client,
            &mut token,
            &csv.path().to_string_lossy().to_string(),
        )
        .await
        .unwrap();

        any_get.assert_calls(0);
        any_post.assert_calls(0);
    }

    #[tokio::test]
    async fn test_bulk_add_power_association_caches_component_map_across_rows() {
        let consumer_a = Uuid::new_v4();
        let consumer_b = Uuid::new_v4();
        let provider = Uuid::new_v4();
        let component_id = Uuid::new_v4();
        let list_path = format!("{RACK_PDU_OUTLETS_API_PREFIX}/{provider}");

        let server = MockServer::start();
        let list_mock = server.mock(|when, then| {
            when.method(GET).path(list_path);
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([{
                    "id": component_id.to_string(),
                    "name": "Outlet 1",
                    "outletNumber": 1,
                    "panelNumber": null
                }]));
        });
        let assoc_mock = server.mock(|when, then| {
            when.method(POST)
                .path(POWER_ASSOCIATION_API_PREFIX)
                .body_includes(component_id.to_string());
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        // Two rows pointing at the same provider — the component list should only be fetched once.
        let csv = write_csv(&[
            &format!("{consumer_a},{provider},RackPdu,1,"),
            &format!("{consumer_b},{provider},RackPdu,1,"),
        ]);

        bulk_add_power_association_async(
            &config,
            &client,
            &mut token,
            &csv.path().to_string_lossy().to_string(),
        )
        .await
        .unwrap();

        list_mock.assert_calls(1);
        assoc_mock.assert_calls(2);
    }

    #[tokio::test]
    async fn test_bulk_add_power_association_continues_after_association_error() {
        let consumer_fail = Uuid::new_v4();
        let consumer_ok = Uuid::new_v4();
        let provider = Uuid::new_v4();

        let server = MockServer::start();
        let fail_mock = server.mock(|when, then| {
            when.method(POST)
                .path(POWER_ASSOCIATION_API_PREFIX)
                .body_includes(consumer_fail.to_string());
            then.status(500);
        });
        let ok_mock = server.mock(|when, then| {
            when.method(POST)
                .path(POWER_ASSOCIATION_API_PREFIX)
                .body_includes(consumer_ok.to_string());
            then.status(200);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let mut token = auth_token();
        // Direct asset-to-asset path so the cache lookup is bypassed.
        let csv = write_csv(&[
            &format!("{consumer_fail},{provider},RackPdu,,"),
            &format!("{consumer_ok},{provider},RackPdu,,"),
        ]);

        bulk_add_power_association_async(
            &config,
            &client,
            &mut token,
            &csv.path().to_string_lossy().to_string(),
        )
        .await
        .expect("bulk add should not abort on a per-row 500");

        fail_mock.assert();
        ok_mock.assert();
    }
}
