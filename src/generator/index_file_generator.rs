use convert_case::{Case, Casing};
use std::path::Path;
#[allow(unused_imports)]
use tera::{Context, Tera};

/// Generates index files for the TypeScript output.
///
/// This function creates `index.ts` files in the `interface`,
/// `tauria-api`, and `mock-api` directories, as well as a root `index.ts` file.
/// These files re-export all the generated command and type files, making them
/// easily accessible to the frontend.
///
/// # Arguments
///
/// * `output_dir` - The root directory where the `index.ts` files will be created.
/// * `file_names` - A mutable vector of strings containing the base names of the generated command files. This vector will be sorted internally.
/// * `generate_mock_api` - A boolean indicating whether mock API index files should be generated.
/// * `global_events` - A slice of `EventInfo` representing global events, used to determine if global event handlers should be exported.
/// * `window_events` - A slice of `WindowEventInfo` representing window-specific events, used to determine if window event handlers should be exported.
///
/// # Returns
///
/// An `anyhow::Result` indicating whether the operation was successful.
#[allow(clippy::ptr_arg)]
pub fn generate_index_files(
    output_dir: &Path,
    file_names: &mut Vec<String>,
    generate_mock_api: bool,
    global_events: &[crate::generator::type_extractor::EventInfo],
    window_events: &[crate::generator::type_extractor::WindowEventInfo],
) -> anyhow::Result<()> {
    std::fs::create_dir_all(output_dir)?;
    file_names.sort();
    let interface_dir = output_dir.join("interface");
    let tauri_api_dir = output_dir.join("tauria-api");
    let mock_api_dir = output_dir.join("mock-api");

    std::fs::create_dir_all(&interface_dir)?;
    std::fs::create_dir_all(&tauri_api_dir)?;
    if generate_mock_api {
        std::fs::create_dir_all(&mock_api_dir)?;
    }

    // interface/index.ts
    let mut interface_index_content = file_names
        .iter()
        .map(|name| {
            format!(
                "export * from \"./commands/{}\";",
                name.to_case(Case::Pascal)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let types_index_path = interface_dir.join("types").join("index.ts");
    if types_index_path.exists() {
        let types_file_content = std::fs::read_to_string(&types_index_path)?;
        if !types_file_content.trim().is_empty() {
            interface_index_content.push_str("\nexport * from \"./types/\";"); // types/index.ts をエクスポート
        }
    }
    std::fs::write(interface_dir.join("index.ts"), interface_index_content)?;

    let mut tauri_api_index_content = file_names
        .iter()
        .map(|name| {
            format!(
                "export * from \"./commands/{}\";",
                name.to_case(Case::Pascal)
            )
        }) // 変更
        .collect::<Vec<_>>()
        .join("\n");

    if !global_events.is_empty() {
        tauri_api_index_content.push_str("\nexport * from \"./events/TauriGlobalEventHandlers\";"); // "event" から "events" に変更
    }

    if !window_events.is_empty() {
        let mut unique_window_names: Vec<String> = window_events
            .iter()
            .map(|e| e.window_name.clone())
            .collect();
        unique_window_names.sort();
        unique_window_names.dedup();
        for window_name in unique_window_names {
            tauri_api_index_content.push_str(&format!(
                "\nexport * from \"./events/Tauri{}WindowEventHandlers\";",
                window_name.to_case(Case::Pascal)
            ));
        }
    }

    std::fs::write(tauri_api_dir.join("index.ts"), tauri_api_index_content)?;

    if generate_mock_api {
        let mock_api_index_content = file_names
            .iter()
            .map(|name| format!("export * from \"./{}\";", name.to_case(Case::Pascal)))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(mock_api_dir.join("index.ts"), mock_api_index_content)?;
    }

    // 最上位の index.ts (切り替え可能にする)
    let root_index_content = r#"// This file is generated by tauria-tsgen.

// You can switch between tauria-api and mock-api by modifying this file.


export * from "./tauria-api";

// export * from "./mock-api";
"#
    .to_string();
    std::fs::write(output_dir.join("index.ts"), root_index_content)?;

    Ok(())
}

/// Generates an `index.ts` file for user-defined types within the `interface/types` directory.
///
/// This function collects all extracted user-defined types (structs and enums)
/// that are marked as serializable or deserializable and generates their TypeScript
/// interfaces/enums into a single `index.ts` file. This allows for easy import
/// of all user-defined types from a single entry point.
///
/// # Arguments
///
/// * `output_dir` - The root output directory where the `interface/types/index.ts` file will be created.
/// * `all_extracted_types` - A slice of `ExtractedTypeInfo` containing all extracted user-defined types.
///
/// # Returns
///
/// An `anyhow::Result` indicating whether the operation was successful.
pub fn generate_user_types_index_file(
    output_dir: &Path,
    all_extracted_types: &[crate::generator::type_extractor::ExtractedTypeInfo],
) -> anyhow::Result<()> {
    // all_extracted_types が空の場合は、types ディレクトリも types/index.ts も生成しない
    if all_extracted_types.is_empty() {
        return Ok(());
    }

    let types_dir = output_dir.join("interface").join("types");
    std::fs::create_dir_all(&types_dir)?;

    let mut tera = Tera::default();
    tera.add_raw_template(
        "user_types.tera",
        std::str::from_utf8(
            crate::generator::ts_file_generator::Asset::get("user_types.tera")
                .unwrap()
                .data
                .as_ref(),
        )?,
    )?;
    tera.autoescape_on(vec![]);

    let mut all_types_content = String::new();
    for extracted_type_info in all_extracted_types {
        // Serialize または Deserialize のどちらか一方でも derive されていればエクスポート対象
        if !extracted_type_info.is_serializable && !extracted_type_info.is_deserializable {
            continue;
        }

        all_types_content.push_str(&format!(
            "//- Generated from {}.rs\n",
            extracted_type_info.original_file_name
        ));
        let mut context = Context::new();
        context.insert("ts_interface", &extracted_type_info.ts_interface);
        let rendered = tera.render("user_types.tera", &context)?;
        all_types_content.push_str(&rendered);
        all_types_content.push('\n');
        all_types_content.push('\n');
    }

    std::fs::write(types_dir.join("index.ts"), all_types_content)?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    use crate::generator::type_extractor::ExtractedTypeInfo;
    use serde_json::json;

    // Helper function to create a dummy file
    fn create_dummy_file(dir: &Path, file_name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(file_name);
        std::fs::write(&file_path, content).expect("Failed to write dummy file");
        file_path
    }

    #[test]
    fn test_generate_index_files_with_non_empty_types_index() {
        let output_dir = tempdir().expect("Failed to create temp dir");
        let interface_dir = output_dir.path().join("interface");
        let types_dir = interface_dir.join("types");
        std::fs::create_dir_all(&types_dir).expect("Failed to create types dir");
        create_dummy_file(&types_dir, "index.ts", "export interface MyType {};");

        let mut file_names = vec!["test_file".to_string()];
        generate_index_files(output_dir.path(), &mut file_names, false, &[], &[])
            .expect("Failed to generate index files");

        let interface_index_content = fs::read_to_string(interface_dir.join("index.ts"))
            .expect("Failed to read interface/index.ts");
        assert!(interface_index_content.contains("export * from \"./types/\";"));
    }

    #[test]
    fn test_generate_index_files_with_empty_types_index() {
        let output_dir = tempdir().expect("Failed to create temp dir");
        let interface_dir = output_dir.path().join("interface");
        let types_dir = interface_dir.join("types");
        std::fs::create_dir_all(&types_dir).expect("Failed to create types dir");
        create_dummy_file(&types_dir, "index.ts", ""); // Empty content

        let mut file_names = vec!["test_file".to_string()];
        generate_index_files(output_dir.path(), &mut file_names, false, &[], &[])
            .expect("Failed to generate index files");

        let interface_index_content = fs::read_to_string(interface_dir.join("index.ts"))
            .expect("Failed to read interface/index.ts");
        assert!(!interface_index_content.contains("export * from \"./types/\";"));
    }

    #[test]
    fn test_generate_index_files_without_types_index() {
        let output_dir = tempdir().expect("Failed to create temp dir");
        // Do not create types/index.ts

        let mut file_names = vec!["test_file".to_string()];
        generate_index_files(output_dir.path(), &mut file_names, false, &[], &[])
            .expect("Failed to generate index files");

        let interface_dir = output_dir.path().join("interface");
        let interface_index_content = fs::read_to_string(interface_dir.join("index.ts"))
            .expect("Failed to read interface/index.ts");
        assert!(!interface_index_content.contains("export * from \"./types/\";"));
    }

    #[test]
    fn test_generate_index_files_no_mock_api() {
        let output_dir = tempdir().expect("Failed to create temp dir");
        let mut file_names = vec!["test_file".to_string()];
        generate_index_files(output_dir.path(), &mut file_names, false, &[], &[])
            .expect("Failed to generate index files");

        assert!(!output_dir.path().join("mock-api").exists());
    }

    #[test]
    fn test_generate_user_types_index_file_empty_interfaces() {
        let output_dir = tempdir().expect("Failed to create temp dir");
        let all_ts_interfaces: Vec<ExtractedTypeInfo> = Vec::new();

        generate_user_types_index_file(output_dir.path(), &all_ts_interfaces)
            .expect("Failed to generate user types index file");

        assert!(
            !output_dir
                .path()
                .join("interface")
                .join("types")
                .join("index.ts")
                .exists()
        );
    }

    #[test]
    fn test_generate_user_types_index_file_with_empty_file_interfaces() {
        let output_dir = tempdir().expect("Failed to create temp dir");
        let all_ts_interfaces = vec![
            ExtractedTypeInfo {
                name: "file1".to_string(),
                ts_interface: json!({}),
                is_serializable: false,
                is_deserializable: false,
                original_file_name: "file1".to_string(),
            },
            ExtractedTypeInfo {
                name: "file2".to_string(),
                ts_interface: json!({"name": "MyType", "type": "interface"}),
                is_serializable: true,
                is_deserializable: true,
                original_file_name: "file2".to_string(),
            },
        ];

        generate_user_types_index_file(output_dir.path(), &all_ts_interfaces)
            .expect("Failed to generate user types index file");

        let types_index_content = fs::read_to_string(
            output_dir
                .path()
                .join("interface")
                .join("types")
                .join("index.ts"),
        )
        .expect("Failed to read types/index.ts");

        assert!(types_index_content.contains("interface MyType"));
    }

    #[test]
    fn test_generate_user_types_index_file_multiple_files_sorted() {
        let output_dir = tempdir().expect("Failed to create temp dir");
        let mut all_ts_interfaces = vec![
            ExtractedTypeInfo {
                name: "file_b".to_string(),
                ts_interface: json!({"name": "TypeB", "type": "interface", "fields": []}),
                is_serializable: true,
                is_deserializable: true,
                original_file_name: "file_b".to_string(),
            },
            ExtractedTypeInfo {
                name: "file_a".to_string(),
                ts_interface: json!({"name": "TypeA", "type": "interface", "fields": []}),
                is_serializable: true,
                is_deserializable: true,
                original_file_name: "file_a".to_string(),
            },
        ];
        all_ts_interfaces.sort_by(|a, b| a.name.cmp(&b.name));

        generate_user_types_index_file(output_dir.path(), &all_ts_interfaces)
            .expect("Failed to generate user types index file");

        let types_index_content = fs::read_to_string(
            output_dir
                .path()
                .join("interface")
                .join("types")
                .join("index.ts"),
        )
        .expect("Failed to read types/index.ts");

        let expected_content = r#"//- Generated from file_a.rs

export interface TypeA {

}


//- Generated from file_b.rs

export interface TypeB {

}

"#;
        assert_eq!(
            types_index_content.trim().replace("\r\n", "\n"),
            expected_content.trim().replace("\r\n", "\n")
        );
    }

    #[test]
    fn test_generate_user_types_index_file_with_struct_and_enum() {
        let output_dir = tempdir().expect("Failed to create temp dir");
        let all_ts_interfaces = vec![
            ExtractedTypeInfo {
                name: "my_types".to_string(),
                ts_interface: json!({"name": "MyStruct", "type": "interface", "fields": []}),
                is_serializable: true,
                is_deserializable: true,
                original_file_name: "my_types".to_string(),
            },
            ExtractedTypeInfo {
                name: "my_types".to_string(),
                ts_interface: json!({"name": "MyEnum", "type": "enum", "variants": []}),
                is_serializable: true,
                is_deserializable: true,
                original_file_name: "my_types".to_string(),
            },
        ];

        generate_user_types_index_file(output_dir.path(), &all_ts_interfaces)
            .expect("Failed to generate user types index file");

        let types_index_content = fs::read_to_string(
            output_dir
                .path()
                .join("interface")
                .join("types")
                .join("index.ts"),
        )
        .expect("Failed to read types/index.ts");

        assert!(types_index_content.contains("interface MyStruct"));
        assert!(types_index_content.contains("enum MyEnum"));
    }

    #[test]
    fn test_generate_index_files_sort_order() {
        let output_dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/test_index_sort_order");
        if output_dir.exists() {
            fs::remove_dir_all(&output_dir)
                .expect("出力ディレクトリのクリーンアップに失敗しました");
        }
        fs::create_dir_all(&output_dir).expect("出力ディレクトリの作成に失敗しました");

        let mut file_names = vec![
            "z_file".to_string(),
            "a_file".to_string(),
            "m_file".to_string(),
        ];

        generate_index_files(&output_dir, &mut file_names, true, &[], &[])
            .expect("indexファイルの生成に失敗しました");

        let interface_index_content =
            fs::read_to_string(output_dir.join("interface").join("index.ts"))
                .expect("interface/index.tsが読み込めません");
        let tauri_api_index_content =
            fs::read_to_string(output_dir.join("tauria-api").join("index.ts"))
                .expect("tauria-api/index.tsが読み込めません");
        let mock_api_index_content =
            fs::read_to_string(output_dir.join("mock-api").join("index.ts"))
                .expect("mock-api/index.tsが読み込めません");

        // interface/index.ts のソート順を確認
        let expected_interface_content = "export * from \"./commands/AFile\";\nexport * from \"./commands/MFile\";\nexport * from \"./commands/ZFile\";";
        assert_eq!(
            interface_index_content.trim().replace("\r\n", "\n"),
            expected_interface_content.trim().replace("\r\n", "\n"),
            "interface/index.ts のソート順が不正です"
        );

        // tauria-api/index.ts のソート順を確認
        let expected_tauri_api_content = "export * from \"./commands/AFile\";\nexport * from \"./commands/MFile\";\nexport * from \"./commands/ZFile\";";
        assert_eq!(
            tauri_api_index_content.trim().replace("\r\n", "\n"),
            expected_tauri_api_content.trim().replace("\r\n", "\n"),
            "tauria-api/index.ts のソート順が不正です"
        );

        // mock-api/index.ts のソート順を確認
        let expected_mock_api_content =
            "export * from \"./AFile\";\nexport * from \"./MFile\";\nexport * from \"./ZFile\";";
        assert_eq!(
            mock_api_index_content.trim().replace("\r\n", "\n"),
            expected_mock_api_content.trim().replace("\r\n", "\n"),
            "mock-api/index.ts のソート順が不正です"
        );
    }
}
