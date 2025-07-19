mod cli;
mod generator;

use anyhow::Context;
use clap::Parser;
use cli::{Cli, load_config};
use generator::index_file_generator::{generate_index_files, generate_user_types_index_file};
use generator::ts_file_generator::generate_ts_files;
use log::{error, info};
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    if let Err(e) = run_app(cli) {
        error!("Application error: {e}");
        std::process::exit(1);
    }

    Ok(())
}

fn run_app(cli: Cli) -> anyhow::Result<()> {
    let config = load_config(&cli).context("Failed to load configuration")?;
    let input_dir = PathBuf::from(config.input_path);
    let output_dir = PathBuf::from(config.output_path);

    info!("Input directory: {input_dir:?}");
    info!("Output directory: {output_dir:?}");

    if !output_dir.exists() {
        info!("Output directory does not exist, creating: {output_dir:?}");
        fs::create_dir_all(&output_dir).context("Failed to create output directory")?;
    }

    let mut file_names = Vec::new();
    let mut all_ts_interfaces: Vec<crate::generator::type_extractor::ExtractedTypeInfo> =
        Vec::new();

    for entry in fs::read_dir(&input_dir).context("Failed to read input directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            info!("Processing file: {path:?}");
            let code = fs::read_to_string(&path).context("Failed to read file")?;
            let file_name = path
                .file_stem()
                .ok_or_else(|| anyhow::anyhow!("File has no stem: {}", path.display()))?;
            dbg!(&file_name);
            let file_name = file_name
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid file name: {}", path.display()))?;
            dbg!(&file_name);

            let (has_command, ts_interfaces) =
                generate_ts_files(&code, &output_dir, file_name, cli.mock_api)
                    .context("Failed to generate TypeScript wrapper")?;
            all_ts_interfaces.extend(ts_interfaces);

            if has_command {
                file_names.push(file_name.to_string());
            }
            info!(
                "Generated: {}.ts",
                output_dir.join(format!("{file_name}.ts")).display()
            );
        } else {
            info!("Skipping: {path:?}");
        }
    }

    file_names.sort();
    all_ts_interfaces.sort_by(|a, b| a.name.cmp(&b.name));

    generate_user_types_index_file(&output_dir, &all_ts_interfaces)?;

    generate_index_files(&output_dir, &mut file_names, cli.mock_api)?;

    info!("✅ Tauri wrapper generation completed.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // Helper function to create a dummy Rust file
    fn create_dummy_rust_file(dir: &PathBuf, file_name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(file_name);
        std::fs::write(&file_path, content).expect("Failed to write dummy Rust file");
        file_path
    }

    #[test]
    fn test_run_app_success() {
        let input_dir = tempdir().expect("Failed to create temp input dir");
        let output_dir = tempdir().expect("Failed to create temp output dir");

        create_dummy_rust_file(
            &input_dir.path().to_path_buf(),
            "test_commands.rs",
            r#"
                #[tauri::command]
                fn greet(name: String) -> String {
                    format!("Hello, {}!", name)
                }
            "#,
        );

        let cli = Cli {
            config: None,
            input_path: Some(input_dir.path().to_str().unwrap().to_string()),
            output_path: Some(output_dir.path().to_str().unwrap().to_string()),
            mock_api: false,
        };

        let result = run_app(cli);
        assert!(result.is_ok());

        // Verify generated files exist
        assert!(
            output_dir
                .path()
                .join("tauria-api")
                .join("TestCommands.ts")
                .exists()
        );
        assert!(
            output_dir
                .path()
                .join("interface")
                .join("commands")
                .join("TestCommands.ts")
                .exists()
        );
        assert!(output_dir.path().join("index.ts").exists());
    }

    #[test]
    fn test_run_app_input_dir_not_found() {
        let input_dir = PathBuf::from("/nonexistent/input/dir");
        let output_dir = tempdir().expect("Failed to create temp output dir");

        let cli = Cli {
            config: None,
            input_path: Some(input_dir.to_str().unwrap().to_string()),
            output_path: Some(output_dir.path().to_str().unwrap().to_string()),
            mock_api: false,
        };

        let result = run_app(cli);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to read input directory")
        );
    }

    #[test]
    fn test_run_app_output_dir_creation_failure() {
        // Simulate a directory that cannot be created (e.g., due to permissions)
        // This is tricky to test reliably without actual permission issues.
        // For now, we'll rely on the `fs::create_dir_all`'s error handling.
        // A more robust test might involve mocking `fs` operations, but that's beyond current scope.

        let input_dir = tempdir().expect("Failed to create temp input dir");
        create_dummy_rust_file(
            &input_dir.path().to_path_buf(),
            "test_commands.rs",
            r#"
                #[tauri::command]
                fn greet(name: String) -> String {
                    format!("Hello, {}!", name)
                }
            "#,
        );

        // Attempt to create output in a read-only location (e.g., root on Unix-like systems)
        // This test might require specific OS permissions to fail as expected.
        let output_dir = PathBuf::from("/root/nonexistent_output"); // This path is usually not writable by normal users

        let cli = Cli {
            config: None,
            input_path: Some(input_dir.path().to_str().unwrap().to_string()),
            output_path: Some(output_dir.to_str().unwrap().to_string()),
            mock_api: false,
        };

        let result = run_app(cli);
        assert!(result.is_err());
        // The error message might vary by OS, but should indicate a creation/permission issue
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to create output directory")
        );
    }

    #[test]
    fn test_run_app_file_read_failure() {
        let input_dir = tempdir().expect("Failed to create temp input dir");
        let output_dir = tempdir().expect("Failed to create temp output dir");

        // Create a file that cannot be read (e.g., due to permissions)
        let file_path = input_dir.path().join("unreadable.rs");
        std::fs::write(&file_path, "// some content").expect("Failed to write file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&file_path, PermissionsExt::from_mode(0o000)).unwrap();
        }

        let cli = Cli {
            config: None,
            input_path: Some(input_dir.path().to_str().unwrap().to_string()),
            output_path: Some(output_dir.path().to_str().unwrap().to_string()),
            mock_api: false,
        };

        let result = run_app(cli);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to read file")
        );
    }

    // #[test]
    // fn test_run_app_invalid_file_name_non_utf8() {
    //     let input_dir = tempdir().expect("Failed to create temp input dir");
    //     let output_dir = tempdir().expect("Failed to create temp output dir");

    //     // Create a file with a non-UTF8 name (tricky to do directly in Rust, often OS-dependent)
    //     // This test might be difficult to make cross-platform and reliable.
    //     // For demonstration, we'll simulate the error path if `to_str()` returns None.
    //     // In a real scenario, you might need to use `std::os::unix::ffi::OsStrExt` for non-UTF8 paths.

    //     // Simulate `to_str()` returning None by creating a path that is not valid UTF-8
    //     // This is a bit of a hack for testing purposes.
    //     let file_path = input_dir.path().join(std::ffi::OsString::from_vec(vec![0xff, 0xfe, 0xfd]));
    //     std::fs::write(&file_path, "// some content").expect("Failed to write file");

    //     let cli = Cli {
    //         config: None,
    //         input_path: Some(input_dir.path().to_str().unwrap().to_string()),
    //         output_path: Some(output_dir.path().to_str().unwrap().to_string()),
    //         mock_api: false,
    //     };

    //     let result = run_app(cli);
    //     assert!(result.is_err());
    //     assert!(result.unwrap_err().to_string().contains("Invalid file name"));
    // }
}
