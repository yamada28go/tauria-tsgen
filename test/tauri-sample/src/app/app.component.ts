import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet } from '@angular/router';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { FormsModule } from '@angular/forms';
import { createLib, MainWindowEventHandlers } from './external/tauri-api';
import { ILib } from './external/tauri-api/interface/commands/Lib';
import { open } from '@tauri-apps/plugin-dialog';
import { MatCheckboxModule } from '@angular/material/checkbox';

import { MatExpansionModule  } from '@angular/material/expansion';
import { UnlistenFn, Event } from '@tauri-apps/api/event';


// --- Global Window ---


class ImpGlobal implements GlobalEventHandlers {
  constructor() {
    super();
  }

  OnGlobal(event: Event<string>): void {
    debugger;
    console.log(event.payload);
  }
}

// --- Main Window ---


class ImpHoge extends MainWindowEventHandlers {
  override OnCoreMsg(event: Event<string>): void {
    throw new Error('Method not implemented.');
  }
  constructor() {
    super();
  }

  OnLoggedIn(event: Event<string>): void {
    debugger;
    console.log(event.payload);
  }
}
// --- Main Window ---

const l = new ImpHoge();
const g = new ImpGlobal();

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [
    CommonModule,
    RouterOutlet,
    MatButtonModule,
    MatFormFieldModule,
    MatInputModule,
    FormsModule,
    MatCheckboxModule,
    MatExpansionModule,
  ],
  templateUrl: './app.component.html',
  styleUrl: './app.component.css',
})
export class AppComponent {
  lib: ILib;

  // greet_command
  greetName = '';
  greetMessage = '';

  // multiplication_command
  multiplicationA = 10;
  multiplicationB = 20;
  multiplicationResult = '';

  // error_handling_command
  isError = false;
  errorMessage = '';

  // read_file_command
  fileContent = '';

  // app_handle_command
  bundleIdentifier = '';

  // webview_window_command
  webviewLabel = '';

  // webview_window_with_arg_command
  webviewLabelWithArgName = 'Guest';
  webviewLabelWithArg = '';

  // complex_state_command
  complexStateNumber = 1;
  complexStateResult = '';

  constructor() {
    this.lib = createLib();
  }

  async greet() {
    this.greetMessage = await this.lib.greetCommand(this.greetName);
  }

  async multiplication() {
    const result = await this.lib.multiplicationCommand({
      a: this.multiplicationA,
      b: this.multiplicationB,
    });
    this.multiplicationResult = result.toString();
  }

  async testError() {
    try {
      const result = await this.lib.errorHandlingCommand(this.isError);
      this.errorMessage = result;
    } catch (e) {
      if (e instanceof Error) {
        this.errorMessage = e.message;
      } else {
        this.errorMessage = String(e);
      }
    }
  }

  async readFile() {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'Text',
          extensions: ['txt', 'rs', 'ts'],
        },
      ],
    });
    if (selected && !Array.isArray(selected)) {
      const result = await this.lib.readFileCommand(selected);
      if (result instanceof Uint8Array) {
        this.fileContent = new TextDecoder().decode(result);
      } else {
        this.fileContent = 'Received non-binary data';
      }
    } else {
      this.fileContent = 'No file selected';
    }
  }

  async getBundleIdentifier() {
    this.bundleIdentifier = await this.lib.appHandleCommand();
  }

  async getWebviewLabel() {
    this.webviewLabel = await this.lib.webviewWindowCommand();
  }

  async getWebviewLabelWithArg() {
    this.webviewLabelWithArg = await this.lib.webviewWindowWithArgCommand(
      this.webviewLabelWithArgName
    );
  }

  async getComplexState() {
    const result = await this.lib.complexStateCommand(this.complexStateNumber);
    this.complexStateResult = JSON.stringify(result, null, 2);
  }
}
function getCurrentWebviewWindow() {
  throw new Error('Function not implemented.');
}
