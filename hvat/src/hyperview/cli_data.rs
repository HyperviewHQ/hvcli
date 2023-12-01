use core::fmt;

use clap::{value_parser, Args, Parser, Subcommand, ValueEnum};
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

#[derive(Debug, ValueEnum, Clone)]
pub enum OutputOptions {
    CsvFile,
    Json,
    Record,
}

#[derive(Debug, ValueEnum, Clone)]
#[clap(rename_all = "PascalCase")]
pub enum AssetTypes {
    BladeEnclosure,
    BladeNetwork,
    BladeServer,
    BladeStorage,
    Busway,
    Camera,
    Chiller,
    Crac,
    Crah,
    Environmental,
    FireControlPanel,
    Generator,
    InRowCooling,
    KvmSwitch,
    Location,
    Monitor,
    NetworkDevice,
    NetworkStorage,
    NodeServer,
    PatchPanel,
    PduAndRpp,
    PowerMeter,
    Rack,
    RackPdu,
    Server,
    SmallUps,
    TransferSwitch,
    Unknown,
    Ups,
    VirtualServer,
}

impl fmt::Display for AssetTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, ValueEnum, Clone)]
pub enum DebugLevels {
    Error,
    Warn,
    Debug,
    Info,
    Trace,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct AppArgs {
    #[arg(short = 'd', long, help = "Debug level", default_value = "error")]
    pub debug_level: DebugLevels,

    #[command(subcommand)]
    pub command: AppArgsSubcommands,
}

#[allow(clippy::enum_variant_names)]
#[derive(Subcommand)]
pub enum AppArgsSubcommands {
    /// List asset properties
    ListAssetProperties(ListPropertiesArgs),

    /// List custom asset properties
    ListCustomAssetProperties(ListPropertiesArgs),

    /// Search assets
    #[clap(alias = "list-assets")]
    SearchAssets(SearchAssetsArgs),
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
        help = "Output type, e.g. csv-file",
        default_value = "record"
    )]
    pub output_type: OutputOptions,

    #[arg(short, long, help = "output filename, e.g. output.csv")]
    pub filename: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct SearchAssetsArgs {
    #[arg(
        short = 'p',
        long,
        help = "Search pattern or string, e.g. chrome",
        default_value = "*"
    )]
    pub search_pattern: String,

    #[arg(short = 't', long, help = "Optional asset type, e.g. Crah")]
    pub asset_type: Option<AssetTypes>,

    #[arg(
        short = 'c',
        long,
        help = "Optional prefix of location path, e.g. \"All/\""
    )]
    pub location_path: Option<String>,

    #[arg(
        short = 'P',
        long,
        help = "Optional property or custom property to filter on, e.g. serialNumer=SN1234567890"
    )]
    pub properties: Option<Vec<String>>,

    #[arg(
        short = 'C',
        long,
        help = "Optional custom property or custom property to filter on, e.g. serialNumer=SN1234567890"
    )]
    pub custom_properties: Option<Vec<String>>,

    #[arg(
        short,
        long,
        help = "Primary ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub id: Option<String>,

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
        help = "Output type, e.g. csv-file",
        default_value = "record"
    )]
    pub output_type: OutputOptions,

    #[arg(short, long, help = "output filename, e.g. output.csv")]
    pub filename: Option<String>,
}
