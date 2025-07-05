<script lang="ts">
  import "uno.css";
  import "@unocss/reset/tailwind.css";

  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { SvelteMap } from "svelte/reactivity";

  interface DeviceUpdatePayload {
    id: string;
    name: string | null;
    power: string;
  }

  let pending = $state<Promise<void>>();
  let devices = new SvelteMap<string, DeviceUpdatePayload>();

  onMount(async () => {
    pending = invoke("discover");
    void listen<DeviceUpdatePayload>("device-update", ({ payload }) => {
      devices.set(payload.id, {
        ...payload,
        name: payload.name ?? devices.get(payload.id)?.name ?? "[Unavailable]",
      });
    });
  });

  function createOnclick(id: string, power: string): () => void {
    return function onclick() {
      switch (power) {
        case "Power on":
          return invoke("power_off", { id });
        case "Power off":
          return invoke("power_on", { id });
      }
    };
  }

  function getAction(power: string): string {
    switch (power) {
      case "Power on":
        return "Turn Off";
      case "Power off":
        return "Turn On";
      default:
        return "Unavailable";
    }
  }
</script>

<main>
  <ul class="p-4 flex flex-col gap-4">
    {#each devices.entries() as [id, { name, power }]}
      {@const action = getAction(power)}
      <li class="p-4 b-(1 black) bg-neutral-800 rounded space-y-2">
        <div>
          <h3>
            <span class="text-2xl font-bold">{name}</span>
            <span class="font-italic">{id}</span>
          </h3>
          <div>Status: {power}</div>
        </div>
        <button
          class="px-4 py-2 bg-blue-900 b-(1 blue-950) rounded disabled:(b-black bg-neutral-900 cursor-not-allowed)"
          disabled={action === "Unavailable"}
          onclick={createOnclick(id, power)}
        >
          {action}
        </button>
      </li>
    {/each}
    {#await pending}
      <li class="p-4 b-(1 black) bg-neutral-800 rounded space-y-2">
        Scanning for lighthouses...
      </li>
    {:then}
      {#if devices.size === 0}
        <li class="p-4 b-(1 black) bg-neutral-800 rounded space-y-2">
          No lighthouses found!
        </li>
      {/if}
    {/await}
  </ul>
</main>

<style>
  :root {
    --at-apply: text-white bg-neutral-700;
  }
</style>
