use csv::Writer;
use log::{LevelFilter, error};
use serde::Serialize;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::{MAIN_SEPARATOR_STR, Path};

use crate::hyperview::api_constants::{
    BUSWAY_TAPOFF_API_PREFIX, PDU_RPP_BREAKERS_API_PREFIX, RACK_PDU_OUTLETS_API_PREFIX,
};
use crate::hyperview::asset_power_api_functions::{
    add_power_association_async, bulk_add_power_association_async,
};

use super::{
    api_constants::{
        ASSET_PROPERTY_ASSET_TAG, ASSET_PROPERTY_DESIGN_VALUE, ASSET_PROPERTY_SERIAL_NUMBER,
    },
    app_errors::AppError,
    asset_alarm_events_functions::{list_alarm_events_async, manage_asset_alarm_events_async},
    asset_api_functions::{
        add_rack_accessory_async, bulk_add_rack_accessory_async, bulk_update_asset_location_async,
        bulk_update_asset_name_async, bulk_update_ports_async, list_any_of_async,
        list_asset_ports_async, search_assets_async, update_asset_location_async,
        update_asset_name_by_id_async,
    },
    asset_power_api_functions::get_power_provider_components_async,
    asset_properties_api_functions::{
        bulk_update_asset_property_async, get_asset_property_list_async,
        update_asset_property_async,
    },
    asset_sensor_api_functions::{bulk_update_asset_sensor_async, get_asset_sensor_list_async},
    cli_data::{AppArgsSubcommands, AppConfig, DebugLevels, OutputOptions},
    custom_asset_properties_api_functions::{
        bulk_update_custom_property_by_name_async, get_custom_asset_property_list_async,
        update_custom_property_by_name_async,
    },
};

pub fn get_config_path() -> String {
    let home_path = dirs::home_dir().expect("Error: Home directory not found");

    format!(
        "{}{}.hyperview{}hyperview.toml",
        home_path.to_str().unwrap(),
        MAIN_SEPARATOR_STR,
        MAIN_SEPARATOR_STR
    )
}

pub fn get_debug_filter(debug_level: DebugLevels) -> LevelFilter {
    match debug_level {
        DebugLevels::Error => LevelFilter::Error,
        DebugLevels::Warn => LevelFilter::Warn,
        DebugLevels::Debug => LevelFilter::Debug,
        DebugLevels::Trace => LevelFilter::Trace,
        DebugLevels::Info => LevelFilter::Info,
    }
}

pub fn write_output<T: Serialize>(filename: String, object_list: Vec<T>) -> color_eyre::Result<()> {
    let mut writer = Writer::from_path(filename)?;

    for object in object_list {
        writer.serialize(object)?;
    }

    Ok(())
}

pub fn handle_output_choice<T: Display + Serialize>(
    output_type: OutputOptions,
    filename: Option<&String>,
    resp: Vec<T>,
) -> color_eyre::Result<()> {
    let mut outfile = String::new();

    if let Some(f) = filename {
        if Path::new(f).exists() {
            error!("Specified file already exists. exiting ...");
            return Err(AppError::FileExists.into());
        }

        f.clone_into(&mut outfile);
    }

    match output_type {
        OutputOptions::CsvFile => {
            if filename.is_none() {
                error!("Must provide a filename. exiting ...");
                return Err(AppError::NoOutputFilename.into());
            }

            write_output(outfile, resp)?;
        }

        OutputOptions::Json => {
            if filename.is_none() {
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
                return Ok(());
            }

            let file_handle = File::create(outfile)?;
            serde_json::to_writer_pretty(file_handle, &resp)?;
        }

        OutputOptions::Record => {
            if filename.is_none() {
                for (i, s) in resp.iter().enumerate() {
                    println!("---- [{i}] ----\n{s}\n");
                }
                return Ok(());
            }

            let mut file_handle = File::create(outfile)?;
            for (i, s) in resp.iter().enumerate() {
                write!(file_handle, "---- [{i}] ----\n{s}\n\n")?;
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
pub async fn route_command_async(
    command: AppArgsSubcommands,
    config: AppConfig,
    auth_header: String,
    req: reqwest::Client,
) -> color_eyre::Result<()> {
    match command {
        AppArgsSubcommands::ListAssetProperties(options) => {
            let resp =
                get_asset_property_list_async(&config, &req, &auth_header, options.id).await?;
            handle_output_choice(options.output_type, options.filename.as_ref(), resp)?;
        }

        AppArgsSubcommands::ListCustomAssetProperties(options) => {
            let resp =
                get_custom_asset_property_list_async(&config, &req, &auth_header, options.id)
                    .await?;

            handle_output_choice(options.output_type, options.filename.as_ref(), resp)?;
        }

        AppArgsSubcommands::SearchAssets(options) => {
            let resp = search_assets_async(&config, &req, &auth_header, options.clone()).await?;

            handle_output_choice(options.output_type, options.filename.as_ref(), resp)?;
        }

        AppArgsSubcommands::ListAnyOf(options) => {
            let resp = list_any_of_async(&config, &req, &auth_header, options.clone()).await?;

            handle_output_choice(options.output_type, options.filename.as_ref(), resp)?;
        }

        AppArgsSubcommands::UpdateAssetName(options) => {
            update_asset_name_by_id_async(
                &config,
                &req,
                &auth_header,
                options.id,
                options.new_name.clone(),
            )
            .await?;
        }

        AppArgsSubcommands::BulkUpdateAssetName(options) => {
            bulk_update_asset_name_async(&config, &req, &auth_header, options.filename.clone())
                .await?;
        }

        AppArgsSubcommands::UpdateAssetLocation(options) => {
            update_asset_location_async(&config, &req, &auth_header, options.clone()).await?;
        }

        AppArgsSubcommands::BulkUpdateAssetLocation(options) => {
            bulk_update_asset_location_async(&config, &req, &auth_header, options.filename.clone())
                .await?;
        }

        AppArgsSubcommands::UpdateAssetSerialNumber(options) => {
            update_asset_property_async(
                &config,
                &req,
                &auth_header,
                options.id,
                options.new_value.clone(),
                ASSET_PROPERTY_SERIAL_NUMBER.to_string(),
            )
            .await?;
        }

        AppArgsSubcommands::BulkUpdateAssetSerialNumber(options) => {
            bulk_update_asset_property_async(
                &config,
                &req,
                &auth_header,
                options.filename.clone(),
                ASSET_PROPERTY_SERIAL_NUMBER.to_string(),
            )
            .await?;
        }

        AppArgsSubcommands::UpdateAssetTag(options) => {
            update_asset_property_async(
                &config,
                &req,
                &auth_header,
                options.id,
                options.new_value.clone(),
                ASSET_PROPERTY_ASSET_TAG.to_string(),
            )
            .await?;
        }

        AppArgsSubcommands::BulkUpdateAssetTag(options) => {
            bulk_update_asset_property_async(
                &config,
                &req,
                &auth_header,
                options.filename.clone(),
                ASSET_PROPERTY_ASSET_TAG.to_string(),
            )
            .await?;
        }

        AppArgsSubcommands::UpdatePowerDesignValue(options) => {
            update_asset_property_async(
                &config,
                &req,
                &auth_header,
                options.id,
                options.new_value.clone(),
                ASSET_PROPERTY_DESIGN_VALUE.to_string(),
            )
            .await?;
        }

        AppArgsSubcommands::BulkUpdatePowerDesignValue(options) => {
            bulk_update_asset_property_async(
                &config,
                &req,
                &auth_header,
                options.filename.clone(),
                ASSET_PROPERTY_DESIGN_VALUE.to_string(),
            )
            .await?;
        }

        AppArgsSubcommands::ListAssetPorts(options) => {
            let resp = list_asset_ports_async(&config, &req, &auth_header, options.clone()).await?;

            handle_output_choice(options.output_type, options.filename.as_ref(), resp)?;
        }

        AppArgsSubcommands::BulkUpdatePatchPanelPorts(options) => {
            bulk_update_ports_async(&config, &req, &auth_header, options.filename.clone(), true)
                .await?;
        }

        AppArgsSubcommands::BulkUpdateAssetPorts(options) => {
            bulk_update_ports_async(&config, &req, &auth_header, options.filename.clone(), false)
                .await?;
        }

        AppArgsSubcommands::UpdateCustomAssetProperty(options) => {
            update_custom_property_by_name_async(
                &config,
                &req,
                &auth_header,
                options.id,
                options.custom_property.clone(),
                options.new_custom_property_value.clone(),
            )
            .await?;
        }

        AppArgsSubcommands::BulkUpdateCustomAssetProperty(options) => {
            bulk_update_custom_property_by_name_async(
                &config,
                &req,
                &auth_header,
                options.filename.clone(),
            )
            .await?;
        }

        AppArgsSubcommands::ListAlarms(options) => {
            let resp = list_alarm_events_async(
                &config,
                &req,
                &auth_header,
                options.skip,
                options.limit,
                options.alarm_filter,
            )
            .await?;

            handle_output_choice(options.output_type, options.filename.as_ref(), resp.data)?;
        }

        AppArgsSubcommands::ManageAlarms(options) => {
            manage_asset_alarm_events_async(
                &config,
                &req,
                &auth_header,
                options.filename.clone(),
                options.manage_action,
            )
            .await?;
        }

        AppArgsSubcommands::AddRackAccessory(options) => {
            add_rack_accessory_async(
                &config,
                &req,
                &auth_header,
                &options.id,
                &options.panel_type,
                &options.rack_side,
                options.rack_u_location,
            )
            .await?;
        }

        AppArgsSubcommands::BulkAddRackAccessory(options) => {
            bulk_add_rack_accessory_async(&config, &req, &auth_header, &options.filename).await?;
        }

        AppArgsSubcommands::ListAssetSensors(options) => {
            let resp = get_asset_sensor_list_async(&config, &req, &auth_header, options.id).await?;

            handle_output_choice(options.output_type, options.filename.as_ref(), resp)?;
        }

        AppArgsSubcommands::BulkUpdateAssetSensor(options) => {
            bulk_update_asset_sensor_async(&config, &req, &auth_header, &options.filename).await?;
        }

        AppArgsSubcommands::ListRackPduOutlets(options) => {
            let resp = get_power_provider_components_async(
                &config,
                &req,
                &auth_header,
                RACK_PDU_OUTLETS_API_PREFIX,
                options.id,
            )
            .await?;

            handle_output_choice(options.output_type, options.filename.as_ref(), resp)?;
        }

        AppArgsSubcommands::ListBuswayTapoffs(options) => {
            let resp = get_power_provider_components_async(
                &config,
                &req,
                &auth_header,
                BUSWAY_TAPOFF_API_PREFIX,
                options.id,
            )
            .await?;

            handle_output_choice(options.output_type, options.filename.as_ref(), resp)?;
        }

        AppArgsSubcommands::ListPduRppBreakers(options) => {
            let resp = get_power_provider_components_async(
                &config,
                &req,
                &auth_header,
                PDU_RPP_BREAKERS_API_PREFIX,
                options.id,
            )
            .await?;

            handle_output_choice(options.output_type, options.filename.as_ref(), resp)?;
        }

        AppArgsSubcommands::AddPowerAssociation(options) => {
            add_power_association_async(
                &config,
                &req,
                &auth_header,
                options.power_consuming_asset_id,
                options.power_providing_asset_id,
            )
            .await?;
        }

        AppArgsSubcommands::BulkAddPowerAssociation(options) => {
            bulk_add_power_association_async(&config, &req, &auth_header, &options.filename)
                .await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppConfig;
    use std::fs::File;
    use std::io::{BufReader, Read, Write};
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_config_path() {
        let config_path = get_config_path();
        let home_path = dirs::home_dir().unwrap();
        let expected_path = format!(
            "{}{}.hyperview{}hyperview.toml",
            home_path.to_str().unwrap(),
            MAIN_SEPARATOR_STR,
            MAIN_SEPARATOR_STR
        );

        assert_eq!(config_path, expected_path);
    }

    #[test]
    fn test_app_config_loading() {
        let mut tmp_file = NamedTempFile::new().unwrap();

        write!(
            tmp_file,
            r#"client_id = "test_id"
            client_secret = "test_secret"
            scope = "test_scope"
            auth_url = "https://test_auth_url"
            token_url = "https://test_token_url"
            instance_url = "https://test_instance_url"
            "#
        )
        .unwrap();

        let config: AppConfig = confy::load_path(tmp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.client_id, "test_id");
        assert_eq!(config.client_secret, "test_secret");
        assert_eq!(config.scope, "test_scope");
        assert_eq!(config.auth_url, "https://test_auth_url");
        assert_eq!(config.token_url, "https://test_token_url");
        assert_eq!(config.instance_url, "https://test_instance_url");
    }

    #[test]
    fn test_get_debug_filter() {
        assert_eq!(get_debug_filter(DebugLevels::Error), LevelFilter::Error);
        assert_eq!(get_debug_filter(DebugLevels::Warn), LevelFilter::Warn);
        assert_eq!(get_debug_filter(DebugLevels::Debug), LevelFilter::Debug);
        assert_eq!(get_debug_filter(DebugLevels::Info), LevelFilter::Info);
        assert_eq!(get_debug_filter(DebugLevels::Trace), LevelFilter::Trace);
    }

    #[test]
    fn test_write_output() {
        // Create test data
        let data = vec![1, 2, 3, 4, 5];

        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let temp_file_path = temp_file.path().to_str().unwrap().to_string();

        // Call the function with the test data and the temporary file path
        let result = write_output(temp_file_path.clone(), data);
        assert!(result.is_ok());

        // Read back the file
        let file = File::open(temp_file_path).unwrap();
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents).unwrap();

        assert_eq!("1\n2\n3\n4\n5\n", contents);
    }

    #[test]
    fn test_handle_output_choice_no_filename() {
        let output_type = OutputOptions::CsvFile;
        let filename = None;
        let resp: Vec<i32> = vec![1, 2, 3, 4, 5];

        match handle_output_choice(output_type, filename, resp) {
            Err(e) => assert_eq!(e.to_string(), AppError::NoOutputFilename.to_string()),
            _ => panic!("Expected Err, but got Ok"),
        }
    }

    #[test]
    fn test_handle_output_choice_file_exists() {
        let output_type = OutputOptions::CsvFile;
        let temp_file = NamedTempFile::new().unwrap();
        let filename = Some(temp_file.path().to_str().unwrap().to_string());
        let resp: Vec<i32> = vec![1, 2, 3, 4, 5];

        match handle_output_choice(output_type, filename.as_ref(), resp) {
            Err(e) => assert_eq!(e.to_string(), AppError::FileExists.to_string()),
            _ => panic!("Expected Err, but got Ok"),
        }
    }

    #[test]
    fn test_handle_output_choice_write_output() {
        let output_type = OutputOptions::CsvFile;
        let temp_file = NamedTempFile::new().unwrap();
        let temp_file_path = temp_file.path().to_str().unwrap().to_string();
        let filename = temp_file_path.clone() + "_new";
        let resp: Vec<i32> = vec![1, 2, 3, 4, 5];

        let result = handle_output_choice(output_type, Some(&filename), resp);
        assert!(result.is_ok());

        let mut file = File::open(filename).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        // Check the contents of the file
        assert_eq!(contents, "1\n2\n3\n4\n5\n");
    }
}
