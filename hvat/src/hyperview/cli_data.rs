use clap::{value_parser, Args, Parser, Subcommand};
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

#[allow(clippy::enum_variant_names)]
#[derive(Subcommand)]
pub enum AppArgsSubcommands {
    /// List assets for a standard type
    ListAssets(ListAssetsArgs),

    /// List a specific asset id
    ListAssetById(ByIdArgs),

    /// List asset properties
    ListAssetProperties(ListPropertiesArgs),

    /// List custom asset properties
    ListCustomAssetProperties(ListPropertiesArgs),

    /// Search assets
    SearchAssets(SearchAssetsArgs),
}

#[derive(Args, Debug)]
pub struct ListAssetsArgs {
    #[arg(
        short = 't',
        long,
        help = "Asset type, e.g. Crah",
        value_parser(ASSET_TYPES)
    )]
    pub asset_type: String,

    #[arg(
        short,
        long,
        help = "Number of records to skip (0 -> 99999), e.g. 100",
        default_value = "0", value_parser(value_parser!(u32).range(0..100000))
    )]
    pub skip: u32,

    #[arg(
        short,
        long,
        help = "Record limit (1 -> 1000), e.g. 100",
        default_value = "100",
        value_parser(value_parser!(u32).range(1..1001))
    )]
    pub limit: u32,

    #[arg(
        short,
        long,
        help = "Output type, e.g. csv",
        default_value = "record",
        value_parser(["record", "csv"])
    )]
    pub output_type: String,

    #[arg(short, long, help = "output filename, e.g. output.csv")]
    pub filename: Option<String>,
}

#[derive(Args, Debug)]
pub struct ByIdArgs {
    #[arg(
        short,
        long,
        help = "Primary ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub id: String,
}

#[derive(Args, Debug)]
pub struct ListPropertiesArgs {
    #[arg(
        short,
        long,
        help = "Asset ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub id: String,

    #[arg(
        short,
        long,
        help = "Output type, e.g. csv",
        default_value = "record",
        value_parser(["record", "csv"])
    )]
    pub output_type: String,

    #[arg(short, long, help = "output filename, e.g. output.csv")]
    pub filename: Option<String>,
}

#[derive(Args, Debug)]
pub struct SearchAssetsArgs {
    #[arg(short = 'p', long, help = "Search pattern or string, e.g. chrome")]
    pub search_pattern: String,

    #[arg(
        short = 't',
        long,
        help = "Optional asset type, e.g. Crah",
        value_parser(ASSET_TYPES)
    )]
    pub asset_type: Option<String>,

    #[arg(
        short = 'c',
        long,
        help = "Optional prefix of location path, e.g. \"All/\""
    )]
    pub location_path: Option<String>,

    #[arg(
        short,
        long,
        help = "Number of records to skip (0 -> 99999), e.g. 100",
        default_value = "0", value_parser(value_parser!(u32).range(0..100000))
    )]
    pub skip: u32,

    #[arg(
        short,
        long,
        help = "Record limit (1 -> 1000), e.g. 100",
        default_value = "100",
        value_parser(value_parser!(u32).range(1..1001))
    )]
    pub limit: u32,

    #[arg(
        short,
        long,
        help = "Output type, e.g. csv",
        default_value = "record",
        value_parser(["record", "csv"])
    )]
    pub output_type: String,

    #[arg(short, long, help = "output filename, e.g. output.csv")]
    pub filename: Option<String>,
}
