<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { XIcon } from "svelte-feather-icons";

  export let open = false;

  const dispatch = createEventDispatcher<{ close: void }>();

  let url = "";
  let videoId = "";

  function parseId(input: string): string {
    const patterns = [
      /(?:youtu\.be\/)([\w-]{11})/,
      /(?:[?&]v=)([\w-]{11})/,
      /(?:\/embed\/)([\w-]{11})/,
      /(?:\/shorts\/)([\w-]{11})/,
    ];
    for (const re of patterns) {
      const m = input.match(re);
      if (m) return m[1];
    }
    return /^[\w-]{11}$/.test(input.trim()) ? input.trim() : "";
  }

  function load() {
    videoId = parseId(url);
  }
</script>

{#if open}
  <div class="yt-popup panel">
    <div class="head">
      <span class="title">▶ YouTube</span>
      <button class="close" title="Close" on:click={() => dispatch("close")}>
        <XIcon size="16" />
      </button>
    </div>
    <div class="body">
      <form class="row" on:submit|preventDefault={load}>
        <!-- svelte-ignore a11y-autofocus -->
        <input
          class="url"
          placeholder="Paste a YouTube link…"
          bind:value={url}
          autofocus
        />
        <button class="go" type="submit">Play</button>
      </form>
      {#if videoId}
        <div class="frame">
          <iframe
            title="YouTube player"
            src={`https://www.youtube.com/embed/${videoId}?autoplay=1`}
            allow="autoplay; encrypted-media; fullscreen; picture-in-picture"
            allowfullscreen
          />
        </div>
      {:else}
        <p class="hint">
          Paste a YouTube link and hit Play to listen while you work 🎵
        </p>
      {/if}
    </div>
  </div>
{/if}

<style lang="postcss">
  .yt-popup {
    @apply fixed z-40 flex flex-col overflow-hidden;
    @apply left-2 right-2 bottom-2 sm:left-auto sm:right-4 sm:w-96;
    max-height: 70vh;
  }
  .head {
    @apply flex items-center justify-between px-3 py-2 border-b border-zinc-700/60;
  }
  .title {
    @apply text-sm font-medium text-red-400;
  }
  .close {
    @apply p-1 rounded hover:bg-white/5 text-zinc-400;
  }
  .body {
    @apply p-3 flex flex-col gap-2;
  }
  .row {
    @apply flex gap-2;
  }
  .url {
    @apply flex-1 px-3 py-2 text-sm rounded-md bg-transparent border border-zinc-700;
    @apply outline-none focus:ring-2 focus:ring-indigo-500/50;
  }
  .go {
    @apply px-3 py-2 text-sm rounded-md bg-indigo-600 text-white hover:bg-indigo-500 whitespace-nowrap;
  }
  .frame {
    @apply relative w-full rounded-md overflow-hidden;
    aspect-ratio: 16 / 9;
  }
  .frame iframe {
    @apply absolute inset-0 w-full h-full border-0;
  }
  .hint {
    @apply text-xs text-zinc-400 text-center py-4;
  }
</style>
