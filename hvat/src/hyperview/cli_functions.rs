use color_eyre::Result;
use csv::Writer;
use log::{error, LevelFilter};
use serde::Serialize;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::{Path, MAIN_SEPARATOR_STR};

use crate::hyperview::{
    app_errors::AppError,
    cli_data::{DebugLevels, OutputOptions},
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

pub fn write_output<T: Serialize>(filename: String, object_list: Vec<T>) -> Result<()> {
    let mut writer = Writer::from_path(filename)?;

    for object in object_list {
        writer.serialize(object)?;
    }

    Ok(())
}

pub fn handle_output_choice<T: Display + Serialize>(
    output_type: OutputOptions,
    filename: Option<String>,
    resp: Vec<T>,
) -> Result<()> {
    let mut outfile = String::new();

    if let Some(f) = filename.clone() {
        if Path::new(&f).exists() {
            error!("Specified file already exists. exiting ...");
            return Err(AppError::FileExists.into());
        }

        outfile = f;
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
                    println!("---- [{}] ----\n{}\n", i, s);
                }
                return Ok(());
            }

            let mut file_handle = File::create(outfile)?;
            for (i, s) in resp.iter().enumerate() {
                write!(file_handle, "---- [{}] ----\n{}\n\n", i, s)?;
            }
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

        match handle_output_choice(output_type, filename, resp) {
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

        let result = handle_output_choice(output_type, Some(filename.clone()), resp);
        assert!(result.is_ok());

        let mut file = File::open(filename).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        // Check the contents of the file
        assert_eq!(contents, "1\n2\n3\n4\n5\n");
    }
}
