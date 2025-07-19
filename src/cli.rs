use anyhow::Context;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the configuration file. You can set the input and output directories in JSON format.
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<String>,

    /// Path to the directory containing the input Rust code.
    /// This argument is required if no configuration file is specified.
    #[arg(long, value_name = "DIR")]
    pub input_path: Option<String>,

    /// Path to the directory where the generated TypeScript files will be output.
    /// This argument is required if no configuration file is specified.
    #[arg(long, value_name = "DIR")]
    pub output_path: Option<String>,

    /// Specify this flag to generate mock API files.
    #[arg(long)]
    pub mock_api: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub input_path: String,
    pub output_path: String,
}

/// Loads the configuration from the CLI arguments or a config file.
///
/// This function first checks for a `--config` file path. If it exists, it reads and
/// parses the JSON configuration file.
///
/// If no config file is provided, it checks for `--input-path` and `--output-path`
/// arguments to construct the configuration.
///
/// # Errors
///
/// This function will return an `Err` if:
/// - The config file cannot be read or parsed.
/// - Neither a config file nor the input/output path arguments are provided.
pub fn load_config(cli: &Cli) -> anyhow::Result<Config> {
    if let Some(config_path) = &cli.config {
        let config_content =
            fs::read_to_string(config_path).context("Could not read config file")?;
        serde_json::from_str(&config_content).context("Could not parse config file")
    } else if let (Some(input), Some(output)) = (&cli.input_path, &cli.output_path) {
        Ok(Config {
            input_path: input.clone(),
            output_path: output.clone(),
        })
    } else {
        anyhow::bail!("Either --config or both --input-path and --output-path must be provided.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_config_from_file() {
        let config_content = r#"{ "input_path": "/tmp/input", "output_path": "/tmp/output" }"#;
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        writeln!(temp_file, "{}", config_content).expect("Failed to write to temp file");
        let config_path = temp_file.path().to_str().unwrap().to_string();

        let cli = Cli {
            config: Some(config_path),
            input_path: None,
            output_path: None,
            mock_api: false,
        };
        let config = load_config(&cli).expect("Failed to load config from file");
        assert_eq!(config.input_path, "/tmp/input");
        assert_eq!(config.output_path, "/tmp/output");
    }

    #[test]
    fn test_load_config_from_args() {
        let cli = Cli {
            config: None,
            input_path: Some("/tmp/input_arg".to_string()),
            output_path: Some("/tmp/output_arg".to_string()),
            mock_api: false,
        };
        let config = load_config(&cli).expect("Failed to load config from args");
        assert_eq!(config.input_path, "/tmp/input_arg");
        assert_eq!(config.output_path, "/tmp/output_arg");
    }

    #[test]
    fn test_load_config_file_not_found() {
        let cli = Cli {
            config: Some("/nonexistent/path/config.json".to_string()),
            input_path: None,
            output_path: None,
            mock_api: false,
        };
        let err = load_config(&cli).unwrap_err();
        assert!(err.to_string().contains("Could not read config file"));
    }

    #[test]
    fn test_load_config_invalid_json() {
        let config_content = r#"{ "input_path": "/tmp/input", "output_path": "/tmp/output""#; // Missing closing brace
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        writeln!(temp_file, "{}", config_content).expect("Failed to write to temp file");
        let config_path = temp_file.path().to_str().unwrap().to_string();

        let cli = Cli {
            config: Some(config_path),
            input_path: None,
            output_path: None,
            mock_api: false,
        };
        let err = load_config(&cli).unwrap_err();
        assert!(err.to_string().contains("Could not parse config file"));
    }

    #[test]
    fn test_load_config_no_args_no_config() {
        let cli = Cli {
            config: None,
            input_path: None,
            output_path: None,
            mock_api: false,
        };
        let err = load_config(&cli).unwrap_err();
        assert!(
            err.to_string().contains(
                "Either --config or both --input-path and --output-path must be provided."
            )
        );
    }

    #[test]
    fn test_load_config_only_input_path() {
        let cli = Cli {
            config: None,
            input_path: Some("/tmp/input_only".to_string()),
            output_path: None,
            mock_api: false,
        };
        let err = load_config(&cli).unwrap_err();
        assert!(
            err.to_string().contains(
                "Either --config or both --input-path and --output-path must be provided."
            )
        );
    }

    #[test]
    fn test_load_config_only_output_path() {
        let cli = Cli {
            config: None,
            input_path: None,
            output_path: Some("/tmp/output_only".to_string()),
            mock_api: false,
        };
        let err = load_config(&cli).unwrap_err();
        assert!(
            err.to_string().contains(
                "Either --config or both --input-path and --output-path must be provided."
            )
        );
    }
}
