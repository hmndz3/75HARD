<script lang="ts">
  import * as api from "../lib/api";
  import { kg, litres, minutesShort, plural, statusColor, statusLabel } from "../lib/format";
  import type { History } from "../lib/types";

  let { onopenday }: { onopenday: (date: string) => void } = $props();

  let data = $state<History | null>(null);
  let error = $state("");
  let filter = $state<"todos" | "complete" | "failed">("todos");
  let search = $state("");

  async function load() {
    try {
      data = await api.getHistory(200);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  load();

  const rows = $derived(
    (data?.rows ?? []).filter((r) => {
      if (filter !== "todos" && r.status !== filter) return false;
      if (!search.trim()) return true;
      const q = search.toLowerCase();
      return (
        r.date.includes(q) ||
        r.weekdayLabel.toLowerCase().includes(q) ||
        String(r.dayNumber ?? "").includes(q)
      );
    })
  );
</script>

<div class="page">
  <div class="card flush">
    <div class="card-head">
      <h1 class="section-title">Historial</h1>
      {#if data}
        <span class="figures">
          <span class="num">{plural(data.summary.complete, "completo", "completos")}</span>
          <span class="sep">·</span>
          <span class="num">{plural(data.summary.partial, "parcial", "parciales")}</span>
          <span class="sep">·</span>
          <span class="num">{plural(data.summary.failed, "fallido", "fallidos")}</span>
          <span class="sep">·</span>
          <span class="num">{plural(data.summary.skipped, "pausado", "pausados")}</span>
        </span>
      {/if}
    </div>

    <div class="filters">
      <div class="segmented" role="group" aria-label="Filtrar por estado">
        {#each [["todos", "Todos"], ["complete", "Completos"], ["failed", "Fallidos"]] as [value, label] (value)}
          <button
            class="seg"
            class:on={filter === value}
            onclick={() => (filter = value as typeof filter)}
          >
            {label}
          </button>
        {/each}
      </div>
      <input class="search" type="search" placeholder="Filtrar registros…" bind:value={search} />
    </div>

    {#if error}
      <p class="error" role="alert">{error}</p>
    {:else if !data}
      <p class="empty muted">Cargando…</p>
    {:else if rows.length === 0}
      <p class="empty muted">
        Todavía no hay días registrados con ese filtro. En cuanto guardes algo aparecerá aquí.
      </p>
    {:else}
      <div class="tablewrap">
        <table>
          <thead>
            <tr>
              <th class="right">Día</th>
              <th>Fecha</th>
              <th>Estado</th>
              <th class="right">Sueño</th>
              <th class="right">Ejercicio</th>
              <th class="right">Comidas</th>
              <th class="right">Agua</th>
              <th class="right">Azúcar</th>
              <th class="right">Peso</th>
            </tr>
          </thead>
          <tbody>
            {#each rows as r (r.date)}
              <tr
                tabindex="0"
                role="button"
                onclick={() => onopenday(r.date)}
                onkeydown={(e) => e.key === "Enter" && onopenday(r.date)}
              >
                <td class="right num">{r.dayNumber ?? "—"}</td>
                <td>{r.weekdayLabel}</td>
                <td>
                  <span class="dot" style:background={statusColor[r.status]}></span>
                  {statusLabel[r.status]}
                </td>
                <td class="right num">{minutesShort(r.sleepMinutes)}</td>
                <td class="right num">{r.workoutMin > 0 ? `${r.workoutMin} min` : "—"}</td>
                <td class="right num">{r.mealsCount || "—"}</td>
                <td class="right num">{r.waterMl > 0 ? `${litres(r.waterMl)} L` : "—"}</td>
                <td class="right num">{r.glucoseCount || "—"}</td>
                <td class="right num">{kg(r.weightKg)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
      <p class="foot hint">
        Haz clic en cualquier fila para abrir el día y editarlo. Todo es editable hacia atrás.
      </p>
    {/if}
  </div>
</div>

<style>
  .page {
    max-width: 1200px;
  }

  .figures {
    font-size: 13px;
    color: var(--ink-secondary);
  }

  .figures .num {
    color: var(--ink-primary);
    font-weight: 600;
  }

  .sep {
    color: var(--border-strong);
    margin: 0 6px;
  }

  .filters {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 20px;
    border-bottom: 1px solid var(--border);
  }

  .segmented {
    display: flex;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-control);
    overflow: hidden;
  }

  .seg {
    height: 32px;
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

  .search {
    width: 260px;
    height: 32px;
  }

  .tablewrap {
    overflow-x: auto;
  }

  tbody tr {
    cursor: pointer;
  }

  .dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 2px;
    margin-right: 8px;
    vertical-align: middle;
  }

  .empty,
  .error {
    padding: 32px 20px;
    margin: 0;
    text-align: center;
  }

  .error {
    color: var(--critical);
  }

  .foot {
    padding: 12px 20px;
    margin: 0;
  }
</style>
