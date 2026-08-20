<script lang="ts">
  import type { ChartConfiguration } from "chart.js";

  type Config = ChartConfiguration<any>;

  import Chart from "../../lib/components/Chart.svelte";
  import StatTile from "../../lib/components/StatTile.svelte";
  import { base, lineaMeta, serie, tinta } from "../../lib/charts";
  import { num } from "../../lib/format";
  import type { Range, WorkoutStats } from "../../lib/types";
  import * as api from "../../lib/api";

  let { range }: { range: Range } = $props();

  // $state.raw y no $state: Chart.js redefine propiedades sobre los objetos
  // que recibe, y un proxy reactivo lo rechaza. Aquí solo se lee y se
  // reemplaza entero, así que no hace falta proxy.
  let d = $state.raw<WorkoutStats | null>(null);
  let error = $state("");

  $effect(() => {
    const r = range;
    let vivo = true;
    api
      .getWorkoutStats(r)
      .then((v) => vivo && (d = v))
      .catch((e) => vivo && (error = e instanceof Error ? e.message : String(e)));
    return () => {
      vivo = false;
    };
  });

  // Dos series apiladas, separadas por 2px del color del panel.
  const semanal = $derived.by((): Config | null => {
    if (!d || d.weekly.length === 0) return null;
    const comun = {
      borderColor: tinta.panel(),
      borderWidth: 2,
      borderRadius: 3,
      borderSkipped: false,
    };
    return {
      type: "bar",
      data: {
        labels: d.weekly.map((w) => w.label),
        datasets: [
          {
            label: "Bajo techo",
            data: d.weekly.map((w) => w.indoorMin),
            backgroundColor: serie.uno(),
            ...comun,
          },
          {
            label: "Al aire libre",
            data: d.weekly.map((w) => w.outdoorMin),
            backgroundColor: serie.dos(),
            ...comun,
          },
        ],
      },
      options: {
        ...base(),
        plugins: { ...base().plugins, legend: { ...base().plugins?.legend, display: true } },
        scales: {
          x: { ...base().scales?.x, stacked: true },
          y: {
            ...base().scales?.y,
            stacked: true,
            ticks: { ...base().scales?.y?.ticks, callback: (v: number | string) => `${v}m` },
          },
        },
      },
    };
  });

  const calorias = $derived.by((): Config | null => {
    if (!d || d.calories.length === 0) return null;
    return {
      type: "bar",
      data: {
        labels: d.calories.map((c) => c.label),
        datasets: [
          {
            label: "Ingeridas",
            data: d.calories.map((c) => c.intake),
            backgroundColor: serie.uno(),
            borderRadius: 3,
            borderSkipped: false,
          },
          {
            label: "Quemadas",
            data: d.calories.map((c) => c.burned),
            backgroundColor: serie.dos(),
            borderRadius: 3,
            borderSkipped: false,
          },
        ],
      },
      options: {
        ...base(),
        plugins: { ...base().plugins, legend: { ...base().plugins?.legend, display: true } },
      },
    };
  });

  // Barras horizontales con rampa secuencial: la mayor, la más oscura.
  const tipos = $derived.by((): Config | null => {
    if (!d || d.byKind.length === 0) return null;
    const max = Math.max(...d.byKind.map((k) => k.minutes), 1);
    const rampa = ["#234269", "#2e5786", "#3a6ea5", "#5f86b2", "#84a4c9"];
    return {
      type: "bar",
      data: {
        labels: d.byKind.map((k) => k.label),
        datasets: [
          {
            label: "Minutos",
            data: d.byKind.map((k) => k.minutes),
            backgroundColor: d.byKind.map(
              (k) => rampa[Math.min(rampa.length - 1, Math.floor((1 - k.minutes / max) * rampa.length))]
            ),
            borderRadius: 3,
            borderSkipped: false,
          },
        ],
      },
      options: {
        ...base(),
        indexAxis: "y",
        scales: {
          x: {
            ...base().scales?.y,
            ticks: { ...base().scales?.y?.ticks, callback: (v: number | string) => `${v}m` },
          },
          y: { ...base().scales?.x },
        },
      },
    };
  });
</script>

{#if error}
  <p class="error" role="alert">{error}</p>
{:else if !d}
  <p class="muted">Cargando…</p>
{:else if d.sessions === 0}
  <div class="card vacio">
    <p class="muted">Sin sesiones registradas en este rango.</p>
  </div>
{:else}
  <div class="tiles">
    <StatTile label="Sesiones" value={num(d.sessions)} />
    <StatTile label="Minutos totales" value={num(d.totalMin)} />
    <StatTile
      label="Promedio semanal"
      value="{num(d.weeklyAvgMin)} min"
      detail="meta {num(d.weeklyGoalMin)} min"
      severity={d.weeklyAvgMin >= d.weeklyGoalMin ? "good" : "warning"}
    />
    <StatTile
      label="Días sin entrenar"
      value={num(d.daysWithout)}
      severity={d.daysWithout > 0 ? "warning" : "good"}
    />
  </div>

  {#if semanal}
    <div class="card">
      <span class="label">Minutos por semana</span>
      <Chart
        config={semanal}
        height={320}
        plugins={[lineaMeta(d.weeklyGoalMin, `Meta ${d.weeklyGoalMin} min`)]}
        label="Minutos de ejercicio por semana"
      />
    </div>
  {/if}

  <div class="mitades">
    <div class="card">
      <span class="label">Calorías: ingeridas vs. quemadas</span>
      {#if calorias}
        <Chart config={calorias} height={220} label="Calorías ingeridas contra quemadas" />
        <p class="hint">Solo se grafican los días con ambos datos registrados.</p>
      {:else}
        <p class="hint">
          Ningún día del rango tiene ambos datos. Las calorías son opcionales: esta gráfica
          aparece sola si algún día registras las dos cosas.
        </p>
      {/if}
    </div>

    {#if tipos}
      <div class="card">
        <span class="label">Tipos de entrenamiento</span>
        <Chart config={tipos} height={220} label="Minutos por tipo de entrenamiento" />
      </div>
    {/if}
  </div>
{/if}

<style>
  .tiles {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 16px;
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .mitades {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    align-items: start;
  }

  .vacio {
    padding: 32px 20px;
    text-align: center;
  }

  .vacio p,
  .hint {
    margin: 0;
  }

  .error {
    color: var(--critical);
  }
</style>
