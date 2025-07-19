# tauria-tsgen

## 概要

`tauria-tsgen` は、Rustで記述されたTauriアプリケーションのコマンドから、TypeScriptのインターフェースとラッパー関数を自動生成するツールです。これにより、Tauriアプリケーションのフロントエンド開発において、型安全で効率的なRustコマンドの呼び出しを可能にします。

## 主な機能

-   **Rustコマンドの自動識別とTypeScript変換:**
    -   指定されたRustファイルから `#[tauri::command]` アトリビュートが付与された関数を自動的に識別します。
    -   識別されたRust関数の引数と戻り値の型に基づいて、対応するTypeScriptの型定義と非同期ラッパー関数を生成します。

-   **Tauri固有の引数の自動無視:**
    -   `tauri::WebviewWindow`、`tauri::State`、`tauri::AppHandle` といったTauriフレームワークが内部的に使用する引数型を自動的に検出し、TypeScriptのインターフェース生成時にこれらを無視します。
    -   これらの型が `use ... as ...` によるエイリアスや参照 (`&`) として使用されている場合でも、正確に識別して無視します。

-   **特殊な戻り値の型安全な変換:**
    -   `tauri::ipc::Response` 型を戻り値とするRustコマンドに対しては、TypeScript側で `unknown` 型を生成します。これにより、低レベルなIPCレスポンスの具体的な型を開発者が明示的にキャストすることを促し、型安全性を維持します。

-   **ディレクトリ構造の維持とモック機能:**
    -   Rustのディレクトリ構造を維持した形でTypeScriptの関数を対応付けて出力します。
    -   各TypeScriptクラスには、JSONデータを読み込み、モック動作が可能な機能を備える予定です。

## 技術スタック

-   **Rust**
    -   `clap`: コマンドライン引数パーシング
    -   `serde`, `serde_json`: 設定ファイルの読み込み
    -   `syn`, `quote`: Rustコードのパースとコード生成
    -   `log`, `env_logger`: ロギング
    -   `tera`: テンプレートエンジン
    -   `anyhow`: エラーハンドリング
    -   `convert_case`: ケース変換
    -   `rust-embed`: テンプレートファイルの埋め込み

## 使用方法

### バージョン情報の表示

アプリケーションのバージョン情報を表示するには、以下のコマンドを実行します。

```bash
cargo run -- --version
```

### 実行

プロジェクトのルートディレクトリで以下のコマンドを実行します。

```bash
cargo run
```

### コマンドライン引数

設定ファイルを指定するか、直接パスを指定して実行します。

-   `-c <FILE>`, `--config <FILE>`: 設定ファイルのパスを指定します。設定ファイルはJSON形式で、`input_path` と `output_path` を含める必要があります。

    **設定ファイルの例 (`config.json`):**

    ```json
    {
      "input_path": "src/commands.rs",
      "output_path": "src/tauriCommands.ts"
    }
    ```

    **設定ファイルを使用した実行例:**

    ```bash
    cargo run -- -c config.json
    ```

-   `--input-path <DIR>`: 入力Rustコードを含むディレクトリへのパスを指定します。
-   `--output-path <DIR>`: 生成されたTypeScriptファイルを保存するディレクトリへのパスを指定します。

    **直接パスを指定した実行例:**

    ```bash
    cargo run -- --input-path ./src-tauri/src --output-path ./src/bindings
    ```

-   `--mock-api`: このフラグを指定すると、モックAPIファイルも生成されます。

### ログ出力

`RUST_LOG`環境変数を設定することで、ログの出力レベルを制御できます。

例: `info`レベル以上のログを出力する

```bash
RUST_LOG=info cargo run -- -c config.json
```

利用可能なログレベル: `error`, `warn`, `info`, `debug`, `trace`

## 開発者向け情報

### テスト方針

和田卓人氏のユニットテスト思想に基づき、「振る舞いを検証するテスト」を重視します。主にビジネスロジックを含むサービス層をテスト対象とし、モックやスタブの使用は最小限に留めます。

### コーディング規約

-   Rustコードは公式の **Rust Style Guide** に準拠します。
-   `clippy` を使用し、すべての警告を解消します。

### レビュー

-   コードレビューを積極的に行い、品質と一貫性を保ちます。