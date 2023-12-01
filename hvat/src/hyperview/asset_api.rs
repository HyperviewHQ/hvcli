use color_eyre::Result;
use log::{debug, info, trace};
use reqwest::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde_json::{json, Value};

use crate::hyperview::{
    api_constants::ASSET_SEARCH_API_PREFIX,
    asset_api_data::AssetDto,
    cli_data::{AppConfig, SearchAssetsArgs},
};

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

    let search_query = compose_search_query(options);

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
                serial_number: a.get("serialNumber").unwrap().to_string(),
            };

            asset_list.push(asset);
        });
    };

    Ok(asset_list)
}

fn compose_search_query(
    options: SearchAssetsArgs
) -> Value {
    let mut search_query = json!({
      "size": options.limit,
      "from": options.skip,
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
                "query": format!("{}", options.search_pattern),
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
                    "query": format!("{}", options.search_pattern),
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
                    "query": format!("{}", options.search_pattern),
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
                    "query": format!("{}", options.search_pattern),
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
                    "query": format!("{}", options.search_pattern),
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
                    "query": format!("{}", options.search_pattern),
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
                    "query": format!("{}", options.search_pattern),
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

    if let Some(t) = options.asset_type {
        let filter = json!({ "match": { "assetType": t.to_string() } });

        search_query["query"]["bool"]["filter"]["bool"]["must"]
            .as_array_mut()
            .unwrap()
            .push(filter);
    }

    if let Some(p) = options.location_path {
        let prepared_path = format!("{}*", p.replace('/', "\t"));
        let path = json!({ "wildcard": { "tabDelimitedPath": prepared_path } });

        search_query["query"]["bool"]["filter"]["bool"]["must"]
            .as_array_mut()
            .unwrap()
            .push(path);
    }

    if let Some(props) = options.properties {
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

    if let Some(props) = options.custom_properties {
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

    if let Some(id_guid) = options.id {
        let subquery = json!({ "match": { "id": { "query": id_guid, "lenient": true } } });
        search_query["query"]["bool"]["must"]
            .as_array_mut()
            .unwrap()
            .push(subquery);
    }

    trace!(
        "search_query:t\n{}",
        serde_json::to_string_pretty(&search_query).unwrap()
    );

    search_query
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;
    use std::fs;

    use crate::hyperview::cli_data::{OutputOptions, AssetTypes};

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
            id: None,
            limit: 100,
            skip: 0,
            filename: None,
            output_type: OutputOptions::Record,
        };

        assert_eq!(
            compose_search_query(options.clone()),
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
        options.asset_type = Some(AssetTypes::Server);

        assert_eq!(
            compose_search_query(options),
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
            id: None,
            limit: 100,
            skip: 0,
            filename: None,
            output_type: OutputOptions::Record,
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
