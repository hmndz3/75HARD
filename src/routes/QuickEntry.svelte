<script lang="ts">
  import EntryForms from "../lib/components/EntryForms.svelte";
  import type { EntryTab } from "../lib/components/EntryForms.svelte";
  import * as api from "../lib/api";

  let tab = $state<EntryTab>("comida");
  let toast = $state("");

  function close() {
    api.closeWindow().catch(() => {});
  }

  function onsaved(mensaje: string) {
    toast = mensaje;
    // Guardó: la ventana cumple su función y se quita de en medio.
    setTimeout(close, 550);
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      close();
    }
  }
</script>

<svelte:window on:keydown={onkeydown} />

<div class="quick">
  <div class="drag" data-tauri-drag-region>
    <span data-tauri-drag-region>Captura rápida</span>
    <button class="ghost x" onclick={close} aria-label="Cerrar">✕</button>
  </div>

  <EntryForms bind:tab tabs={["comida", "ejercicio", "azucar", "agua"]} compact {onsaved} />

  {#if toast}
    <div class="toast" role="status">{toast}</div>
  {/if}
</div>

<style>
  .quick {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    position: relative;
  }

  .drag {
    height: 32px;
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-left: 12px;
    border-bottom: 1px solid var(--border);
    background: var(--surface-1);
    font-size: 12px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--ink-muted);
  }

  .x {
    width: 32px;
    height: 32px;
    padding: 0;
    border-radius: 0;
    font-size: 11px;
  }

  .toast {
    position: absolute;
    left: 12px;
    right: 12px;
    bottom: 12px;
    background: var(--ink-primary);
    color: var(--bg);
    border-radius: var(--radius-control);
    padding: 8px 12px;
    font-size: 12px;
    text-align: center;
  }
</style>
