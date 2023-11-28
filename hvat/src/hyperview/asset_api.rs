use color_eyre::Result;
use log::{debug, info, trace};
use reqwest::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde_json::{json, Value};

use crate::hyperview::{
    api_constants::{ASSET_API_PREFIX, ASSET_SEARCH_API_PREFIX},
    asset_api_data::AssetDto,
    cli_data::AppConfig,
};

use super::cli_data::SearchAssetsArgs;

pub async fn get_asset_list_async(
    config: &AppConfig,
    req: Client,
    auth_header: String,
    query: Vec<(String, String)>,
) -> Result<Vec<AssetDto>> {
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

    if let Some(metadata) = &resp.get("_metadata") {
        let total = metadata["total"].as_u64().unwrap();
        let limit = metadata["limit"].as_u64().unwrap();
        info!("Meta Data: | Total: {} | Limit: {} |", total, limit);
    }

    let mut asset_list = Vec::new();

    if let Some(assets) = resp.get("data") {
        assets.as_array().unwrap().iter().for_each(|a| {
            debug!("RAW: {}", serde_json::to_string_pretty(&a).unwrap());

            let asset = AssetDto {
                id: a.get("id").unwrap().to_string(),
                name: a.get("name").unwrap().to_string(),
                asset_lifecycle_state: a.get("assetLifecycleState").unwrap().to_string(),
                asset_type_id: a.get("assetTypeId").unwrap().to_string(),
                manufacturer_id: a.get("manufacturerId").unwrap().to_string(),
                manufacturer_name: a.get("manufacturerName").unwrap().to_string(),
                monitoring_state: a.get("monitoringState").unwrap().to_string(),
                parent_id: a.get("parentId").unwrap().to_string(),
                parent_name: a.get("parentName").unwrap().to_string(),
                product_id: a.get("productId").unwrap().to_string(),
                product_name: a.get("productName").unwrap().to_string(),
                status: a.get("status").unwrap().to_string(),
                path: a
                    .get("tabDelimitedPath")
                    .unwrap()
                    .to_string()
                    .replace("\\t", "/"),
            };

            asset_list.push(asset);
        });
    };

    Ok(asset_list)
}

pub async fn get_asset_by_id_async(
    config: &AppConfig,
    req: Client,
    auth_header: String,
    id: String,
) -> Result<AssetDto> {
    // format the target URL
    let target_url = format!("{}{}/{}", config.instance_url, ASSET_API_PREFIX, id);
    debug!("Request URL: {:?}", target_url);

    let resp = req
        .get(target_url)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await?
        .json::<Value>()
        .await?;

    debug!("RAW: {}", serde_json::to_string_pretty(&resp).unwrap());

    let asset = AssetDto {
        id: resp.get("id").unwrap().to_string(),
        name: resp.get("name").unwrap().to_string(),
        asset_lifecycle_state: resp.get("assetLifecycleState").unwrap().to_string(),
        asset_type_id: resp.get("assetTypeId").unwrap().to_string(),
        manufacturer_id: resp.get("manufacturerId").unwrap().to_string(),
        manufacturer_name: resp.get("manufacturerName").unwrap().to_string(),
        monitoring_state: resp.get("monitoringState").unwrap().to_string(),
        parent_id: resp.get("parentId").unwrap().to_string(),
        parent_name: resp.get("parentName").unwrap().to_string(),
        product_id: resp.get("productId").unwrap().to_string(),
        product_name: resp.get("productName").unwrap().to_string(),
        status: resp.get("status").unwrap().to_string(),
        path: resp
            .get("tabDelimitedPath")
            .unwrap()
            .to_string()
            .replace("\\t", "/"),
    };

    Ok(asset)
}

pub async fn search_assets_async(
    config: &AppConfig,
    req: Client,
    auth_header: String,
    options: SearchAssetsArgs,
) -> Result<Vec<AssetDto>> {
    // format the target URL
    let target_url = format!("{}{}", config.instance_url, ASSET_SEARCH_API_PREFIX);
    debug!("Request URL: {:?}", target_url);
    debug!("Options: {:#?}", options);

    let search_query = compose_search_query(
        options.search_pattern,
        options.limit,
        options.skip,
        options.asset_type,
        options.location_path,
        options.properties,
        options.custom_properties,
    );

    trace!("{}", serde_json::to_string_pretty(&search_query).unwrap());

    let resp = req
        .post(target_url)
        .header(AUTHORIZATION, auth_header)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .json(&search_query)
        .send()
        .await?
        .json::<Value>()
        .await?;

    if let Some(metadata) = &resp.get("_metadata") {
        let total = metadata["total"].as_u64().unwrap();
        let limit = metadata["limit"].as_u64().unwrap();
        info!("Meta Data: | Total: {} | Limit: {} |", total, limit);
    }

    let mut asset_list = Vec::new();

    if let Some(assets) = resp.get("data") {
        assets.as_array().unwrap().iter().for_each(|a| {
            debug!("RAW: {}", serde_json::to_string_pretty(&a).unwrap());

            let asset = AssetDto {
                id: a.get("id").unwrap().to_string(),
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
                    .get("tabDelimitedPath")
                    .unwrap()
                    .to_string()
                    .replace("\\t", "/"),
            };

            asset_list.push(asset);
        });
    };

    Ok(asset_list)
}

fn compose_search_query(
    search_pattern: String,
    limit: u32,
    skip: u32,
    asset_type: Option<String>,
    location_path: Option<String>,
    properties: Option<Vec<String>>,
    custom_properties: Option<Vec<String>>,
) -> Value {
    let mut search_query = json!({
      "size": limit,
      "from": skip,
      "query": {
        "bool": {
          "filter": {
            "bool": {
              "must": []
            }
          },
          "must": [],
          "should": [
            {
              "query_string": {
                "query": format!("{}",search_pattern),
                "fields": [
                  "displayNameLowerCase^5",
                  "*"
                ]
              }
            },
            {
              "nested": {
                "path": "componentAssets",
                "query": {
                  "query_string": {
                    "query": format!("{}",search_pattern),
                    "fields": [
                      "componentAssets.displayName"
                    ]
                  }
                }
              }
            },
            {
              "nested": {
                "path": "stringCustomProperties",
                "query": {
                  "query_string": {
                    "query": format!("{}",search_pattern),
                    "fields": [
                      "stringCustomProperties.name",
                      "stringCustomProperties.value"
                    ]
                  }
                }
              }
            },
            {
              "nested": {
                "path": "dateTimeCustomProperties",
                "query": {
                  "query_string": {
                    "query": format!("{}",search_pattern),
                    "fields": [
                      "dateTimeCustomProperties.name",
                      "dateTimeCustomProperties.searchableValue"
                    ]
                  }
                }
              }
            },
            {
              "nested": {
                "path": "numericCustomProperties",
                "query": {
                  "query_string": {
                    "query": format!("{}",search_pattern),
                    "fields": [
                      "numericCustomProperties.name",
                      "numericCustomProperties.searchableValue"
                    ]
                  }
                }
              }
            },
            {
              "nested": {
                "path": "stringSensors",
                "query": {
                  "query_string": {
                    "query": format!("{}",search_pattern),
                    "fields": [
                      "stringSensors.value"
                    ]
                  }
                }
              }
            },
            {
              "nested": {
                "path": "numericSensors",
                "query": {
                  "query_string": {
                    "query": format!("{}",search_pattern),
                    "fields": [
                      "numericSensors.searchableValue"
                    ]
                  }
                }
              }
            }
          ],
          "minimum_should_match": 1
        }
      }
    });

    if let Some(t) = asset_type {
        let filter = json!({ "match": { "assetType": t } });

        search_query["query"]["bool"]["filter"]["bool"]["must"]
            .as_array_mut()
            .unwrap()
            .push(filter);
    }

    if let Some(p) = location_path {
        let prepared_path = format!("{}*", p.replace('/', "\t"));
        let path = json!({ "wildcard": { "tabDelimitedPath": prepared_path } });

        search_query["query"]["bool"]["filter"]["bool"]["must"]
            .as_array_mut()
            .unwrap()
            .push(path);
    }

    if let Some(props) = properties {
        let kv: Vec<(String, String)> = props
            .iter()
            .filter_map(|x| {
                let mut s = x.splitn(2, '=');
                match (s.next(), s.next()) {
                    (Some(k), Some(v)) => Some((k.to_string(), v.to_string())),
                    _ => None,
                }
            })
            .collect();

        kv.iter().for_each(|(k, v)| {
            let subquery = json!({ "match": { k: { "query": v, "lenient": true } } });
            search_query["query"]["bool"]["must"]
                .as_array_mut()
                .unwrap()
                .push(subquery);
        });
    }

    if let Some(props) = custom_properties {
        let kv: Vec<(String, String)> = props
            .iter()
            .filter_map(|x| {
                let mut s = x.splitn(2, '=');
                match (s.next(), s.next()) {
                    (Some(k), Some(v)) => Some((k.to_string(), v.to_string())),
                    _ => None,
                }
            })
            .collect();

        kv.iter().for_each(|(k, v)| {
            let subquery = json!({
              "nested": {
                "path": "stringCustomProperties",
                "inner_hits": {},
                "query": {
                  "bool": {
                    "must": [
                      {
                        "match": {
                          "stringCustomProperties.name": k
                        }
                      },
                      {
                        "match": {
                          "stringCustomProperties.value": {
                            "query": v,
                            "lenient": true
                          }
                        }
                      }
                    ]
                  }
                }
              }
            });

            search_query["query"]["bool"]["must"]
                .as_array_mut()
                .unwrap()
                .push(subquery);
        });
    }

    trace!(
        "search_query:t\n{}",
        serde_json::to_string_pretty(&search_query).unwrap()
    );

    search_query
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_asset_by_id_async() {
        // Arrange
        let asset_id = "3fa85f64-5717-4562-b3fc-2c963f66afa6".to_string();
        let url_path = format!("{}/{}", ASSET_API_PREFIX, asset_id);

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path(url_path);

            then.status(200)
                .header("Content-Type", "application/json")
                .body(
                    json!({
                        "hasChildren": false,
                        "locationData": null,
                        "baseInformationLastUpdated": "2021-10-25T18:17:09.979662+00:00",
                        "accessState": "full",
                        "tabDelimitedPath": "All\tEU\tLoc-001\tTestRack",
                        "accessPolicyId": "eea77bbe-c1fb-464e-841c-bce66ae5beb4",
                        "id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
                        "name": "TestRack",
                        "status": "normal",
                        "assetTypeId": "rack",
                        "assetTypeCategory": "rack",
                        "parentId": "a23f3ec8-89a4-4caa-95b0-0f6f0a77073f",
                        "parentName": "Loc-001",
                        "productId": "3afd7bbb-95e8-4bd0-924a-ccee26ac33bc",
                        "productName": "AR3100",
                        "manufacturerId": "e417483a-20b0-4b86-b0e0-2c2be6592892",
                        "manufacturerName": "APC",
                        "dimension": {},
                        "assetLifecycleState": "active",
                        "discoveryState": "manuallyCreated",
                        "monitoringState": "off",
                        "sensorMonitoringProfileType": "discovered"
                    })
                    .to_string(),
                );
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = reqwest::Client::new();
        let auth_header = "Bearer test_token".to_string();

        // Act
        let result = get_asset_by_id_async(&config, client, auth_header, asset_id.clone()).await;

        // Assert
        m.assert();
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.id, format!("\"{}\"", asset_id));
        assert_eq!(asset.name, "\"TestRack\"");
    }

    #[tokio::test]
    async fn test_get_asset_list_async() {
        // Arrange
        let query = vec![
            ("assetType".to_string(), "RackPdu".to_string()),
            ("(after)".to_string(), 0.to_string()),
            ("(limit)".to_string(), 2.to_string()),
            ("(sort)".to_string(), "+Id".to_string()),
        ];

        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET)
                .path(ASSET_API_PREFIX)
                .query_param("assetType", "RackPdu")
                .query_param("(after)", "0")
                .query_param("(limit)", "2")
                .query_param("(sort)", "+Id");

            then.status(200)
                .header("Content-Type", "application/json")
                .body(
                    json!({
                            "_metadata": {
                            "limit": 2,
                            "offset": 0,
                            "total": 182
                        },
                        "data": [{
                            "hasChildren": false,
                            "locationData": null,
                            "baseInformationLastUpdated": "2021-10-25T18:17:09.979662+00:00",
                            "accessState": "full",
                            "tabDelimitedPath": "All\tEU\tLoc-001\tTestPdu1",
                            "accessPolicyId": "eea77bbe-c1fb-464e-841c-bce66ae5beb4",
                            "id": "08e1c24d-6134-4709-99af-3e7e4b3ef161",
                            "name": "TestPdu1",
                            "status": "normal",
                            "assetTypeId": "rackPdu",
                            "assetTypeCategory": "device",
                            "parentId": "a23f3ec8-89a4-4caa-95b0-0f6f0a77073f",
                            "parentName": "Loc-001",
                            "productId": "0a5efdb2-fd5f-4902-8cdf-0985e50863e8",
                            "productName": "RPC-28",
                            "manufacturerId": "8502393d-e8a0-4cb2-970e-d5dda8fca355",
                            "manufacturerName": "Baytech",
                            "dimension": {},
                            "assetLifecycleState": "active",
                            "discoveryState": "manuallyCreated",
                            "monitoringState": "off",
                            "sensorMonitoringProfileType": "discovered"
                        }, {
                            "hasChildren": false,
                            "locationData": null,
                            "baseInformationLastUpdated": "2021-10-25T18:17:09.979662+00:00",
                            "accessState": "full",
                            "tabDelimitedPath": "All\tEU\tLoc-001\tTestPdu2",
                            "accessPolicyId": "eea77bbe-c1fb-464e-841c-bce66ae5beb4",
                            "id": "09ba0f43-6ca7-48c6-abc1-a2cb1962f626",
                            "name": "TestPdu2",
                            "status": "normal",
                            "assetTypeId": "rackPdu",
                            "assetTypeCategory": "device",
                            "parentId": "a23f3ec8-89a4-4caa-95b0-0f6f0a77073f",
                            "parentName": "Loc-001",
                            "productId": "0a5efdb2-fd5f-4902-8cdf-0985e50863e8",
                            "productName": "RPC-28",
                            "manufacturerId": "8502393d-e8a0-4cb2-970e-d5dda8fca355",
                            "manufacturerName": "Baytech",
                            "dimension": {},
                            "assetLifecycleState": "active",
                            "discoveryState": "manuallyCreated",
                            "monitoringState": "off",
                            "sensorMonitoringProfileType": "discovered"
                    }]})
                    .to_string(),
                );
        });

        let config = AppConfig {
            instance_url: format!("http://{}", server.address()),
            ..Default::default()
        };
        let client = reqwest::Client::new();
        let auth_header = "Bearer test_token".to_string();

        // Act
        let result = get_asset_list_async(&config, client, auth_header, query).await;

        // Assert
        m.assert();
        assert!(result.is_ok());
        let assets = result.unwrap();
        assert_eq!(assets.len(), 2);
        assert_eq!(assets[0].id, "\"08e1c24d-6134-4709-99af-3e7e4b3ef161\"");
        assert_eq!(assets[1].id, "\"09ba0f43-6ca7-48c6-abc1-a2cb1962f626\"");
    }

    #[test]
    fn test_compose_search_query() {
        let mut query1 = json!({
          "size": 100,
          "from": 0,
          "query": {
            "bool": {
              "filter": {
                "bool": {
                  "must": []
                }
              },
              "must": [],
              "should": [
                {
                  "query_string": {
                    "query": format!("{}","search_pattern"),
                    "fields": [
                      "displayNameLowerCase^5",
                      "*"
                    ]
                  }
                },
                {
                  "nested": {
                    "path": "componentAssets",
                    "query": {
                      "query_string": {
                        "query": format!("{}","search_pattern"),
                        "fields": [
                          "componentAssets.displayName"
                        ]
                      }
                    }
                  }
                },
                {
                  "nested": {
                    "path": "stringCustomProperties",
                    "query": {
                      "query_string": {
                        "query": format!("{}","search_pattern"),
                        "fields": [
                          "stringCustomProperties.name",
                          "stringCustomProperties.value"
                        ]
                      }
                    }
                  }
                },
                {
                  "nested": {
                    "path": "dateTimeCustomProperties",
                    "query": {
                      "query_string": {
                        "query": format!("{}","search_pattern"),
                        "fields": [
                          "dateTimeCustomProperties.name",
                          "dateTimeCustomProperties.searchableValue"
                        ]
                      }
                    }
                  }
                },
                {
                  "nested": {
                    "path": "numericCustomProperties",
                    "query": {
                      "query_string": {
                        "query": format!("{}","search_pattern"),
                        "fields": [
                          "numericCustomProperties.name",
                          "numericCustomProperties.searchableValue"
                        ]
                      }
                    }
                  }
                },
                {
                  "nested": {
                    "path": "stringSensors",
                    "query": {
                      "query_string": {
                        "query": format!("{}","search_pattern"),
                        "fields": [
                          "stringSensors.value"
                        ]
                      }
                    }
                  }
                },
                {
                  "nested": {
                    "path": "numericSensors",
                    "query": {
                      "query_string": {
                        "query": format!("{}","search_pattern"),
                        "fields": [
                          "numericSensors.searchableValue"
                        ]
                      }
                    }
                  }
                }
              ],
              "minimum_should_match": 1
            }
          }
        });

        let mut options = SearchAssetsArgs {
            search_pattern: "search_pattern".to_string(),
            asset_type: None,
            location_path: None,
            properties: None,
            custom_properties: None,
            limit: 100,
            skip: 0,
            filename: None,
            output_type: "record".to_string(),
        };

        assert_eq!(
            compose_search_query(
                options.search_pattern.clone(),
                options.limit,
                options.skip,
                options.asset_type,
                options.location_path,
                options.properties.clone(),
                options.custom_properties.clone(),
            ),
            query1
        );

        // Test with asset type and location set

        let filter = json!({ "match": { "assetType": "Server" } });

        query1["query"]["bool"]["filter"]["bool"]["must"]
            .as_array_mut()
            .unwrap()
            .push(filter);

        let input_path = "All/".to_string();
        let prepared_path = format!("{}*", input_path.replace('/', "\t"));
        let path = json!({ "wildcard": { "tabDelimitedPath": prepared_path } });

        query1["query"]["bool"]["filter"]["bool"]["must"]
            .as_array_mut()
            .unwrap()
            .push(path);

        options.location_path = Some("All/".to_string());
        options.asset_type = Some("Server".to_string());

        assert_eq!(
            compose_search_query(
                options.search_pattern.clone(),
                options.limit,
                options.skip,
                options.asset_type,
                options.location_path,
                options.properties,
                options.custom_properties,
            ),
            query1
        );
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
            search_pattern: "labworker16".to_string(),
            asset_type: None,
            location_path: None,
            properties: None,
            custom_properties: None,
            limit: 100,
            skip: 0,
            filename: None,
            output_type: "record".to_string(),
        };
        // Act
        let result = search_assets_async(&config, client, auth_header, options).await;

        // Assert
        m.assert();
        assert!(result.is_ok());
        let assets = result.unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].name, "\"labworker16\"".to_string());
        assert_eq!(assets[0].asset_type_id, "\"Server\"".to_string())
    }
}
