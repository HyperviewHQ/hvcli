use clap::Parser;
use color_eyre::eyre::Result;
use hyperview::cli::{get_debug_filter, AppArgs};
use log::info;

use crate::hyperview::{
    api_constants::ASSET_TYPES,
    auth::get_auth_header_async,
    cli::{get_config_path, AppConfig},
};

mod hyperview;

#[tokio::main]
async fn main() -> Result<()> {
    let args = AppArgs::parse();
    let debug_level = args.debug_level;
    let level_filter = get_debug_filter(&debug_level);
    env_logger::builder().filter(None, level_filter).init();

    info!("Starting hyperview asset import tool");
    info!("Startup options:\n|debug_level: {debug_level} |\n");

    let config: AppConfig = confy::load_path(get_config_path())?;

    ASSET_TYPES.iter().for_each(|t| println!("{t}"));

    let token = get_auth_header_async(&config).await?;

    println!("TOKEN: {}", token);

    Ok(())
}
