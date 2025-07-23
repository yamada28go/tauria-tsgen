import { Event, UnlistenFn } from "@tauri-apps/api/event";
import * as W from "@tauri-apps/api/window";
import * as T from "../../interface/types";

abstract class MainWindowEventHandlers {
    private readonly unlistenFns: Promise<UnlistenFn>[] = [];

    protected constructor() {
        const appWebview = W.getCurrentWebviewWindow();
        
        this.unlistenFns.push(
            appWebview.listen<T.EventPayload>('window-event', (event) => { this.OnWindowEvent(event); }));
        
        this.unlistenFns.push(
            appWebview.listen<T.MainPayload>('main_event', (event) => { this.OnMainEvent(event); }));
        
        this.unlistenFns.push(
            appWebview.listen<string>('another_main_event', (event) => { this.OnAnotherMainEvent(event); }));
        
    }

    public async Unlisten() {
        for (const x of this.unlistenFns) {
            await x;
        }
    }

    
    abstract OnWindowEvent(event: Event<T.EventPayload>): void;
    
    abstract OnMainEvent(event: Event<T.MainPayload>): void;
    
    abstract OnAnotherMainEvent(event: Event<string>): void;
    
}
