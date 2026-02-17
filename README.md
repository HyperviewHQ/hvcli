# Hyperview CLI

> [!NOTE]
> This project is under active development. Please remember to check for new release.

Hyperview CLI (hvcli) is a command line program to interact with data within Hyperview. 

** Important Reminders **

> - _Powerful Capabilities_: This tool can make changes to data in Hyperview. Please take the time to familiarize yourself with its features.
> - _Check Twice, Act Once_: Ensure that all your inputs are accurate. A small oversight can lead to unintended consequences.
> - _Test with a small sample first_: Test and verify bulk changes with a small sample before proceeding to make big changes.

Your success is important to us! Enjoy using the Hyperview CLI (hvcli), and remember to proceed with caution! 

# Download

To use this tool simply download a pre-built binary from the [Releases](https://github.com/HyperviewHQ/hvcli/releases) section.

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
$ hvcli --help
Usage: hvcli [OPTIONS] <COMMAND>

Commands:
  list-asset-properties              List asset properties
  list-custom-asset-properties       List asset custom properties
  search-assets                      Search assets
  list-any-of                        List assets matching any of the provided property values
  update-asset-name                  Update asset name
  bulk-update-asset-name             Bulk update asset name
  update-asset-location              Update asset location
  bulk-update-asset-location         Bulk update asset location
  update-asset-serial-number         Update asset serial number. This applies to manually created assets and assets discovered without a serial number
  bulk-update-asset-serial-number    Bulk update asset serial number. This applies to manually created assets and assets discovered without a serial number
  update-asset-tag                   Update asset "asset tag" Property
  bulk-update-asset-tag              Bulk update asset "asset tag" Property
  update-power-design-value          Update asset power "design value" Property Applies to Rack and Location asset types
  bulk-update-power-design-value     Bulk update asset power "design value" Property Applies to Rack and Location asset types
  list-asset-ports                   List asset ports
  bulk-update-patch-panel-ports      Bulk update patch panel port names
  bulk-update-asset-ports            Bulk update asset port names
  update-custom-asset-property       Update asset custom property
  bulk-update-custom-asset-property  Bulk  update asset custom property
  list-alarms                        List alarm events
  manage-alarms                      Acknowledge or close alarm events using the CSV output from the list-alarms command
  add-rack-accessory                 Add a blanking panel or cable management panel to a rack
  bulk-add-rack-accessory            Bulk add a blanking panel or cable management panel to a rack
  list-asset-sensors                 List asset sensors
  bulk-update-asset-sensor           Bulk update asset sensor name and access policy.
                                     IMPORTANT: Keep access policy field empty to maintain original and only change the name.
                                     Use a NIL UUID (00000000-0000-0000-0000-000000000000) to reset to parent access policy
  list-rack-pdu-outlets              List Rack PDU outlets
  list-busway-tapoffs                List busway tap-offs
  list-pdu-rpp-breakers              List PDU/RPP Breakers
  add-power-association              Add power association
  bulk-add-power-association         Bulk add power power association
  help                               Print this message or the help of the given subcommand(s)

Options:
  -d, --debug-level <DEBUG_LEVEL>  Debug level [default: error] [possible values: error, warn, info, debug, trace]
  -h, --help                       Print help
  -V, --version                    Print version
```

## Subcommands

### 1. list-asset-properties

This subcommand lists all available properties for an asset identified by its unique id.

### 2. list-custom-asset-properties

This subcommand lists all available custom properties for an asset  identified by its unique id.

### 3. search-assets

This subcommand provides methods for searching for assets in Hyperview.

### 4. list-any-of

This subcommand will list assets that match a specific set of property values. For example, a list of serial numbers. Please note that the matches are exact.

### 5. update-asset-name

This subcommand will update the display name of a single asset.

### 6. bulk-update-asset-name

This subcommand updates multiple assets from a CSV file. Example data is in the example_input folder.

### 7. update-asset-location

This subcommand will update the location of a single asset.

### 8. bulk-update-asset-location

This subcommand updates the location of multiple assets from a CSV file. Example Data is in the example_input folder.

### 9. update-asset-serial-number

This subcommand updates the asset serial number. Applies to manually created assets and assets discovered without a serial number.

### 10. bulk-update-asset-serial-number

This subcommand updates the serial numbers of multiple assets from a CSV file. Example data is in the example_input folder.

### 11. update-asset-tag

Update asset “asset tag” Property.

### 12. bulk-update-asset-tag

This subcommand updates the asset tag of multiple assets from a CSV file. Example data is in the example_input folder.

### 13. update-power-design-value

Update the asset power “design value” property. This applies to Rack and Location asset types.

### 14. bulk-update-power-design-value

This subcommand updates the asset power “design value” property of multiple assets from a CSV file. This applies to Rack and Location asset types. Example data is in the example_input folder.

### 15. list-asset-ports

This subcommand will list asset physical network ports.

### 16. bulk-update-patch-panel-ports

This subcommand bulk updates the physical network port names of patch panels from a CSV file. Example data is in the example_input folder.

### 17. bulk-update-asset-ports

This subcommand updates non-patch panel asset physical network port names, E.g. a network switch. Example data is in the example_input folder.

### 18. update-custom-asset-property

This subcommand will update the value of an asset custom property.

### 19. bulk-update-custom-asset-property

This subcommand updates the custom property value of multiple assets using a CSV file. Example data is in the example_input folder.

### 22. list-alarms

This subcommand lists alarm events. By default, it will list active events. It can also list unacknowledged events via a command-line option.

### 21. manage-alarms

This subcommand either acknowledges or closes alarm events using a CSV file generated by the list-alarms command. By default, this command closes the events; it can also acknowledge them via a command-line option.

### 22. add-rack-accessory

Add a blanking panel or cable management panel to a rack.

### 23. bulk-add-rack-accessory

Add blanking panels or cable management panels to multiple racks using a CSV file. Example data is in the example_input folder.

### 24. list-asset-sensors

List asset sensors for an asset  identified by its unique id.

### 25. bulk-update-asset-sensor

Bulk update asset sensor name and access policy. IMPORTANT: Keep the access policy field empty to maintain the original and only change the name. Use a NIL UUID (00000000-0000-0000-0000-000000000000) to reset to the parent access policy. Example data is in the example_input folder.

### 26. list-rack-pdu-outlets

List Rack PDU outlets for an asset  identified by its unique id.

### 27. list-busway-tapoffs

List busway tap-offs for an asset  identified by its unique id.

### 28. list-pdu-rpp-breakers

List PDU/RPP Breakers for an asset  identified by its unique id.

### 29. add-power-association

Add power association. Note that associations are asset-to-asset. For example, when associating with a specific outlet or tap-off, you need its id.

### 30. bulk-add-power-association

Bulk add power associations between assets using a CSV. Example data is in the example_input folder.

> [!NOTE]
> Use --help to explore the various options available within the main command and each subcommand.

#### Subcommand help examples

```bash
$ hvcli list-alarms --help
List alarm events

Usage: hvcli list-alarms [OPTIONS]

Options:
  -s, --skip <SKIP>                  Number of records to skip (0 -> 99999), e.g. 100 [default: 0]
  -l, --limit <LIMIT>                Record limit (1 -> 100000), e.g. 100 [default: 100]
  -a, --alarm-filter <ALARM_FILTER>  Asset alarm event filter option, e.g. active [default: active] [possible values: unacknowledged, active]
  -o, --output-type <OUTPUT_TYPE>    Output type, e.g. csv-file [default: record] [possible values: csv-file, json, record]
  -f, --filename <FILENAME>          Output filename, e.g. output.csv
  -h, --help                         Print help
  -V, --version                      Print version

$  hvcli manage-alarms --help
Acknowledge or close alarm events using CSV file output from the list-alarms command

Usage: hvcli manage-alarms [OPTIONS] --filename <FILENAME>

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
$ hvcli search-assets -P serialNumber=SERIALNUMBEREXAMPLE1234 -o json
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
$ hvcli search-assets -p "UpsExampl*"
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
$ hvcli search-assets -p "UpsExample" --location-path "All/Simulated SNMP Devices/" -M "Liebert" -o json
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

# Building from source

## Linux, Windows and MacOS

### Debug build 

```
cargo build
```

The binary will be under `target/debug/hvcli`.

### Release build

```
cargo build --release
```

The binary will be under `target/release/hvcli`.

## Linux static binary

Install the **x86_64-unknown-linux-musl** target and run the command to build a statically-linked version:

```
PKG_CONFIG_SYSROOT_DIR=/ RUSTFLAGS='-C target-feature=+crt-static' cargo build --target x86_64-unknown-linux-musl --release
```

## Docker

```
docker build --tag hvcli:latest -f docker/Dockerfile .
```

### Running the Docker image

To run the docker image generated you need to: 

1. Map the application configuration directory to the container.
2. Optional, map an output folder to the container

#### Example

Assuming the username is **albert**

```
docker run -v /home/albert/.hyperview:/root/.hyperview hvcli search-assets

```

If you are planing to output to csv

```
docker run -v /home/albert/.hyperview:/root/.hyperview -v /tmp:/output hvcli search-assets -o csv-file -f /output/assets.csv
```
