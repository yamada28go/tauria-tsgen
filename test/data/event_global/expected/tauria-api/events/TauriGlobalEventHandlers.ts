import { Event, listen, UnlistenFn } from "@tauri-apps/api/event";



abstract class TauriGlobalEventHandlers {
    private readonly unlistenFns: Promise<UnlistenFn>[] = [];

    protected constructor() {
        
        this.unlistenFns.push(
            listen<string>('global', (event) => { this.OnGlobal(event); }));
        
    }

    public async Unlisten() {
        for (const x of this.unlistenFns) {
            await x;
        }
    }

    
    abstract OnGlobal(event: Event<string>): void;
    
}
