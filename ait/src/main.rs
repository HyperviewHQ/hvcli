use color_eyre::eyre::Result;
use clap::Parser;
use hyperview::cli::{AppArgs, get_debug_filter};
use log::info;

use crate::hyperview::{api_constants::ASSET_TYPES, cli::{AppConfig, get_config_path}};

mod hyperview;

fn main() -> Result<()>{
    let args = AppArgs::parse();
    let debug_level = args.debug_level;
    let level_filter = get_debug_filter(&debug_level);
    env_logger::builder().filter(None, level_filter).init();

    info!("Starting hyperview asset import tool");
    info!("Startup options:\n|debug_level: {debug_level} |\n");

    let _config: AppConfig = confy::load_path(get_config_path())?;

    ASSET_TYPES.iter().for_each(|t| println!("{t}"));

    Ok(())
}
