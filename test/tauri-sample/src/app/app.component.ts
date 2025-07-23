import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet } from '@angular/router';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { FormsModule } from '@angular/forms';
import { createLib } from './external/tauri-api';
import { ILib } from './external/tauri-api/interface/commands/Lib';
import { open } from '@tauri-apps/plugin-dialog';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';

import { MatExpansionModule } from '@angular/material/expansion';
import { UnlistenFn, Event } from '@tauri-apps/api/event';
import { TauriMainWindowEventHandlers } from './external/tauri-api/tauria-api/events/TauriMainWindowEventHandlers';
import { TauriGlobalEventHandlers } from './external/tauri-api/tauria-api/events/TauriGlobalEventHandlers';

// --- Global Window ---

class ImpGlobal extends TauriGlobalEventHandlers {
  constructor(private snackBar: MatSnackBar) {
    super();
  }

     OnGlobal(event: Event<string>): void{
      this.snackBar.open(`Global Event: ${event.payload}`, 'Close', { duration: 3000 });
     }


}

// --- Main Window ---

class ImpWindow extends TauriMainWindowEventHandlers {
  override OnCoreMsg(event: Event<string>): void {
    this.snackBar.open(`Core Message: ${event.payload}`, 'Close', { duration: 3000 });
  }
  constructor(private snackBar: MatSnackBar) {
    super();
  }

  OnLoggedIn(event: Event<string>): void {
    debugger;
    console.log(event.payload);
    this.snackBar.open(`Logged In: ${event.payload}`, 'Close', { duration: 3000 });
  }
}
// --- Main Window ---


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
    MatSnackBarModule,
  ],
  templateUrl: './app.component.html',
  styleUrl: './app.component.css',
})
export class AppComponent {
  lib: ILib;

  // イベントハンドラ
  private  eventWindows ;
private  eventGlobal ;


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

  // call_msg_global
  callMsgGlobalResult = '';

  // call_msg_main
  callMsgMainResult = '';

  constructor(private snackBar: MatSnackBar) {
    this.lib = createLib();
    this.eventWindows = new ImpWindow(this.snackBar);
    this.eventGlobal = new ImpGlobal(this.snackBar);
  }

async  ngOnDestroy() {

   await this.eventWindows.Unlisten();
   await this.eventGlobal.Unlisten();

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

  async callMsgGlobal() {
    this.callMsgGlobalResult = await this.lib.callMsgGlobal();
  }

  async callMsgMain() {
    this.callMsgMainResult = await this.lib.callMsgMain();
  }

}