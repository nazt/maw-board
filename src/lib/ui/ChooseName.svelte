<script lang="ts">
  import { browser } from "$app/environment";

  import OverlayMenu from "./OverlayMenu.svelte";
  import { settings, updateSettings } from "$lib/settings";

  let value = "";

  function handleSubmit() {
    updateSettings({ name: value });
  }
</script>

<OverlayMenu
  title="Welcome!"
  description="Before you join — what should we call you?"
  maxWidth={640}
  open={browser && !$settings.name}
>
  <form class="flex gap-2" on:submit|preventDefault={handleSubmit}>
    <input
      class="flex-1 w-full px-3 py-2 rounded outline-none text-zinc-300 bg-zinc-800"
      placeholder="Your name"
      required
      minlength="2"
      maxlength="50"
      bind:value
    />
    <button
      class="flex-shrink-0 px-4 py-2 bg-indigo-600 hover:bg-indigo-500 active:ring-4 active:ring-indigo-400/40 rounded-lg font-medium transition-colors"
      >Join</button
    >
  </form>
</OverlayMenu>
