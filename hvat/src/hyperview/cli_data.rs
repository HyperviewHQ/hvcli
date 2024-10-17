use clap::{value_parser, Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::fmt;

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

#[derive(Debug, ValueEnum, Clone, Serialize, Deserialize)]
#[clap(rename_all = "PascalCase")]
pub enum RackSide {
    Front,
    Rear,
    Unknown,
}

#[derive(Debug, ValueEnum, Clone, Serialize, Deserialize)]
#[clap(rename_all = "PascalCase")]
pub enum RackPosition {
    Left,
    Right,
    Top,
    Bottom,
    Above,
    Below,
    Unknown,
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

    /// Update asset name
    UpdateAssetName(UpdateAssetNameArgs),

    /// Bulk update asset name
    BulkUpdateAssetName(BulkUpdateAssetNameArgs),

    /// Update asset location
    UpdateAssetLocation(UpdateAssetLocationArgs),

    /// Bulk update asset location
    BulkUpdateAssetLocation(BulkUpdateAssetLocationArgs),

    /// List asset ports
    ListAssetPorts(ListAssetPortsArgs),

    /// Bulk update patch panel port names
    BulkUpdatePatchPanelPorts(BulkUpdatePortsArgs),

    /// Bulk update asset port names
    BulkUpdateAssetPorts(BulkUpdatePortsArgs),

    /// List alarm events
    ListAlarms(ListAlarmsArgs),

    /// Acknowledge or close alarm events
    ManageAlarms(ManageAlarmArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ManageAlarmArgs {
    #[arg(short, long, help = "Input filename, e.g. port_name_update.csv")]
    pub filename: String,
}

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum AlarmEventFilterOption {
    Unacknowledged,
    Active,
}

#[derive(Args, Debug, Clone)]
pub struct ListAlarmsArgs {
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
        help = "Record limit (1 -> 100000), e.g. 100",
        default_value = "100",
        value_parser(value_parser!(u32).range(1..100001))
    )]
    pub limit: u32,

    #[arg(
        short,
        long,
        help = "Asset alarm event filter option, e.g. active",
        default_value = "active"
    )]
    pub alarm_filter: AlarmEventFilterOption,

    #[arg(
        short,
        long,
        help = "Output type, e.g. csv-file",
        default_value = "record"
    )]
    pub output_type: OutputOptions,

    #[arg(short, long, help = "Output filename, e.g. output.csv")]
    pub filename: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct BulkUpdatePortsArgs {
    #[arg(short, long, help = "Input filename, e.g. port_name_update.csv")]
    pub filename: String,
}

#[derive(Args, Debug, Clone)]
pub struct ListAssetPortsArgs {
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

    #[arg(short, long, help = "Output filename, e.g. output.csv")]
    pub filename: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct UpdateAssetLocationArgs {
    #[arg(
        short,
        long,
        help = "Asset ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub id: String,

    #[arg(
        short = 'n',
        long,
        help = "New location ID. It must be a valid GUID/UUID, e.g. 68713cf3-2f5b-45b3-97a3-592e70537c4d"
    )]
    pub new_location_id: String,

    #[arg(
        short = 'p',
        long,
        help = "Optional rack position attribute for zero-u rack mounted assets. e.g. Front"
    )]
    pub rack_position: Option<RackPosition>,

    #[arg(
        short = 's',
        long,
        help = "Optional rack side attribute for rack mounted and zero-u assets. e.g. Front"
    )]
    pub rack_side: Option<RackSide>,

    #[arg(
        short = 'u',
        long,
        help = "Optional rack unit elevation attribute for rack mounted assets. e.g. Front"
    )]
    pub rack_u_location: Option<usize>,
}

#[derive(Args, Debug)]
pub struct BulkUpdateAssetLocationArgs {
    #[arg(short, long, help = "Input filename, e.g. name_changes.csv")]
    pub filename: String,
}

#[derive(Args, Debug)]
pub struct UpdateAssetNameArgs {
    #[arg(
        short,
        long,
        help = "Asset ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub id: String,

    #[arg(
        short = 'n',
        long,
        help = "New Name. It must be a string value, e.g. \"Main_Generator\""
    )]
    pub new_name: String,
}

#[derive(Args, Debug)]
pub struct BulkUpdateAssetNameArgs {
    #[arg(short, long, help = "Input filename, e.g. name_changes.csv")]
    pub filename: String,
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

    #[arg(short, long, help = "Output filename, e.g. output.csv")]
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

    #[arg(short = 'M', long, help = "Manufacturer name, e.g. dell")]
    pub manufacturer: Option<String>,

    #[arg(short = 'R', long, help = "Product name, e.g. poweredge")]
    pub product: Option<String>,

    #[arg(short = 'U', long, help = "Show property in output, e.g. dnsName")]
    pub show_property: Option<String>,

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

    #[arg(short, long, help = "Output filename, e.g. output.csv")]
    pub filename: Option<String>,
}
