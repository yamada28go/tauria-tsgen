# tauria-tsgen (npm package)

This is the npm package for `tauria-tsgen`, a CLI tool that automatically generates TypeScript interfaces and wrapper functions from Rust Tauri commands.

## Installation

To install the `tauria-tsgen` CLI tool, run the following command:

```bash
npm install tauria-tsgen --save-dev
```

## Usage

After installation, you can use the `tauria-tsgen` command directly:

```bash
taura-tsgen --help
```

### Recommended Usage (with configuration file)

For more complex projects, it is recommended to use a configuration file.

1.  **Create a configuration file**

    Create a file named `tauria-tsgen-config.json` in your project root:

    ```json
    {
      "input_path": "src-tauri/src",
      "output_path": "src/app/external/tauri-api"
    }
    ```

2.  **Add a script to `package.json`**

    Add the following script to your `package.json`:

    ```json
    "scripts": {
      "maketsif": "tauria-tsgen --config tauria-tsgen-config.json"
    }
    ```

3.  **Run the script**

    You can now generate the TypeScript interfaces by running:

    ```bash
    npm run maketsif
    ```

For detailed usage instructions, command-line arguments, and examples, please refer to the main GitHub repository:

[tauria-tsgen GitHub Repository](https://github.com/yamada28go/tauria-tsgen#readme)

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

### Generated TypeScript Output Example (output_directory/interface/commands/Cmd1.ts)

```typescript
// src/bindings/commands/Cmd1.ts

import { invoke } from '@tauri-apps/api/tauri';

export class Cmd1 {
  static async command1(): Promise<string> {
    return await invoke('command1');
  }
}
```

### Generated TypeScript Output Example (output_directory/interface/commands/Cmd2.ts)

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
import { createCmd } from './tauria-api'; // Assuming your TypeScript config or bundler resolves from the output directory
// Or, if the output directory is 'src/generated-api' and you're importing from 'src/main.ts':
// import { createCmd } from './generated-api/tauria-api';

async function callTauriCommands() {
  const cmdApi = createCmd(); // Instantiate Cmd class

  try {
    // Call the Rust command1
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

## License

This project is licensed under the MIT License. See the [LICENSE](../../LICENSE) file for details.
