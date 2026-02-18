use clap::{Args, Parser, Subcommand, ValueEnum, value_parser};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

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
    #[arg(short = 'd', long, help = "Debug level", default_value = "error")]
    pub debug_level: DebugLevels,

    #[command(subcommand)]
    pub command: AppArgsSubcommands,
}

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum OutputOptions {
    CsvFile,
    Json,
    Record,
}

#[derive(Debug, ValueEnum, Clone, Serialize, Deserialize)]
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
        write!(f, "{self:?}")
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

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum DebugLevels {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, ValueEnum, Clone, Copy, Deserialize)]
#[clap(rename_all = "PascalCase")]
pub enum RackPanelType {
    BlankingPanel,
    CableManagement,
}

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum ManageActionOptions {
    Acknowledge,
    Close,
}

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum AlarmEventFilterOptions {
    Unacknowledged,
    Active,
}

#[allow(clippy::enum_variant_names)]
#[derive(Subcommand)]
pub enum AppArgsSubcommands {
    /// List all available properties for an asset identified by its unique id
    ListAssetProperties(ListRecordsByAssetIdArgs),

    /// List all available custom properties for an asset identified by its unique id
    ListCustomAssetProperties(ListRecordsByAssetIdArgs),

    /// Search for assets in Hyperview
    #[clap(alias = "list-assets")]
    SearchAssets(SearchAssetsArgs),

    /// List assets that match a specific set of property values. For example, a list of serial numbers. Please note that the matches are exact
    ListAnyOf(ListAnyOfArgs),

    /// Update the display name of an asset identified by its unique id
    UpdateAssetName(UpdateAssetNameArgs),

    /// Update multiple assets from a CSV file
    BulkUpdateAssetName(BulkUpdateSingleInputFileArgs),

    /// Update the location of an asset identified by its unique id
    UpdateAssetLocation(UpdateAssetLocationArgs),

    /// Update the location of multiple assets from a CSV file
    BulkUpdateAssetLocation(BulkUpdateSingleInputFileArgs),

    /// Update the serial number of an asset identified by its unique id. Applies to manually created assets and assets discovered without a serial number
    UpdateAssetSerialNumber(UpdateAssetPropertyArgs),

    /// Update the serial numbers of multiple assets from a CSV file. Applies to manually created assets and assets discovered without a serial number
    BulkUpdateAssetSerialNumber(BulkUpdateSingleInputFileArgs),

    /// Update the asset tag of an asset identified by its unique id
    UpdateAssetTag(UpdateAssetPropertyArgs),

    /// Update the asset tag of multiple assets from a CSV file
    BulkUpdateAssetTag(BulkUpdateSingleInputFileArgs),

    /// Update the power design value property of an asset identified by its unique id. This applies to Rack and Location asset types
    UpdatePowerDesignValue(UpdateAssetPropertyArgs),

    /// Update the power “design value” property of multiple assets from a CSV file. This applies to Rack and Location asset types
    BulkUpdatePowerDesignValue(BulkUpdateSingleInputFileArgs),

    /// List the physical network ports of an asset identified by its unique id
    ListAssetPorts(ListRecordsByAssetIdArgs),

    /// Update the physical network port names of patch panel assets from a CSV file
    BulkUpdatePatchPanelPorts(BulkUpdateSingleInputFileArgs),

    /// Update the physical network port names of other (non-patch-panel) assets from a CSV file
    BulkUpdateAssetPorts(BulkUpdateSingleInputFileArgs),

    /// Update the value of a custom property of an asset identified by its unique id
    UpdateCustomAssetProperty(UpdateCustomAssetPropertyArgs),

    /// Update the custom property value of multiple assets using a CSV file
    BulkUpdateCustomAssetProperty(BulkUpdateSingleInputFileArgs),

    /// List alarm events. By default, it will list active events. It can also list unacknowledged events via a command-line option
    ListAlarms(ListAlarmsArgs),

    /// Acknowledge or close alarm events using a CSV file generated by the list-alarms command. By default, this command closes the events; it can also acknowledge them via a command-line option
    ManageAlarms(ManageAlarmsArgs),

    /// Add a blanking panel or cable management panel to a rack identified by its unique id
    AddRackAccessory(AddRackAccessoryArgs),

    /// Add blanking panels or cable management panels to multiple racks using a CSV file
    BulkAddRackAccessory(BulkUpdateSingleInputFileArgs),

    /// List sensors for an asset identified by its unique id
    ListAssetSensors(ListRecordsByAssetIdArgs),

    /// Update asset sensor name and/or access policy using a CSV file. IMPORTANT: Keep the access policy field empty to maintain the original and only change the name. Use a NIL UUID (00000000-0000-0000-0000-000000000000) to reset to the parent access policy
    BulkUpdateAssetSensor(BulkUpdateSingleInputFileArgs),

    /// List Rack PDU outlets for an asset identified by its unique id
    ListRackPduOutlets(ListRecordsByAssetIdArgs),

    /// List busway tap-offs for an asset identified by its unique id
    ListBuswayTapoffs(ListRecordsByAssetIdArgs),

    /// List PDU/RPP Breakers for an asset identified by its unique id
    ListPduRppBreakers(ListRecordsByAssetIdArgs),

    /// Add power association. Note that associations are asset-to-asset. For example, when associating with a specific outlet or tap-off, you need its id
    AddPowerAssociation(AddPowerAssociationArgs),

    /// Add power associations between assets using a CSV
    BulkAddPowerAssociation(BulkUpdateSingleInputFileArgs),
}

#[derive(Args, Debug, Clone)]
pub struct BulkUpdateSingleInputFileArgs {
    #[arg(short, long, help = "Input filename, e.g. input.csv")]
    pub filename: String,
}

#[derive(Args, Debug, Clone)]
pub struct ListRecordsByAssetIdArgs {
    #[arg(
        short,
        long,
        help = "Asset ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub id: Uuid,

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
pub struct AddPowerAssociationArgs {
    #[arg(
        short = 'c',
        long,
        help = "Power consuming asset id, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub power_consuming_asset_id: Uuid,

    #[arg(
        short = 'p',
        long,
        help = "Power providing asset id, e.g. 61d2dcf3-65f0-4f84-89d4-3110a1e1f196. Use the component id if associating with a specific outlet, tap-off, or breaker."
    )]
    pub power_providing_asset_id: Uuid,
}

#[derive(Args, Debug, Clone)]
pub struct AddRackAccessoryArgs {
    #[arg(
        short,
        long,
        help = "Asset ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub id: Uuid,

    #[arg(short = 'l', long, help = "Panel type value. e.g. CableManagement")]
    pub panel_type: RackPanelType,

    #[arg(
        short = 's',
        long,
        help = "Rack side attribute for accessory. e.g. Front"
    )]
    pub rack_side: RackSide,

    #[arg(
        short = 'u',
        long,
        help = "Rack unit elevation attribute for rack mounted assets. e.g. 22"
    )]
    pub rack_u_location: usize,
}

#[derive(Args, Debug, Clone)]
pub struct ManageAlarmsArgs {
    #[arg(short, long, help = "Input filename, e.g. input.csv")]
    pub filename: String,

    #[arg(
        short,
        long,
        help = "Manage action to use, e.g. close",
        default_value = "close"
    )]
    pub manage_action: ManageActionOptions,
}

#[derive(Args, Debug, Clone)]
pub struct ListAlarmsArgs {
    #[arg(
        short,
        long,
        help = "Number of records to skip (0 -> 1_000_000_000), e.g. 100",
        default_value = "0", value_parser(value_parser!(u32).range(0..=1_000_000_000))
    )]
    pub skip: u32,

    #[arg(
        short,
        long,
        help = "Record limit (1 -> 100_000), e.g. 100",
        default_value = "100",
        value_parser(value_parser!(u32).range(1..=100_000))
    )]
    pub limit: u32,

    #[arg(
        short,
        long,
        help = "Asset alarm event filter option, e.g. active",
        default_value = "active"
    )]
    pub alarm_filter: AlarmEventFilterOptions,

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
pub struct UpdateCustomAssetPropertyArgs {
    #[arg(
        short,
        long,
        help = "Asset ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub id: Uuid,

    #[arg(
        short = 'N',
        long,
        help = "Custom property to update, e.g. testCustomPropertyName"
    )]
    pub custom_property: String,

    #[arg(short = 'D', long, help = "New custom property value, e.g. testValue")]
    pub new_custom_property_value: String,
}

#[derive(Args, Debug, Clone)]
pub struct UpdateAssetPropertyArgs {
    #[arg(
        short,
        long,
        help = "Asset ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub id: Uuid,

    #[arg(short = 'T', long, help = "New property value, e.g. EPDU123456789")]
    pub new_value: String,
}

#[derive(Args, Debug, Clone)]
pub struct UpdateAssetLocationArgs {
    #[arg(
        short,
        long,
        help = "Asset ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub id: Uuid,

    #[arg(
        short = 'n',
        long,
        help = "New location ID. It must be a valid GUID/UUID, e.g. 68713cf3-2f5b-45b3-97a3-592e70537c4d"
    )]
    pub new_location_id: Uuid,

    #[arg(
        short = 'p',
        long,
        help = "Optional rack position attribute for zero-u rack mounted assets. e.g. Left"
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
        help = "Optional rack unit elevation attribute for rack mounted assets. e.g. 22"
    )]
    pub rack_u_location: Option<usize>,
}

#[derive(Args, Debug)]
pub struct UpdateAssetNameArgs {
    #[arg(
        short,
        long,
        help = "Asset ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub id: Uuid,

    #[arg(
        short = 'n',
        long,
        help = "New Name. It must be a string value, e.g. \"Main_Generator\""
    )]
    pub new_name: String,
}

#[derive(Args, Debug, Clone)]
pub struct ListAnyOfArgs {
    #[arg(
        short = 'k',
        long,
        help = "Property key to filter on, e.g. serialNumber"
    )]
    pub property_key: String,

    #[arg(
        short = 'v',
        long,
        value_delimiter = ',',
        help = "A list of property values to filter on, e.g. serialNumber1,serialNumber2"
    )]
    pub property_value: Vec<String>,

    #[arg(short = 't', long, help = "Optional asset type, e.g. Crah")]
    pub asset_type: Option<AssetTypes>,

    #[arg(
        short = 'c',
        long,
        help = "Optional prefix of location path, e.g. \"All/\""
    )]
    pub location_path: Option<String>,

    #[arg(
        short = 'C',
        long,
        help = "Optional custom property to filter on, e.g. testCustomProperty=testValue"
    )]
    pub custom_properties: Option<Vec<String>>,

    #[arg(
        short,
        long,
        help = "Optional asset ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub id: Option<Uuid>,

    #[arg(short = 'M', long, help = "Manufacturer name, e.g. dell")]
    pub manufacturer: Option<String>,

    #[arg(short = 'R', long, help = "Product name, e.g. poweredge")]
    pub product: Option<String>,

    #[arg(short = 'U', long, help = "Show property in output, e.g. ratedVoltage")]
    pub show_property: Option<String>,

    #[arg(
        short,
        long,
        help = "Number of records to skip (0 -> 1_000_000_000), e.g. 100",
        default_value = "0", value_parser(value_parser!(u32).range(0..=1_000_000_000))
    )]
    pub skip: u32,

    #[arg(
        short,
        long,
        help = "Record limit (1 -> 1000), e.g. 100",
        default_value = "100",
        value_parser(value_parser!(u32).range(1..=1000))
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

#[derive(Args, Debug, Clone)]
pub struct SearchAssetsArgs {
    #[arg(short = 'p', long, help = "Search pattern or string, e.g. chrome")]
    pub search_pattern: Option<String>,

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
        help = "Optional property to filter on, e.g. serialNumber=SN1234567890"
    )]
    pub properties: Option<Vec<String>>,

    #[arg(
        short = 'C',
        long,
        help = "Optional custom property to filter on, e.g. testCustomProperty=testValue"
    )]
    pub custom_properties: Option<Vec<String>>,

    #[arg(
        short,
        long,
        help = "Optional asset ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub id: Option<Uuid>,

    #[arg(short = 'M', long, help = "Manufacturer name, e.g. dell")]
    pub manufacturer: Option<String>,

    #[arg(short = 'R', long, help = "Product name, e.g. poweredge")]
    pub product: Option<String>,

    #[arg(short = 'U', long, help = "Show property in output, e.g. ratedVoltage")]
    pub show_property: Option<String>,

    #[arg(
        short,
        long,
        help = "Number of records to skip (0 -> 1_000_000_000), e.g. 100",
        default_value = "0", value_parser(value_parser!(u32).range(0..=1_000_000_000))
    )]
    pub skip: u32,

    #[arg(
        short,
        long,
        help = "Record limit (1 -> 1000), e.g. 100",
        default_value = "100",
        value_parser(value_parser!(u32).range(1..=1000))
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
