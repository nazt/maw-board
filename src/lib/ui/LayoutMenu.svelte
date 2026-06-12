<!--
  Apple-style dropdown for choosing how to tile the terminals. Offers a few
  layout shapes plus a remembered custom column count.
-->
<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { settings, updateSettings } from "$lib/settings";

  const dispatch = createEventDispatcher<{
    select: string | number;
    close: void;
  }>();

  const LAYOUTS = [
    { mode: "grid", label: "Grid", hint: "auto" },
    { mode: "2col", label: "2 columns", hint: "▦" },
    { mode: "3col", label: "3 columns", hint: "▦▦" },
    { mode: "cols", label: "Side by side", hint: "▥" },
    { mode: "rows", label: "Stacked rows", hint: "☰" },
  ];

  let customCols = $settings.tileCols || 2;

  function pick(mode: string | number) {
    dispatch("select", mode);
    dispatch("close");
  }

  function applyCustom() {
    const n = Math.max(1, Math.min(8, Math.round(customCols) || 1));
    updateSettings({ tileCols: n }); // remember as the user's preset
    pick(n);
  }
</script>

<svelte:window on:keydown={(e) => e.key === "Escape" && dispatch("close")} />

<div class="menu panel">
  <p class="title">Arrange windows</p>
  {#each LAYOUTS as l}
    <button class="row" on:click={() => pick(l.mode)}>
      <span>{l.label}</span>
      <span class="hint">{l.hint}</span>
    </button>
  {/each}

  <div class="sep" />

  <div class="custom">
    <span class="lbl">Custom</span>
    <input
      type="number"
      min="1"
      max="8"
      bind:value={customCols}
      on:keydown={(e) => e.key === "Enter" && applyCustom()}
    />
    <span class="unit">cols</span>
    <button class="apply" on:click={applyCustom}>Apply</button>
  </div>
</div>

<style lang="postcss">
  .menu {
    @apply absolute top-12 left-0 z-50 w-52 p-1.5 flex flex-col;
    /* Apple-style: soft frosted card with a gentle pop-in */
    animation: pop 120ms cubic-bezier(0.2, 0.9, 0.3, 1.2);
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(-4px) scale(0.97);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }
  .title {
    @apply px-2.5 py-1 text-[11px] uppercase tracking-wide text-zinc-500;
  }
  .row {
    @apply flex items-center justify-between px-2.5 py-1.5 rounded-lg text-sm text-zinc-200;
    @apply hover:bg-white/10 transition-colors;
  }
  .hint {
    @apply text-zinc-500 text-xs;
  }
  .sep {
    @apply my-1 border-t border-zinc-700/60;
  }
  .custom {
    @apply flex items-center gap-1.5 px-2.5 py-1.5;
  }
  .lbl {
    @apply text-sm text-zinc-300;
  }
  .custom input {
    @apply w-12 px-1.5 py-1 rounded-md bg-zinc-800 border border-zinc-700 text-sm text-zinc-100;
    @apply outline-none focus:ring-2 focus:ring-indigo-500/50 appearance-none;
  }
  .unit {
    @apply text-xs text-zinc-500;
  }
  .apply {
    @apply ml-auto px-2 py-1 rounded-md bg-indigo-600/80 text-white text-xs hover:bg-indigo-600;
  }
</style>
