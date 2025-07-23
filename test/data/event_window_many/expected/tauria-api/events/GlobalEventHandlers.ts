import { Event, listen, UnlistenFn } from "@tauri-apps/api/event";


import * as T from "../../interface/types/index"


abstract class GlobalEventHandlers {
    private readonly unlistenFns: Promise<UnlistenFn>[] = [];

    protected constructor() {
        
        this.unlistenFns.push(
            listen<T.SubPayload>('sub_event', (event) => { this.OnSubEvent(event); }));
        
        this.unlistenFns.push(
            listen<string>('another_main_event', (event) => { this.OnAnotherMainEvent(event); }));
        
    }

    public async Unlisten() {
        for (const x of this.unlistenFns) {
            await x;
        }
    }

    
    abstract OnSubEvent(event: Event<T.SubPayload>): void;
    
    abstract OnAnotherMainEvent(event: Event<string>): void;
    
}
