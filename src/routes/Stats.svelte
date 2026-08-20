<script lang="ts">
  import { save } from "@tauri-apps/plugin-dialog";

  import SleepStats from "./stats/SleepStats.svelte";
  import WorkoutStats from "./stats/WorkoutStats.svelte";
  import GlucoseStats from "./stats/GlucoseStats.svelte";
  import BodyStats from "./stats/BodyStats.svelte";
  import * as api from "../lib/api";
  import { todayIso } from "../lib/format";
  import type { Correlation, Range } from "../lib/types";

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

  let correlaciones = $state<Correlation[] | null>(null);
  $effect(() => {
    if (tab !== "correlaciones" || correlaciones) return;
    api
      .getCorrelations()
      .then((c) => (correlaciones = c))
      .catch((e) => (error = e instanceof Error ? e.message : String(e)));
  });

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

  /** Una r suelta no dice nada; esto la traduce a lenguaje humano. */
  function fuerza(r: number): string {
    const a = Math.abs(r);
    const grado = a >= 0.7 ? "fuerte" : a >= 0.4 ? "moderada" : "débil";
    return `Relación ${grado} y ${r >= 0 ? "positiva" : "negativa"}`;
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
      <div class="card">
        <span class="label">Correlaciones</span>
        <p class="muted">
          Relaciones observadas entre tus propios datos. <strong>Correlación no es causalidad</strong>:
          esto no dice que una cosa cause la otra, solo que se mueven juntas.
        </p>
        {#if correlaciones === null}
          <p class="muted">Calculando…</p>
        {:else if correlaciones.length === 0}
          <p class="hint">
            Todavía no hay suficientes pares de datos. Hacen falta al menos cinco días con las
            dos cosas registradas para que el número signifique algo.
          </p>
        {:else}
          <ul class="corr">
            {#each correlaciones as c (c.label)}
              <li>
                <div class="stack grow">
                  <span class="nombre">{c.label}</span>
                  <span class="hint">{c.caption}</span>
                </div>
                <div class="stack derecha-txt">
                  <span class="num r">r = {c.r.toFixed(2)}</span>
                  <span class="hint">{fuerza(c.r)} · n = {c.n}</span>
                </div>
                <div class="barra" aria-hidden="true">
                  <div
                    class="relleno"
                    style:width="{Math.abs(c.r) * 100}%"
                    style:background={c.r >= 0 ? "var(--accent)" : "#c05a30"}
                  ></div>
                </div>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
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

  .card {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .corr {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .corr li {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px 16px;
    padding: 14px 0;
    border-bottom: 1px solid var(--border);
    align-items: center;
  }

  .corr li:last-child {
    border-bottom: none;
  }

  .stack {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .derecha-txt {
    text-align: right;
  }

  .nombre {
    font-weight: 500;
  }

  .r {
    font-size: 18px;
    font-weight: 600;
  }

  .barra {
    grid-column: 1 / -1;
    height: 6px;
    background: var(--surface-sunken);
    border-radius: 3px;
    overflow: hidden;
  }

  .relleno {
    height: 100%;
  }

  .error {
    margin: 0;
    color: var(--critical);
  }

  .ok {
    margin: 0;
    color: var(--good);
  }

  .hint {
    margin: 0;
  }
</style>
