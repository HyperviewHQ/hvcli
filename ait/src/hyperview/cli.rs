use std::path::MAIN_SEPARATOR_STR;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AppConfig {
    pub client_id: String,
    pub client_secret: String,
    pub scope: String,
    pub auth_url: String,
    pub token_url: String,
    pub instance_url: String,
}

pub fn get_config_path() -> String {
    let home_path = dirs::home_dir().expect("Error: Home directory not found");

    format!(
        "{}{}.hyperview{}hyperview.toml",
        home_path.to_str().unwrap(),
        MAIN_SEPARATOR_STR,
        MAIN_SEPARATOR_STR
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_config_path() {
        let config_path = get_config_path();
        let home_path = dirs::home_dir().unwrap();
        let expected_path = format!(
            "{}{}.hyperview{}hyperview.toml",
            home_path.to_str().unwrap(),
            MAIN_SEPARATOR_STR,
            MAIN_SEPARATOR_STR
        );

        assert_eq!(config_path, expected_path);
    }

    #[test]
    fn test_app_config_loading() {
        let mut tmp_file = NamedTempFile::new().unwrap();

        write!(
            tmp_file,
            r#"client_id = "test_id"
            client_secret = "test_secret"
            scope = "test_scope"
            auth_url = "https://test_auth_url"
            token_url = "https://test_token_url"
            instance_url = "https://test_instance_url"
            "#
        )
        .unwrap();

        let config: AppConfig =
            confy::load_path(tmp_file.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(config.client_id, "test_id");
        assert_eq!(config.client_secret, "test_secret");
        assert_eq!(config.scope, "test_scope");
        assert_eq!(config.auth_url, "https://test_auth_url");
        assert_eq!(config.token_url, "https://test_token_url");
        assert_eq!(config.instance_url, "https://test_instance_url");
    }
}
