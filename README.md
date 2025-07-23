# tauria-tsgen

[View the Japanese version here](./README.ja.md)

## Overview

`tauria-tsgen` is a tool that automatically generates TypeScript interfaces and wrapper functions from the commands of a Tauri application written in Rust. This enables type-safe and efficient calling of Rust commands in the front-end development of Tauri applications.

## Key Features

-   **Automatic Identification and TypeScript Conversion of Rust Code Tauri Commands:**
    -   Automatically identifies functions with the `#[tauri::command]` attribute from specified Rust files.
    -   Generates corresponding TypeScript type definitions and asynchronous wrapper functions based on the argument and return types of the identified Rust functions.

-   **Automatic Ignoring of Tauri-Specific Arguments:**
    -   Automatically detects and ignores argument types used internally by the Tauri framework, such as `tauri::WebviewWindow`, `tauri::State`, and `tauri::AppHandle`, during TypeScript interface generation.
    -   Accurately identifies and ignores these types even when used with aliases (`use ... as ...`) or as references (`&`).

-   **Type-Safe Conversion of Special Return Types:**
    -   For Rust commands that return the `tauri::ipc::Response` type, it generates the `unknown` type on the TypeScript side. This encourages developers to explicitly cast the specific type of the low-level IPC response, maintaining type safety.

-   **Automatic Generation of Event Handlers:**
    -   Automatically generates type-safe TypeScript event handlers for both global and window-specific Tauri events, simplifying event subscription and handling in the frontend.

-   **Directory Structure Preservation and Mocking Feature:**
    -   Outputs TypeScript functions in a way that preserves the Rust directory structure.
    -   Each TypeScript class is planned to have a feature that allows loading JSON data for mock behavior.

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
      "input_path": "src-tauri/src",
      "output_path": "src/bindings"
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

## Input and Output Examples

### Rust Input Example (`src/cmd1.rs`)

```rust
use tauri::command;

#[command]
pub fn command1() -> String {
    "Command 1 executed".to_string()
}
```

### Rust Input Example (`src/cmd2.rs`)

```rust
use tauri::command;

#[command]
pub fn command2() -> String {
    "Command 2 executed".to_string()
}
```

### Generated TypeScript Output Example (`src/bindings/commands/Cmd1.ts`)

```typescript
// src/bindings/commands/Cmd1.ts

import { invoke } from '@tauri-apps/api/tauri';

export class Cmd1 {
  static async command1(): Promise<string> {
    return await invoke('command1');
  }
}
```

### Generated TypeScript Output Example (output_directory/interface/commands/Cmd1.ts)

```typescript
// src/bindings/commands/Cmd2.ts

import { invoke } from '@tauri-apps/api/tauri';

export class Cmd2 {
  static async command2(): Promise<string> {
    return await invoke('command2');
  }
}
```

## Generated File Directory Structure

`tauria-tsgen` generates TypeScript files while maintaining the directory structure of the input Rust files.
For example, if you specify `./src/bindings` for `--output-path`, files will be generated directly within that directory with the following structure:

```
./src/bindings/  <-- This is the directory specified by --output-path
├───tauria-api/
│   ├───Cmd1.ts
│   ├───Cmd2.ts
│   └───index.ts
├───interface/
│   ├───commands/
│   │   ├───Cmd1.ts
│   │   └───Cmd2.ts
│   └───types/
│       └───index.ts
└───index.ts
```

- `tauria-api/`: Wrapper functions that directly call Tauri's `invoke` function are generated.
- `interface/commands/`: TypeScript interfaces corresponding to functions with `#[tauri::command]` are generated. The file names are determined based on the Rust module names.
- `interface/types/`: TypeScript interfaces and types corresponding to Rust `struct`s, `enum`s, etc., are generated.
- `index.ts`: This is an entry point file that exports all generated commands and types.

### Usage Example of Generated API

The Tauri command wrappers generated by `tauria-tsgen` can be instantiated via a factory function, allowing type-safe calls to Rust commands.

#### TypeScript Usage Example

```typescript
import { createCmd } from './src/bindings/tauria-api'; // Adjust according to your output path

async function callTauriCommands() {
  const cmdApi = createCmd(); // Instantiate Cmd class

  try {
    // Call the Rust get_user_data command
    const result = await cmdApi.command1();
    console.log('Result of command1:', result);

    // Other commands can be called similarly
    // const result = await cmdApi.some_other_command();
    // console.log('Result of other command:', result);

  } catch (error) {
    console.error('Error calling Tauri command:', error);
  }
}

callTauriCommands();
```

## For Developers

### Testing Policy

Based on the unit testing philosophy of Mr. Takuto Wada, we emphasize "tests that verify behavior." The primary targets for testing are the service layers containing business logic, and the use of mocks and stubs is kept to a minimum.

### Coding Conventions

-   Rust code adheres to the official **Rust Style Guide**.
-   Use `clippy` and resolve all warnings.

### Reviews

-   We actively conduct code reviews to maintain quality and consistency.
