#!/bin/zsh

# /test/dataの結果ファイルを一括で作成する

# /test/data ディレクトリの絶対パス
TEST_DATA_DIR="test/data"

# TEST_DATA_DIR 以下のディレクトリを一覧表示し、各ディレクトリに対して処理を実行
for dir in "$TEST_DATA_DIR"/*; do
    if [ -d "$dir" ]; then
        DIR_NAME=$(basename "$dir")
        INPUT_PATH="${TEST_DATA_DIR}/${DIR_NAME}/src"
        OUTPUT_PATH="${TEST_DATA_DIR}/${DIR_NAME}/expected"

        echo "Processing directory: $DIR_NAME"
        echo "Input Path: $INPUT_PATH"
        echo "Output Path: $OUTPUT_PATH"

        # 既存の出力ディレクトリを削除 (冪等性を保つため)
        if [ -d "$OUTPUT_PATH" ]; then
            echo "Removing existing output directory: $OUTPUT_PATH"
            rm -rf "$OUTPUT_PATH"
        fi

        # cargo run コマンドを実行
        # --input-path と --output-path を指定
        ( RUST_LOG=debug cargo run -- --input-path "$INPUT_PATH" --output-path "$OUTPUT_PATH" )

        echo "----------------------------------------"
    fi
done

echo "All test directories processed."
