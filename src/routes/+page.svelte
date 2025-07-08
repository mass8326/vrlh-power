<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { SvelteMap } from "svelte/reactivity";
  import toggle from "$lib/icons/power.svg?raw";
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
    name: string | null;
    power: { code: DevicePowerCode; detail: string | null };
  }

  let pending = $state(true);
  let devices = new SvelteMap<string, DeviceUpdatePayload>();

  onMount(async () => {
    void discover();
    void listen<DeviceUpdatePayload>("device-update", ({ payload }) => {
      const existing = devices.get(payload.addr);
      if (!existing)
        status.push(`Discovered "${payload.name ?? payload.addr}"`);
      const name = payload.name ?? existing?.name ?? "[Unavailable]";
      devices.set(payload.addr, { ...payload, name });
    });
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

  function createOnclick(id: unknown, power: DevicePowerCode): () => void {
    return function onclick() {
      switch (power) {
        case "POWERED_ON":
          return invoke("power_off", { id }).catch((err) =>
            status.push(JSON.stringify(err))
          );
        case "POWERED_OFF":
          return invoke("power_on", { id }).catch((err) =>
            status.push(JSON.stringify(err))
          );
      }
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

  function getColors(power: DevicePowerCode): string | undefined {
    switch (power) {
      case "POWERED_ON":
        return "bg-blue-900 b-blue-950 hover:(bg-blue-700 b-blue-800)";
      case "POWERED_OFF":
        return "bg-red-900 b-red-950 hover:(bg-red-700 b-red-800)";
    }
  }
</script>

<div class="h-full flex flex-col justify-between">
  <main class="p-4 space-y-4">
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
      {#each devices.values() as { addr, id, name, power }}
        {@const action = getAction(power.code)}
        {@const colors = getColors(power.code)}
        <li
          class="p-4 flex justify-between b-(1 black) bg-neutral-800 rounded font-mono"
        >
          <div>
            <h3 class="text-3xl font-bold" title={addr}>
              {name}
            </h3>
            <div class="font-italic">
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
            onclick={createOnclick(id, power.code)}
            title={action}
          >
            {@html toggle}
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
