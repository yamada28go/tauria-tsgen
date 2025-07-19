// SerdeクレートのSerialize（シリアライズ）とDeserialize（デシリアライズ）をインポート
use serde::{Deserialize, Serialize};

/// 商品情報を表す構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    /// 商品ID（ユニークな識別子）
    pub product_id: String,

    /// 商品の価格（小数対応）
    pub price: f64,

    /// 在庫数（単位数）
    pub quantity: u32,
}

/// 商品情報を取得するTauriコマンド
///
/// # 引数
/// * `product_id` - 商品の識別子
///
/// # 戻り値
/// 指定された商品IDに対応する商品情報（ダミーデータ）
#[tauri::command]
pub fn get_product_data(product_id: String) -> Product {
    // ダミーの商品データを返す
    Product { 
        product_id, 
        price: 99.99, 
        quantity: 1 
    }
}