use clap::Parser;
use color_eyre::Result;
use hyperview::asset_api::get_raw_asset_by_id_async;
use log::info;
use reqwest::Client;

use crate::hyperview::{
    asset_api::search_assets_async,
    asset_properties_api::get_asset_property_list_async,
    auth::get_auth_header_async,
    cli::{get_config_path, get_debug_filter, handle_output_choice},
    cli_data::{AppArgs, AppArgsSubcommands, AppConfig},
    custom_asset_properties_api::get_custom_asset_property_list_async,
};

mod hyperview;

#[tokio::main]
async fn main() -> Result<()> {
    let args = AppArgs::parse();
    let debug_level = args.debug_level;
    let level_filter = get_debug_filter(debug_level.clone());
    env_logger::builder().filter(None, level_filter).init();

    info!("Starting Hyperview Asset Tool");
    info!("Startup options: | Debug Level: {debug_level:?} |");

    let config: AppConfig = confy::load_path(get_config_path())?;
    let auth_header = get_auth_header_async(&config).await?;
    let req = Client::new();

    match &args.command {
        AppArgsSubcommands::ListAssetProperties(options) => {
            let id = options.id.clone();
            let output_type = options.output_type.clone();
            let filename = options.filename.clone();

            let resp = get_asset_property_list_async(&config, req, auth_header, id).await?;
            handle_output_choice(output_type, filename, resp)?;
        }

        AppArgsSubcommands::ListCustomAssetProperties(options) => {
            let id = options.id.clone();
            let output_type = options.output_type.clone();
            let filename = options.filename.clone();

            let resp = get_custom_asset_property_list_async(&config, req, auth_header, id).await?;
            handle_output_choice(output_type, filename, resp)?;
        }

        AppArgsSubcommands::SearchAssets(options) => {
            let resp = search_assets_async(&config, req, auth_header, options.clone()).await?;

            handle_output_choice(options.output_type.clone(), options.filename.clone(), resp)?;
        }

        AppArgsSubcommands::UpdateAssetName(options) => {
            let resp =
                get_raw_asset_by_id_async(&config, req, auth_header, options.id.clone()).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
    }

    Ok(())
}
