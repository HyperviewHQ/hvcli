use std::path::MAIN_SEPARATOR_STR;

use clap::{value_parser, Args, Parser, Subcommand};
use log::LevelFilter;
use serde::{Deserialize, Serialize};

use crate::ASSET_TYPES;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AppConfig {
    pub client_id: String,
    pub client_secret: String,
    pub scope: String,
    pub auth_url: String,
    pub token_url: String,
    pub instance_url: String,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct AppArgs {
    #[arg(
        short = 'd',
        long,
        help = "Debug level",
        default_value = "error",
        value_parser(["error", "warn", "debug", "info", "trace"])
    )]
    pub debug_level: String,

    #[command(subcommand)]
    pub command: AppArgsSubcommands,
}

#[derive(Subcommand)]
pub enum AppArgsSubcommands {
    /// List assets for a standard type
    ListAssets(ListAssetsArgs),

    /// List asset properties
    ListAssetProperties,
}

#[derive(Args, Debug)]
pub struct ListAssetsArgs {
    #[arg(
        short = 't',
        long,
        help = "Asset type. e.g. Crah",
        value_parser(ASSET_TYPES)
    )]
    pub asset_type: String,

    #[arg(
        short,
        long,
        help = "Offset number (0 -> 99999). e.g. 100", 
        default_value = "0", value_parser(value_parser!(u32).range(0..100000))
    )]
    pub offset: u32,

    #[arg(
        short,
        long,
        help = "Record limit (1 -> 1000). e.g. 100", 
        default_value = "100", 
        value_parser(value_parser!(u32).range(1..1001))
    )]
    pub limit: u32,
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

pub fn get_debug_filter(debug_level: &str) -> LevelFilter {
    match debug_level {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    }
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

        let config: AppConfig = confy::load_path(tmp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.client_id, "test_id");
        assert_eq!(config.client_secret, "test_secret");
        assert_eq!(config.scope, "test_scope");
        assert_eq!(config.auth_url, "https://test_auth_url");
        assert_eq!(config.token_url, "https://test_token_url");
        assert_eq!(config.instance_url, "https://test_instance_url");
    }

    #[test]
    fn test_get_debug_filter() {
        assert_eq!(get_debug_filter("error"), LevelFilter::Error);
        assert_eq!(get_debug_filter("warn"), LevelFilter::Warn);
        assert_eq!(get_debug_filter("debug"), LevelFilter::Debug);
        assert_eq!(get_debug_filter("info"), LevelFilter::Info);
        assert_eq!(get_debug_filter("trace"), LevelFilter::Trace);
        assert_eq!(get_debug_filter("unknown"), LevelFilter::Info);
    }
}
