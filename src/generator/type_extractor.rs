use log::debug;
use serde_json;
use std::collections::HashMap;
use syn::{
    Attribute, Expr, ExprMethodCall, Fields, FnArg, Item, ItemEnum, ItemStruct, Lit, Meta, Pat,
    Type, UseTree,
    visit::{self, Visit},
};

const IGNORED_TAURI_TYPES: &[&str] = &[
    "tauri::WebviewWindow",
    "tauri::State",
    "tauri::AppHandle",
    "tauri::Window",
];

#[derive(Debug, Clone, serde::Serialize)]
pub struct EventInfo {
    pub event_name: String,
    pub payload_type: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WindowEventInfo {
    pub window_name: String,
    pub event_name: String,
    pub payload_type: String,
}

struct EventVisitor<'a> {
    global_events: Vec<EventInfo>,
    window_events: Vec<WindowEventInfo>,
    defined_types: &'a [String],
}

impl<'ast> Visit<'ast> for EventVisitor<'ast> {
    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let method_name = node.method.to_string();
        if method_name == "emit" {
            if let Some(Expr::Lit(expr_lit)) = node.args.get(0) {
                if let Lit::Str(lit_str) = &expr_lit.lit {
                    let event_name = lit_str.value();
                    let payload_type = if let Some(arg) = node.args.get(1) {
                        let rust_type = match arg {
                            Expr::Path(expr_path) => expr_path
                                .path
                                .segments
                                .last()
                                .map(|s| s.ident.to_string())
                                .unwrap_or_else(|| "any".to_string()),
                            Expr::Struct(expr_struct) => expr_struct
                                .path
                                .segments
                                .last()
                                .map(|s| s.ident.to_string())
                                .unwrap_or_else(|| "any".to_string()),
                            _ => "any".to_string(),
                        };
                        type_to_ts(
                            &syn::parse_str(&rust_type).unwrap_or(syn::parse_str("any").unwrap()),
                            self.defined_types,
                            true,
                        )
                    } else {
                        "void".to_string()
                    };
                    self.global_events.push(EventInfo {
                        event_name,
                        payload_type,
                    });
                }
            }
        } else if method_name == "emit_to" {
            if let (Some(Expr::Lit(win_lit)), Some(Expr::Lit(event_lit))) =
                (node.args.get(0), node.args.get(1))
            {
                if let (Lit::Str(win_str), Lit::Str(event_str)) = (&win_lit.lit, &event_lit.lit) {
                    let window_name = win_str.value();
                    let event_name = event_str.value();
                    let payload_type = if let Some(arg) = node.args.get(2) {
                        let rust_type = match arg {
                            Expr::Path(expr_path) => expr_path
                                .path
                                .segments
                                .last()
                                .map(|s| s.ident.to_string())
                                .unwrap_or_else(|| "any".to_string()),
                            Expr::Struct(expr_struct) => expr_struct
                                .path
                                .segments
                                .last()
                                .map(|s| s.ident.to_string())
                                .unwrap_or_else(|| "any".to_string()),
                            _ => "any".to_string(),
                        };
                        type_to_ts(
                            &syn::parse_str(&rust_type).unwrap_or(syn::parse_str("any").unwrap()),
                            self.defined_types,
                            true,
                        )
                    } else {
                        "void".to_string()
                    };
                    self.window_events.push(WindowEventInfo {
                        window_name,
                        event_name,
                        payload_type,
                    });
                }
            }
        }

        visit::visit_expr_method_call(self, node);
    }
}

pub fn extract_events(
    items: &[Item],
    all_extracted_types: &[ExtractedTypeInfo],
) -> (Vec<EventInfo>, Vec<WindowEventInfo>) {
    let defined_types_names: Vec<String> = all_extracted_types
        .iter()
        .map(|info| info.name.clone())
        .collect();

    let mut visitor = EventVisitor {
        global_events: Vec::new(),
        window_events: Vec::new(),
        defined_types: &defined_types_names,
    };

    for item in items {
        visitor.visit_item(item);
    }

    (visitor.global_events, visitor.window_events)
}

pub struct ExtractedTypeInfo {
    pub name: String,
    pub ts_interface: serde_json::Value,
    pub is_serializable: bool,
    pub is_deserializable: bool,
}

/// Extracts and converts Rust structs and enums to TypeScript interfaces.
///
/// This function iterates through the given Rust items and converts any structs or enums
/// that derive `Serialize` and `Deserialize` into a `serde_json::Value` representation
/// for TypeScript generation.
///
/// # Arguments
///
/// * `items` - A slice of `syn::Item` representing the parsed Rust code.
///
/// # Returns
///
/// A vector of `ExtractedTypeInfo` representing the extracted TypeScript interfaces with serialization/deserialization info.
pub fn extract_and_convert_types(items: &[Item]) -> Vec<ExtractedTypeInfo> {
    let mut extracted_types = Vec::new();
    let mut defined_types_names = Vec::new(); // Keep track of defined type names for type_to_ts

    for item in items {
        match item {
            Item::Struct(s) => {
                let is_serializable = has_derive_macro(&s.attrs, "Serialize");
                let is_deserializable = has_derive_macro(&s.attrs, "Deserialize");
                let struct_name = s.ident.to_string();

                // Always convert to TS interface if it's a user-defined type, regardless of Serde derives
                let ts_interface = convert_struct_to_ts_interface(s, &defined_types_names);
                extracted_types.push(ExtractedTypeInfo {
                    name: struct_name.clone(),
                    ts_interface,
                    is_serializable,
                    is_deserializable,
                });
                defined_types_names.push(struct_name);
            }
            Item::Enum(e) => {
                let is_serializable = has_derive_macro(&e.attrs, "Serialize");
                let is_deserializable = has_derive_macro(&e.attrs, "Deserialize");
                let enum_name = e.ident.to_string();

                // Always convert to TS enum if it's a user-defined type, regardless of Serde derives
                let ts_interface = convert_enum_to_ts_enum(e, &defined_types_names);
                extracted_types.push(ExtractedTypeInfo {
                    name: enum_name.clone(),
                    ts_interface,
                    is_serializable,
                    is_deserializable,
                });
                defined_types_names.push(enum_name);
            }
            _ => {}
        }
    }
    extracted_types
}

pub(crate) fn has_derive_macro(attrs: &[Attribute], macro_name: &str) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("derive") {
            if let Ok(list) = attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            ) {
                return list
                    .iter()
                    .any(|path| path.segments.last().is_some_and(|s| s.ident == macro_name));
            }
        }
        false
    })
}

pub(crate) fn convert_struct_to_ts_interface(
    s: &ItemStruct,
    defined_types: &[String],
) -> serde_json::Value {
    let struct_name = s.ident.to_string();
    let doc_comment = extract_doc_comments(&s.attrs);
    let mut fields_ts = Vec::new();

    if let Fields::Named(fields) = &s.fields {
        for field in &fields.named {
            let field_name = field.ident.as_ref().unwrap().to_string();
            let field_type = type_to_ts(&field.ty, defined_types, false);
            let field_doc_comment = extract_doc_comments(&field.attrs);
            fields_ts.push(serde_json::json!({
                "name": field_name,
                "type": field_type,
                "doc_comment": field_doc_comment,
            }));
        }
    }

    serde_json::json!({
        "type": "interface",
        "name": struct_name,
        "doc_comment": doc_comment,
        "fields": fields_ts,
    })
}

pub(crate) fn convert_enum_to_ts_enum(e: &ItemEnum, defined_types: &[String]) -> serde_json::Value {
    let enum_name = e.ident.to_string();
    let doc_comment = extract_doc_comments(&e.attrs);
    let mut variants_ts = Vec::new();

    for variant in &e.variants {
        let variant_name = variant.ident.to_string();
        let variant_doc_comment = extract_doc_comments(&variant.attrs);
        let mut variant_info = serde_json::Map::new();
        variant_info.insert(
            "name".to_string(),
            serde_json::Value::String(variant_name.clone()),
        );
        variant_info.insert(
            "doc_comment".to_string(),
            serde_json::Value::String(variant_doc_comment),
        );

        match &variant.fields {
            Fields::Unit => {
                variant_info.insert(
                    "type".to_string(),
                    serde_json::Value::String("unit".to_string()),
                );
            }
            Fields::Unnamed(fields) => {
                // Tuple Variant
                let types: Vec<String> = fields
                    .unnamed
                    .iter()
                    .map(|f| type_to_ts(&f.ty, defined_types, false))
                    .collect();
                variant_info.insert(
                    "type".to_string(),
                    serde_json::Value::String("tuple".to_string()),
                );
                variant_info.insert(
                    "members".to_string(),
                    serde_json::Value::Array(
                        types.into_iter().map(serde_json::Value::String).collect(),
                    ),
                );
            }
            Fields::Named(fields) => {
                // Struct Variant
                let fields_str: Vec<serde_json::Value> = fields
                    .named
                    .iter()
                    .map(|f| {
                        let field_name = f.ident.as_ref().unwrap().to_string();
                        let field_type = type_to_ts(&f.ty, defined_types, false);
                        let field_doc_comment = extract_doc_comments(&f.attrs);
                        serde_json::json!({
                            "name": field_name,
                            "type": field_type,
                            "doc_comment": field_doc_comment
                        })
                    })
                    .collect();
                variant_info.insert(
                    "type".to_string(),
                    serde_json::Value::String("struct".to_string()),
                );
                variant_info.insert("members".to_string(), serde_json::Value::Array(fields_str));
            }
        }
        variants_ts.push(serde_json::Value::Object(variant_info));
    }

    serde_json::json!({
        "type": "enum",
        "name": enum_name,
        "doc_comment": doc_comment,
        "variants": variants_ts,
    })
}

pub(crate) fn type_to_ts(
    ty: &Type,
    defined_types: &[String],
    is_tauri_command_type: bool,
) -> String {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let ident_str = segment.ident.to_string();
                match ident_str.as_str() {
                    "String" => "string".to_string(),
                    "bool" => "boolean".to_string(),
                    "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64"
                    | "i128" | "usize" | "isize" | "f32" | "f64" => "number".to_string(),
                    "Option" => {
                        // Option<T> を T | undefined に変換
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
                            {
                                let inner_ts_type =
                                    type_to_ts(inner_type, defined_types, is_tauri_command_type);
                                return format!("{inner_ts_type} | undefined");
                            }
                        }
                        "any".to_string() // 内部型が特定できない場合のフォールバック
                    }
                    "Vec" => {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first()
                            {
                                let inner_ts_type =
                                    type_to_ts(inner_type, defined_types, is_tauri_command_type);
                                return format!("{inner_ts_type}[]");
                            }
                        }
                        "any[]".to_string() // 内部型が特定できない場合のフォールバック
                    }
                    "HashMap" => {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            let types: Vec<&syn::GenericArgument> = args.args.iter().collect();
                            if types.len() == 2 {
                                if let (
                                    syn::GenericArgument::Type(key_type),
                                    syn::GenericArgument::Type(value_type),
                                ) = (types[0], types[1])
                                {
                                    let key_ts_type =
                                        type_to_ts(key_type, defined_types, is_tauri_command_type);
                                    let value_ts_type = type_to_ts(
                                        value_type,
                                        defined_types,
                                        is_tauri_command_type,
                                    );
                                    return format!("Record<{key_ts_type}, {value_ts_type}>");
                                }
                            }
                        }
                        "Record<any, any>".to_string() // 内部型が特定できない場合のフォールバック
                    }
                    "Result" => {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(ok_type)) = args.args.first() {
                                let ok_ts_type =
                                    type_to_ts(ok_type, defined_types, is_tauri_command_type);
                                return ok_ts_type;
                            }
                        }
                        "any".to_string() // 内部型が特定できない場合のフォールバック
                    }
                    _ => {
                        if is_tauri_command_type || !defined_types.contains(&ident_str) {
                            format!("T.{ident_str}")
                        } else {
                            ident_str
                        }
                    }
                }
            } else {
                "any".to_string() // パスが空の場合のフォールバック
            }
        }
        Type::Reference(type_ref) => {
            // &str を string に変換
            if let Type::Path(path) = &*type_ref.elem {
                if let Some(segment) = path.path.segments.last() {
                    if segment.ident == "str" {
                        return "string".to_string();
                    }
                }
            }
            type_to_ts(&type_ref.elem, defined_types, is_tauri_command_type) // 参照されている型を再帰的に変換
        }
        Type::Tuple(type_tuple) => {
            if type_tuple.elems.is_empty() {
                "void".to_string()
            } else {
                let elems_ts: Vec<String> = type_tuple
                    .elems
                    .iter()
                    .map(|elem| type_to_ts(elem, defined_types, is_tauri_command_type))
                    .collect();
                format!("[{}]", elems_ts.join(", "))
            }
        }
        _ => "any".to_string(), // その他の複雑な型に対するフォールバック
    }
}

/// `use`文を解析してエイリアスのマップを作成する
fn extract_use_aliases(items: &[Item]) -> HashMap<String, String> {
    let mut aliases = HashMap::new();
    for item in items {
        if let Item::Use(use_item) = item {
            parse_use_tree(&mut aliases, &use_item.tree, Vec::new());
        }
    }
    aliases
}

/// `UseTree`を再帰的に解析してエイリアスを見つける
fn parse_use_tree(
    aliases: &mut HashMap<String, String>,
    tree: &UseTree,
    current_path: Vec<String>,
) {
    match tree {
        UseTree::Path(use_path) => {
            let mut new_path = current_path.clone();
            new_path.push(use_path.ident.to_string());
            parse_use_tree(aliases, &use_path.tree, new_path);
        }
        UseTree::Name(use_name) => {
            let mut full_path = current_path;
            full_path.push(use_name.ident.to_string());
            // use a::B; のようなケース。 B as B と同じとして扱う
            aliases.insert(full_path.last().unwrap().clone(), full_path.join("::"));
        }
        UseTree::Rename(use_rename) => {
            let mut full_path = current_path;
            full_path.push(use_rename.ident.to_string());
            aliases.insert(use_rename.rename.to_string(), full_path.join("::"));
        }
        UseTree::Group(use_group) => {
            for tree_in_group in &use_group.items {
                parse_use_tree(aliases, tree_in_group, current_path.clone());
            }
        }
        UseTree::Glob(_) => {
            // `*` は今回は無視
        }
    }
}

/// Extracts Tauri commands from the given Rust items.
///
/// This function iterates through the given Rust items and extracts any functions
/// marked with the `#[tauri::command]` attribute. It then converts them into a
/// `serde_json::Value` representation for TypeScript generation.
///
/// # Arguments
///
/// * `items` - A slice of `syn::Item` representing the parsed Rust code.
/// * `defined_types` - A slice of strings containing the names of user-defined types.
///
/// # Returns
///
/// A vector of `serde_json::Value` representing the extracted Tauri commands.
pub fn extract_tauri_commands(
    items: &[Item],
    all_extracted_types: &[ExtractedTypeInfo],
) -> Vec<serde_json::Value> {
    let mut functions = Vec::new();
    let aliases = extract_use_aliases(items);

    // defined_types_names を all_extracted_types から構築
    let defined_types_names: Vec<String> = all_extracted_types
        .iter()
        .map(|info| info.name.clone())
        .collect();

    for item in items {
        if let Item::Fn(func) = item {
            if has_tauri_command(&func.attrs) {
                let fn_name = func.sig.ident.to_string();
                let doc_comment = extract_doc_comments(&func.attrs);
                let mut args_ts = Vec::new();
                let mut invoke_obj = Vec::new();

                for input in &func.sig.inputs {
                    if let FnArg::Typed(pat_type) = input {
                        if is_ignored_tauri_type(&pat_type.ty, &aliases) {
                            continue; // 無視対象のTauri型はスキップ
                        }

                        let name = match &*pat_type.pat {
                            Pat::Ident(ident) => ident.ident.to_string(),
                            _ => "arg".to_string(),
                        };
                        let ty_str = type_to_ts(&pat_type.ty, &defined_types_names, true);

                        // 引数の型がユーザー定義型の場合、Deserializeが必須
                        let user_defined_types_in_arg =
                            get_user_defined_type_names(&pat_type.ty, &defined_types_names);
                        let mut all_args_deserializable = true;
                        for user_type_name in &user_defined_types_in_arg {
                            if let Some(type_info) = all_extracted_types
                                .iter()
                                .find(|info| &info.name == user_type_name)
                            {
                                if !type_info.is_deserializable {
                                    debug!(
                                        "Skipping argument {name} because its nested type {user_type_name} is not Deserializable.",
                                    );
                                    all_args_deserializable = false;
                                    break;
                                }
                            }
                        }

                        if !all_args_deserializable {
                            continue; // Deserializable でない型を含む場合はスキップ
                        }

                        args_ts.push(format!("{name}: {ty_str}"));
                        invoke_obj.push(format!("{name}: {name}"));
                    }
                }

                let ret_ty = match &func.sig.output {
                    syn::ReturnType::Type(_, ty) => {
                        // Result<(), E> を void に変換する処理
                        let mut is_result_unit = false;
                        if let Type::Path(type_path) = &**ty {
                            if let Some(segment) = type_path.path.segments.last() {
                                if segment.ident == "Result" {
                                    if let syn::PathArguments::AngleBracketed(args) =
                                        &segment.arguments
                                    {
                                        if let Some(syn::GenericArgument::Type(Type::Tuple(
                                            tuple,
                                        ))) = args.args.first()
                                        {
                                            if tuple.elems.is_empty() {
                                                is_result_unit = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        let mut final_ret_ty = if is_result_unit {
                            "void".to_string()
                        } else if is_tauri_ipc_response(ty, &aliases) {
                            "unknown".to_string()
                        } else {
                            type_to_ts(ty, &defined_types_names, true)
                        };

                        // 戻り値の型がユーザー定義型の場合、Serializeが必須
                        let user_defined_types_in_ret =
                            get_user_defined_type_names(ty, &defined_types_names);
                        for user_type_name in &user_defined_types_in_ret {
                            if let Some(type_info) = all_extracted_types
                                .iter()
                                .find(|info| &info.name == user_type_name)
                            {
                                if !type_info.is_serializable {
                                    debug!(
                                        "Changing return type of function {fn_name} to unknown because its nested type {user_type_name} is not Serializable.",
                                    );
                                    final_ret_ty = "unknown".to_string(); // Serializable でない場合は unknown に変更
                                    break;
                                }
                            }
                        }
                        final_ret_ty
                    }
                    _ => "void".to_string(),
                };

                let func_json = serde_json::json!({
                    "name": fn_name,
                    "doc_comment": doc_comment,
                    "args": args_ts,
                    "invoke_args": invoke_obj,
                    "return_type": ret_ty,
                });
                debug!("DEBUG: func_json = {func_json:?}");
                functions.push(func_json);
            }
        }
    }
    functions
}

// Helper to get user-defined type names from a syn::Type, searching recursively.
fn get_user_defined_type_names(ty: &Type, defined_types_names: &[String]) -> Vec<String> {
    let mut user_defined_types = Vec::new();

    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let ident_str = segment.ident.to_string();
                if defined_types_names.contains(&ident_str) {
                    user_defined_types.push(ident_str);
                }

                // Recursively search for generic arguments
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg {
                            user_defined_types
                                .extend(get_user_defined_type_names(inner_ty, defined_types_names));
                        }
                    }
                }
            }
        }
        Type::Reference(type_ref) => {
            user_defined_types.extend(get_user_defined_type_names(
                &type_ref.elem,
                defined_types_names,
            ));
        }
        Type::Tuple(type_tuple) => {
            for elem_ty in &type_tuple.elems {
                user_defined_types
                    .extend(get_user_defined_type_names(elem_ty, defined_types_names));
            }
        }
        _ => {}
    }

    user_defined_types
}

fn is_ignored_tauri_type(ty: &Type, aliases: &HashMap<String, String>) -> bool {
    match ty {
        Type::Path(type_path) => {
            let segments: Vec<_> = type_path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect();
            let path_str = segments.join("::");

            let final_path = if segments.len() == 1 {
                // エイリアスの場合 (例: `MyWindow`)
                aliases
                    .get(&path_str)
                    .map(|s| s.as_str())
                    .unwrap_or(&path_str)
            } else {
                // フルパスの場合 (例: `tauri::WebviewWindow`)
                &path_str
            };

            // `State` はジェネリクスを持つため、前方一致で判定
            if final_path.starts_with("tauri::State") {
                return true;
            }

            IGNORED_TAURI_TYPES.contains(&final_path)
        }
        Type::Reference(type_ref) => {
            // 参照型の場合、再帰的にチェック
            is_ignored_tauri_type(&type_ref.elem, aliases)
        }
        _ => false,
    }
}

fn is_tauri_ipc_response(ty: &Type, aliases: &HashMap<String, String>) -> bool {
    match ty {
        Type::Path(type_path) => {
            let segments: Vec<_> = type_path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect();
            let path_str = segments.join("::");

            let final_path = if segments.len() == 1 {
                aliases
                    .get(&path_str)
                    .map(|s| s.as_str())
                    .unwrap_or(&path_str)
            } else {
                &path_str
            };

            final_path == "tauri::ipc::Response"
        }
        Type::Reference(type_ref) => is_tauri_ipc_response(&type_ref.elem, aliases),
        _ => false,
    }
}

pub(crate) fn has_tauri_command(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        let path = attr.path();
        // #[command] の形式
        if path.is_ident("command") {
            return true;
        }
        // #[tauri::command] の形式
        if path.segments.len() == 2
            && path.segments[0].ident == "tauri"
            && path.segments[1].ident == "command"
        {
            return true;
        }
        false
    })
}

/// Extracts documentation comments from the given attributes.
///
/// This function filters the attributes for `#[doc]` comments and concatenates them
/// into a single string, with each comment on a new line.
///
/// # Arguments
///
/// * `attrs` - A slice of `syn::Attribute` to extract the doc comments from.
///
/// # Returns
///
/// A string containing the concatenated doc comments.
pub fn extract_doc_comments(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let Meta::NameValue(meta_name_value) = &attr.meta {
                    if let syn::Expr::Lit(expr_lit) = &meta_name_value.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            return Some(lit_str.value().trim().to_string());
                        }
                    }
                }
            }
            None
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_and_convert_types() {
        let rust_code = r#"
            #[derive(Serialize, Deserialize)]
            struct MyStruct {
                field1: String,
            }

            #[derive(Serialize, Deserialize)]
            enum MyEnum {
                VariantA,
            }

            struct NotConvertedStruct;

            #[derive(Serialize)]
            struct OnlySerializable;

            #[derive(Deserialize)]
            struct OnlyDeserializable;

            struct NoDerive;
        "#;
        let syntax = syn::parse_file(rust_code).unwrap();
        let extracted_types = extract_and_convert_types(&syntax.items);

        assert_eq!(extracted_types.len(), 6);

        let my_struct = extracted_types
            .iter()
            .find(|info| info.name == "MyStruct")
            .unwrap();
        assert!(my_struct.is_serializable);
        assert!(my_struct.is_deserializable);

        let my_enum = extracted_types
            .iter()
            .find(|info| info.name == "MyEnum")
            .unwrap();
        assert!(my_enum.is_serializable);
        assert!(my_enum.is_deserializable);

        let not_converted_struct = extracted_types
            .iter()
            .find(|info| info.name == "NotConvertedStruct")
            .unwrap();
        assert!(!not_converted_struct.is_serializable);
        assert!(!not_converted_struct.is_deserializable);

        let only_serializable = extracted_types
            .iter()
            .find(|info| info.name == "OnlySerializable")
            .unwrap();
        assert!(only_serializable.is_serializable);
        assert!(!only_serializable.is_deserializable);

        let only_deserializable = extracted_types
            .iter()
            .find(|info| info.name == "OnlyDeserializable")
            .unwrap();
        assert!(!only_deserializable.is_serializable);
        assert!(only_deserializable.is_deserializable);

        let no_derive = extracted_types
            .iter()
            .find(|info| info.name == "NoDerive")
            .unwrap();
        assert!(!no_derive.is_serializable);
        assert!(!no_derive.is_deserializable);
    }

    #[test]
    fn test_extract_tauri_commands_with_special_types() {
        let rust_code = r#"
            use tauri::{WebviewWindow as MyWindow, State, AppHandle, Window};
            use tauri::ipc::Response as IpcResponse;

            #[tauri::command]
            fn command_with_aliases(window: MyWindow, state: State<String>, app: AppHandle, tauri_window: Window, message: String) {}

            #[tauri::command]
            fn command_returns_response() -> tauri::ipc::Response {}

            #[tauri::command]
            fn command_returns_alias_response() -> IpcResponse {}

            #[tauri::command]
            fn command_with_window(window: Window, message: String) {}
        "#;
        let syntax = syn::parse_file(rust_code).unwrap();
        let defined_types = vec![];
        let functions = extract_tauri_commands(&syntax.items, &defined_types);

        assert_eq!(functions.len(), 4);

        // エイリアスを使ったコマンド
        assert_eq!(functions[0]["name"], "command_with_aliases");
        let args0 = functions[0]["args"].as_array().unwrap();
        assert_eq!(args0.len(), 1);
        assert_eq!(args0[0], "message: string");

        // tauri::ipc::Response を返すコマンド
        assert_eq!(functions[1]["name"], "command_returns_response");
        assert_eq!(functions[1]["return_type"], "unknown");

        // エイリアスされた tauri::ipc::Response を返すコマンド
        assert_eq!(functions[2]["name"], "command_returns_alias_response");
        assert_eq!(functions[2]["return_type"], "unknown");

        // tauri::Window を引数に持つコマンド
        assert_eq!(functions[3]["name"], "command_with_window");
        let args3 = functions[3]["args"].as_array().unwrap();
        assert_eq!(args3.len(), 1);
        assert_eq!(args3[0], "message: string");
    }

    #[test]
    fn test_is_ignored_tauri_type() {
        let rust_code = r#"
            use tauri::{WebviewWindow as MyWindow, State as MyState, AppHandle as MyAppHandle, Window as MyTauriWindow};
            struct WebviewWindow {}
        "#;
        let syntax = syn::parse_file(rust_code).unwrap();
        let aliases = extract_use_aliases(&syntax.items);

        // WebviewWindow
        let ty1: Type = syn::parse_str("tauri::WebviewWindow").unwrap();
        assert!(is_ignored_tauri_type(&ty1, &aliases));
        let ty2: Type = syn::parse_str("&tauri::WebviewWindow").unwrap();
        assert!(is_ignored_tauri_type(&ty2, &aliases));
        let ty3: Type = syn::parse_str("MyWindow").unwrap();
        assert!(is_ignored_tauri_type(&ty3, &aliases));

        // State
        let ty4: Type = syn::parse_str("tauri::State<String>").unwrap();
        assert!(is_ignored_tauri_type(&ty4, &aliases));
        let ty5: Type = syn::parse_str("MyState<String>").unwrap();
        assert!(is_ignored_tauri_type(&ty5, &aliases));

        // AppHandle
        let ty6: Type = syn::parse_str("tauri::AppHandle").unwrap();
        assert!(is_ignored_tauri_type(&ty6, &aliases));
        let ty7: Type = syn::parse_str("MyAppHandle").unwrap();
        assert!(is_ignored_tauri_type(&ty7, &aliases));

        // Window
        let ty8: Type = syn::parse_str("tauri::Window").unwrap();
        assert!(is_ignored_tauri_type(&ty8, &aliases));
        let ty9: Type = syn::parse_str("&tauri::Window").unwrap();
        assert!(is_ignored_tauri_type(&ty9, &aliases));
        let ty10: Type = syn::parse_str("MyTauriWindow").unwrap();
        assert!(is_ignored_tauri_type(&ty10, &aliases));

        // 無関係な型
        let ty11: Type = syn::parse_str("String").unwrap();
        assert!(!is_ignored_tauri_type(&ty11, &aliases));
        let ty12: Type = syn::parse_str("my_tauri::WebviewWindow").unwrap();
        assert!(!is_ignored_tauri_type(&ty12, &aliases));
        let ty13: Type = syn::parse_str("WebviewWindow").unwrap();
        assert!(!is_ignored_tauri_type(&ty13, &aliases));
    }

    #[test]
    fn test_is_tauri_ipc_response() {
        let rust_code = r#"use tauri::ipc::Response as IpcResponse;"#;
        let syntax = syn::parse_file(rust_code).unwrap();
        let aliases = extract_use_aliases(&syntax.items);

        // 正しい tauri::ipc::Response 型
        let ty1: Type = syn::parse_str("tauri::ipc::Response").unwrap();
        assert!(is_tauri_ipc_response(&ty1, &aliases));

        // 参照型
        let ty2: Type = syn::parse_str("&tauri::ipc::Response").unwrap();
        assert!(is_tauri_ipc_response(&ty2, &aliases));

        // エイリアス型
        let ty3: Type = syn::parse_str("IpcResponse").unwrap();
        assert!(is_tauri_ipc_response(&ty3, &aliases));

        // 無関係な型
        let ty4: Type = syn::parse_str("String").unwrap();
        assert!(!is_tauri_ipc_response(&ty4, &aliases));
        let ty5: Type = syn::parse_str("tauri::Response").unwrap();
        assert!(!is_tauri_ipc_response(&ty5, &aliases));
    }

    #[test]
    fn test_has_tauri_command() {
        let item_code = r#"
            #[tauri::command]
            fn my_command() {}
        "#;
        let parsed_item: Item = syn::parse_str(item_code).unwrap();
        if let Item::Fn(func) = parsed_item {
            assert!(has_tauri_command(&func.attrs));
        } else {
            panic!("Expected a function");
        }

        let item_code = r#"
            #[command]
            fn another_command() {}
        "#;
        let parsed_item: Item = syn::parse_str(item_code).unwrap();
        if let Item::Fn(func) = parsed_item {
            assert!(has_tauri_command(&func.attrs));
        } else {
            panic!("Expected a function");
        }

        let item_code = r#"
            fn not_a_command() {}
        "#;
        let parsed_item: Item = syn::parse_str(item_code).unwrap();
        if let Item::Fn(func) = parsed_item {
            assert!(!has_tauri_command(&func.attrs));
        }
    }

    #[test]
    fn test_extract_doc_comments() {
        let item_code = r#"
            /// This is a doc comment.
            /// # Arguments
            /// * `arg1` - The first argument.
            /// * `arg1` - The first argument.
            /// # Returns
            /// The result.
            fn my_function() {}
        "#;
        let parsed_item: Item = syn::parse_str(item_code).unwrap();
        if let Item::Fn(func) = parsed_item {
            let doc_comment = extract_doc_comments(&func.attrs);
            assert_eq!(
                doc_comment.replace("\r\n", "\n"),
                "This is a doc comment.\n# Arguments\n* `arg1` - The first argument.\n* `arg1` - The first argument.\n# Returns\nThe result.".replace("\r\n", "\n")
            );
        } else {
            panic!("Expected a function");
        }

        let item_code = r#"
            /// Single line doc comment.
            fn another_function() {}
        "#;
        let parsed_item: Item = syn::parse_str(item_code).unwrap();
        if let Item::Fn(func) = parsed_item {
            let doc_comment = extract_doc_comments(&func.attrs);
            assert_eq!(doc_comment, "Single line doc comment.");
        }

        let item_code = r#"
            fn no_doc_function() {}
        "#;
        let parsed_item: Item = syn::parse_str(item_code).unwrap();
        if let Item::Fn(func) = parsed_item {
            let doc_comment = extract_doc_comments(&func.attrs);
            assert_eq!(doc_comment, "");
        }
    }

    #[test]
    fn test_has_derive_macro() {
        let item_code = r#"
            #[derive(Serialize, Deserialize)]
            struct MyStruct;
        "#;
        let parsed_item: Item = syn::parse_str(item_code).unwrap();
        if let Item::Struct(s) = parsed_item {
            assert!(has_derive_macro(&s.attrs, "Serialize"));
            assert!(has_derive_macro(&s.attrs, "Deserialize"));
            assert!(!has_derive_macro(&s.attrs, "Debug"));
        } else {
            panic!("Expected a struct");
        }

        let item_code = r#"
            #[derive(Debug)]
            struct AnotherStruct;
        "#;
        let parsed_item: Item = syn::parse_str(item_code).unwrap();
        if let Item::Struct(s) = parsed_item {
            assert!(has_derive_macro(&s.attrs, "Debug"));
            assert!(!has_derive_macro(&s.attrs, "Serialize"));
        } else {
            panic!("Expected a struct");
        }

        let item_code = r#"
            struct NoDeriveStruct;
        "#;
        let parsed_item: Item = syn::parse_str(item_code).unwrap();
        if let Item::Struct(s) = parsed_item {
            assert!(!has_derive_macro(&s.attrs, "Serialize"));
        } else {
            panic!("Expected a struct");
        }

        let item_code = r#"
            #[derive(serde::Serialize)]
            struct NamespacedStruct;
        "#;
        let parsed_item: Item = syn::parse_str(item_code).unwrap();
        if let Item::Struct(s) = parsed_item {
            assert!(has_derive_macro(&s.attrs, "Serialize"));
            assert!(!has_derive_macro(&s.attrs, "Deserialize"));
        } else {
            panic!("Expected a struct");
        }
    }

    #[test]
    fn test_convert_struct_to_ts_interface() {
        // 基本的な構造体
        let struct_code = r#"
            struct MyStruct {
                field1: String,
                field2: u32,
            }
        "#;
        let parsed_struct: ItemStruct = syn::parse_str(struct_code).unwrap();
        let defined_types = vec!["MyStruct".to_string()];
        let result = convert_struct_to_ts_interface(&parsed_struct, &defined_types);
        assert_eq!(result["name"], "MyStruct");
        assert_eq!(result["type"], "interface");
        assert_eq!(result["doc_comment"], "");
        assert_eq!(result["fields"].as_array().unwrap().len(), 2);
        assert_eq!(result["fields"][0]["name"], "field1");
        assert_eq!(result["fields"][0]["type"], "string");
        assert_eq!(result["fields"][1]["name"], "field2");
        assert_eq!(result["fields"][1]["type"], "number");

        // ドキュメントコメントを持つ構造体
        let struct_code = r#"
            /// This is a struct doc comment.
            struct AnotherStruct {
                field1: bool,
            }
        "#;
        let parsed_struct: ItemStruct = syn::parse_str(struct_code).unwrap();
        let defined_types = vec!["AnotherStruct".to_string()];
        let result = convert_struct_to_ts_interface(&parsed_struct, &defined_types);
        assert_eq!(result["name"], "AnotherStruct");
        assert_eq!(result["doc_comment"], "This is a struct doc comment.");

        // フィールドにドキュメントコメントを持つ構造体
        let struct_code = r#"
            struct StructWithFieldDoc {
                /// Doc for field_a
                field_a: String,
                field_b: u32,
            }
        "#;
        let parsed_struct: ItemStruct = syn::parse_str(struct_code).unwrap();
        let defined_types = vec!["StructWithFieldDoc".to_string()];
        let result = convert_struct_to_ts_interface(&parsed_struct, &defined_types);
        assert_eq!(result["name"], "StructWithFieldDoc");
        assert_eq!(result["fields"][0]["name"], "field_a");
        assert_eq!(result["fields"][0]["doc_comment"], "Doc for field_a");
        assert_eq!(result["fields"][1]["name"], "field_b");
        assert_eq!(result["fields"][1]["doc_comment"], "");
    }

    #[test]
    fn test_convert_enum_to_ts_enum() {
        // Unit Variant
        let enum_code = r#"
            #[derive(Serialize, Deserialize)]
            enum MyEnum {
                /// Doc for VariantA
                VariantA,
                VariantB,
            }
        "#;
        let parsed_enum: ItemEnum = syn::parse_str(enum_code).unwrap();
        let defined_types = vec!["MyEnum".to_string()];
        let result = convert_enum_to_ts_enum(&parsed_enum, &defined_types);
        assert_eq!(result["name"], "MyEnum");
        assert_eq!(result["doc_comment"], ""); // Enum level doc comment is empty
        assert_eq!(result["variants"][0]["name"], "VariantA");
        assert_eq!(result["variants"][0]["doc_comment"], "Doc for VariantA");
        assert_eq!(result["variants"][0]["type"], "unit");

        // Tuple Variant
        let enum_code = r#"
            #[derive(Serialize, Deserialize)]
            enum MyTupleEnum {
                VariantC(String, u32),
            }
        "#;
        let parsed_enum: ItemEnum = syn::parse_str(enum_code).unwrap();
        let defined_types = vec!["MyTupleEnum".to_string()];
        let result = convert_enum_to_ts_enum(&parsed_enum, &defined_types);
        assert_eq!(result["name"], "MyTupleEnum");
        assert_eq!(result["variants"][0]["name"], "VariantC");
        assert_eq!(result["variants"][0]["type"], "tuple");
        assert_eq!(result["variants"][0]["members"][0], "string");
        assert_eq!(result["variants"][0]["members"][1], "number");

        // Struct Variant
        let enum_code = r#"
            #[derive(Serialize, Deserialize)]
            enum MyStructEnum {
                VariantD {
                    /// Doc for field1
                    field1: String,
                    field2: bool,
                },
            }
        "#;
        let parsed_enum: ItemEnum = syn::parse_str(enum_code).unwrap();
        let defined_types = vec!["MyStructEnum".to_string()];
        let result = convert_enum_to_ts_enum(&parsed_enum, &defined_types);
        assert_eq!(result["name"], "MyStructEnum");
        assert_eq!(result["variants"][0]["name"], "VariantD");
        assert_eq!(result["variants"][0]["type"], "struct");
        assert_eq!(result["variants"][0]["members"][0]["name"], "field1");
        assert_eq!(result["variants"][0]["members"][0]["type"], "string");
        assert_eq!(
            result["variants"][0]["members"][0]["doc_comment"],
            "Doc for field1"
        );
        assert_eq!(result["variants"][0]["members"][1]["name"], "field2");
        assert_eq!(result["variants"][0]["members"][1]["type"], "boolean");

        // Enum level doc comment
        let enum_code = r#"
            /// Doc for MyEnumWithDoc
            #[derive(Serialize, Deserialize)]
            enum MyEnumWithDoc {
                VariantA,
            }
        "#;
        let parsed_enum: ItemEnum = syn::parse_str(enum_code).unwrap();
        let defined_types = vec!["MyEnumWithDoc".to_string()];
        let result = convert_enum_to_ts_enum(&parsed_enum, &defined_types);
        assert_eq!(result["name"], "MyEnumWithDoc");
        assert_eq!(result["doc_comment"], "Doc for MyEnumWithDoc");
    }

    #[test]
    fn test_type_to_ts() {
        let defined_types = vec!["MyStruct".to_string(), "MyEnum".to_string()];

        // 基本的な型
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>("String").unwrap(),
                &defined_types,
                false
            ),
            "string"
        );
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>("bool").unwrap(),
                &defined_types,
                false
            ),
            "boolean"
        );
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>("u32").unwrap(),
                &defined_types,
                false
            ),
            "number"
        );
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>("f64").unwrap(),
                &defined_types,
                false
            ),
            "number"
        );

        // Option<T>型
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>(r#"Option<String>"#).unwrap(),
                &defined_types,
                false
            ),
            "string | undefined"
        );
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>(r#"Option<u32>"#).unwrap(),
                &defined_types,
                false
            ),
            "number | undefined"
        );
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>(r#"Option<MyStruct>"#).unwrap(),
                &defined_types,
                false
            ),
            "MyStruct | undefined"
        );

        // 参照型
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>(r#"&str"#).unwrap(),
                &defined_types,
                false
            ),
            "string"
        );

        // カスタム型
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>(r#"MyStruct"#).unwrap(),
                &defined_types,
                false
            ),
            "MyStruct"
        );
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>(r#"MyEnum"#).unwrap(),
                &defined_types,
                false
            ),
            "MyEnum"
        );

        // その他の型（フォールバック）
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>(r#"Vec<String>"#).unwrap(),
                &defined_types,
                false
            ),
            "string[]"
        );
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>(r#"HashMap<String, u32>"#).unwrap(),
                &defined_types,
                false
            ),
            "Record<string, number>"
        );

        // Tauriコマンドの型の場合
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>(r#"MyStruct"#).unwrap(),
                &defined_types,
                true
            ),
            "T.MyStruct"
        );
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>(r#"String"#).unwrap(),
                &defined_types,
                true
            ),
            "string"
        );

        // Result<T, E> 型
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>(r#"Result<String, String>"#).unwrap(),
                &defined_types,
                false
            ),
            "string"
        );
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>(r#"Result<u32, MyError>"#).unwrap(),
                &defined_types,
                false
            ),
            "number"
        );
        assert_eq!(
            type_to_ts(
                &syn::parse_str::<Type>(r#"Result<MyStruct, MyError>"#).unwrap(),
                &defined_types,
                false
            ),
            "MyStruct"
        );
    }
}
