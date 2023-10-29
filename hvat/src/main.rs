use clap::Parser;
use color_eyre::Result;
use hyperview::cli::{get_debug_filter, AppArgs};
use log::info;
use reqwest::Client;

use crate::hyperview::{
    api_constants::ASSET_TYPES,
    asset_api::{get_asset_by_id_async, get_asset_list_async},
    auth::get_auth_header_async,
    cli::{get_config_path, handle_output_choice, AppArgsSubcommands, AppConfig},
};

mod hyperview;

#[tokio::main]
async fn main() -> Result<()> {
    let args = AppArgs::parse();
    let debug_level = args.debug_level;
    let level_filter = get_debug_filter(&debug_level);
    env_logger::builder().filter(None, level_filter).init();

    info!("Starting Hyperview Asset Tool");
    info!("Startup options: | Debug Level: {debug_level} |");

    let config: AppConfig = confy::load_path(get_config_path())?;
    let auth_header = get_auth_header_async(&config).await?;
    let req = Client::new();

    match &args.command {
        AppArgsSubcommands::ListAssets(options) => {
            let asset_type = options.asset_type.clone();
            let skip = options.skip.to_string();
            let limit = options.limit.to_string();
            let output_type = options.output_type.clone();
            let filename = options.filename.clone();

            let query = vec![
                ("assetType".to_string(), asset_type),
                ("(after)".to_string(), skip),
                ("(limit)".to_string(), limit),
                ("(sort)".to_string(), "+Id".to_string()),
            ];

            let resp = get_asset_list_async(&config, req, auth_header, query).await?;
            handle_output_choice(output_type, filename, resp)?;
        }

        AppArgsSubcommands::ListAssetById(options) => {
            let id = options.id.clone();

            let resp = get_asset_by_id_async(&config, req, auth_header, id).await?;
            println!("---- [0] ----\n{}", resp);
        }
        AppArgsSubcommands::ListAssetProperties(options) => {
            info!("List asset properties options: {:#?}", options);
        }
    }

    Ok(())
}
