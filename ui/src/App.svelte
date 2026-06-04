<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  type DeviceSummary = { model: string; keys: number };
  type Status = { version: string; devices: DeviceSummary[] };

  let status = $state<Status | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      status = await invoke<Status>("get_status");
    } catch (e) {
      error = String(e);
    }
  });
</script>

<main>
  <header>
    <h1>soomfonLinux</h1>
    <p class="tagline">Pilotez votre stream deck Soomfon sous Linux.</p>
  </header>

  <section class="card">
    {#if error}
      <p class="error">Erreur backend : {error}</p>
    {:else if status}
      <p>Cœur applicatif <code>v{status.version}</code></p>
      {#if status.devices.length === 0}
        <p class="muted">
          Aucun appareil détecté — la couche matérielle (mirajazz) sera branchée
          dans une prochaine étape.
        </p>
      {:else}
        <ul>
          {#each status.devices as device}
            <li>{device.model} — {device.keys} touches</li>
          {/each}
        </ul>
      {/if}
    {:else}
      <p class="muted">Connexion au cœur applicatif…</p>
    {/if}
  </section>
</main>
