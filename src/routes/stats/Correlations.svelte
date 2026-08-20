<script lang="ts">
  import type { ChartConfiguration } from "chart.js";

  type Config = ChartConfiguration<any>;

  import Chart from "../../lib/components/Chart.svelte";
  import { base, serie, tinta } from "../../lib/charts";
  import type { Correlation, XY } from "../../lib/types";
  import * as api from "../../lib/api";

  // $state.raw: los puntos van directos a Chart.js, que no admite proxies.
  let datos = $state.raw<Correlation[] | null>(null);
  let error = $state("");

  api
    .getCorrelations()
    .then((c) => (datos = c))
    .catch((e) => (error = e instanceof Error ? e.message : String(e)));

  /** Mínimo de pares para que el número signifique algo. Igual que en Rust. */
  const MINIMO = 5;

  /** Recta de mínimos cuadrados, solo entre el primer y el último x. */
  function tendencia(points: XY[]): XY[] {
    if (points.length < MINIMO) return [];
    const n = points.length;
    const mx = points.reduce((a, p) => a + p.x, 0) / n;
    const my = points.reduce((a, p) => a + p.y, 0) / n;
    let num = 0;
    let den = 0;
    for (const p of points) {
      num += (p.x - mx) * (p.y - my);
      den += (p.x - mx) ** 2;
    }
    if (den === 0) return [];
    const m = num / den;
    const b = my - m * mx;
    const xs = points.map((p) => p.x);
    const x0 = Math.min(...xs);
    const x1 = Math.max(...xs);
    return [
      { x: x0, y: m * x0 + b },
      { x: x1, y: m * x1 + b },
    ];
  }

  function config(c: Correlation): Config {
    const linea = tendencia(c.points);
    return {
      type: "scatter",
      data: {
        datasets: [
          {
            label: c.yLabel,
            data: c.points.map((p) => ({ x: p.x, y: p.y })),
            backgroundColor: serie.uno(),
            pointRadius: 4,
          },
          ...(linea.length === 2
            ? [
                {
                  label: "Tendencia",
                  data: linea,
                  type: "line" as const,
                  borderColor: serie.apagada(),
                  borderWidth: 2,
                  pointRadius: 0,
                },
              ]
            : []),
        ],
      },
      options: {
        ...base(),
        scales: {
          x: {
            ...base().scales?.x,
            type: "linear",
            title: { display: true, text: c.xLabel, color: tinta.eje() },
          },
          y: {
            ...base().scales?.y,
            beginAtZero: false,
            title: { display: true, text: c.yLabel, color: tinta.eje() },
          },
        },
      },
    };
  }

  /** Traduce la r a algo que se pueda leer sin haber estudiado estadística. */
  function fuerza(r: number): string {
    const a = Math.abs(r);
    const grado = a >= 0.7 ? "fuerte" : a >= 0.4 ? "moderada" : "débil";
    return `Relación ${grado} y ${r >= 0 ? "positiva" : "negativa"}`;
  }
</script>

<div class="aviso sunken">
  <svg viewBox="0 0 24 24" aria-hidden="true">
    <circle cx="12" cy="12" r="9" />
    <path d="M12 11v5M12 8h.01" />
  </svg>
  <span>
    Correlación no es causalidad. Que dos cosas se muevan juntas no significa que una cause la
    otra, y con pocos datos el número se mueve mucho.
  </span>
</div>

{#if error}
  <p class="error" role="alert">{error}</p>
{:else if !datos}
  <p class="muted">Calculando…</p>
{:else}
  <div class="rejilla">
    {#each datos as c (c.key)}
      <div class="card">
        <div class="spread">
          <span class="label">{c.label}</span>
          {#if c.r !== null}
            <span class="num r">r = {c.r.toFixed(2)}</span>
          {/if}
        </div>
        <p class="hint">{c.caption}</p>

        {#if c.points.length === 0}
          <p class="none">
            Sin datos todavía. Hace falta registrar {c.xLabel.toLowerCase()} y
            {c.yLabel.toLowerCase()} el mismo día.
          </p>
        {:else}
          <Chart config={config(c)} height={220} label="{c.xLabel} contra {c.yLabel}" />
          {#if c.r !== null}
            <div class="pie">
              <span class="fuerza">{fuerza(c.r)}</span>
              <span class="hint num">{c.n} días con ambos datos</span>
            </div>
          {:else}
            <p class="none">
              {c.n}
              {c.n === 1 ? "día" : "días"} con ambos datos. Con menos de {MINIMO} el número sería
              ruido, así que todavía no se calcula.
            </p>
          {/if}
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .aviso {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 16px;
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    font-size: 13px;
    color: var(--ink-secondary);
  }

  .aviso svg {
    width: 16px;
    height: 16px;
    flex: none;
    fill: none;
    stroke: var(--ink-muted);
    stroke-width: 1.6;
    stroke-linecap: round;
  }

  .rejilla {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(360px, 1fr));
    gap: 16px;
    align-items: start;
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .r {
    font-size: 18px;
    font-weight: 600;
  }

  .pie {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
  }

  .fuerza {
    font-size: 13px;
    color: var(--ink-secondary);
  }

  .none {
    margin: 0;
    color: var(--ink-muted);
    font-size: 13px;
  }

  .hint {
    margin: 0;
  }

  .error {
    color: var(--critical);
  }
</style>
