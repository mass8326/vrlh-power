<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import * as remeda from "remeda";
  import { onMount } from "svelte";
  import { SvelteMap } from "svelte/reactivity";
  import { slide } from "svelte/transition";
  import play from "$lib/icons/mingcute--play-fill.svg?raw";
  import pause from "$lib/icons/mingcute--pause-fill.svg?raw";
  import stop from "$lib/icons/mingcute--stop-fill.svg?raw";
  import { status } from "$lib/status.svelte";

  interface Device {
    /** Serializes differently per platorm */
    id: unknown;
    /** Should be consistent across platorms */
    addr: string;
    name: string;
    local?: string;
    remote?: string;
  }

  let pending = $state(true);
  const devices = new SvelteMap<string, Device>();

  onMount(() => {
    const cleanup: (() => void)[] = [];
    void discover();
    void listen<Device>("device-update", ({ payload }) => {
      const existing = devices.get(payload.addr);
      if (!existing) status.push(`Discovered "${payload.name}"`);
      devices.set(payload.addr, {
        ...payload,
        local: payload.local ?? existing?.local,
        remote: payload.remote ?? existing?.remote,
      });
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

  function createOnclick(id: unknown, cmd: number): () => void {
    return function onclick() {
      invoke("power", { id, cmd }).catch((err: unknown) => {
        status.push(JSON.stringify(err));
      });
    };
  }

  const arr = $derived(
    remeda.pipe(
      [...devices.values()],
      remeda.filter(({ local }) => local !== "IGNORED"),
      remeda.sortBy(remeda.prop("name")),
    ),
  );

  function localizeRemoteStatus(device: Device) {
    switch (device.remote) {
      case "00":
        return "STOPPED";
      case "01":
        return "INITIATED";
      case "02":
        return "STANDBY";
      case "08":
        return "ACKNOWLEDGED";
      case "09":
        return "SPINUP";
      case "0B":
        return "ACTIVE";
      default:
        return device.local ?? "[UNAVAILABLE]";
    }
  }
</script>

<div class="h-full flex flex-col justify-between">
  <main class="p-2 space-y-2 min-h-0 flex-1 overflow-auto">
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
    <ul class="flex flex-col gap-2">
      {#each arr as device (device.addr)}
        {@const { id, addr, name, local, remote } = device}
        {@const disabled = local !== "DISCONNECTED"}
        <li
          class="p-2 pl-3 flex justify-between b-(1 black) bg-neutral-800 rounded font-mono"
          transition:slide
        >
          <div>
            <div class="flex gap-2 items-center">
              <h3 class="text-3xl font-bold">
                {name}
              </h3>
              {#if disabled}
                <div
                  class={[
                    "h-2 w-2 rounded-full animate-pulse",
                    local === "INITIALIZING"
                      ? "bg-yellow-800"
                      : local === "CONNECTED"
                        ? "bg-blue-800"
                        : "bg-red-800",
                  ]}
                  title={localizeRemoteStatus(device)}
                ></div>
              {/if}
            </div>
            <div class="-mt-1 text-sm font-italic">
              {addr}
            </div>
          </div>
          <div
            class="flex b-(1 black) rounded divide-(x-1 black) overflow-hidden"
          >
            {@render command({
              device: id,
              cmd: 0,
              icon: stop,
              disabled,
              active: remote === "00",
            })}
            {@render command({
              device: id,
              cmd: 2,
              icon: pause,
              disabled,
              active: remote === "02",
            })}
            {@render command({
              device: id,
              cmd: 1,
              icon: play,
              disabled,
              active: remote === "0B",
            })}
          </div>
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

{#snippet command(opts: {
  device: unknown;
  class?: string;
  disabled?: boolean;
  active?: boolean;
  icon: string;
  cmd: number;
})}
  <button
    class={[
      "h-full p-2 transition-colors bg-neutral-900 hover:bg-neutral-700",
      "disabled:(bg-neutral-950 cursor-not-allowed data-[active]:bg-neutral-700)",
      opts.class,
    ]}
    disabled={opts.disabled || opts.active || undefined}
    data-active={opts.active || undefined}
    onclick={createOnclick(opts.device, opts.cmd)}
  >
    {@html opts.icon}
  </button>
{/snippet}
