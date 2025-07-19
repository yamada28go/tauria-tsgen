# tauria-tsgen

[View the Japanese version here](./README.ja.md)

## Overview

`tauria-tsgen` is a tool that automatically generates TypeScript interfaces and wrapper functions from the commands of a Tauri application written in Rust. This enables type-safe and efficient calling of Rust commands in the front-end development of Tauri applications.

## Key Features

-   **Automatic Identification and TypeScript Conversion of Rust Commands:**
    -   Automatically identifies functions with the `#[tauri::command]` attribute from specified Rust files.
    -   Generates corresponding TypeScript type definitions and asynchronous wrapper functions based on the argument and return types of the identified Rust functions.

-   **Automatic Ignoring of Tauri-Specific Arguments:**
    -   Automatically detects and ignores argument types used internally by the Tauri framework, such as `tauri::WebviewWindow`, `tauri::State`, and `tauri::AppHandle`, during TypeScript interface generation.
    -   Accurately identifies and ignores these types even when used with aliases (`use ... as ...`) or as references (`&`).

-   **Type-Safe Conversion of Special Return Types:**
    -   For Rust commands that return the `tauri::ipc::Response` type, it generates the `unknown` type on the TypeScript side. This encourages developers to explicitly cast the specific type of the low-level IPC response, maintaining type safety.

-   **Directory Structure Preservation and Mocking Feature:**
    -   Outputs TypeScript functions in a way that preserves the Rust directory structure.
    -   Each TypeScript class is planned to have a feature that allows loading JSON data for mock behavior.

## Tech Stack

-   **Rust**
    -   `clap`: Command-line argument parsing
    -   `serde`, `serde_json`: Configuration file reading
    -   `syn`, `quote`: Rust code parsing and code generation
    -   `log`, `env_logger`: Logging
    -   `tera`: Template engine
    -   `anyhow`: Error handling
    -   `convert_case`: Case conversion
    -   `rust-embed`: Embedding template files

## Usage

### Displaying Version Information

To display the application's version information, run the following command.

```bash
cargo run -- --version
```

### Execution

Run the following command in the project's root directory.

```bash
cargo run
```

### Command-Line Arguments

You can run the tool by specifying a configuration file or by providing paths directly.

-   `-c <FILE>`, `--config <FILE>`: Specifies the path to the configuration file. The configuration file must be in JSON format and include `input_path` and `output_path`.

    **Example `config.json`:**

    ```json
    {
      "input_path": "src/commands.rs",
      "output_path": "src/tauriCommands.ts"
    }
    ```

    **Example execution with a configuration file:**

    ```bash
    cargo run -- -c config.json
    ```

-   `--input-path <DIR>`: Specifies the path to the directory containing the input Rust code.
-   `--output-path <DIR>`: Specifies the path to the directory where the generated TypeScript files will be saved.

    **Example execution with direct paths:**

    ```bash
    cargo run -- --input-path ./src-tauri/src --output-path ./src/bindings
    ```

-   `--mock-api`: If this flag is specified, mock API files will also be generated.

### Logging

You can control the log output level by setting the `RUST_LOG` environment variable.

Example: Output logs at the `info` level or higher

```bash
RUST_LOG=info cargo run -- -c config.json
```

Available log levels: `error`, `warn`, `info`, `debug`, `trace`

## For Developers

### Testing Policy

Based on the unit testing philosophy of Mr. Takuto Wada, we emphasize "tests that verify behavior." The primary targets for testing are the service layers containing business logic, and the use of mocks and stubs is kept to a minimum.

### Coding Conventions

-   Rust code adheres to the official **Rust Style Guide**.
-   Use `clippy` and resolve all warnings.

### Reviews

-   We actively conduct code reviews to maintain quality and consistency.
