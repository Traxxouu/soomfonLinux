<script lang="ts">
  import type { Button } from "./config";
  import { rgbToHex, hexToRgb } from "./config";

  let {
    keyIndex,
    button,
    exists,
    onchange,
    onclear,
  }: {
    keyIndex: number;
    button: Button;
    exists: boolean;
    onchange: (next: Button) => void;
    onclear: () => void;
  } = $props();

  function setLabel(value: string) {
    onchange({ ...button, label: value.trim() === "" ? undefined : value });
  }
</script>

<div class="editor">
  <h2>Touche {keyIndex + 1}</h2>

  <label class="field">
    <span>Libellé</span>
    <input
      type="text"
      maxlength="16"
      placeholder="(vide)"
      value={button.label ?? ""}
      oninput={(e) => setLabel(e.currentTarget.value)}
    />
  </label>

  <div class="colors">
    <label class="field">
      <span>Fond</span>
      <input
        type="color"
        value={rgbToHex(button.color)}
        oninput={(e) => onchange({ ...button, color: hexToRgb(e.currentTarget.value) })}
      />
    </label>
    <label class="field">
      <span>Texte</span>
      <input
        type="color"
        value={rgbToHex(button.text_color)}
        oninput={(e) =>
          onchange({ ...button, text_color: hexToRgb(e.currentTarget.value) })}
      />
    </label>
  </div>

  <button type="button" class="clear" onclick={onclear} disabled={!exists}>
    Effacer la touche
  </button>
</div>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  h2 {
    margin: 0;
    font-size: 1.1rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.85rem;
    color: var(--muted);
  }

  input[type="text"] {
    padding: 0.5rem 0.6rem;
    border-radius: 8px;
    border: 1px solid #2a2a3a;
    background: #14141b;
    color: var(--fg);
    font: inherit;
  }

  input[type="text"]:focus {
    outline: 2px solid var(--accent);
    outline-offset: 0;
    border-color: transparent;
  }

  .colors {
    display: flex;
    gap: 1rem;
  }

  input[type="color"] {
    width: 100%;
    height: 2.4rem;
    padding: 0;
    border: 1px solid #2a2a3a;
    border-radius: 8px;
    background: #14141b;
    cursor: pointer;
  }

  .clear {
    align-self: flex-start;
    padding: 0.45rem 0.8rem;
    border-radius: 8px;
    border: 1px solid #4a2a32;
    background: transparent;
    color: #ff8a8a;
    font: inherit;
    cursor: pointer;
  }

  .clear:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .clear:not(:disabled):hover {
    background: rgba(255, 107, 107, 0.12);
  }
</style>
