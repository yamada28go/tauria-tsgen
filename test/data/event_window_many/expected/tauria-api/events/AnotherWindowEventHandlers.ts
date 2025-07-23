import { Event, listen, UnlistenFn } from "@tauri-apps/api/event";
import * as T from "../../interface/types";

abstract class AnotherWindowEventHandlers {
    private readonly unlistenFns: Promise<UnlistenFn>[] = [];

    protected constructor() {
        
        this.unlistenFns.push(
            listen<string>('another_main_event', (event) => { this.OnAnotherMainEvent(event); }));
        
    }

    public async Unlisten() {
        for (const x of this.unlistenFns) {
            await x;
        }
    }

    
    abstract OnAnotherMainEvent(event: Event<string>): void;
    
}
