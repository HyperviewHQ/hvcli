use clap::Parser;
use log::info;
use reqwest::Client;

use crate::hyperview::{
    auth::get_auth_header_async,
    cli_data::{AppArgs, AppConfig},
    cli_functions::{get_config_path, get_debug_filter, route_command_async},
};

mod hyperview;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = AppArgs::parse();
    let debug_level = args.debug_level;
    let level_filter = get_debug_filter(debug_level);
    env_logger::builder().filter(None, level_filter).init();

    info!("Starting Hyperview Asset Tool");
    info!("Startup options: | Debug Level: {debug_level:?} |");

    let config: AppConfig = confy::load_path(get_config_path())?;
    let auth_header = get_auth_header_async(&config).await?;
    let req = Client::new();

    route_command_async(args.command, config, auth_header, req).await
}
