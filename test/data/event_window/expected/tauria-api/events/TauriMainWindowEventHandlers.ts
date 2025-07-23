import { Event, listen, UnlistenFn } from "@tauri-apps/api/event";
import * as T from "../../interface/types";

export abstract class TauriMainWindowEventHandlers {
    private readonly unlistenFns: Promise<UnlistenFn>[] = [];

    protected constructor() {
        
        this.unlistenFns.push(
            listen<T.EventPayload>('window-event', (event) => { this.OnWindowEvent(event); }));
        
    }

    public async Unlisten() {
        for (const x of this.unlistenFns) {
            await x;
        }
    }

    
    abstract OnWindowEvent(event: Event<T.EventPayload>): void;
    
}
