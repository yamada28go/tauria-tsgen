import { Event, listen, UnlistenFn } from '@tauri-apps/api/event';
import * as T from '../../interface/types';

export abstract class TauriMainWindowEventHandlers {
  private readonly unlistenFns: Promise<UnlistenFn>[] = [];

  protected constructor() {
    this.unlistenFns.push(
      listen<string>('core-msg', (event) => {
        this.OnCoreMsg(event);
      })
    );
  }

  public async Unlisten() {
    for (const x of this.unlistenFns) {
      await x;
    }
  }

  abstract OnCoreMsg(event: Event<string>): void;
}
