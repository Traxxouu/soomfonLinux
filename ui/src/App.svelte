<script lang="ts">
  import { onMount } from "svelte";
  import KeyGrid from "./lib/KeyGrid.svelte";
  import ButtonEditor from "./lib/ButtonEditor.svelte";
  import { getConfig, saveConfig, getStatus, type Status } from "./lib/api";
  import {
    activePage,
    activeProfileIndex,
    defaultButton,
    GRID_COLS,
    LCD_KEYS,
    type Button,
    type Config,
  } from "./lib/config";

  let config = $state<Config | null>(null);
  let status = $state<Status | null>(null);
  let loadError = $state<string | null>(null);

  let selected = $state<number | null>(null);
  let dirty = $state(false);
  let saving = $state(false);
  let saved = $state(false);
  let saveError = $state<string | null>(null);

  const page = $derived(config ? activePage(config) : null);
  const profile = $derived(
    config ? config.profiles[activeProfileIndex(config)] : null,
  );
  const storedButton = $derived(
    page && selected !== null ? (page.buttons[String(selected)] ?? null) : null,
  );

  onMount(async () => {
    try {
      [config, status] = await Promise.all([getConfig(), getStatus()]);
    } catch (e) {
      loadError = String(e);
    }
  });

  function selectKey(key: number) {
    selected = key;
    saved = false;
  }

  function updateButton(next: Button) {
    if (!page || selected === null) return;
    page.buttons[String(selected)] = next;
    dirty = true;
    saved = false;
  }

  function clearKey() {
    if (!page || selected === null) return;
    delete page.buttons[String(selected)];
    dirty = true;
    saved = false;
  }

  async function save() {
    if (!config) return;
    saving = true;
    saveError = null;
    try {
      await saveConfig(config);
      dirty = false;
      saved = true;
    } catch (e) {
      saveError = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<main>
  <header>
    <h1>soomfonLinux</h1>
    <p class="tagline">Pilotez votre stream deck Soomfon sous Linux.</p>
  </header>

  {#if loadError}
    <section class="card">
      <p class="error">Erreur backend : {loadError}</p>
    </section>
  {:else if config && page && profile}
    <p class="device muted">
      {#if status && status.devices.length > 0}
        Appareil : {status.devices[0].model}
      {:else}
        Aucun appareil détecté — l'édition reste possible.
      {/if}
    </p>

    <section class="editor-layout">
      <div class="card">
        <h2 class="context">{profile.name} · {page.name}</h2>
        <KeyGrid
          {page}
          keyCount={LCD_KEYS}
          cols={GRID_COLS}
          {selected}
          onselect={selectKey}
        />
      </div>

      <div class="card">
        {#if selected === null}
          <p class="muted">Sélectionnez une touche pour la configurer.</p>
        {:else}
          {#key selected}
            <ButtonEditor
              keyIndex={selected}
              button={storedButton ?? defaultButton()}
              exists={storedButton !== null}
              onchange={updateButton}
              onclear={clearKey}
            />
          {/key}
        {/if}
      </div>
    </section>

    <footer class="actions">
      <button type="button" class="save" onclick={save} disabled={!dirty || saving}>
        {saving ? "Enregistrement…" : "Enregistrer"}
      </button>
      {#if saveError}
        <span class="error">Échec : {saveError}</span>
      {:else if saved}
        <span class="ok">Enregistré ✓</span>
      {:else if dirty}
        <span class="muted">Modifications non enregistrées</span>
      {/if}
    </footer>
  {:else}
    <section class="card">
      <p class="muted">Chargement de la configuration…</p>
    </section>
  {/if}
</main>

<style>
  .device {
    margin-top: 1.5rem;
  }

  .editor-layout {
    margin-top: 0.75rem;
    display: grid;
    grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr);
    gap: 1.5rem;
    align-items: start;
  }

  .context {
    margin: 0 0 1rem;
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--muted);
  }

  .actions {
    margin-top: 1.5rem;
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .save {
    padding: 0.55rem 1.1rem;
    border-radius: 8px;
    border: none;
    background: var(--accent);
    color: white;
    font: inherit;
    font-weight: 600;
    cursor: pointer;
  }

  .save:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .ok {
    color: #7ee0a0;
  }

  @media (max-width: 620px) {
    .editor-layout {
      grid-template-columns: 1fr;
    }
  }
</style>
