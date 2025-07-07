<script lang="ts">
  // During Linux development, "virtual:uno.css" may prevent initial mount
  // If it does: comment, save, uncomment, reload
  // This fix only works when styles are in a separate layout
  import "virtual:uno.css";
  import "@unocss/reset/tailwind.css";
  import "./styles.css";
</script>

<slot />
