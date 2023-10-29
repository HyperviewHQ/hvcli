use color_eyre::eyre::Result;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, ClientId, ClientSecret, Scope,
    TokenResponse, TokenUrl,
};

use crate::hyperview::cli_data::AppConfig;

pub async fn get_auth_header_async(config: &AppConfig) -> Result<String> {
    // create client
    let client = BasicClient::new(
        ClientId::new(config.client_id.clone()),
        Some(ClientSecret::new(config.client_secret.clone())),
        AuthUrl::new(config.auth_url.clone())?,
        Some(TokenUrl::new(config.token_url.clone())?),
    );

    // fetch token
    let token_response = client
        .exchange_client_credentials()
        .add_scope(Scope::new(config.scope.clone()))
        .request_async(async_http_client)
        .await?;

    Ok(format!("Bearer {}", token_response.access_token().secret()))
}
