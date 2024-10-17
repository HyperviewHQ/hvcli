use clap::Parser;
use color_eyre::Result;
use hyperview::asset_alarm_events_functions::list_alarm_events_async;
use log::info;
use reqwest::Client;

use crate::hyperview::{
    asset_api_functions::{
        bulk_update_asset_location_async, bulk_update_asset_name_async, bulk_update_ports_async,
        list_asset_ports_async, search_assets_async, update_asset_location_async,
        update_asset_name_by_id_async,
    },
    asset_properties_api_functions::get_asset_property_list_async,
    auth::get_auth_header_async,
    cli_data::{AppArgs, AppArgsSubcommands, AppConfig},
    cli_functions::{get_config_path, get_debug_filter, handle_output_choice},
    custom_asset_properties_api_functions::get_custom_asset_property_list_async,
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
            update_asset_name_by_id_async(
                &config,
                req,
                auth_header,
                options.id.clone(),
                options.new_name.clone(),
            )
            .await?;
        }

        AppArgsSubcommands::BulkUpdateAssetName(options) => {
            bulk_update_asset_name_async(&config, req, auth_header, options.filename.clone())
                .await?;
        }

        AppArgsSubcommands::UpdateAssetLocation(options) => {
            update_asset_location_async(&config, req, auth_header, options.clone()).await?;
        }

        AppArgsSubcommands::BulkUpdateAssetLocation(options) => {
            bulk_update_asset_location_async(&config, req, auth_header, options.filename.clone())
                .await?;
        }

        AppArgsSubcommands::ListAssetPorts(options) => {
            let resp = list_asset_ports_async(&config, req, auth_header, options.clone()).await?;

            handle_output_choice(options.output_type.clone(), options.filename.clone(), resp)?;
        }

        AppArgsSubcommands::BulkUpdatePatchPanelPorts(options) => {
            bulk_update_ports_async(&config, req, auth_header, options.filename.clone(), true)
                .await?;
        }

        AppArgsSubcommands::BulkUpdateAssetPorts(options) => {
            bulk_update_ports_async(&config, req, auth_header, options.filename.clone(), false)
                .await?;
        }

        AppArgsSubcommands::ListUnacknowledgedAlarms(options) => {
            let resp =
                list_alarm_events_async(&config, req, auth_header, options.skip, options.limit)
                    .await?;

            handle_output_choice(
                options.output_type.clone(),
                options.filename.clone(),
                resp.data,
            )?;
        }
    }

    Ok(())
}
