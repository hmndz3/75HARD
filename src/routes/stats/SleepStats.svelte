<script lang="ts">
  import type { ChartConfiguration } from "chart.js";

  type Config = ChartConfiguration<any>;

  import Chart from "../../lib/components/Chart.svelte";
  import StatTile from "../../lib/components/StatTile.svelte";
  import { base, lineaMeta, serie, tinta } from "../../lib/charts";
  import { minutes } from "../../lib/format";
  import type { Range, SleepStats } from "../../lib/types";
  import * as api from "../../lib/api";

  let { range }: { range: Range } = $props();

  // $state.raw y no $state: Chart.js redefine propiedades sobre los objetos
  // que recibe, y un proxy reactivo lo rechaza. Aquí solo se lee y se
  // reemplaza entero, así que no hace falta proxy.
  let d = $state.raw<SleepStats | null>(null);
  let error = $state("");

  $effect(() => {
    const r = range;
    let vivo = true;
    api
      .getSleepStats(r)
      .then((v) => vivo && (d = v))
      .catch((e) => vivo && (error = e instanceof Error ? e.message : String(e)));
    return () => {
      vivo = false;
    };
  });

  const horas = (m: number | null | undefined) => (m == null ? null : m / 60);

  // Barras por noche. Las que quedan bajo la meta van en gris: el color se
  // reserva para lo que cumple, no para decorar.
  const barras = $derived.by((): Config | null => {
    if (!d) return null;
    const meta = d.goalMin / 60;
    return {
      type: "bar",
      data: {
        labels: d.daily.map((p) => p.label),
        datasets: [
          {
            label: "Horas dormidas",
            data: d.daily.map((p) => horas(p.minutes)),
            backgroundColor: d.daily.map((p) =>
              (p.minutes ?? 0) / 60 >= meta ? serie.uno() : serie.apagada()
            ),
            borderRadius: 4,
            borderSkipped: false,
            barPercentage: 0.7,
            categoryPercentage: 0.9,
          },
          {
            label: "Promedio 7 días",
            data: d.movingAvg.map((v) => (v == null ? null : v / 60)),
            type: "line",
            borderColor: tinta.texto(),
            borderWidth: 2,
            pointRadius: 0,
            tension: 0.35,
            spanGaps: true,
          },
        ],
      },
      options: {
        ...base(),
        plugins: {
          ...base().plugins,
          legend: { ...base().plugins?.legend, display: true },
        },
        scales: {
          ...base().scales,
          y: {
            ...base().scales?.y,
            suggestedMax: Math.max(10, meta + 2),
            ticks: {
              ...base().scales?.y?.ticks,
              callback: (v: number | string) => `${v}h`,
            },
          },
        },
      },
    };
  });

  const histograma = $derived.by((): Config | null => {
    if (!d || d.bedtimes.length === 0) return null;
    return {
      type: "bar",
      data: {
        labels: d.bedtimes.map((b) => b.label),
        datasets: [
          {
            label: "Noches",
            data: d.bedtimes.map((b) => b.count),
            backgroundColor: d.bedtimes.map((b) =>
              b.label === d?.modalBedtime ? serie.uno() : serie.apagada()
            ),
            borderRadius: 3,
            borderSkipped: false,
          },
        ],
      },
      options: {
        ...base(),
        scales: {
          ...base().scales,
          y: { ...base().scales?.y, ticks: { ...base().scales?.y?.ticks, precision: 0 } },
        },
      },
    };
  });

  const dispersion = $derived.by((): Config | null => {
    if (!d || d.sleepVsEnergy.length === 0) return null;
    return {
      type: "scatter",
      data: {
        datasets: [
          {
            label: "Energía",
            data: d.sleepVsEnergy,
            backgroundColor: serie.uno(),
            pointRadius: 4,
          },
        ],
      },
      options: {
        ...base(),
        scales: {
          x: {
            ...base().scales?.x,
            title: { display: true, text: "Horas dormidas", color: tinta.eje() },
          },
          y: {
            ...base().scales?.y,
            min: 0,
            max: 5,
            ticks: { ...base().scales?.y?.ticks, stepSize: 1 },
            title: { display: true, text: "Energía", color: tinta.eje() },
          },
        },
      },
    };
  });
</script>

{#if error}
  <p class="error" role="alert">{error}</p>
{:else if !d}
  <p class="muted">Cargando…</p>
{:else if d.nights === 0}
  <div class="card vacio">
    <p class="muted">
      Todavía no hay noches registradas en este rango. El check-in matutino las va llenando.
    </p>
  </div>
{:else}
  <div class="tiles">
    <StatTile
      label="Promedio"
      value={d.avgMin !== null ? minutes(d.avgMin) : "—"}
      detail={d.avgMin !== null
        ? `${d.avgMin >= d.goalMin ? "+" : "−"}${minutes(Math.abs(d.avgMin - d.goalMin))} vs. meta`
        : ""}
      severity={d.avgMin !== null && d.avgMin >= d.goalMin ? "good" : "warning"}
    />
    <StatTile label="Mejor noche" value={d.bestMin !== null ? minutes(d.bestMin) : "—"} />
    <StatTile label="Peor noche" value={d.worstMin !== null ? minutes(d.worstMin) : "—"} />
    <StatTile
      label={d.balanceMin < 0 ? "Déficit acumulado" : "Superávit acumulado"}
      value="{d.balanceMin < 0 ? '−' : '+'}{minutes(Math.abs(d.balanceMin))}"
      detail="sobre {d.nights} {d.nights === 1 ? 'noche' : 'noches'}"
      severity={d.balanceMin < 0 ? "critical" : "good"}
    />
  </div>

  {#if barras}
    <div class="card">
      <span class="label">Duración del sueño</span>
      <Chart
        config={barras}
        height={320}
        plugins={[lineaMeta(d.goalMin / 60, `Meta ${d.goalMin / 60}h`)]}
        label="Horas dormidas por noche"
      />
    </div>
  {/if}

  <div class="mitades">
    {#if histograma}
      <div class="card">
        <span class="label">Hora de dormir</span>
        <Chart config={histograma} height={220} label="Histograma de la hora de dormir" />
        {#if d.modalBedtime}
          <p class="hint">La hora más repetida es las {d.modalBedtime}.</p>
        {/if}
      </div>
    {/if}

    {#if dispersion}
      <div class="card">
        <div class="spread">
          <span class="label">Sueño vs. energía</span>
          {#if d.correlation !== null}
            <span class="hint num">r = {d.correlation.toFixed(2)}</span>
          {/if}
        </div>
        <Chart config={dispersion} height={220} label="Sueño contra energía" />
        <p class="hint">Correlación observada, no causalidad.</p>
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

  .vacio p {
    margin: 0;
  }

  .error {
    color: var(--critical);
  }

  .hint {
    margin: 0;
  }
</style>
