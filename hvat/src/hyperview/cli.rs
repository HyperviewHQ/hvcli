use clap::{value_parser, Args, Parser, Subcommand};
use color_eyre::Result;
use csv::Writer;
use log::{error, LevelFilter};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::{Path, MAIN_SEPARATOR_STR};

use crate::{hyperview::app_errors::AppError, ASSET_TYPES};

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
    ListAssetProperties(ListAssetPropertiesArgs),
}

#[derive(Args, Debug)]
pub struct ListAssetsArgs {
    #[arg(
        short = 't',
        long,
        help = "Asset type. e.g. Crah",
        value_parser(ASSET_TYPES)
    )]
    pub asset_type: String,

    #[arg(
        short,
        long,
        help = "Number of records to skip (0 -> 99999). e.g. 100", 
        default_value = "0", value_parser(value_parser!(u32).range(0..100000))
    )]
    pub skip: u32,

    #[arg(
        short,
        long,
        help = "Record limit (1 -> 1000). e.g. 100", 
        default_value = "100", 
        value_parser(value_parser!(u32).range(1..1001))
    )]
    pub limit: u32,

    #[arg(
        short,
        long,
        help = "Output type. E.g. csv",
        default_value = "record",
        value_parser(["record", "csv"])
    )]
    pub output_type: String,

    #[arg(short, long, help = "output filename. E.g. output.csv")]
    pub filename: Option<String>,
}

#[derive(Args, Debug)]
pub struct ByIdArgs {
    #[arg(
        short,
        long,
        help = "Primary ID. It must be a valid GUID/UUID, E.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub id: String,
}

#[derive(Args, Debug)]
pub struct ListAssetPropertiesArgs {
    #[arg(
        short,
        long,
        help = "Asset ID. It must be a valid GUID/UUID, E.g. 2776f6c6-78da-4087-ab9e-e7b52275cd9e"
    )]
    pub id: String,

    #[arg(
        short,
        long,
        help = "Output type. E.g. csv",
        default_value = "record",
        value_parser(["record", "csv"])
    )]
    pub output_type: String,

    #[arg(short, long, help = "output filename. E.g. output.csv")]
    pub filename: Option<String>,
}

pub fn get_config_path() -> String {
    let home_path = dirs::home_dir().expect("Error: Home directory not found");

    format!(
        "{}{}.hyperview{}hyperview.toml",
        home_path.to_str().unwrap(),
        MAIN_SEPARATOR_STR,
        MAIN_SEPARATOR_STR
    )
}

pub fn get_debug_filter(debug_level: &str) -> LevelFilter {
    match debug_level {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    }
}

pub fn write_output<T: Serialize>(filename: String, object_list: Vec<T>) -> Result<()> {
    let mut writer = Writer::from_path(filename)?;

    for object in object_list {
        writer.serialize(object)?;
    }

    Ok(())
}

pub fn handle_output_choice<T: Display + Serialize>(
    output_type: String,
    filename: Option<String>,
    resp: Vec<T>,
) -> Result<()> {
    if output_type == *"csv" {
        if filename.is_none() {
            error!("Must provide a filename. exiting ...");
            return Err(AppError::NoOutputFilename.into());
        } else if let Some(f) = filename {
            if Path::new(&f).exists() {
                error!("Specified file already exists. exiting ...");
                return Err(AppError::FileExists.into());
            }

            write_output(f, resp)?;
        }
    } else {
        for (i, s) in resp.iter().enumerate() {
            println!("---- [{}] ----", i);
            println!("{}\n", s);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(get_debug_filter("error"), LevelFilter::Error);
        assert_eq!(get_debug_filter("warn"), LevelFilter::Warn);
        assert_eq!(get_debug_filter("debug"), LevelFilter::Debug);
        assert_eq!(get_debug_filter("info"), LevelFilter::Info);
        assert_eq!(get_debug_filter("trace"), LevelFilter::Trace);
        assert_eq!(get_debug_filter("unknown"), LevelFilter::Info);
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
        let output_type = "csv".to_string();
        let filename = None;
        let resp: Vec<i32> = vec![1, 2, 3, 4, 5];

        match handle_output_choice(output_type, filename, resp) {
            Err(e) => assert_eq!(e.to_string(), AppError::NoOutputFilename.to_string()),
            _ => panic!("Expected Err, but got Ok"),
        }
    }

    #[test]
    fn test_handle_output_choice_file_exists() {
        let output_type = "csv".to_string();
        let temp_file = NamedTempFile::new().unwrap();
        let filename = Some(temp_file.path().to_str().unwrap().to_string());
        let resp: Vec<i32> = vec![1, 2, 3, 4, 5];

        match handle_output_choice(output_type, filename, resp) {
            Err(e) => assert_eq!(e.to_string(), AppError::FileExists.to_string()),
            _ => panic!("Expected Err, but got Ok"),
        }
    }

    #[test]
    fn test_handle_output_choice_write_output() {
        let output_type = "csv".to_string();
        let temp_file = NamedTempFile::new().unwrap();
        let temp_file_path = temp_file.path().to_str().unwrap().to_string();
        let filename = temp_file_path.clone() + "_new";
        let resp: Vec<i32> = vec![1, 2, 3, 4, 5];

        let result = handle_output_choice(output_type, Some(filename.clone()), resp);
        assert!(result.is_ok());

        let mut file = File::open(filename).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        // Check the contents of the file
        assert_eq!(contents, "1\n2\n3\n4\n5\n");
    }
}
