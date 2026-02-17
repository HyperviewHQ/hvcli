use log::debug;
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

use super::{
    api_constants::POWER_ASSOCIATION_API_PREFIX,
    asset_power_api_data::{
        BulkPowerAssociationCreateDto, PowerAssociationCreateDto, PowerProviderComponentDto,
    },
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
        .json::<Vec<PowerProviderComponentDto>>()
        .await?;

    Ok(resp)
}

pub async fn bulk_add_power_association_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    filename: &String,
) -> color_eyre::Result<()> {
    // Asset ID : (Component Number, Optional Panel Number): Component Id
    let mut power_provider_component_map: HashMap<Uuid, HashMap<(u64, Option<u64>), Uuid>> =
        HashMap::new();

    let mut reader = csv::Reader::from_path(filename)?;

    while let Some(Ok(record)) = reader.deserialize::<BulkPowerAssociationCreateDto>().next() {
        debug!("updating asset id {}", record.asset_id);

        if record.provider_component_number.is_none() {
            debug!("Component number is not asset, assuming direct asset to asset association");
            add_power_association_async(
                config,
                req,
                auth_header,
                record.asset_id,
                record.provider_asset_id,
            )
            .await?;

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

            get_provider_component_map_async(
                config,
                req,
                auth_header,
                record.provider_asset_id,
                api_path.expect("Expect API path variable to be set at this point"),
                &mut power_provider_component_map,
            )
            .await?;
        }

        // Add power association
        if let Some(component_map) = power_provider_component_map.get(&record.provider_asset_id)
            && let Some(component_id) = component_map.get(&(
                record
                    .provider_component_number
                    .expect("Expect component number to be ser"),
                record.provider_panel_number,
            ))
        {
            add_power_association_async(config, req, auth_header, record.asset_id, *component_id)
                .await?;
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

    let _resp = req
        .post(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(&association_data)
        .send()
        .await?;

    Ok(())
}
