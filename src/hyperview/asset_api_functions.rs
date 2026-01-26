use log::{debug, error, info, trace};
use reqwest::{
    Client,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use serde_json::{Value, json};
use std::str::FromStr;
use uuid::Uuid;

use crate::hyperview::asset_api_data::AddRackAccessoryRecord;

use super::{
    api_constants::{
        ASSET_ASSETS_API_PREFIX, ASSET_LOCATION_API_PREFIX, ASSET_PORTS_API_PREFIX,
        ASSET_SEARCH_API_PREFIX, RACK_PANEL_API_PREFIX,
    },
    app_errors::AppError,
    asset_api_data::{
        AssetDto, AssetLocationDTO, AssetPortDto, UpdateAssetLocationRecord, UpdateAssetNameRecord,
    },
    asset_properties_api_functions::get_named_asset_property_async,
    cli_data::{
        AppConfig, ListAnyOfArgs, ListAssetPortsArgs, RackPanelType, RackSide, SearchAssetsArgs,
        UpdateAssetLocationArgs,
    },
};

pub async fn bulk_add_rack_accessory(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    filename: &String,
) -> color_eyre::Result<()> {
    let mut reader = csv::Reader::from_path(filename)?;
    while let Some(Ok(record)) = reader.deserialize::<AddRackAccessoryRecord>().next() {
        debug!("Adding rack accessory to rack_id {}", record.id);
        add_rack_accessory(
            config,
            req,
            auth_header,
            &record.id,
            &record.panel_type,
            &record.side,
            record.u_location,
        )
        .await?;
    }

    Ok(())
}

pub async fn add_rack_accessory(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    rack_id: &Uuid,
    rack_panel_type: &RackPanelType,
    rack_side: &RackSide,
    rack_u_location: usize,
) -> color_eyre::Result<()> {
    let target_url = format!("{}{}", config.instance_url, RACK_PANEL_API_PREFIX);
    debug!("Request URL: {target_url}");

    let display_name_annotation = match rack_side {
        RackSide::Rear => "(R)",
        _ => "",
    };

    let display_name = match rack_panel_type {
        RackPanelType::BlankingPanel => {
            format!("Blanking Panel at {rack_u_location}U{display_name_annotation}")
        }

        RackPanelType::CableManagement => {
            format!("Cable Management at {rack_u_location}U{display_name_annotation}")
        }
    };

    let panel = match rack_panel_type {
        RackPanelType::BlankingPanel => "blankingPanel",
        RackPanelType::CableManagement => "cableManagement",
    };

    let side = match rack_side {
        RackSide::Front => "front",
        RackSide::Rear => "rear",
        RackSide::Unknown => "",
    };

    let payload = serde_json::json!({
        "panelType": panel,
        "rackId": rack_id,
        "rackSide": side,
        "rackPanelDataCollection": [
            {
                "rackUnit": rack_u_location,
                "displayName": display_name,
            }
        ]
    });
    trace!(
        "Add rack accessory payload: {}",
        serde_json::to_string_pretty(&payload)?
    );

    let resp = req
        .post(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(&payload)
        .send()
        .await?;

    trace!("Server response: {}", resp.status());

    Ok(())
}

pub async fn bulk_update_ports_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    filename: String,
    is_patchpanel: bool,
) -> color_eyre::Result<()> {
    let mut reader = csv::Reader::from_path(filename)?;
    while let Some(Ok(record)) = reader.deserialize::<AssetPortDto>().next() {
        debug!("Updating port id: {}", record.id);

        // Patch panel flow
        if is_patchpanel {
            trace!("Updating patch panel port");
            let target_url = format!(
                "{}{}/patchPanel/{}",
                config.instance_url, ASSET_PORTS_API_PREFIX, record.id
            );
            debug!("Request URL: {target_url}");

            let payload = json!({
              "id": record.id,
              "name": record.name,
              "parentId": record.parent_id,
              "portNumber": record.port_number,
              "connectorTypeValueId": record.connector_type_value_id,
              "portSideValueId": record.port_side_value_id,
            });
            debug!("Payload: {}", serde_json::to_string_pretty(&payload)?);

            update_port_async(req, auth_header, target_url, payload).await?;

            // Go to next record
            continue;
        }

        // Asset flow
        trace!("Updating asset port");
        let target_url = format!(
            "{}{}/{}",
            config.instance_url, ASSET_PORTS_API_PREFIX, record.id
        );
        debug!("Request URL: {target_url}");

        let payload = json!({
          "id": record.id,
          "name": record.name,
          "parentId": record.parent_id,
          "portNumber": record.port_number,
          "portSideValueId": record.port_side_value_id,
          "portTypeValueId": record.port_type_value_id
        });

        debug!("Payload: {}", serde_json::to_string_pretty(&payload)?);

        update_port_async(req, auth_header, target_url, payload).await?;
    }

    Ok(())
}

async fn update_port_async(
    req: &Client,
    auth_header: &String,
    target_url: String,
    payload: Value,
) -> color_eyre::Result<()> {
    let resp = req
        .put(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(&payload)
        .send()
        .await?
        .json::<Value>()
        .await?;

    debug!(
        "Update port return: {}",
        serde_json::to_string_pretty(&resp)?
    );

    Ok(())
}

pub async fn list_asset_ports_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    list_asset_ports_args: ListAssetPortsArgs,
) -> color_eyre::Result<Vec<AssetPortDto>> {
    let target_url = format!(
        "{}{}/detailed/{}",
        config.instance_url, ASSET_PORTS_API_PREFIX, list_asset_ports_args.id
    );

    debug!("Request URL: {target_url}");

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .json::<Vec<Value>>()
        .await?;

    let mut asset_ports = Vec::new();

    for record in resp {
        let mut port = AssetPortDto {
            ..Default::default()
        };

        if let Some(id) = record["id"].as_str() {
            port.id = Uuid::parse_str(id).unwrap();
        }

        if let Some(name) = record["name"].as_str() {
            port.name = name.to_string();
        }

        if let Some(parent_id) = record["parentId"].as_str() {
            port.parent_id = parent_id.to_string();
        }

        if let Some(port_number) = record["portNumber"].as_i64() {
            port.port_number = port_number;
        }

        if let Some(port_side) = record["portSide"].as_str() {
            port.port_side = Some(port_side.to_string());
        }

        if let Some(port_side_value_id) = record["portSideValueId"].as_str() {
            port.port_side_value_id = Some(port_side_value_id.to_string());
        }

        if let Some(connector_type_value_id) = record["connectorTypeValueId"].as_str() {
            port.connector_type_value_id = Some(connector_type_value_id.to_string());
        }

        if let Some(port_type_value_id) = record["portTypeValueId"].as_str() {
            port.port_type_value_id = Some(port_type_value_id.to_string());
        }

        asset_ports.push(port);
    }

    Ok(asset_ports)
}

pub async fn update_asset_location_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    update_location_data: UpdateAssetLocationArgs,
) -> color_eyre::Result<()> {
    let target_url = format!(
        "{}{}/{}?id={}",
        config.instance_url,
        ASSET_LOCATION_API_PREFIX,
        update_location_data.id,
        update_location_data.id
    );

    debug!("Request URL: {target_url}");

    let asset_location_dto = AssetLocationDTO {
        parent_id: update_location_data.new_location_id,
        rack_position: update_location_data.rack_position,
        rack_side: update_location_data.rack_side,
        rack_u_location: update_location_data.rack_u_location,
    };

    debug!(
        "New location payload: {}",
        serde_json::to_string_pretty(&asset_location_dto)?
    );

    let resp = req
        .put(target_url)
        .header(AUTHORIZATION, auth_header)
        .json(&asset_location_dto)
        .send()
        .await?
        .json::<Value>()
        .await?;

    debug!(
        "Update location return: {}",
        serde_json::to_string_pretty(&resp)?
    );

    Ok(())
}

pub async fn bulk_update_asset_location_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    filename: String,
) -> color_eyre::Result<()> {
    let mut reader = csv::Reader::from_path(filename)?;
    while let Some(Ok(record)) = reader.deserialize::<UpdateAssetLocationRecord>().next() {
        debug!(
            "Updating asset id: {} with new location: {}",
            record.asset_id, record.new_location_id
        );

        let id = record.asset_id;
        let new_location_id = record.new_location_id;

        let update_location_data = UpdateAssetLocationArgs {
            id,
            new_location_id,
            rack_position: record.rack_position,
            rack_side: record.rack_side,
            rack_u_location: record.rack_u_location,
        };

        update_asset_location_async(config, req, auth_header, update_location_data).await?;
    }

    Ok(())
}

async fn get_raw_asset_by_id_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    id: &Uuid,
) -> color_eyre::Result<Value> {
    let target_url = format!("{}{}/{}", config.instance_url, ASSET_ASSETS_API_PREFIX, id);

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .json::<Value>()
        .await?;

    Ok(resp)
}

pub async fn update_asset_name_by_id_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    id: Uuid,
    new_name: String,
) -> color_eyre::Result<()> {
    let target_url = format!("{}{}/{}", config.instance_url, ASSET_ASSETS_API_PREFIX, id);
    debug!("Request URL: {target_url}");

    let mut asset_value = get_raw_asset_by_id_async(config, req, auth_header, &id).await?;

    trace!(
        "Returned asset value: {}",
        serde_json::to_string_pretty(&asset_value)?
    );

    match asset_value.get_mut("name") {
        Some(name) => {
            debug!(
                "Old name: {}, new name: {}",
                serde_json::to_string_pretty(name)?,
                new_name
            );

            if let Value::String(name_string) = name {
                *name_string = new_name;
            }

            let _resp = req
                .put(target_url)
                .header(AUTHORIZATION, auth_header)
                .json(&asset_value)
                .send()
                .await?;

            Ok(())
        }

        None => Err(AppError::AssetNotFound.into()),
    }
}

pub async fn bulk_update_asset_name_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    filename: String,
) -> color_eyre::Result<()> {
    let mut reader = csv::Reader::from_path(filename)?;
    while let Some(Ok(record)) = reader.deserialize::<UpdateAssetNameRecord>().next() {
        debug!(
            "Updating asset id: {} with new name: {}",
            record.asset_id, record.new_name
        );

        let new_name = record.new_name.trim().replace('"', "");

        if new_name.is_empty() {
            error!("New name can't be empty for asset id: {}", record.asset_id);
            continue;
        }

        update_asset_name_by_id_async(config, req, auth_header, record.asset_id, new_name).await?;
    }

    Ok(())
}

pub async fn list_any_of_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    options: ListAnyOfArgs,
) -> color_eyre::Result<Vec<AssetDto>> {
    let target_url = format!("{}{}", config.instance_url, ASSET_SEARCH_API_PREFIX);
    debug!("Request URL: {target_url}");
    debug!("Options: {options:#?}");

    let search_query = compose_any_of_query(options.clone());

    trace!("{}", serde_json::to_string_pretty(&search_query).unwrap());

    let resp = req
        .post(target_url)
        .header(AUTHORIZATION, auth_header.clone())
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .json(&search_query)
        .send()
        .await?
        .json::<Value>()
        .await?;

    let total = resp
        .get("estimatedTotalHits")
        .expect("Expected estimatedTotalHits to be defined in response body.")
        .as_u64()
        .unwrap();

    let limit = resp
        .get("limit")
        .expect("Expected limit to be defined in response body.")
        .as_u64()
        .unwrap();

    info!("Meta Data: | Total: {total} | Limit: {limit} |");

    let mut asset_list = Vec::new();

    if total == 0 {
        return Ok(asset_list);
    }

    if let Some(assets) = resp.get("hits") {
        assets.as_array().unwrap().iter().for_each(|a| {
            debug!("RAW: {}", serde_json::to_string_pretty(&a).unwrap());

            let asset = AssetDto {
                id: Uuid::from_str(a.get("id").unwrap().as_str().unwrap()).unwrap(),
                name: a.get("displayName").unwrap().to_string(),
                asset_lifecycle_state: a.get("assetLifecycleState").unwrap().to_string(),
                asset_type_id: a.get("assetType").unwrap().to_string(),
                manufacturer_id: a.get("manufacturerId").unwrap().to_string(),
                manufacturer_name: a.get("manufacturerName").unwrap().to_string(),
                monitoring_state: a.get("monitoringState").unwrap().to_string(),
                parent_id: a.get("parentId").unwrap().to_string(),
                parent_name: a.get("parentDisplayName").unwrap().to_string(),
                product_id: a.get("productId").unwrap().to_string(),
                product_name: a.get("productName").unwrap().to_string(),
                status: a.get("status").unwrap().to_string(),
                path: a
                    .get("delimitedPath")
                    .unwrap()
                    .to_string()
                    .replace('~', "/"),
                serial_number: a
                    .get("assetProperty_serialNumber")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| serde_json::to_string(arr).ok())
                    .unwrap_or_else(|| "[]".to_string()),
                property: None,
            };

            asset_list.push(asset);
        });
    }

    if let Some(property_type) = options.show_property {
        for a in &mut asset_list {
            let props = get_named_asset_property_async(
                config,
                req,
                auth_header,
                a.id,
                property_type.clone(),
            )
            .await?;

            let prop_values: String = props.iter().fold(String::new(), |mut a, v| {
                let v = format!("{} ", v.value);
                a.push_str(&v);
                a
            });

            a.property = Some(prop_values);
        }
    }

    Ok(asset_list)
}

fn compose_any_of_query(options: ListAnyOfArgs) -> serde_json::Value {
    let mut search_query = json!({
      "limit": options.limit,
      "offset": options.skip,
      "attributesToRetrieve": [
        "id",
        "displayName",
        "assetLifecycleState",
        "assetType",
        "manufacturerId",
        "manufacturerName",
        "monitoringState",
        "parentId",
        "parentDisplayName",
        "productId",
        "productName",
        "status",
        "delimitedPath",
        "assetProperty_serialNumber"
      ],
      "filter": "",
    });

    let mut filters = Vec::new();

    filters.push(format!(
        "assetProperty_{} EXISTS AND assetProperty_{} IN {:?}",
        options.property_key, options.property_key, options.property_value
    ));

    if let Some(t) = options.asset_type {
        let asset_type = t.to_string();
        filters.push(format!("assetType = '{asset_type}'"));
    }

    if let Some(p) = options.location_path {
        let prepared_path = p.replace('/', "~").clone();
        filters.push(format!("delimitedPath STARTS WITH '{prepared_path}'"));
    }

    if let Some(custom_properties) = options.custom_properties {
        for custom_property in custom_properties {
            if let Some((custom_property_key_name, custom_property_key_value)) =
                custom_property.split_once('=')
            {
                let custom_property_key_attribute =
                    format!("customProperty_{} ", custom_property_key_name.trim());
                filters.push(format!(
                    "{} = '{}'",
                    custom_property_key_attribute,
                    custom_property_key_value.trim()
                ));
            } else {
                error!(
                    "Custom asset property filter was formatted incorrectly. Skipping... '{custom_property}'"
                );
            }
        }
    }

    if let Some(id_guid) = options.id {
        let id_query = format!("id = '{id_guid}'");
        filters.push(id_query);
    }

    if let Some(manufacturer) = options.manufacturer {
        let manufacturer_name_query = format!("manufacturerName = '{manufacturer}'");
        filters.push(manufacturer_name_query);
    }

    if let Some(product) = options.product {
        let product_name_query = format!("productName CONTAINS '{product}'");
        filters.push(product_name_query);
    }

    let filter_str = filters.join(" AND ");

    if let Some(filter_field) = search_query.get_mut("filter") {
        *filter_field = Value::String(filter_str);
    }

    search_query
}

pub async fn search_assets_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    options: SearchAssetsArgs,
) -> color_eyre::Result<Vec<AssetDto>> {
    let target_url = format!("{}{}", config.instance_url, ASSET_SEARCH_API_PREFIX);
    debug!("Request URL: {target_url}");
    debug!("Options: {options:#?}");

    let all_location_uuid = Uuid::parse_str("11223344-5566-7788-99aa-bbccddeeff00")?;
    let all_location =
        get_raw_asset_by_id_async(config, req, auth_header, &all_location_uuid).await?;
    let all_location_name = all_location
        .get("name")
        .expect("All location did not have a name")
        .as_str()
        .expect("Unable to parse location name as str");

    let search_query = compose_search_query(options.clone(), all_location_name);

    trace!("{}", serde_json::to_string_pretty(&search_query).unwrap());

    let resp = req
        .post(target_url)
        .header(AUTHORIZATION, auth_header.clone())
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .json(&search_query)
        .send()
        .await?
        .json::<Value>()
        .await?;

    let total = resp
        .get("estimatedTotalHits")
        .expect("Expected estimatedTotalHits to be defined in response body.")
        .as_u64()
        .unwrap();

    let limit = resp
        .get("limit")
        .expect("Expected limit to be defined in response body.")
        .as_u64()
        .unwrap();

    info!("Meta Data: | Total: {total} | Limit: {limit} |");

    let mut asset_list = Vec::new();

    if total == 0 {
        return Ok(asset_list);
    }

    if let Some(assets) = resp.get("hits") {
        assets.as_array().unwrap().iter().for_each(|a| {
            debug!("RAW: {}", serde_json::to_string_pretty(&a).unwrap());

            let asset = AssetDto {
                id: Uuid::from_str(a.get("id").unwrap().as_str().unwrap()).unwrap(),
                name: a.get("displayName").unwrap().to_string(),
                asset_lifecycle_state: a.get("assetLifecycleState").unwrap().to_string(),
                asset_type_id: a.get("assetType").unwrap().to_string(),
                manufacturer_id: a.get("manufacturerId").unwrap().to_string(),
                manufacturer_name: a.get("manufacturerName").unwrap().to_string(),
                monitoring_state: a.get("monitoringState").unwrap().to_string(),
                parent_id: a.get("parentId").unwrap().to_string(),
                parent_name: a.get("parentDisplayName").unwrap().to_string(),
                product_id: a.get("productId").unwrap().to_string(),
                product_name: a.get("productName").unwrap().to_string(),
                status: a.get("status").unwrap().to_string(),
                path: a
                    .get("delimitedPath")
                    .unwrap()
                    .to_string()
                    .replace('~', "/"),
                serial_number: a
                    .get("assetProperty_serialNumber")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| serde_json::to_string(arr).ok())
                    .unwrap_or_else(|| "[]".to_string()),
                property: None,
            };

            asset_list.push(asset);
        });
    }

    if let Some(property_type) = options.show_property {
        for a in &mut asset_list {
            let props = get_named_asset_property_async(
                config,
                req,
                auth_header,
                a.id,
                property_type.clone(),
            )
            .await?;

            let prop_values: String = props.iter().fold(String::new(), |mut a, v| {
                let v = format!("{} ", v.value);
                a.push_str(&v);
                a
            });

            a.property = Some(prop_values);
        }
    }

    Ok(asset_list)
}

fn compose_search_query(options: SearchAssetsArgs, all_location_name: &str) -> serde_json::Value {
    let mut search_query = json!({
      "limit": options.limit,
      "offset": options.skip,
      "attributesToRetrieve": [
        "id",
        "displayName",
        "assetLifecycleState",
        "assetType",
        "manufacturerId",
        "manufacturerName",
        "monitoringState",
        "parentId",
        "parentDisplayName",
        "productId",
        "productName",
        "status",
        "delimitedPath",
        "assetProperty_serialNumber"
      ],
      "q": options.search_pattern,
      "filter": "",
    });

    let mut filters = Vec::new();

    if let Some(t) = options.asset_type {
        let asset_type = t.to_string();
        filters.push(format!("assetType = '{asset_type}'"));
    }

    let mut path_filter = format!("delimitedPath STARTS WITH '{all_location_name}~'");
    if let Some(p) = options.location_path {
        let prepared_path = p.replace('/', "~");
        path_filter = format!("delimitedPath STARTS WITH '{prepared_path}'");
    }
    filters.push(path_filter);

    if let Some(properties) = options.properties {
        for property in properties {
            if let Some((property_key_name, property_key_value)) = property.split_once('=') {
                let property_key_attribute = format!("assetProperty_{} ", property_key_name.trim());
                filters.push(format!(
                    "{} = '{}'",
                    property_key_attribute,
                    property_key_value.trim()
                ));
            } else {
                error!("Asset property filter was formatted incorrectly. Skipping... '{property}'");
            }
        }
    }

    if let Some(custom_properties) = options.custom_properties {
        for custom_property in custom_properties {
            if let Some((custom_property_key_name, custom_property_key_value)) =
                custom_property.split_once('=')
            {
                let custom_property_key_attribute =
                    format!("customProperty_{} ", custom_property_key_name.trim());
                filters.push(format!(
                    "{} = '{}'",
                    custom_property_key_attribute,
                    custom_property_key_value.trim()
                ));
            } else {
                error!(
                    "Custom asset property filter was formatted incorrectly. Skipping... '{custom_property}'"
                );
            }
        }
    }

    if let Some(id_guid) = options.id {
        let id_query = format!("id = '{id_guid}'");
        filters.push(id_query);
    }

    if let Some(manufacturer) = options.manufacturer {
        let manufacturer_name_query = format!("manufacturerName = '{manufacturer}'");
        filters.push(manufacturer_name_query);
    }

    if let Some(product) = options.product {
        let product_name_query = format!("productName CONTAINS '{product}'");
        filters.push(product_name_query);
    }

    let filter_str = filters.join(" AND ");

    if let Some(filter_field) = search_query.get_mut("filter") {
        *filter_field = Value::String(filter_str);
    }

    search_query
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hyperview::cli_data::*;

    use httpmock::prelude::*;
    use serde_json::json;
    use std::fs;

    #[test]
    fn test_compose_search_query() {
        let mut query1 = json!({
            "limit": 100,
            "offset": 0,
            "attributesToRetrieve": [
                "id",
                "displayName",
                "assetLifecycleState",
                "assetType",
                "manufacturerId",
                "manufacturerName",
                "monitoringState",
                "parentId",
                "parentDisplayName",
                "productId",
                "productName",
                "status",
                "delimitedPath",
                "assetProperty_serialNumber"
            ],
            "q": "search_pattern",
            "filter": "delimitedPath STARTS WITH 'All~'"
        });

        let mut options = SearchAssetsArgs {
            search_pattern: Some("search_pattern".to_string()),
            asset_type: None,
            location_path: None,
            properties: None,
            custom_properties: None,
            id: None,
            manufacturer: None,
            product: None,
            limit: 100,
            skip: 0,
            filename: None,
            output_type: OutputOptions::Record,
            show_property: None,
        };

        assert_eq!(compose_search_query(options.clone(), "All"), query1);

        // Test with asset type and location set
        let mut filter = Vec::new();

        filter.push(format!("assetType = '{}'", "Server"));

        let input_path = "All/".to_string();
        let prepared_path = input_path.replace('/', "~").to_string();
        filter.push(format!("delimitedPath STARTS WITH '{}'", prepared_path));

        let filter_str = filter.join(" AND ");

        if let Some(filter_field) = query1.get_mut("filter") {
            *filter_field = Value::String(filter_str);
        }

        options.location_path = Some("All/".to_string());
        options.asset_type = Some(AssetTypes::Server);

        assert_eq!(compose_search_query(options, "All"), query1);
    }

    #[tokio::test]
    async fn test_search_assets_async() {
        //Arrange
        let search_resp1 = fs::read_to_string("test_data/search_resp1.json")
            .expect("Unable to open test data file");
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(POST).path(ASSET_SEARCH_API_PREFIX);

            then.status(200)
                .header("Content-Type", "application/json")
                .body(search_resp1);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = reqwest::Client::new();
        let auth_header = "Bearer test_token".to_string();

        let options = SearchAssetsArgs {
            search_pattern: Some("labworker16".to_string()),
            asset_type: None,
            location_path: None,
            properties: None,
            custom_properties: None,
            id: None,
            manufacturer: None,
            product: None,
            limit: 100,
            skip: 0,
            filename: None,
            output_type: OutputOptions::Record,
            show_property: None,
        };
        // Act
        let result = search_assets_async(&config, &client, &auth_header, options).await;

        // Assert
        m.assert();
        assert!(result.is_ok());
        let assets = result.unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].name, "\"labworker16\"".to_string());
        assert_eq!(assets[0].asset_type_id, "\"Server\"".to_string())
    }
}
