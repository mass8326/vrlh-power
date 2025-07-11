<script lang="ts">
  import play from "$lib/icons/mingcute--play-fill.svg?raw";
  import pause from "$lib/icons/mingcute--pause-fill.svg?raw";
  import stop from "$lib/icons/mingcute--stop-fill.svg?raw";
  import type { DeviceInfo } from "@vrlh/core";
  import { slide } from "svelte/transition";
  import Command from "./command.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { status } from "$lib/status.svelte";

  interface Props {
    device: DeviceInfo;
  }
  const { device }: Props = $props();
  const { addr, name, local, remote } = $derived(device);
  const pending = $derived(local !== "Disconnected");

  function createOnclick(cmd: number): () => void {
    return function onclick() {
      invoke("power", { id: device.id, cmd }).catch((err: unknown) => {
        status.push(JSON.stringify(err));
      });
    };
  }
</script>

<li
  class="p-2 pl-3 flex justify-between b-(1 black) bg-neutral-800 rounded font-mono"
  transition:slide
>
  <div>
    <div class="flex gap-2 items-center">
      <h3 class="text-3xl font-bold">
        {name}
      </h3>
      {#if pending}
        <div
          class={[
            "h-2 w-2 rounded-full animate-pulse",
            local === "Initializing"
              ? "bg-yellow-800"
              : local === "Connected"
                ? "bg-blue-800"
                : "bg-red-800",
          ]}
          title={typeof device.local === "string"
            ? device.local
            : device.local && device.local.Error}
        ></div>
      {/if}
    </div>
    <div class="-mt-1 text-sm font-italic">
      {addr}
    </div>
  </div>
  <div class="flex b-(1 black) rounded divide-(x-1 black) overflow-hidden">
    <Command
      onclick={createOnclick(0)}
      icon={stop}
      {pending}
      active={remote === "Stopped"}
    />
    <Command
      onclick={createOnclick(2)}
      icon={pause}
      {pending}
      active={remote === "Standby"}
    />
    <Command
      onclick={createOnclick(1)}
      icon={play}
      {pending}
      active={remote === "Active"}
    />
  </div>
</li>
