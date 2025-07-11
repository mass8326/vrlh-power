<script lang="ts">
  interface Props {
    onclick: () => void;
    class?: string;
    pending?: boolean;
    active?: boolean;
    icon: string;
  }
  const props: Props = $props();
</script>

<button
  class={[
    "h-full p-2 transition-colors bg-neutral-900 hover:bg-neutral-700",
    "disabled:cursor-not-allowed",
    "data-[pending]:bg-neutral-950",
    "data-[active]:bg-neutral-700",
    props.class,
  ]}
  disabled={props.active || props.pending || undefined}
  data-active={props.active || undefined}
  data-pending={props.pending || undefined}
  onclick={props.onclick}
>
  {@html props.icon}
</button>
