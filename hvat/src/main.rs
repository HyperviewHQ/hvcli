use clap::Parser;
use color_eyre::eyre::Result;
use hyperview::cli::{get_debug_filter, AppArgs};
use log::{info, trace};
use reqwest::Client;

use crate::hyperview::{
    api_constants::ASSET_TYPES,
    asset_api::get_asset_list_async,
    auth::get_auth_header_async,
    cli::{get_config_path, AppArgsSubcommands, AppConfig},
};

mod hyperview;

#[tokio::main]
async fn main() -> Result<()> {
    let args = AppArgs::parse();
    let debug_level = args.debug_level;
    let level_filter = get_debug_filter(&debug_level);
    env_logger::builder().filter(None, level_filter).init();

    info!("Starting hyperview asset import tool");
    info!("Startup options: | Debug Level: {debug_level} |");

    let config: AppConfig = confy::load_path(get_config_path())?;
    let auth_header = get_auth_header_async(&config).await?;
    trace!("Authorization Header: {}", auth_header);
    let req = Client::new();

    match &args.command {
        AppArgsSubcommands::ListAssets(a) => {
            let asset_type = a.asset_type.clone();
            let offset = a.offset.to_string();
            let limit = a.limit.to_string();

            let query = vec![
                ("assetType".to_string(), asset_type),
                ("(after)".to_string(), offset),
                ("(limit)".to_string(), limit),
                ("(sort)".to_string(), "+Id".to_string()),
            ];

            let _ = get_asset_list_async(&config, req, auth_header, query).await;
        }

        AppArgsSubcommands::ListAssetProperties => {
            info!("List asset properties branch");
        }
    }

    Ok(())
}
