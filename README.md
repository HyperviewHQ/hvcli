# Hyperview Asset Tool

> [!NOTE]  
> This project is under active development. 

Hyperview asset tool (hvat) is a command line program to interact with data within Hyperview.

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

```console
client_id = 'c33472d0-c66b-4659-a8f8-73c289ba4dbe'
client_secret = '2c239e21-f81b-472b-a8c3-82296d5f250d'
scope = 'HyperviewManagerApi'
auth_url = 'https://example.hyperviewhq.com/connect/authorize'
token_url = 'https://example.hyperviewhq.com/connect/token'
instance_url = 'https://example.hyperviewhq.com'
```

# Usage

```console
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
  help                           Print this message or the help of the given subcommand(s)

Options:
  -d, --debug-level <DEBUG_LEVEL>  Debug level [default: error] [possible values: error, warn, debug, info, trace]
  -h, --help                       Print help
  -V, --version                    Print version
```

## Subcommands

- `list-asset-properties` will list all set, inherited and available properties for an asset id.
- `list-custom-asset-properties` will list all set and available custom properties for an asset id.
- `search-assets` is the main entry point for the application and it provides various methods to search for
assets in Hyperview.
- `update-asset-name` will update the name of a single asset.
- `bulk-update-asset-name` will update multiple assets using a CSV input file. Example input is in the example_input folder.
- `update-asset-location` will update the location of a single asset.
- `bulk-update-asset-location` will update multiple assets using a CSV input file. Example input is in the example_input folder.
- `list-asset-ports` will List asset ports.
- `bulk-update-patch-panel-ports`  will bulk update *patch panel* port names using a CSV input file. Example input is in the example_input folder.
- `bulk-update-asset-ports` will bulk update *other asset* port names, E.g. a network switch. Example input is in the example_input folder.

Use `--help` to explore the various options available within each sub command.

A master debug level can be set to troubleshoot issues using `-d` or `--debug-level`.

Where applicable some subcommands allow the user to set to output to `record`, `json` or `csv-file`. 

## Examples

### Search by property (JSON output)

```console
hvat search-assets -P serialNumber=SERIALNUMBEREXAMPLE1234 -o json
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

```console
hvat search-assets -p "UpsExampl*"
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

```console
hvat search-assets -p "UpsExample" --location-path "All/Simulated SNMP Devices/" -M "Liebert" -o json
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
