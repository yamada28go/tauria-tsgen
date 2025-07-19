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

## Input and Output Examples

### Rust Input Example (`src/commands.rs`)

```rust
use tauri::{command, AppHandle, State, WebviewWindow};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
}

#[command]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[command]
pub fn get_user(id: u32) -> User {
    User { id, name: "Test User".to_string() }
}

#[command]
pub fn update_user(user: User, app_handle: AppHandle) -> Result<String, String> {
    // app_handle is ignored on the TypeScript side
    Ok(format!("User {} updated.", user.name))
}

#[command]
pub fn get_state(state: State<'_, String>) -> String {
    // state is ignored on the TypeScript side
    state.inner().clone()
}

#[command]
pub fn close_window(window: WebviewWindow) {
    // window is ignored on the TypeScript side
    window.close().unwrap();
}
```

### Generated TypeScript Output Example (`src/bindings/commands/Commands.ts`)

```typescript
// src/bindings/commands/Commands.ts

import { invoke } from '@tauri-apps/api/tauri';

export interface User {
  id: number;
  name: string;
}

export class Commands {
  static async greet(name: string): Promise<string> {
    return await invoke('greet', { name });
  }

  static async getUser(id: number): Promise<User> {
    return await invoke('get_user', { id });
  }

  static async updateUser(user: User): Promise<string> {
    return await invoke('update_user', { user });
  }

  static async getState(): Promise<string> {
    return await invoke('get_state');
  }

  static async closeWindow(): Promise<void> {
    return await invoke('close_window');
  }
}
```

### Generated TypeScript Output Example (`src/bindings/types/User.ts`)

```typescript
// src/bindings/types/User.ts

export interface User {
  id: number;
  name: string;
}
```

### Generated JavaScript Output Example (`src/bindings/commands/Commands.js`)

```javascript
// src/bindings/commands/Commands.js

import { invoke } from '@tauri-apps/api/tauri';

export class Commands {
  static async greet(name) {
    return await invoke('greet', { name });
  }

  static async getUser(id) {
    return await invoke('get_user', { id });
  }

  static async updateUser(user) {
    return await invoke('update_user', { user });
  }

  static async getState() {
    return await invoke('get_state');
  }

  static async closeWindow() {
    return await invoke('close_window');
  }
}
```

## Generated File Directory Structure

`tauria-tsgen` generates TypeScript files while maintaining the directory structure of the input Rust files.
For example, if you specify `./src-tauri/src` for `--input-path` and `./src/bindings` for `--output-path`, files will be generated with the following directory structure:

```
./src/bindings/
├───commands/
│   └───Commands.ts
├───types/
│   └───User.ts
└───index.ts
```

- `commands/`: TypeScript wrapper functions corresponding to Rust files with `#[tauri::command]` are generated. The file names are determined based on the Rust module names.
- `types/`: TypeScript interfaces and types corresponding to Rust `struct`s, `enum`s, etc., are generated.
- `index.ts`: This is an entry point file that exports all generated commands and types.

## For Developers

### Testing Policy

Based on the unit testing philosophy of Mr. Takuto Wada, we emphasize "tests that verify behavior." The primary targets for testing are the service layers containing business logic, and the use of mocks and stubs is kept to a minimum.

### Coding Conventions

-   Rust code adheres to the official **Rust Style Guide**.
-   Use `clippy` and resolve all warnings.

### Reviews

-   We actively conduct code reviews to maintain quality and consistency.
