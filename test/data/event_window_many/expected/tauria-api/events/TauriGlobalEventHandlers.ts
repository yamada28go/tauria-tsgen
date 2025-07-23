import { Event, listen, UnlistenFn } from "@tauri-apps/api/event";


import * as T from "../../interface/types/index"


export abstract class TauriGlobalEventHandlers {
    private readonly unlistenFns: Promise<UnlistenFn>[] = [];

    protected constructor() {
        
        this.unlistenFns.push(
            listen<T.SubPayload>('sub_event', (event) => { this.OnSubEvent(event); }));
        
    }

    public async Unlisten() {
        for (const x of this.unlistenFns) {
            await x;
        }
    }

    
    abstract OnSubEvent(event: Event<T.SubPayload>): void;
    
}
