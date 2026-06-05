<script lang="ts">
  import type { Page } from "./config";
  import { rgbToHex } from "./config";

  let {
    page,
    keyCount,
    cols,
    selected,
    onselect,
  }: {
    page: Page;
    keyCount: number;
    cols: number;
    selected: number | null;
    onselect: (key: number) => void;
  } = $props();

  const keys = $derived(Array.from({ length: keyCount }, (_, i) => i));
</script>

<div class="grid" style="--cols: {cols}">
  {#each keys as key (key)}
    {@const button = page.buttons[String(key)]}
    <button
      type="button"
      class="key"
      class:selected={selected === key}
      class:empty={!button}
      style={button
        ? `background:${rgbToHex(button.color)}; color:${rgbToHex(button.text_color)}`
        : ""}
      onclick={() => onselect(key)}
      aria-label={`Touche ${key + 1}`}
      aria-pressed={selected === key}
    >
      {#if button?.label}
        <span class="label">{button.label}</span>
      {:else}
        <span class="index">{key + 1}</span>
      {/if}
    </button>
  {/each}
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(var(--cols), 1fr);
    gap: 0.6rem;
  }

  .key {
    aspect-ratio: 1;
    border: 1px solid #2a2a3a;
    border-radius: 10px;
    background: #1a1a24;
    color: var(--fg);
    font: inherit;
    font-weight: 600;
    cursor: pointer;
    display: grid;
    place-items: center;
    padding: 0.25rem;
    transition:
      outline-color 0.1s ease,
      transform 0.05s ease;
    outline: 2px solid transparent;
    outline-offset: 2px;
  }

  .key:hover {
    transform: translateY(-1px);
  }

  .key.selected {
    outline-color: var(--accent);
  }

  .key.empty {
    border-style: dashed;
    color: var(--muted);
    font-weight: 400;
  }

  .label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }

  .index {
    opacity: 0.5;
  }
</style>
