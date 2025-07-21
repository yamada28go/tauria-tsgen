import { Event, UnlistenFn } from "@tauri-apps/api/event";
import * as W from "@tauri-apps/api/window";

abstract class MainWindowEventHandlers {
    private readonly unlistenFns: Promise<UnlistenFn>[] = [];

    protected constructor() {
        const appWebview = W.getCurrentWebviewWindow();
        
        this.unlistenFns.push(
            appWebview.listen<T.EventPayload>('window-event', (event) => { this.OnWindowEvent(event); }));
        
    }

    public async Unlisten() {
        for (const x of this.unlistenFns) {
            await x;
        }
    }

    
    abstract OnWindowEvent(event: Event<T.EventPayload>): void;
    
}
