import { listen } from "@tauri-apps/api/event";

class Status {
  private _history = $state(["Initializing..."]);
  
  public constructor() {
     listen<string>("status", ({ payload }) => 
    void  status.push(payload)
    );
  }

  public get current(): string {
    return this._history[0];
  }

  public push(payload: string) {
    const len = this._history.unshift(payload);
    if (len > 20) this._history.pop();
  }
}

export const status = new Status();
