// SerdeクレートのSerialize（シリアライズ）とDeserialize（デシリアライズ）をインポート
use serde::{Deserialize, Serialize};

/// ユーザー情報を表す構造体
#[derive( Serialize, Deserialize)]
pub struct User {
    /// ユーザーID（ユニークな識別子）
    pub id: u32,

    /// ユーザーの名前
    pub name: String,

    /// ユーザーのメールアドレス（オプション）
    pub email: Option<String>,
}

/// ユーザー情報を取得するTauriコマンド
///
/// # 引数
/// * `id` - ユーザーのID
///
/// # 戻り値
/// 指定されたIDに対応するユーザー情報（ダミーデータ）
#[tauri::command]
pub fn get_user_data(id: u32) -> User {
    // ダミーのユーザーデータを返す
    User { 
        id, 
        name: "Test User".to_string(), 
        email: None 
    }
}
