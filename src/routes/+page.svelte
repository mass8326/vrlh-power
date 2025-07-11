<script lang="ts">
  import "overlayscrollbars/overlayscrollbars.css";

  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import * as remeda from "remeda";
  import { onMount } from "svelte";
  import type { DeviceInfo, DeviceRemoteStatus } from "@vrlh/core";
  import { SvelteMap } from "svelte/reactivity";
  import play from "$lib/icons/mingcute--play-fill.svg?raw";
  import pause from "$lib/icons/mingcute--pause-fill.svg?raw";
  import stop from "$lib/icons/mingcute--stop-fill.svg?raw";
  import { status } from "$lib/status.svelte";
  import Device from "./device.svelte";
  import Command from "./command.svelte";
  import { OverlayScrollbarsComponent } from "overlayscrollbars-svelte";

  let pending = $state(true);
  const devices = new SvelteMap<string, DeviceInfo>();

  onMount(() => {
    const cleanup: (() => void)[] = [];
    void discover(5);
    void listen<DeviceInfo>("device-update", ({ payload }) => {
      const existing = devices.get(payload.addr);
      devices.set(payload.addr, {
        ...payload,
        local: payload.local ?? existing?.local ?? null,
        remote: payload.remote ?? existing?.remote ?? null,
      });
    }).then((unlisten) => cleanup.push(unlisten));
    return () => {
      for (const fn of cleanup) fn();
    };
  });

  async function discover(duration: number) {
    pending = true;
    try {
      await invoke("discover", { duration });
    } finally {
      pending = false;
    }
  }

  const COMMAND_MAP = new Map<number, DeviceRemoteStatus>([
    [0, "Stopped"],
    [1, "Active"],
    [2, "Standby"],
  ]);
  function createOnclick(cmd: number): () => void {
    return function onclick() {
      for (const device of devices.values()) {
        const matcher = COMMAND_MAP.get(cmd);
        if (matcher === device.remote) continue;
        invoke("power", { id: device.id, cmd }).catch((err: unknown) => {
          status.push(JSON.stringify(err));
        });
      }
    };
  }

  const arr = $derived(
    remeda.pipe(
      [...devices.values()],
      remeda.filter(({ local }) => local !== "Ignored"),
      remeda.sortBy(remeda.prop("name")),
    ),
  );
</script>

<OverlayScrollbarsComponent
  element="main"
  class="p-2.5 min-h-0 flex-1 overflow-auto"
>
  <div class="space-y-2">
    <div class="flex gap-2 items-center">
      <button
        class={[
          "px-4 py-2 bg-blue-900 b-(1 blue-950) rounded font-bold transition-colors",
          "hover:(bg-blue-700 b-blue-800)",
          "disabled:(b-black bg-neutral-900 cursor-not-allowed)",
        ]}
        onclick={() => discover(10)}
        disabled={pending || undefined}
      >
        Refresh
      </button>
      <div class="flex b-(1 black) rounded divide-(x-1 black) overflow-hidden">
        <Command onclick={createOnclick(0)} icon={stop} />
        <Command onclick={createOnclick(2)} icon={pause} />
        <Command onclick={createOnclick(1)} icon={play} />
      </div>
    </div>
    <ul class="flex flex-col gap-2">
      {#each arr as device (device.addr)}
        <Device {device} />
      {/each}
      {#if !pending && devices.size === 0}
        <li class="p-4 b-(1 black) bg-neutral-800 rounded space-y-2">
          No lighthouses found!
        </li>
      {/if}
    </ul>
  </div>
</OverlayScrollbarsComponent>
<footer class="px-4 py-1 bg-neutral-800 b-t-(1 black) font-italic">
  {status.current}
</footer>
