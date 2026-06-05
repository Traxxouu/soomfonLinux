<script lang="ts">
  import { untrack } from "svelte";
  import type { Action, Button } from "./config";
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

  // Local text for the arguments field, kept separate from the parsed `args`
  // array so spaces survive while typing. The editor is re-keyed per key, so
  // this initialises correctly for each selection.
  let argsText = $state(
    untrack(() =>
      button.action.type === "run_command" ? button.action.args.join(" ") : "",
    ),
  );

  // Same idea for the hotkey combo: the raw "ctrl+shift+m" text is kept locally
  // so a trailing "+" survives while the next key is being typed.
  let hotkeyText = $state(
    untrack(() =>
      button.action.type === "hotkey" ? button.action.keys.join("+") : "",
    ),
  );

  function setLabel(value: string) {
    onchange({ ...button, label: value.trim() === "" ? undefined : value });
  }

  function setAction(action: Action) {
    onchange({ ...button, action });
  }

  function setActionType(type: Action["type"]) {
    if (type === "run_command") {
      argsText = "";
      setAction({ type: "run_command", program: "", args: [] });
    } else if (type === "hotkey") {
      hotkeyText = "";
      setAction({ type: "hotkey", keys: [] });
    } else {
      setAction({ type: "none" });
    }
  }

  function setProgram(program: string) {
    if (button.action.type !== "run_command") return;
    setAction({ ...button.action, program });
  }

  function setArgs(value: string) {
    argsText = value;
    if (button.action.type !== "run_command") return;
    const args = value.split(/\s+/).filter((s) => s.length > 0);
    setAction({ ...button.action, args });
  }

  function setHotkey(value: string) {
    hotkeyText = value;
    if (button.action.type !== "hotkey") return;
    const keys = value
      .split("+")
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
    setAction({ ...button.action, keys });
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

  <div class="action">
    <label class="field">
      <span>Action</span>
      <select
        value={button.action.type}
        onchange={(e) =>
          setActionType(e.currentTarget.value as Action["type"])}
      >
        <option value="none">Aucune</option>
        <option value="run_command">Lancer une commande</option>
        <option value="hotkey">Raccourci clavier</option>
      </select>
    </label>

    {#if button.action.type === "run_command"}
      <label class="field">
        <span>Programme</span>
        <input
          type="text"
          placeholder="ex. firefox"
          value={button.action.program}
          oninput={(e) => setProgram(e.currentTarget.value)}
        />
      </label>
      <label class="field">
        <span>Arguments (séparés par des espaces)</span>
        <input
          type="text"
          placeholder="ex. --new-window https://twitch.tv"
          value={argsText}
          oninput={(e) => setArgs(e.currentTarget.value)}
        />
      </label>
    {:else if button.action.type === "hotkey"}
      <label class="field">
        <span>Combinaison (touches séparées par «&nbsp;+&nbsp;»)</span>
        <input
          type="text"
          placeholder="ex. ctrl+shift+m"
          value={hotkeyText}
          oninput={(e) => setHotkey(e.currentTarget.value)}
        />
      </label>
      <p class="hint">
        Modificateurs : ctrl, shift, alt, super. Plus lettres, chiffres,
        f1–f12, enter, space, tab, esc, flèches…
      </p>
    {/if}
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
    gap: 0.4rem;
  }

  .field > span {
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .hint {
    margin: 0;
    font-size: 0.78rem;
    line-height: 1.45;
    color: var(--muted);
  }

  input[type="text"],
  select {
    padding: 0.55rem 0.65rem;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--fg);
    font: inherit;
  }

  input[type="text"]::placeholder {
    color: color-mix(in srgb, var(--muted) 65%, transparent);
  }

  select {
    cursor: pointer;
  }

  input[type="text"]:focus,
  select:focus {
    outline: 2px solid var(--accent);
    outline-offset: 0;
    border-color: transparent;
  }

  .action {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding-top: 1.25rem;
    border-top: 1px solid var(--border);
  }

  .colors {
    display: flex;
    gap: 1rem;
  }

  input[type="color"] {
    width: 100%;
    height: 2.4rem;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--surface);
    cursor: pointer;
  }

  .clear {
    align-self: flex-start;
    padding: 0.45rem 0.8rem;
    border-radius: var(--radius-sm);
    border: 1px solid color-mix(in srgb, var(--danger) 40%, var(--border));
    background: transparent;
    color: var(--danger);
    font: inherit;
    cursor: pointer;
    transition: background 0.12s ease;
  }

  .clear:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .clear:not(:disabled):hover {
    background: color-mix(in srgb, var(--danger) 14%, transparent);
  }
</style>
