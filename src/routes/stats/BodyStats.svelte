<script lang="ts">
  import type { ChartConfiguration } from "chart.js";

  type Config = ChartConfiguration<any>;

  import Chart from "../../lib/components/Chart.svelte";
  import StatTile from "../../lib/components/StatTile.svelte";
  import { base, serie, tinta } from "../../lib/charts";
  import type { BodyStats, Range } from "../../lib/types";
  import * as api from "../../lib/api";

  let { range }: { range: Range } = $props();

  // $state.raw y no $state: Chart.js redefine propiedades sobre los objetos
  // que recibe, y un proxy reactivo lo rechaza. Aquí solo se lee y se
  // reemplaza entero, así que no hace falta proxy.
  let d = $state.raw<BodyStats | null>(null);
  let error = $state("");

  $effect(() => {
    const r = range;
    let vivo = true;
    api
      .getBodyStats(r)
      .then((v) => vivo && (d = v))
      .catch((e) => vivo && (error = e instanceof Error ? e.message : String(e)));
    return () => {
      vivo = false;
    };
  });

  // El dato crudo va como puntitos grises sin línea; la línea azul es el
  // promedio móvil. Es la decisión 2 del §6 y no se negocia: el peso diario
  // oscila ±1.5 kg por agua y sal.
  const peso = $derived.by((): Config | null => {
    if (!d || d.points.every((p) => p.kg === null)) return null;
    return {
      type: "line",
      data: {
        labels: d.points.map((p) => p.label),
        datasets: [
          {
            label: "Peso diario",
            data: d.points.map((p) => p.kg),
            showLine: false,
            pointRadius: 3,
            pointBackgroundColor: serie.apagada(),
            pointBorderWidth: 0,
          },
          {
            label: "Promedio 7 días",
            data: d.movingAvg,
            borderColor: serie.uno(),
            borderWidth: 2,
            pointRadius: 0,
            tension: 0.35,
            spanGaps: true,
          },
        ],
      },
      options: {
        ...base(),
        plugins: { ...base().plugins, legend: { ...base().plugins?.legend, display: true } },
        scales: {
          ...base().scales,
          y: {
            ...base().scales?.y,
            beginAtZero: false,
            ticks: { ...base().scales?.y?.ticks, callback: (v: number | string) => `${v} kg` },
          },
        },
      },
    };
  });

  const trabajo = $derived.by((): Config | null => {
    if (!d || d.categories.length === 0) return null;
    const colores = [serie.uno(), serie.dos(), serie.tres()];
    return {
      type: "bar",
      data: {
        labels: d.workDaily.map((w) => w.label),
        datasets: d.categories.map((cat, i) => ({
          label: cat,
          data: d?.workDaily.map((w) => (w.values[i] ?? 0) / 60) ?? [],
          backgroundColor: colores[i % colores.length],
          borderColor: tinta.panel(),
          borderWidth: 2,
          borderRadius: 3,
          borderSkipped: false,
        })),
      },
      options: {
        ...base(),
        plugins: { ...base().plugins, legend: { ...base().plugins?.legend, display: true } },
        scales: {
          x: { ...base().scales?.x, stacked: true },
          y: {
            ...base().scales?.y,
            stacked: true,
            ticks: { ...base().scales?.y?.ticks, callback: (v: number | string) => `${v}h` },
          },
        },
      },
    };
  });

  const signo = (v: number) => (v > 0 ? "+" : "−");
</script>

{#if error}
  <p class="error" role="alert">{error}</p>
{:else if !d}
  <p class="muted">Cargando…</p>
{:else}
  <section>
    <div class="tiles tres">
      <StatTile
        label="Peso actual"
        value={d.currentKg !== null ? `${d.currentKg.toFixed(1)} kg` : "—"}
      />
      <StatTile
        label="Cambio en el rango"
        value={d.deltaKg !== null
          ? `${signo(d.deltaKg)}${Math.abs(d.deltaKg).toFixed(1)} kg`
          : "—"}
        severity={d.deltaKg === null ? "neutral" : d.deltaKg < 0 ? "good" : "warning"}
      />
      <StatTile
        label="Promedio semanal"
        value={d.weeklyDeltaKg !== null
          ? `${signo(d.weeklyDeltaKg)}${Math.abs(d.weeklyDeltaKg).toFixed(2)} kg`
          : "—"}
      />
    </div>

    <div class="card">
      <span class="label">Peso</span>
      {#if peso}
        <Chart config={peso} height={280} label="Peso diario y promedio de 7 días" />
        <p class="hint">
          El peso diario oscila por agua y sal. La línea azul es lo que realmente importa.
        </p>
      {:else}
        <p class="hint">Sin pesajes en este rango. Se registra en el check-in matutino.</p>
      {/if}
    </div>
  </section>

  <section>
    <div class="tiles dos">
      <StatTile label="Horas esta semana" value={`${d.workHoursWeek} h`} />
      <StatTile label="Promedio diario" value={`${d.workAvgDailyH} h`} />
    </div>

    <div class="card">
      <span class="label">Trabajo por día</span>
      {#if trabajo}
        <Chart config={trabajo} height={280} label="Horas de trabajo por día y categoría" />
      {:else}
        <p class="hint">Sin sesiones de trabajo en este rango.</p>
      {/if}
    </div>
  </section>
{/if}

<style>
  section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .tiles {
    display: grid;
    gap: 16px;
  }

  .tiles.tres {
    grid-template-columns: repeat(3, 1fr);
  }

  .tiles.dos {
    grid-template-columns: repeat(2, 1fr);
    max-width: 640px;
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .hint {
    margin: 0;
  }

  .error {
    color: var(--critical);
  }
</style>
