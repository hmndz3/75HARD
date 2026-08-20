<script lang="ts">
  import { save } from "@tauri-apps/plugin-dialog";

  import SleepStats from "./stats/SleepStats.svelte";
  import WorkoutStats from "./stats/WorkoutStats.svelte";
  import GlucoseStats from "./stats/GlucoseStats.svelte";
  import BodyStats from "./stats/BodyStats.svelte";
  import Correlations from "./stats/Correlations.svelte";
  import * as api from "../lib/api";
  import { todayIso } from "../lib/format";
  import type { Range } from "../lib/types";

  let { onreport }: { onreport: () => void } = $props();

  type Tab = "sueno" | "ejercicio" | "glucosa" | "cuerpo" | "correlaciones";

  const tabs: [Tab, string][] = [
    ["sueno", "Sueño"],
    ["ejercicio", "Ejercicio"],
    ["glucosa", "Glucosa"],
    ["cuerpo", "Peso y trabajo"],
    ["correlaciones", "Correlaciones"],
  ];

  const rangos: [Range, string][] = [
    ["7", "7 días"],
    ["30", "30 días"],
    ["all", "Todo"],
  ];

  let tab = $state<Tab>("sueno");
  let range = $state<Range>("30");
  let aviso = $state("");
  let error = $state("");

  async function exportar(formato: "csv" | "json") {
    error = "";
    try {
      const destino = await save({
        defaultPath: `75hard-${todayIso()}.${formato}`,
        filters: [{ name: formato.toUpperCase(), extensions: [formato] }],
      });
      if (!destino) return;
      const ruta = await api.exportData(formato, destino);
      aviso = `Guardado en ${ruta}`;
      setTimeout(() => (aviso = ""), 6000);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<div class="page">
  <div class="cabecera">
    <div class="tabbar" role="tablist">
      {#each tabs as [id, label] (id)}
        <button class="tab" class:on={tab === id} role="tab" aria-selected={tab === id}
          onclick={() => (tab = id)}>{label}</button>
      {/each}
    </div>

    <div class="derecha">
      {#if tab !== "correlaciones"}
        <div class="segmented" role="group" aria-label="Rango de tiempo">
          {#each rangos as [id, label] (id)}
            <button class="seg" class:on={range === id} onclick={() => (range = id)}>{label}</button>
          {/each}
        </div>
      {/if}
      <button onclick={() => exportar("csv")}>Exportar CSV</button>
      <button onclick={() => exportar("json")}>JSON</button>
    </div>
  </div>

  {#if error}<p class="error" role="alert">{error}</p>{/if}
  {#if aviso}<p class="ok" role="status">{aviso}</p>{/if}

  <div class="contenido">
    {#if tab === "sueno"}
      <SleepStats {range} />
    {:else if tab === "ejercicio"}
      <WorkoutStats {range} />
    {:else if tab === "glucosa"}
      <GlucoseStats {range} {onreport} />
    {:else if tab === "cuerpo"}
      <BodyStats {range} />
    {:else}
      <Correlations />
    {/if}
  </div>
</div>

<style>
  .page {
    max-width: 1200px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .cabecera {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }

  .tabbar {
    display: flex;
    gap: 4px;
  }

  .tab {
    height: 34px;
    border: none;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--ink-secondary);
  }

  .tab.on {
    background: var(--surface-sunken);
    color: var(--ink-primary);
    font-weight: 500;
  }

  .derecha {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .segmented {
    display: flex;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-control);
    overflow: hidden;
  }

  .seg {
    height: 34px;
    border: none;
    border-radius: 0;
    border-right: 1px solid var(--border-strong);
    background: var(--surface-2);
    color: var(--ink-secondary);
    font-size: 13px;
  }

  .seg:last-child {
    border-right: none;
  }

  .seg.on {
    background: var(--surface-sunken);
    color: var(--ink-primary);
    font-weight: 500;
  }

  .contenido {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }











  .error {
    margin: 0;
    color: var(--critical);
  }

  .ok {
    margin: 0;
    color: var(--good);
  }

</style>
