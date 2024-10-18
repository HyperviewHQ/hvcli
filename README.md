# Hyperview Asset Tool

> [!NOTE]
> This project is under active development.

Asset tool (hvat) is a command line program to interact with data within Hyperview.

# Download

To use this tool simply download a pre-built binary from the [Releases](https://github.com/HyperviewHQ/asset_tool/releases) section.

# Build from source

## Linux
If you are experimenting with the code on a single platform the usual `cargo build` and `cargo build --release` will work. However, if the desire is to build a binary that can run on multiple Linux distributions it is recommended to install the `x86_64-unknown-linux-musl` target and to build a statically-linked binary to avoid dependency problems. 

The command to build a statically-linked version is:

```console
PKG_CONFIG_SYSROOT_DIR=/ RUSTFLAGS='-C target-feature=+crt-static' cargo build --target x86_64-unknown-linux-musl --release
```

## Windows & MacOS
The usual `cargo build` and `cargo build --release` will work. 

# Configuration
A valid Hyperview API client must be used. The API client must have the appropriate access. The configuration file must be placed in `$HOME/.hyperview/hyperview.toml`

## Example

```toml
client_id = 'c33472d0-c66b-4659-a8f8-73c289ba4dbe'
client_secret = '2c239e21-f81b-472b-a8c3-82296d5f250d'
scope = 'HyperviewManagerApi'
auth_url = 'https://example.hyperviewhq.com/connect/authorize'
token_url = 'https://example.hyperviewhq.com/connect/token'
instance_url = 'https://example.hyperviewhq.com'
```

# Usage

```bash
$  hvat --help
A command line interface to interact with asset data stored in Hyperview

Usage: hvat [OPTIONS] <COMMAND>

Commands:
  list-asset-properties          List asset properties
  list-custom-asset-properties   List custom asset properties
  search-assets                  Search assets
  update-asset-name              Update asset name
  bulk-update-asset-name         Bulk update asset name
  update-asset-location          Update asset location
  bulk-update-asset-location     Bulk update asset location
  list-asset-ports               List asset ports
  bulk-update-patch-panel-ports  Bulk update patch panel port names
  bulk-update-asset-ports        Bulk update asset port names
  list-alarms                    List alarm events
  manage-alarms                  Acknowledge or close alarm events using CSV file output from the list-alarms command
  help                           Print this message or the help of the given subcommand(s)

Options:
  -d, --debug-level <DEBUG_LEVEL>  Debug level [default: error] [possible values: error, warn, debug, info, trace]
  -h, --help                       Print help
  -V, --version                    Print version
```

## Subcommands

### 1. list-asset-properties
This subcommand will list all _set_, _inherited_ and _available_ properties for an asset identified by its unique id.

### 2. list-custom-asset-properties
This subcommand will list all _set_ and _available_ custom properties for an asset  identified by its unique id.

### 3. search-assets
This subcommand is the main entry point for the application and it provides various methods to search for assets in Hyperview.

### 4. update-asset-name
This subcommand will update the display name of a single asset.

### 5. bulk-update-asset-name
This subcommand will update multiple assets using a _CSV_ input file. Example input is in the **example_input** folder in
this repo.

### 6. update-asset-location
This subcommand will update the location of a single asset.

### 7. bulk-update-asset-location
This subcommand will update multiple assets using a _CSV_ input file. Example input is in the **example_input** folder.

### 8. list-asset-ports
This subcommand will List asset physical network ports.

### 9. bulk-update-patch-panel-ports
This subcommand will bulk update **patch panel** physical network port names using a _CSV_ input file. Example input is in the **example_input** folder.

### 10. bulk-update-asset-ports
This subcommand will bulk update **other asset** physical network port names, E.g. a network switch. Example input is in the example_input folder.

### 11. list-alarms
This subcommand will list alarm events. By default it will list _active_ events and it can also list _unacknowledged_
events via a command line toggle.

### 12. manage-alarms
This subcommand will _acknowledge_ or _close_ alarm events using _CSV_ file output from the list-alarms command. By
default this command will close event and it can also acknowledge events via a command line toggle.

### Help
Use `--help` to explore the various options available within the main command and each subcommand.

#### Subcommand help examples

```bash
$ hvat list-alarms --help
List alarm events

Usage: hvat list-alarms [OPTIONS]

Options:
  -s, --skip <SKIP>                  Number of records to skip (0 -> 99999), e.g. 100 [default: 0]
  -l, --limit <LIMIT>                Record limit (1 -> 100000), e.g. 100 [default: 100]
  -a, --alarm-filter <ALARM_FILTER>  Asset alarm event filter option, e.g. active [default: active] [possible values: unacknowledged, active]
  -o, --output-type <OUTPUT_TYPE>    Output type, e.g. csv-file [default: record] [possible values: csv-file, json, record]
  -f, --filename <FILENAME>          Output filename, e.g. output.csv
  -h, --help                         Print help
  -V, --version                      Print version

$  hvat manage-alarms --help
Acknowledge or close alarm events using CSV file output from the list-alarms command

Usage: hvat manage-alarms [OPTIONS] --filename <FILENAME>

Options:
  -f, --filename <FILENAME>            Input filename, e.g. port_name_update.csv
  -m, --manage-action <MANAGE_ACTION>  Manage action to use, e.g. close [default: close] [possible values: acknowledge, close]
  -h, --help                           Print help
  -V, --version                        Print version
```

For troubleshooting, a master debug level can be set to troubleshoot issues using `-d` or `--debug-level`.

Some subcommands allow the user to set to output to `record`, `json` or `csv-file`. Refer to the command help for more
information.

## Output examples

### Search by property (JSON output)

```bash
$ hvat search-assets -P serialNumber=SERIALNUMBEREXAMPLE1234 -o json
[
  {
    "id": "\"58af63dc-1e9e-4b8b-b2b7-e0451aaca8fb\"",
    "name": "\"UpsExample\"",
    "assetLifecycleState": "\"Active\"",
    "assetTypeId": "\"Ups\"",
    "manufacturerId": "\"cd85e92d-869c-470a-a3ba-df8b2b7196e3\"",
    "manufacturerName": "\"Liebert\"",
    "monitoringState": "\"On\"",
    "parentId": "\"9a877a93-1f21-4895-a078-5c67f531ea0b\"",
    "parentName": "\"Simulated SNMP Devices\"",
    "productId": "\"aedbd4b9-06ae-4768-ba4a-64847b60d334\"",
    "productName": "\"eXM\"",
    "status": "\"Normal\"",
    "path": "\"All/Simulated SNMP Devices/UpsExample\"",
    "serialNumber": "[\"SERIALNUMBEREXAMPLE1234\"]"
  }
]

```

### Search by text pattern (record output)

```bash
$ hvat search-assets -p "UpsExampl*"
---- [0] ----
id: "58af63dc-1e9e-4b8b-b2b7-e0451aaca8fb"
name: "UpsExample"
asset_lifecycle_state: "Active"
asset_type_id: "Ups"
manufacturer_id: "cd85e92d-869c-470a-a3ba-df8b2b7196e3"
manufacturer_name: "Liebert"
monitoring_state: "On"
parent_id: "9a877a93-1f21-4895-a078-5c67f531ea0b"
parent_name: "Simulated SNMP Devices"
product_id: "aedbd4b9-06ae-4768-ba4a-64847b60d334"
product_name: "eXM"
status: "Normal"
path: "All/Simulated SNMP Devices/UpsExample"
serial_number: ["SERIALNUMBEREXAMPLE1234"]
```

### Combination search (JSON output)

```bash
$ hvat search-assets -p "UpsExample" --location-path "All/Simulated SNMP Devices/" -M "Liebert" -o json
[
  {
    "id": "\"58af63dc-1e9e-4b8b-b2b7-e0451aaca8fb\"",
    "name": "\"UpsExample\"",
    "assetLifecycleState": "\"Active\"",
    "assetTypeId": "\"Ups\"",
    "manufacturerId": "\"cd85e92d-869c-470a-a3ba-df8b2b7196e3\"",
    "manufacturerName": "\"Liebert\"",
    "monitoringState": "\"On\"",
    "parentId": "\"9a877a93-1f21-4895-a078-5c67f531ea0b\"",
    "parentName": "\"Simulated SNMP Devices\"",
    "productId": "\"aedbd4b9-06ae-4768-ba4a-64847b60d334\"",
    "productName": "\"eXM\"",
    "status": "\"Normal\"",
    "path": "\"All/Simulated SNMP Devices/UpsExample\"",
    "serialNumber": "[\"SERIALNUMBEREXAMPLE1234\"]"
  }
]
```
