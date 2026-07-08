use std::time::{Duration, Instant};

use log::debug;
use oauth2::{AuthUrl, ClientId, ClientSecret, Scope, TokenResponse, TokenUrl, basic::BasicClient};
use reqwest::{ClientBuilder, redirect};

use super::cli_data::AppConfig;

/// Refresh ahead of the token's reported expiry so a request is never sent right on the edge.
const REFRESH_MARGIN: Duration = Duration::from_mins(1);
/// Lifetime to assume when the token endpoint does not report `expires_in`.
const DEFAULT_TOKEN_LIFETIME: Duration = Duration::from_mins(5);

pub struct AuthToken {
    pub header: String,
    expires_at: Instant,
}

impl AuthToken {
    pub async fn fetch_async(config: &AppConfig) -> color_eyre::Result<Self> {
        let auth_client = BasicClient::new(ClientId::new(config.client_id.clone()))
            .set_client_secret(ClientSecret::new(config.client_secret.clone()))
            .set_auth_uri(AuthUrl::new(config.auth_url.clone())?)
            .set_token_uri(TokenUrl::new(config.token_url.clone())?);

        let http_client = ClientBuilder::new()
            .redirect(redirect::Policy::none())
            .build()?;

        let token_response = auth_client
            .exchange_client_credentials()
            .add_scope(Scope::new(config.scope.clone()))
            .request_async(&http_client)
            .await?;

        let lifetime = token_response
            .expires_in()
            .unwrap_or(DEFAULT_TOKEN_LIFETIME)
            .saturating_sub(REFRESH_MARGIN);

        Ok(Self {
            header: format!("Bearer {}", token_response.access_token().secret()),
            expires_at: Instant::now() + lifetime,
        })
    }

    /// Re-fetches the token if it is at or past its expiry, replacing `self` in place.
    pub async fn refresh_if_needed_async(&mut self, config: &AppConfig) -> color_eyre::Result<()> {
        if Instant::now() >= self.expires_at {
            debug!("Auth token nearing expiry; fetching a new one");
            *self = Self::fetch_async(config).await?;
        }

        Ok(())
    }

    /// Unconditionally fetches a new token, replacing `self` in place. Used as a fallback when
    /// the server rejects a request as unauthorized despite the token appearing unexpired.
    pub async fn force_refresh_async(&mut self, config: &AppConfig) -> color_eyre::Result<()> {
        *self = Self::fetch_async(config).await?;

        Ok(())
    }

    /// Test-only constructor: builds a token with the given header that does not expire for
    /// `lifetime`. Lets sibling modules' tests construct an `AuthToken` without touching the
    /// private `expires_at` field.
    #[cfg(test)]
    pub fn for_test(header: impl Into<String>, lifetime: Duration) -> Self {
        Self {
            header: header.into(),
            expires_at: Instant::now() + lifetime,
        }
    }
}

/// True if `err` wraps a `reqwest::Error` carrying a `401 Unauthorized` response status.
pub fn is_unauthorized(err: &color_eyre::Report) -> bool {
    err.downcast_ref::<reqwest::Error>()
        .and_then(reqwest::Error::status)
        .is_some_and(|status| status == reqwest::StatusCode::UNAUTHORIZED)
}

/// Runs `$call`, an already-awaited `color_eyre::Result<_>` expression; if it failed with a
/// 401 Unauthorized, force-refreshes `$auth_token` and re-evaluates `$call` exactly once before
/// giving up. Proactive expiry tracking (see `refresh_if_needed_async`) handles the common case;
/// this is the fallback for clock skew or server-side token revocation.
///
/// # Contract
///
/// On a 401 the **entire** `$call` expression re-runs from the top (it is spliced into the
/// expansion twice — once per attempt), so it must be safe to run again:
///
/// - **Idempotency:** `$call` may contain at most one non-idempotent request (e.g. a creating
///   `POST`), and it must be the *last* request in the operation. Otherwise a token that dies
///   between an earlier write and a later 401 would replay that earlier write on retry — e.g.
///   creating a duplicate resource. An operation that needs a non-idempotent request followed by
///   further requests should retry at the individual-request level rather than wrap the whole
///   operation here.
/// - **Re-evaluable expression:** because `$call` is written twice, it must not move owned values
///   (clone any owned arguments it passes).
/// - **Caller returns `Result`/`Option`:** the expansion uses `?` to propagate a refresh failure,
///   so the surrounding function must accept `?`.
#[macro_export]
macro_rules! retry_on_unauthorized_async {
    ($config:expr, $auth_token:expr, $call:expr) => {{
        match $call {
            Err(e) if $crate::hyperview::auth::is_unauthorized(&e) => {
                log::debug!("Request was unauthorized; refreshing token and retrying once");
                $auth_token.force_refresh_async($config).await?;
                $call
            }
            result => result,
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;

    fn test_config(token_url: String) -> AppConfig {
        AppConfig {
            client_id: "client_id".to_string(),
            client_secret: "client_secret".to_string(),
            scope: "scope".to_string(),
            auth_url: "https://example.test/auth".to_string(),
            token_url,
            instance_url: "https://example.test".to_string(),
        }
    }

    #[tokio::test]
    async fn test_fetch_async_sets_header_and_future_expiry() {
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "access_token": "test_access_token",
                    "token_type": "bearer",
                    "expires_in": 3600
                }));
        });

        let config = test_config(format!("http://{}/token", server.address()));

        let token = AuthToken::fetch_async(&config).await.unwrap();

        m.assert();
        assert_eq!(token.header, "Bearer test_access_token");
        assert!(token.expires_at > Instant::now());
    }

    #[tokio::test]
    async fn test_refresh_if_needed_skips_refresh_when_not_expired() {
        let mut token = AuthToken {
            header: "Bearer still_valid".to_string(),
            expires_at: Instant::now() + Duration::from_mins(1),
        };

        // No mock server is set up; a refresh attempt here would fail to connect.
        let config = test_config("http://127.0.0.1:0/token".to_string());

        token.refresh_if_needed_async(&config).await.unwrap();

        assert_eq!(token.header, "Bearer still_valid");
    }

    #[tokio::test]
    async fn test_refresh_if_needed_refreshes_when_expired() {
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "access_token": "refreshed_access_token",
                    "token_type": "bearer",
                    "expires_in": 3600
                }));
        });

        let config = test_config(format!("http://{}/token", server.address()));

        let mut token = AuthToken {
            header: "Bearer expired".to_string(),
            expires_at: Instant::now().checked_sub(Duration::from_secs(1)).unwrap(),
        };

        token.refresh_if_needed_async(&config).await.unwrap();

        m.assert();
        assert_eq!(token.header, "Bearer refreshed_access_token");
    }

    async fn fetch_resource_async(
        client: &reqwest::Client,
        url: &str,
        auth_header: &str,
    ) -> color_eyre::Result<()> {
        client
            .get(url)
            .header(reqwest::header::AUTHORIZATION, auth_header)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    #[tokio::test]
    async fn test_is_unauthorized_true_for_401_response() {
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(GET).path("/resource");
            then.status(401);
        });

        let client = reqwest::Client::new();
        let err = fetch_resource_async(
            &client,
            &format!("http://{}/resource", server.address()),
            "Bearer anything",
        )
        .await
        .unwrap_err();

        m.assert();
        assert!(is_unauthorized(&err));
    }

    #[test]
    fn test_is_unauthorized_false_for_non_http_error() {
        let err = color_eyre::eyre::eyre!("some other failure");
        assert!(!is_unauthorized(&err));
    }

    #[tokio::test]
    async fn test_retry_on_unauthorized_async_refreshes_token_and_retries() -> color_eyre::Result<()>
    {
        let api_server = MockServer::start();
        let unauthorized_mock = api_server.mock(|when, then| {
            when.method(GET)
                .path("/resource")
                .header("Authorization", "Bearer stale_token");
            then.status(401);
        });
        let authorized_mock = api_server.mock(|when, then| {
            when.method(GET)
                .path("/resource")
                .header("Authorization", "Bearer fresh_token");
            then.status(200);
        });

        let auth_server = MockServer::start();
        let auth_mock = auth_server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body(json!({
                    "access_token": "fresh_token",
                    "token_type": "bearer",
                    "expires_in": 3600
                }));
        });

        let config = test_config(format!("http://{}/token", auth_server.address()));
        let mut auth_token = AuthToken {
            header: "Bearer stale_token".to_string(),
            expires_at: Instant::now() + Duration::from_mins(5),
        };

        let client = reqwest::Client::new();
        let url = format!("http://{}/resource", api_server.address());

        crate::retry_on_unauthorized_async!(
            &config,
            auth_token,
            fetch_resource_async(&client, &url, &auth_token.header).await
        )?;

        unauthorized_mock.assert();
        authorized_mock.assert();
        auth_mock.assert();
        assert_eq!(auth_token.header, "Bearer fresh_token");

        Ok(())
    }
}
