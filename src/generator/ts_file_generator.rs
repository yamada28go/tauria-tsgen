use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::path::Path;
#[allow(unused_imports)]
use syn::{Attribute, Fields, FnArg, Item, ItemEnum, ItemStruct, Lit, Meta, Pat, Type};
use tera::{Context, Tera};

#[derive(RustEmbed)]
#[folder = "templates/"]
pub struct Asset;

pub fn register_tera_filters(tera: &mut Tera) {
    use convert_case::{Case, Casing};
    tera.register_filter(
        "camelcase",
        move |value: &tera::Value, _: &std::collections::HashMap<String, tera::Value>| {
            let s = match value.as_str() {
                Some(s) => s,
                None => {
                    return Err(tera::Error::msg(
                        "camelcase filter can only be used on strings",
                    ));
                }
            };
            Ok(tera::Value::String(s.to_case(Case::Camel)))
        },
    );
    tera.register_filter(
        "pascalcase",
        move |value: &tera::Value, _: &std::collections::HashMap<String, tera::Value>| {
            let s = match value.as_str() {
                Some(s) => s,
                None => {
                    return Err(tera::Error::msg(
                        "pascalcase filter can only be used on strings",
                    ));
                }
            };
            Ok(tera::Value::String(s.to_case(Case::Pascal)))
        },
    );
}

pub fn generate_event_handler_files(
    output_dir: &Path,
    global_events: &[crate::generator::type_extractor::EventInfo],
    window_events: &[crate::generator::type_extractor::WindowEventInfo],
) -> anyhow::Result<()> {
    use convert_case::{Case, Casing};
    let mut tera = Tera::default();
    register_tera_filters(&mut tera);
    tera.add_raw_template(
        "global_event_handler.tera",
        std::str::from_utf8(
            Asset::get("global_event_handler.tera")
                .unwrap()
                .data
                .as_ref(),
        )?,
    )?;
    tera.add_raw_template(
        "window_event_handler.tera",
        std::str::from_utf8(
            Asset::get("window_event_handler.tera")
                .unwrap()
                .data
                .as_ref(),
        )?,
    )?;
    tera.autoescape_on(vec![]);

    let interface_dir = output_dir.join("interface");
    let tauri_api_dir = output_dir.join("tauria-api");
    let events_dir = interface_dir.join("events");

    std::fs::create_dir_all(&events_dir)?;
    std::fs::create_dir_all(&tauri_api_dir)?;

    if !global_events.is_empty() {
        let mut context = Context::new();
        let mut unique_events = HashMap::new();
        for event in global_events {
            unique_events.insert(event.event_name.clone(), event.clone());
        }
        let unique_global_events: Vec<_> = unique_events.values().cloned().collect();
        context.insert("global_events", &unique_global_events);

        // GlobalEventHandlers.ts の生成
        let rendered = tera.render("global_event_handler.tera", &context).unwrap();
        std::fs::write(tauri_api_dir.join("GlobalEventHandlers.ts"), rendered)?;
    }

    if !window_events.is_empty() {
        let mut window_events_map: HashMap<
            String,
            HashMap<String, crate::generator::type_extractor::WindowEventInfo>,
        > = HashMap::new();
        for event in window_events {
            window_events_map
                .entry(event.window_name.clone())
                .or_default()
                .insert(event.event_name.clone(), event.clone());
        }

        for (window_name, events_map) in window_events_map {
            let mut context = Context::new();
            let unique_events: Vec<_> = events_map.values().cloned().collect();
            context.insert("window_name", &window_name);
            context.insert("events", &unique_events);
            let rendered = tera.render("window_event_handler.tera", &context).unwrap();
            let pascal_case_window_name = window_name.to_case(Case::Pascal);
            std::fs::write(
                events_dir.join(format!("{pascal_case_window_name}WindowEventHandlers.ts")),
                rendered,
            )?;
        }
    }

    Ok(())
}

#[allow(unused_variables)]
/// Generates TypeScript files from Rust code.
///
/// This function parses Rust code, extracts Tauri commands and user-defined types,
/// and generates corresponding TypeScript files for interfaces, Tauri APIs, and mock APIs.
///
/// # Arguments
///
/// * `rust_code` - A string slice containing the Rust code to be parsed.
/// * `output_dir` - The root directory where the generated TypeScript files will be saved.
/// * `file_name` - The base name for the generated TypeScript files.
///
/// # Returns
///
/// An `anyhow::Result<(bool, Vec<serde_json::Value>)>` indicating whether any command files were generated and the extracted TypeScript interfaces.
pub fn generate_ts_files(
    rust_code: &str,
    output_dir: &Path,
    file_name: &str,
    generate_mock_api: bool,
) -> anyhow::Result<(
    bool,
    Vec<crate::generator::type_extractor::ExtractedTypeInfo>,
    Vec<crate::generator::type_extractor::EventInfo>,
    Vec<crate::generator::type_extractor::WindowEventInfo>,
)> {
    let mut tera = Tera::default();
    register_tera_filters(&mut tera);
    // テンプレートの読み込み
    tera.add_raw_template(
        "command_interfaces.tera",
        std::str::from_utf8(Asset::get("command_interfaces.tera").unwrap().data.as_ref())?,
    )?;
    tera.add_raw_template(
        "tauria_api.tera",
        std::str::from_utf8(Asset::get("tauria_api.tera").unwrap().data.as_ref())?,
    )?;
    tera.add_raw_template(
        "mock_api.tera",
        std::str::from_utf8(Asset::get("mock_api.tera").unwrap().data.as_ref())?,
    )?;
    tera.add_raw_template(
        "user_types.tera",
        std::str::from_utf8(Asset::get("user_types.tera").unwrap().data.as_ref())?,
    )?;
    tera.autoescape_on(vec![]);

    let syntax = syn::parse_file(rust_code)?;

    let all_extracted_types = super::type_extractor::extract_and_convert_types(&syntax.items);
    let (global_events, window_events) =
        super::type_extractor::extract_events(&syntax.items, &all_extracted_types);

    let functions =
        super::type_extractor::extract_tauri_commands(&syntax.items, &all_extracted_types);

    // 出力ディレクトリの作成
    std::fs::create_dir_all(output_dir)?;
    let interface_dir = output_dir.join("interface");
    let tauri_api_dir = output_dir.join("tauria-api");
    let mock_api_dir = output_dir.join("mock-api");

    std::fs::create_dir_all(&interface_dir)?;
    std::fs::create_dir_all(&tauri_api_dir)?;
    if generate_mock_api {
        std::fs::create_dir_all(&mock_api_dir)?;
    }

    // コマンドインターフェースファイルの生成
    if !functions.is_empty() {
        let commands_dir = interface_dir.join("commands");
        std::fs::create_dir_all(&commands_dir)?;
        let mut command_context = Context::new();
        command_context.insert("functions", &functions);
        use convert_case::{Case, Casing};
        let interface_name = file_name.to_case(Case::Pascal);
        command_context.insert("interface_name", &interface_name);
        let has_user_defined_types_in_file = functions.iter().any(|f| {
            let args = f["args"].as_array().unwrap();
            let return_type = f["return_type"].as_str().unwrap();
            args.iter().any(|arg| arg.as_str().unwrap().contains("T."))
                || return_type.contains("T.")
        });
        command_context.insert(
            "has_user_defined_types_in_file",
            &has_user_defined_types_in_file,
        );
        let command_interfaces_rendered =
            tera.render("command_interfaces.tera", &command_context)?;
        let pascal_case_file_name = file_name.to_case(Case::Pascal);
        std::fs::write(
            commands_dir.join(format!("{pascal_case_file_name}.ts")),
            command_interfaces_rendered,
        )?;
    }

    // Tauri APIファイルの生成
    if !functions.is_empty() {
        let mut tauri_api_context = Context::new();
        tauri_api_context.insert("functions", &functions);
        tauri_api_context.insert("file_name", &file_name);
        let tauri_api_rendered = tera.render("tauria_api.tera", &tauri_api_context)?;
        use convert_case::{Case, Casing};
        let pascal_case_file_name = file_name.to_case(Case::Pascal);
        std::fs::write(
            tauri_api_dir.join(format!("{pascal_case_file_name}.ts")),
            tauri_api_rendered,
        )?;
    }

    if generate_mock_api && !functions.is_empty() {
        std::fs::create_dir_all(&mock_api_dir)?;
        let mut mock_api_context = Context::new();
        mock_api_context.insert("functions", &functions);
        mock_api_context.insert("file_name", &file_name);
        let mock_api_rendered = tera.render("mock_api.tera", &mock_api_context)?;
        use convert_case::{Case, Casing};
        let pascal_case_file_name = file_name.to_case(Case::Pascal);
        std::fs::write(
            mock_api_dir.join(format!("{pascal_case_file_name}.ts")),
            mock_api_rendered,
        )?;
    }

    Ok((
        !functions.is_empty(),
        all_extracted_types,
        global_events,
        window_events,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[allow(dead_code)]
    fn run_ts_wrapper_test(test_case_name: &str) {
        use convert_case::{Case, Casing};
        let pascal_case_file_name = test_case_name.to_case(Case::Pascal);
        #[allow(unused_variables)]
        let rust_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test/data")
            .join(test_case_name)
            .join("src")
            .join(format!("{}.rs", test_case_name));
        let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/generated_ts")
            .join(test_case_name);

        // 既存の出力ディレクトリをクリーンアップ
        if output_dir.exists() {
            fs::remove_dir_all(&output_dir)
                .expect("出力ディレクトリのクリーンアップに失敗しました");
        }
        fs::create_dir_all(&output_dir).expect("出力ディレクトリの作成に失敗しました");

        let rust_code = fs::read_to_string(&rust_file_path).expect("Rustファイルが読み込めません");
        let file_name = test_case_name;

        let result = generate_ts_files(&rust_code, &output_dir, file_name, false);

        assert!(result.is_ok());

        let (has_command, _all_types, global_events, window_events) = result.unwrap();

        if !global_events.is_empty() || !window_events.is_empty() {
            let event_result =
                generate_event_handler_files(&output_dir, &global_events, &window_events);
            assert!(event_result.is_ok());
        }

        if has_command {
            // コマンド関連ファイルの比較
            compare_generated_files(
                &output_dir,
                test_case_name,
                &format!("interface/commands/{}.ts", pascal_case_file_name),
            );
            compare_generated_files(
                &output_dir,
                test_case_name,
                &format!("tauria-api/{}.ts", pascal_case_file_name),
            );
        }

        if !global_events.is_empty() {
            // グローバルイベントハンドラファイルの比較
            compare_generated_files(
                &output_dir,
                test_case_name,
                "interface/events/GlobalEventHandlers.ts",
            );
        }

        if !window_events.is_empty() {
            // ウィンドウイベントハンドラファイルの比較
            let mut window_names: Vec<_> = window_events.iter().map(|e| e.window_name.clone()).collect();
            window_names.sort();
            window_names.dedup();

            for window_name in window_names {
                let pascal_case_window_name = window_name.to_case(Case::Pascal);
                compare_generated_files(
                    &output_dir,
                    test_case_name,
                    &format!(
                        "interface/events/{}WindowEventHandlers.ts",
                        pascal_case_window_name
                    ),
                );
            }
        }

        // mock-api ディレクトリとファイルが存在しないことを確認
        assert!(!output_dir.join("mock-api").exists());
    }

    fn compare_generated_files(output_dir: &Path, test_case_name: &str, file_path: &str) {
        let generated_path = output_dir.join(file_path);
        let expected_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test/data")
            .join(test_case_name)
            .join("expected")
            .join(file_path);

        let generated_content = fs::read_to_string(&generated_path)
            .unwrap_or_else(|_| panic!("生成されたファイルが読み込めません: {:?}", generated_path));
        let expected_content = fs::read_to_string(&expected_path)
            .unwrap_or_else(|_| panic!("期待されるファイルが読み込めません: {:?}", expected_path));

        assert_eq!(
            generated_content.trim().replace("\r\n", "\n"),
            expected_content.trim().replace("\r\n", "\n"),
            "ファイルの内容が一致しません: {}",
            file_path
        );
    }

    #[test]
    fn test_generate_ts_wrapper_for_basic_file() {
        run_ts_wrapper_test("basic");
    }

    #[test]
    fn test_generate_ts_wrapper_for_struct_test_file() {
        run_ts_wrapper_test("struct_test");
    }

    #[test]
    fn test_generate_ts_wrapper_for_enum_test_file() {
        run_ts_wrapper_test("enum_test");
    }

    #[test]
    fn test_generate_ts_wrapper_for_nesting_type_test() {
        run_ts_wrapper_test("nesting_type_test");
    }

    #[test]
    fn test_generate_ts_wrapper_for_app_handle() {
        run_ts_wrapper_test("app_handle");
    }

    #[test]
    fn test_generate_ts_wrapper_for_webview_window() {
        run_ts_wrapper_test("webview_window");
    }

    #[test]
    fn test_generate_ts_wrapper_for_state() {
        run_ts_wrapper_test("state");
    }

    #[test]
    fn test_generate_ts_wrapper_for_response() {
        run_ts_wrapper_test("response");
    }

    #[test]
    fn test_generate_ts_wrapper_for_window() {
        run_ts_wrapper_test("window");
    }

    #[test]
    fn test_generate_ts_wrapper_for_event_test() {
        run_ts_wrapper_test("event_test");
    }
}
