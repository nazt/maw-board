<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { XIcon, PlusIcon, ClipboardIcon } from "svelte-feather-icons";
  import { settings, updateSettings } from "../settings";

  export let open = false;

  const dispatch = createEventDispatcher<{ paste: string; close: void }>();

  let draft = "";

  function add() {
    const value = draft.trim();
    if (!value) return;
    updateSettings({ snippets: [...$settings.snippets, value] });
    draft = "";
  }

  function remove(index: number) {
    updateSettings({
      snippets: $settings.snippets.filter((_, i) => i !== index),
    });
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      add();
    }
  }
</script>

{#if open}
  <div class="panel snippet-bar">
    <div class="header">
      <div class="title">
        <ClipboardIcon size="15" strokeWidth={2} />
        <span>Snippets</span>
      </div>
      <button class="x" on:click={() => dispatch("close")} title="Close">
        <XIcon size="16" strokeWidth={2} />
      </button>
    </div>

    <div class="chips">
      {#each $settings.snippets as snippet, i (i + snippet)}
        <div class="chip">
          <button
            class="chip-text"
            title="Paste into the focused terminal"
            on:click={() => dispatch("paste", snippet)}
          >
            {snippet}
          </button>
          <button class="chip-x" on:click={() => remove(i)} title="Delete">
            <XIcon size="13" strokeWidth={2} />
          </button>
        </div>
      {/each}
      {#if $settings.snippets.length === 0}
        <span class="empty">No snippets yet — add one below.</span>
      {/if}
    </div>

    <div class="add">
      <input
        bind:value={draft}
        on:keydown={onKeydown}
        placeholder="New command…"
        spellcheck="false"
      />
      <button class="add-btn" on:click={add} title="Add snippet">
        <PlusIcon size="16" strokeWidth={2} />
      </button>
    </div>
  </div>
{/if}

<style lang="postcss">
  .snippet-bar {
    @apply absolute left-1/2 top-16 z-30 -translate-x-1/2 w-[min(90vw,32rem)] p-3;
    @apply flex flex-col gap-2;
  }

  .header {
    @apply flex items-center justify-between;
  }

  .title {
    @apply flex items-center gap-1.5 text-indigo-400 font-semibold text-sm;
  }

  .x {
    @apply rounded-md p-1 text-zinc-400 hover:text-white hover:bg-zinc-700/80;
  }

  .chips {
    @apply flex flex-wrap gap-1.5;
  }

  .chip {
    @apply flex items-center rounded-md bg-zinc-800 ring-1 ring-zinc-700 overflow-hidden;
  }

  .chip-text {
    @apply px-2 py-1 font-mono text-xs text-zinc-200 hover:bg-indigo-600 hover:text-white;
  }

  .chip-x {
    @apply px-1 text-zinc-500 hover:text-red-400 hover:bg-zinc-700;
  }

  .empty {
    @apply text-xs text-zinc-500 italic;
  }

  .add {
    @apply flex items-center gap-1.5;
  }

  .add input {
    @apply flex-1 rounded-md bg-zinc-900 ring-1 ring-zinc-700 px-2 py-1 text-sm;
    @apply text-zinc-100 font-mono placeholder:text-zinc-600 focus:ring-indigo-500 outline-none;
  }

  .add-btn {
    @apply rounded-md p-1.5 bg-indigo-600 text-white hover:bg-indigo-500;
  }
</style>
