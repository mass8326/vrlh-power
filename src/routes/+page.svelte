<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import * as remeda from "remeda";
  import { onMount } from "svelte";
  import { SvelteMap, SvelteSet } from "svelte/reactivity";
  import { slide } from "svelte/transition";
  import loadingSvg from "$lib/icons/loading.svg?raw";
  import toggleSvg from "$lib/icons/power.svg?raw";
  import { status } from "$lib/status.svelte";

  type DevicePowerCode =
    | "LOADING"
    | "ERROR"
    | "POWERED_ON"
    | "POWERED_OFF"
    | "POWER_PENDING"
    | "POWER_INITIATED"
    | "POWER_UNKNOWN";

  interface DeviceUpdatePayload {
    /** Serializes differently per platorm */
    id: unknown;
    /** Should be consistent across platorms */
    addr: string;
    name: string;
    power: { code: DevicePowerCode; detail: string | null };
  }

  let pending = $state(true);
  const powering = new SvelteSet<string>();
  const devices = new SvelteMap<string, DeviceUpdatePayload>();

  onMount(() => {
    const cleanup: (() => void)[] = [];
    void discover();
    void listen<DeviceUpdatePayload>("device-update", ({ payload }) => {
      const existing = devices.get(payload.addr);
      if (!existing) status.push(`Discovered "${payload.name}"`);
      devices.set(payload.addr, payload);
    }).then((unlisten) => cleanup.push(unlisten));
    return () => {
      for (const fn of cleanup) fn();
    };
  });

  async function discover(seconds?: number) {
    pending = true;
    devices.clear();
    try {
      await invoke("discover", { duration: seconds ?? 15 });
    } finally {
      pending = false;
    }
  }

  const toggles = new Map<DevicePowerCode, string>([
    ["POWERED_ON", "power_off"],
    ["POWERED_OFF", "power_on"],
  ]);

  function createOnclick(device: DeviceUpdatePayload): () => void {
    return function onclick() {
      if (!device) return;
      const { id, power } = device;
      const command = toggles.get(power.code);
      if (!command) return;
      powering.add(device.addr);
      invoke(command, { id })
        .catch((err: unknown) => {
          status.push(JSON.stringify(err));
        })
        .finally(() => {
          powering.delete(device.addr);
        });
    };
  }

  function getAction(power: DevicePowerCode): string {
    switch (power) {
      case "POWERED_ON":
        return "Turn Off";
      case "POWERED_OFF":
        return "Turn On";
      default:
        return "Unavailable";
    }
  }

  function getColors(device: DeviceUpdatePayload): string | undefined {
    if (powering.has(device.addr)) return;
    switch (device.power.code) {
      case "POWERED_ON":
        return "bg-blue-900 b-blue-950 hover:(bg-red-700 b-red-800)";
      case "POWERED_OFF":
        return "bg-red-900 b-red-950 hover:(bg-blue-700 b-blue-800)";
    }
  }
</script>

<div class="h-full flex flex-col justify-between">
  <main class="p-4 space-y-4 min-h-0 flex-1 overflow-auto">
    <div class="flex gap-4 items-center">
      <button
        class={[
          "px-4 py-2 bg-blue-900 b-(1 blue-950) rounded font-bold transition-colors",
          "hover:(bg-blue-700 b-blue-800)",
          "disabled:(b-black bg-neutral-900 cursor-not-allowed)",
        ]}
        onclick={() => discover(30)}
        disabled={pending || undefined}
      >
        Refresh
      </button>
    </div>
    <ul class="flex flex-col gap-4">
      {#each remeda.sortBy([...devices.values()], (device) => device.name) as device (device.addr)}
        {@const { addr, name, power } = device}
        {@const action = getAction(power.code)}
        {@const colors = getColors(device)}
        <li
          class="p-4 flex justify-between b-(1 black) bg-neutral-800 rounded font-mono"
          transition:slide
        >
          <div>
            <h3 class="text-3xl font-bold">
              {name}
            </h3>
            <div class="-mt-1 text-sm font-italic">
              {addr}
            </div>
            <div class="mt-2">
              {power.code}
              {#if power.detail}({JSON.stringify(power.detail)}){/if}
            </div>
          </div>
          <button
            class={[
              "px-4 py-2 b-1 rounded transition-colors",
              "disabled:(b-black bg-neutral-900 cursor-not-allowed)",
              colors,
            ]}
            disabled={colors ? undefined : true}
            onclick={createOnclick(device)}
            title={action}
          >
            <div class={[colors ? undefined : "animate-spin"]}>
              {@html colors ? toggleSvg : loadingSvg}
            </div>
          </button>
        </li>
      {/each}
      {#if !pending && devices.size === 0}
        <li class="p-4 b-(1 black) bg-neutral-800 rounded space-y-2">
          No lighthouses found!
        </li>
      {/if}
    </ul>
  </main>
  <footer class="px-4 py-1 bg-neutral-800 b-t-(1 black) font-italic">
    {status.current}
  </footer>
</div>
