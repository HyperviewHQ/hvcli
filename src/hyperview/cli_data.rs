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

// camelCase across clap (CLI input), serde (CSV input), and Display (wire output) so the CLI's
// representation of an asset type is a single value that matches the Hyperview API's
// AssetTypeEnum (which is camelCase). The API is case-insensitive on input but emits camelCase.
#[derive(Debug, ValueEnum, Clone, Serialize, Deserialize)]
#[clap(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub enum AssetTypes {
    BatteryBank,
    BladeEnclosure,
    BladeNetwork,
    BladeServer,
    BladeStorage,
    Busway,
    Camera,
    Chiller,
    Crac,
    Crah,
    DcRectifier,
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
    OtherDevice,
    PatchPanel,
    PduAndRpp,
    PowerMeter,
    Rack,
    RackPdu,
    Server,
    SmallUps,
    Switchboard,
    Switchgear,
    TransferSwitch,
    Unknown,
    Ups,
    VirtualServer,
}

impl fmt::Display for AssetTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Emit the API's camelCase form: the PascalCase variant name with its first letter
        // lowercased. This matches `#[serde(rename_all = "camelCase")]` and the clap value names.
        let name = format!("{self:?}");
        let mut chars = name.chars();
        match chars.next() {
            Some(first) => write!(f, "{}{}", first.to_ascii_lowercase(), chars.as_str()),
            None => Ok(()),
        }
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

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum SensorValueClass {
    Numeric,
    Enum,
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

    /// Generate a monthly (or arbitrary date-range) report of daily-summary statistics (avg/max/min/last) for a named sensor across all assets of a given type, optionally enriched with a custom-property value.
    GenerateSensorReport(GenerateSensorReportArgs),

    /// List current `BACnet IP` sensor definitions
    ListBacnetDefinitions(ListDefinitionsArgs),

    /// Add a new `BACnet IP` sensor definition
    AddBacnetDefinition(AddDefinitionArgs),

    /// List numeric sensors for a `BACnet IP` sensor definition
    ListBacnetNumericSensorDefinitions(ListSensorDefinitionsArgs),

    /// List non-numeric sensors for a `BACnet IP` sensor definition
    ListBacnetNonNumericSensorDefinitions(ListSensorDefinitionsArgs),

    /// Bulk create or update numeric sensors on a `BACnet IP` sensor definition from a CSV file. Rows with a blank id are created; rows with a valid UUID id are updated
    BulkImportBacnetNumericSensorDefinitions(BulkImportSensorDefinitionsArgs),

    /// Bulk create or update non-numeric sensors on a `BACnet IP` sensor definition from a CSV file. Rows with a blank id are created; rows with a valid UUID id are updated
    BulkImportBacnetNonNumericSensorDefinitions(BulkImportSensorDefinitionsArgs),

    /// List current Modbus TCP sensor definitions
    ListModbusDefinitions(ListDefinitionsArgs),

    /// Add a new Modbus TCP sensor definition
    AddModbusDefinition(AddDefinitionArgs),

    /// List numeric sensors for a Modbus TCP sensor definition
    ListModbusNumericSensorDefinitions(ListSensorDefinitionsArgs),

    /// List non-numeric sensors for a Modbus TCP sensor definition
    ListModbusNonNumericSensorDefinitions(ListSensorDefinitionsArgs),

    /// Bulk create or update numeric sensors on a Modbus TCP sensor definition from a CSV file. Rows with a blank id are created; rows with a valid UUID id are updated
    BulkImportModbusNumericSensorDefinitions(BulkImportSensorDefinitionsArgs),

    /// Bulk create or update non-numeric sensors on a Modbus TCP sensor definition from a CSV file. Rows with a blank id are created; rows with a valid UUID id are updated
    BulkImportModbusNonNumericSensorDefinitions(BulkImportSensorDefinitionsArgs),

    /// List valid sensor types for an asset type, optionally filtered by sensor class (numeric or enum)
    ListSensorDefinitionTypes(ListSensorDefinitionTypesArgs),

    /// List components of a Modbus TCP sensor definition
    ListModbusComponents(ListModbusComponentsArgs),

    /// Add a new component to a Modbus TCP sensor definition
    AddModbusComponent(AddModbusComponentArgs),

    /// Rename a component of a Modbus TCP sensor definition
    UpdateModbusComponent(UpdateModbusComponentArgs),

    /// Delete a component from a Modbus TCP sensor definition
    DeleteModbusComponent(DeleteModbusComponentArgs),

    /// Get a single `BACnet IP` sensor definition by its id
    GetBacnetDefinition(GetDefinitionArgs),

    /// Update the name, asset type, and description of a `BACnet IP` sensor definition
    UpdateBacnetDefinition(UpdateDefinitionArgs),

    /// Delete a `BACnet IP` sensor definition by its id
    DeleteBacnetDefinition(DeleteDefinitionArgs),

    /// Get a single Modbus TCP sensor definition by its id
    GetModbusDefinition(GetDefinitionArgs),

    /// Update the name, asset type, and description of a Modbus TCP sensor definition
    UpdateModbusDefinition(UpdateDefinitionArgs),

    /// Delete a Modbus TCP sensor definition by its id
    DeleteModbusDefinition(DeleteDefinitionArgs),

    /// Delete a numeric sensor from a `BACnet IP` sensor definition
    DeleteBacnetNumericSensorDefinition(DeleteSensorDefinitionArgs),

    /// Delete a non-numeric sensor from a `BACnet IP` sensor definition
    DeleteBacnetNonNumericSensorDefinition(DeleteSensorDefinitionArgs),

    /// Delete a numeric sensor from a Modbus TCP sensor definition
    DeleteModbusNumericSensorDefinition(DeleteSensorDefinitionArgs),

    /// Delete a non-numeric sensor from a Modbus TCP sensor definition
    DeleteModbusNonNumericSensorDefinition(DeleteSensorDefinitionArgs),
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
pub struct GenerateSensorReportArgs {
    #[arg(short = 't', long, help = "Asset type to include, e.g. Rack")]
    pub asset_type: AssetTypes,

    #[arg(
        short = 's',
        long,
        help = "Sensor name to report on, e.g. averageKwhByHour"
    )]
    pub sensor: String,

    #[arg(
        short = 'y',
        long,
        help = "Year for the report month (2020-2029). Required unless --start and --end are supplied.",
        value_parser(value_parser!(i32).range(2020..=2029))
    )]
    pub year: Option<i32>,

    #[arg(
        short = 'm',
        long,
        help = "Month for the report (1-12). Required unless --start and --end are supplied.",
        value_parser(value_parser!(u32).range(1..=12))
    )]
    pub month: Option<u32>,

    #[arg(
        short = 'S',
        long,
        help = "Optional start date (YYYY-MM-DD, inclusive). Must be paired with --end and mutually exclusive with --year/--month."
    )]
    pub start: Option<String>,

    #[arg(
        short = 'E',
        long,
        help = "Optional end date (YYYY-MM-DD, exclusive). Must be paired with --start and mutually exclusive with --year/--month."
    )]
    pub end: Option<String>,

    #[arg(
        short = 'c',
        long,
        help = "Optional custom property name whose value is added as a column, e.g. \"Business Unit\""
    )]
    pub custom_property: Option<String>,

    #[arg(
        short = 'C',
        long,
        help = "Optional prefix of location path, e.g. \"All/Datacenter 1/\""
    )]
    pub location_path: Option<String>,

    #[arg(
        short = 'M',
        long,
        help = "Optional manufacturer name filter, e.g. dell"
    )]
    pub manufacturer: Option<String>,

    #[arg(
        short = 'R',
        long,
        help = "Optional product name filter, e.g. poweredge"
    )]
    pub product: Option<String>,

    #[arg(
        long,
        help = "Number of assets to skip (0 -> 1_000_000_000), e.g. 100",
        default_value = "0", value_parser(value_parser!(u32).range(0..=1_000_000_000))
    )]
    pub skip: u32,

    #[arg(
        long,
        help = "Asset page size (1 -> 1000), e.g. 100",
        default_value = "100",
        value_parser(value_parser!(u32).range(1..=1000))
    )]
    pub limit: u32,

    #[arg(
        short = 'o',
        long,
        help = "Output type, e.g. csv-file",
        default_value = "csv-file"
    )]
    pub output_type: OutputOptions,

    #[arg(short = 'f', long, help = "Output filename, e.g. report.csv")]
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

#[derive(Args, Debug, Clone)]
pub struct ListDefinitionsArgs {
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
pub struct AddDefinitionArgs {
    #[arg(short, long, help = "Definition name")]
    pub name: String,

    #[arg(short = 't', long, help = "Asset type, e.g. Crah")]
    pub asset_type: AssetTypes,

    #[arg(short = 'D', long, help = "Optional definition description")]
    pub description: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct ListSensorDefinitionsArgs {
    #[arg(
        short,
        long,
        help = "Definition ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub definition_id: Uuid,

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
pub struct BulkImportSensorDefinitionsArgs {
    #[arg(short, long, help = "Input filename, e.g. input.csv")]
    pub filename: String,

    #[arg(
        short,
        long,
        help = "Definition ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub definition_id: Uuid,

    #[arg(
        long,
        help = "Ignore the id column and create every row as a new sensor. Use to clone an exported file into a different definition."
    )]
    pub create_as_new: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ListSensorDefinitionTypesArgs {
    #[arg(short = 't', long, help = "Asset type, e.g. Crah")]
    pub asset_type: AssetTypes,

    #[arg(
        short,
        long,
        help = "Sensor class, e.g. numeric",
        default_value = "numeric"
    )]
    pub sensor_class: SensorValueClass,

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
pub struct ListModbusComponentsArgs {
    #[arg(
        short,
        long,
        help = "Definition ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub definition_id: Uuid,

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
pub struct AddModbusComponentArgs {
    #[arg(
        short,
        long,
        help = "Definition ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub definition_id: Uuid,

    #[arg(short, long, help = "Component name")]
    pub name: String,
}

#[derive(Args, Debug, Clone)]
pub struct UpdateModbusComponentArgs {
    #[arg(
        short,
        long,
        help = "Definition ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub definition_id: Uuid,

    #[arg(
        short,
        long,
        help = "Component ID. It must be a valid GUID/UUID, e.g. 61d2dcf3-65f0-4f84-89d4-3110a1e1f196"
    )]
    pub component_id: Uuid,

    #[arg(short, long, help = "New component name")]
    pub name: String,
}

#[derive(Args, Debug, Clone)]
pub struct DeleteModbusComponentArgs {
    #[arg(
        short,
        long,
        help = "Definition ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub definition_id: Uuid,

    #[arg(
        short,
        long,
        help = "Component ID. It must be a valid GUID/UUID, e.g. 61d2dcf3-65f0-4f84-89d4-3110a1e1f196"
    )]
    pub component_id: Uuid,
}

#[derive(Args, Debug, Clone)]
pub struct GetDefinitionArgs {
    #[arg(
        short = 'i',
        long,
        help = "Definition ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub definition_id: Uuid,

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
pub struct UpdateDefinitionArgs {
    #[arg(
        short = 'i',
        long,
        help = "Definition ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub definition_id: Uuid,

    #[arg(short, long, help = "Definition name")]
    pub name: String,

    #[arg(short = 't', long, help = "Asset type, e.g. Crah")]
    pub asset_type: AssetTypes,

    #[arg(short = 'D', long, help = "Optional definition description")]
    pub description: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct DeleteDefinitionArgs {
    #[arg(
        short = 'i',
        long,
        help = "Definition ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub definition_id: Uuid,
}

#[derive(Args, Debug, Clone)]
pub struct DeleteSensorDefinitionArgs {
    #[arg(
        short,
        long,
        help = "Definition ID. It must be a valid GUID/UUID, e.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub definition_id: Uuid,

    #[arg(
        short,
        long,
        help = "Sensor ID. It must be a valid GUID/UUID, e.g. 61d2dcf3-65f0-4f84-89d4-3110a1e1f196"
    )]
    pub sensor_id: Uuid,
}
