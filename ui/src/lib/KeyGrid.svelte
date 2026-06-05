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
    gap: 0.7rem;
  }

  .key {
    aspect-ratio: 1;
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--surface);
    color: var(--fg);
    font: inherit;
    font-weight: 600;
    font-size: 0.95rem;
    cursor: pointer;
    display: grid;
    place-items: center;
    padding: 0.35rem;
    /* Inner hairline keeps bright fills legible against the dark frame. */
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.05);
    transition:
      box-shadow 0.12s ease,
      transform 0.08s ease;
  }

  .key:hover {
    transform: translateY(-2px);
  }

  .key:active {
    transform: translateY(0);
  }

  .key.selected {
    box-shadow:
      inset 0 0 0 1px rgba(255, 255, 255, 0.05),
      0 0 0 2px var(--accent),
      0 0 0 6px var(--accent-soft);
  }

  .key.empty {
    border-style: dashed;
    background: transparent;
    box-shadow: none;
    color: var(--muted);
    font-weight: 400;
  }

  .key.empty.selected {
    box-shadow:
      0 0 0 2px var(--accent),
      0 0 0 6px var(--accent-soft);
  }

  .label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.35);
  }

  .index {
    opacity: 0.45;
    font-size: 1.05rem;
  }
</style>
