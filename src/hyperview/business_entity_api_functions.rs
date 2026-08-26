use log::{debug, info};
use reqwest::{Client, header::AUTHORIZATION};
use serde_json::{Map, Value};
use uuid::Uuid;

use super::{
    api_constants::{
        ASSET_ASSETS_API_PREFIX, BUSINESS_ENTITY_ADDRESS_API_PREFIX, BUSINESS_ENTITY_API_PREFIX,
        BUSINESS_ENTITY_CONTACT_API_PREFIX, BUSINESS_ENTITY_PAGE_SIZE,
    },
    business_entity_api_data::{
        BusinessEntityAddressDto, BusinessEntityAssetListResponse, BusinessEntityChild,
        BusinessEntityContactDto, BusinessEntityDto,
    },
    cli_data::AppConfig,
};

/// Lists business entities. The API has no collection endpoint for business entities, so their ids
/// are discovered through the asset collection (a business entity is an asset of type
/// `businessEntity`) and each entity is then fetched individually. The individual record is the
/// only place the entity type is available.
pub async fn list_business_entities_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    skip: u32,
    limit: u32,
) -> color_eyre::Result<Vec<BusinessEntityDto>> {
    let resp = list_business_entity_assets_async(config, req, auth_header, skip, limit).await?;

    let total = resp.metadata.total;
    info!("Meta Data: | Total: {total} | Limit: {limit} |");

    let mut business_entity_list = Vec::new();

    for asset in &resp.data {
        let business_entity = get_business_entity_async(config, req, auth_header, asset.id).await?;

        business_entity_list.push(business_entity);
    }

    Ok(business_entity_list)
}

/// Lists business entity contacts. With `business_entity_id` set, only that entity's contacts are
/// returned; otherwise every entity's contacts are returned, ordered by business entity name and
/// then by contact name.
pub async fn list_business_entity_contacts_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    business_entity_id: Option<Uuid>,
    skip: u32,
    limit: u32,
) -> color_eyre::Result<Vec<BusinessEntityContactDto>> {
    list_business_entity_children_async(
        config,
        req,
        auth_header,
        BUSINESS_ENTITY_CONTACT_API_PREFIX,
        business_entity_id,
        skip,
        limit,
    )
    .await
}

/// Lists business entity addresses. With `business_entity_id` set, only that entity's addresses are
/// returned; otherwise every entity's addresses are returned, ordered by business entity name and
/// then by address description.
pub async fn list_business_entity_addresses_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    business_entity_id: Option<Uuid>,
    skip: u32,
    limit: u32,
) -> color_eyre::Result<Vec<BusinessEntityAddressDto>> {
    list_business_entity_children_async(
        config,
        req,
        auth_header,
        BUSINESS_ENTITY_ADDRESS_API_PREFIX,
        business_entity_id,
        skip,
        limit,
    )
    .await
}

/// Gathers contacts or addresses across one or every business entity. Both collections are keyed by
/// business entity id and neither can be queried across entities, so listing them all means walking
/// the entity collection and fetching each entity's records.
async fn list_business_entity_children_async<T: BusinessEntityChild>(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    api_prefix: &str,
    business_entity_id: Option<Uuid>,
    skip: u32,
    limit: u32,
) -> color_eyre::Result<Vec<T>> {
    let needed = skip as usize + limit as usize;
    let mut records: Vec<T> = Vec::new();

    if let Some(id) = business_entity_id {
        let business_entity = get_business_entity_async(config, req, auth_header, id).await?;

        records.extend(
            get_business_entity_children_async(
                config,
                req,
                auth_header,
                api_prefix,
                business_entity.id,
                &business_entity.name,
            )
            .await?,
        );
    } else {
        let mut offset = 0;

        loop {
            let page = list_business_entity_assets_async(
                config,
                req,
                auth_header,
                offset,
                BUSINESS_ENTITY_PAGE_SIZE,
            )
            .await?;

            if page.data.is_empty() {
                break;
            }

            for asset in &page.data {
                records.extend(
                    get_business_entity_children_async(
                        config,
                        req,
                        auth_header,
                        api_prefix,
                        asset.id,
                        &asset.name,
                    )
                    .await?,
                );

                offset += 1;

                // Entities arrive in name order and that is the primary sort of the output, so once
                // enough records are in hand no later entity can contribute to the requested page.
                if records.len() >= needed {
                    break;
                }
            }

            if records.len() >= needed || i64::from(offset) >= page.metadata.total {
                break;
            }
        }
    }

    info!("Meta Data: | Records: {} | Limit: {limit} |", records.len());

    Ok(records
        .into_iter()
        .skip(skip as usize)
        .take(limit as usize)
        .collect())
}

/// Fetches one business entity's contacts or addresses, sorted, and stamped with the entity name
/// (neither response carries it).
async fn get_business_entity_children_async<T: BusinessEntityChild>(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    api_prefix: &str,
    business_entity_id: Uuid,
    business_entity_name: &str,
) -> color_eyre::Result<Vec<T>> {
    let target_url = format!(
        "{}{}/{}",
        config.instance_url, api_prefix, business_entity_id
    );
    debug!("Request URL: {target_url}");

    let mut records = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<T>>()
        .await?;

    for record in &mut records {
        record.set_business_entity_name(business_entity_name);
    }

    records.sort_by(|a, b| a.sort_key().cmp(b.sort_key()));

    Ok(records)
}

/// Fetches a page of business entities from the asset collection, sorted by name.
async fn list_business_entity_assets_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    skip: u32,
    limit: u32,
) -> color_eyre::Result<BusinessEntityAssetListResponse> {
    let target_url = format!("{}{}", config.instance_url, ASSET_ASSETS_API_PREFIX);
    debug!("Request URL: {target_url}");

    let mut query_params = Map::new();

    query_params.insert(
        "assetType".to_string(),
        Value::String("businessEntity".to_string()),
    );
    query_params.insert("(after)".to_string(), Value::Number(skip.into()));
    query_params.insert("(limit)".to_string(), Value::Number(limit.into()));
    query_params.insert("(sort)".to_string(), Value::String("+Name".to_string()));

    debug!(
        "Query parameters: {}",
        serde_json::to_string(&query_params).unwrap()
    );

    let resp = req
        .get(target_url)
        .query(&query_params)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .error_for_status()?
        .json::<BusinessEntityAssetListResponse>()
        .await?;

    Ok(resp)
}

async fn get_business_entity_async(
    config: &AppConfig,
    req: &Client,
    auth_header: &String,
    id: Uuid,
) -> color_eyre::Result<BusinessEntityDto> {
    let target_url = format!(
        "{}{}/{}",
        config.instance_url, BUSINESS_ENTITY_API_PREFIX, id
    );
    debug!("Request URL: {target_url}");

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .error_for_status()?
        .json::<BusinessEntityDto>()
        .await?;

    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;
    use uuid::Uuid;

    fn asset_list_body(entities: &[(Uuid, &str)]) -> Value {
        json!({
            "_metadata": { "limit": 100, "offset": 0, "total": entities.len() },
            "data": entities.iter().map(|(id, name)| json!({
                "id": id.to_string(),
                "name": name,
                "status": "normal",
                "assetTypeId": "businessEntity"
            })).collect::<Vec<Value>>()
        })
    }

    fn contact_body(id: Uuid, parent_id: Uuid, name: &str) -> Value {
        json!({
            "id": id.to_string(),
            "parentId": parent_id.to_string(),
            "name": name,
            "phoneNumberOne": "555-0100",
            "phoneNumberTwo": null,
            "emailAddress": "contact@example.com",
            "note": null
        })
    }

    fn test_config(server: &MockServer) -> AppConfig {
        AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_list_business_entities_async_returns_entities() {
        let first_id = Uuid::new_v4();
        let second_id = Uuid::new_v4();
        let access_policy_id = Uuid::new_v4();
        let server = MockServer::start();

        let list_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/api/asset/assets")
                .query_param("assetType", "businessEntity")
                .query_param("(after)", "10")
                .query_param("(limit)", "25")
                .query_param("(sort)", "+Name");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(asset_list_body(&[
                    (first_id, "Test customer 1"),
                    (second_id, "Test vendor 1"),
                ]));
        });

        let first_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/asset/businessEntities/{first_id}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "id": first_id.to_string(),
                    "name": "Test customer 1",
                    "businessEntityTypeValue": "customer",
                    "businessEntityTypeValueId": Uuid::new_v4().to_string(),
                    "accessPolicyId": access_policy_id.to_string()
                }));
        });

        let second_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/asset/businessEntities/{second_id}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "id": second_id.to_string(),
                    "name": "Test vendor 1",
                    "businessEntityTypeValue": "vendor",
                    "businessEntityTypeValueId": Uuid::new_v4().to_string(),
                    "accessPolicyId": access_policy_id.to_string()
                }));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let resp = list_business_entities_async(&config, &client, &auth_header, 10, 25)
            .await
            .unwrap();

        list_mock.assert();
        first_mock.assert();
        second_mock.assert();
        assert_eq!(resp.len(), 2);
        assert_eq!(resp[0].id, first_id);
        assert_eq!(resp[0].name, "Test customer 1");
        assert_eq!(resp[0].business_entity_type_value, "customer");
        assert_eq!(resp[0].access_policy_id, access_policy_id);
        assert_eq!(resp[1].business_entity_type_value, "vendor");
    }

    #[tokio::test]
    async fn test_list_business_entities_async_tolerates_null_type_value() {
        // An entity with no type value must not fail the whole list.
        let id = Uuid::new_v4();
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(GET).path("/api/asset/assets");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(asset_list_body(&[(id, "Untyped")]));
        });

        let entity_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/asset/businessEntities/{id}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "id": id.to_string(),
                    "name": "Untyped",
                    "businessEntityTypeValue": null,
                    "accessPolicyId": Uuid::new_v4().to_string()
                }));
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let resp = list_business_entities_async(&config, &client, &auth_header, 0, 100)
            .await
            .unwrap();

        entity_mock.assert();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].business_entity_type_value, "");
    }

    #[tokio::test]
    async fn test_list_business_entities_async_errors_when_asset_list_fails() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(GET).path("/api/asset/assets");
            then.status(500);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let result = list_business_entities_async(&config, &client, &auth_header, 0, 100).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_business_entities_async_errors_when_entity_fetch_fails() {
        let id = Uuid::new_v4();
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(GET).path("/api/asset/assets");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(asset_list_body(&[(id, "Untyped")]));
        });

        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/asset/businessEntities/{id}"));
            then.status(500);
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let result = list_business_entities_async(&config, &client, &auth_header, 0, 100).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_business_entity_contacts_async_walks_every_entity() {
        // Contacts are keyed by business entity and cannot be queried across entities, so with no
        // id the entity collection is walked and each entity's contacts are fetched and stamped
        // with that entity's name.
        let alpha_id = Uuid::new_v4();
        let beta_id = Uuid::new_v4();
        let server = MockServer::start();

        let list_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/api/asset/assets")
                .query_param("assetType", "businessEntity")
                .query_param("(limit)", "100")
                .query_param("(sort)", "+Name");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(asset_list_body(&[(alpha_id, "Alpha"), (beta_id, "Beta")]));
        });

        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/asset/businessEntityContacts/{alpha_id}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([
                    contact_body(Uuid::new_v4(), alpha_id, "Zoe"),
                    contact_body(Uuid::new_v4(), alpha_id, "Adam")
                ]));
        });

        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/asset/businessEntityContacts/{beta_id}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([contact_body(Uuid::new_v4(), beta_id, "Carl")]));
        });

        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let resp = list_business_entity_contacts_async(
            &test_config(&server),
            &client,
            &auth_header,
            None,
            0,
            100,
        )
        .await
        .unwrap();

        list_mock.assert();
        // Ordered by business entity name, then by contact name within each entity.
        let rows: Vec<(&str, &str)> = resp
            .iter()
            .map(|c| (c.business_entity_name.as_str(), c.name.as_str()))
            .collect();
        assert_eq!(
            rows,
            vec![("Alpha", "Adam"), ("Alpha", "Zoe"), ("Beta", "Carl")]
        );
        assert_eq!(resp[0].parent_id, alpha_id);
        assert_eq!(resp[0].email_address, "contact@example.com");
        // Nullable fields come back as empty strings rather than failing the row.
        assert_eq!(resp[0].phone_number_two, "");
        assert_eq!(resp[0].note, "");
    }

    #[tokio::test]
    async fn test_list_business_entity_contacts_async_scoped_to_one_entity() {
        let entity_id = Uuid::new_v4();
        let server = MockServer::start();

        let list_mock = server.mock(|when, then| {
            when.method(GET).path("/api/asset/assets");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(asset_list_body(&[(entity_id, "Alpha")]));
        });

        let entity_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/asset/businessEntities/{entity_id}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "id": entity_id.to_string(),
                    "name": "Alpha",
                    "businessEntityTypeValue": "customer",
                    "accessPolicyId": Uuid::new_v4().to_string()
                }));
        });

        let contacts_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/asset/businessEntityContacts/{entity_id}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([contact_body(Uuid::new_v4(), entity_id, "Adam")]));
        });

        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let resp = list_business_entity_contacts_async(
            &test_config(&server),
            &client,
            &auth_header,
            Some(entity_id),
            0,
            100,
        )
        .await
        .unwrap();

        entity_mock.assert();
        contacts_mock.assert();
        // Scoping to one entity must not walk the entity collection.
        assert_eq!(list_mock.calls(), 0);
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].business_entity_name, "Alpha");
    }

    #[tokio::test]
    async fn test_list_business_entity_contacts_async_applies_skip_and_limit() {
        let alpha_id = Uuid::new_v4();
        let beta_id = Uuid::new_v4();
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(GET).path("/api/asset/assets");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(asset_list_body(&[(alpha_id, "Alpha"), (beta_id, "Beta")]));
        });

        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/asset/businessEntityContacts/{alpha_id}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([
                    contact_body(Uuid::new_v4(), alpha_id, "Adam"),
                    contact_body(Uuid::new_v4(), alpha_id, "Zoe")
                ]));
        });

        let beta_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/asset/businessEntityContacts/{beta_id}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([contact_body(Uuid::new_v4(), beta_id, "Carl")]));
        });

        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let resp = list_business_entity_contacts_async(
            &test_config(&server),
            &client,
            &auth_header,
            None,
            1,
            1,
        )
        .await
        .unwrap();

        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].name, "Zoe");
        // Alpha alone covers the requested page, so Beta is never fetched.
        assert_eq!(beta_mock.calls(), 0);
    }

    #[tokio::test]
    async fn test_list_business_entity_addresses_async_returns_addresses() {
        let entity_id = Uuid::new_v4();
        let address_id = Uuid::new_v4();
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(GET).path("/api/asset/assets");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(asset_list_body(&[(entity_id, "Dell")]));
        });

        let addresses_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/asset/businessEntityAddresses/{entity_id}"));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!([
                    {
                        "id": address_id.to_string(),
                        "parentId": entity_id.to_string(),
                        "description": "Office",
                        "streetAddressValue": "155 Gordon Baker Road, North York, ON, Canada",
                        "streetAddressTypeValue": "headquarters",
                        "streetAddressTypeValueId": Uuid::new_v4().to_string()
                    },
                    // The readable type is absent from the published schema, so a response without
                    // it must still deserialize.
                    {
                        "id": Uuid::new_v4().to_string(),
                        "parentId": entity_id.to_string(),
                        "description": "Annex",
                        "streetAddressValue": null,
                        "streetAddressTypeValueId": Uuid::new_v4().to_string()
                    }
                ]));
        });

        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let resp = list_business_entity_addresses_async(
            &test_config(&server),
            &client,
            &auth_header,
            None,
            0,
            100,
        )
        .await
        .unwrap();

        addresses_mock.assert();
        assert_eq!(resp.len(), 2);
        assert_eq!(resp[0].description, "Annex");
        assert_eq!(resp[0].business_entity_name, "Dell");
        assert_eq!(resp[0].street_address_value, "");
        assert_eq!(resp[0].street_address_type_value, "");
        assert_eq!(resp[1].description, "Office");
        assert_eq!(resp[1].street_address_type_value, "headquarters");
    }

    #[tokio::test]
    async fn test_list_business_entity_contacts_async_errors_when_contact_fetch_fails() {
        let entity_id = Uuid::new_v4();
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(GET).path("/api/asset/assets");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(asset_list_body(&[(entity_id, "Alpha")]));
        });

        server.mock(|when, then| {
            when.method(GET)
                .path(format!("/api/asset/businessEntityContacts/{entity_id}"));
            then.status(500);
        });

        let client = Client::new();
        let auth_header = "Bearer test_token".to_string();

        let result = list_business_entity_contacts_async(
            &test_config(&server),
            &client,
            &auth_header,
            None,
            0,
            100,
        )
        .await;

        assert!(result.is_err());
    }
}
