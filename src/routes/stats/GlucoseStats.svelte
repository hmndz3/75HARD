<script lang="ts">
  import type { ChartConfiguration } from "chart.js";

  type Config = ChartConfiguration<any>;

  import Chart from "../../lib/components/Chart.svelte";
  import StatTile from "../../lib/components/StatTile.svelte";
  import { bandasGlucosa, base, serie } from "../../lib/charts";
  import { num, shortDate } from "../../lib/format";
  import type { GlucoseStats, Range } from "../../lib/types";
  import * as api from "../../lib/api";

  let { range, onreport }: { range: Range; onreport: () => void } = $props();

  // $state.raw y no $state: Chart.js redefine propiedades sobre los objetos
  // que recibe, y un proxy reactivo lo rechaza. Aquí solo se lee y se
  // reemplaza entero, así que no hace falta proxy.
  let d = $state.raw<GlucoseStats | null>(null);
  let error = $state("");

  $effect(() => {
    const r = range;
    let vivo = true;
    api
      .getGlucoseStats(r)
      .then((v) => vivo && (d = v))
      .catch((e) => vivo && (error = e instanceof Error ? e.message : String(e)));
    return () => {
      vivo = false;
    };
  });

  const colorContexto = (ctx: string) =>
    ctx === "fasting" ? serie.uno() : ctx === "post_meal_2h" ? serie.dos() : serie.tres();

  // Dispersión temporal: el eje X es el índice de la lectura y las marcas
  // muestran la fecha. Evita meter un adaptador de fechas entero por esto.
  const dispersion = $derived.by((): Config | null => {
    if (!d || d.readings.length === 0) return null;

    const grupos = new Map<string, { x: number; y: number }[]>();
    d.readings.forEach((r, i) => {
      const clave = r.contextLabel;
      if (!grupos.has(clave)) grupos.set(clave, []);
      grupos.get(clave)?.push({ x: i, y: r.value });
    });

    const lecturas = d.readings;
    return {
      type: "scatter",
      data: {
        datasets: [...grupos.entries()].map(([label, puntos]) => ({
          label,
          data: puntos,
          backgroundColor: colorContexto(
            lecturas.find((r) => r.contextLabel === label)?.context ?? "random"
          ),
          pointRadius: 4.5,
        })),
      },
      options: {
        ...base(),
        plugins: {
          ...base().plugins,
          legend: { ...base().plugins?.legend, display: true },
          tooltip: {
            ...base().plugins?.tooltip,
            callbacks: {
              title: (items: any[]) => {
                const r = lecturas[items[0]?.parsed.x ?? 0];
                return r ? `${shortDate(r.date)} · ${r.time}` : "";
              },
              label: (item: any) => {
                const r = lecturas[item.parsed.x];
                if (!r) return "";
                const linea = `${r.value} mg/dL · ${r.contextLabel}`;
                return r.meal ? [linea, `tras ${r.meal}`] : linea;
              },
            },
          },
        },
        scales: {
          x: {
            ...base().scales?.x,
            type: "linear",
            ticks: {
              ...base().scales?.x?.ticks,
              stepSize: 1,
              callback: (v: number | string) => {
                const r = lecturas[Number(v)];
                return r && Number.isInteger(Number(v)) ? shortDate(r.date) : "";
              },
            },
          },
          y: {
            ...base().scales?.y,
            beginAtZero: false,
            min: 60,
            suggestedMax: 220,
            ticks: { ...base().scales?.y?.ticks, callback: (v: number | string) => `${v}` },
          },
        },
      },
    };
  });

  // Bandas apenas teñidas: sitúan la lectura sin gritar un diagnóstico.
  const bandas = [
    { desde: 60, hasta: 99, color: "rgb(12 163 12 / 6%)", label: "70–99" },
    { desde: 100, hasta: 125, color: "rgb(250 178 25 / 8%)", label: "100–125" },
    { desde: 126, hasta: 400, color: "rgb(208 59 59 / 6%)", label: "126+" },
  ];
</script>

<div class="aviso sunken">
  <svg viewBox="0 0 24 24" aria-hidden="true">
    <circle cx="12" cy="12" r="9" />
    <path d="M12 11v5M12 8h.01" />
  </svg>
  <span>Esta app registra y grafica. No diagnostica ni sugiere tratamiento.</span>
</div>

{#if error}
  <p class="error" role="alert">{error}</p>
{:else if !d}
  <p class="muted">Cargando…</p>
{:else if d.total === 0}
  <div class="card vacio">
    <p class="muted">Sin lecturas registradas en este rango.</p>
  </div>
{:else}
  <div class="tiles">
    <StatTile
      label="Promedio en ayunas"
      value={d.avgFasting !== null ? `${d.avgFasting} mg/dL` : "—"}
      severity={d.avgFasting !== null && d.avgFasting <= 99 ? "good" : "warning"}
      detail={d.avgFasting !== null ? "referencia 70–99" : ""}
    />
    <StatTile
      label="Promedio post-comida"
      value={d.avgPostMeal !== null ? `${d.avgPostMeal} mg/dL` : "—"}
      severity={d.avgPostMeal !== null && d.avgPostMeal < 140 ? "good" : "warning"}
      detail={d.avgPostMeal !== null ? "referencia < 140" : ""}
    />
    <StatTile label="Lecturas" value={num(d.total)} />
    <StatTile
      label="Fuera de rango"
      value={num(d.outOfRange)}
      severity={d.outOfRange > 0 ? "warning" : "good"}
    />
  </div>

  {#if dispersion}
    <div class="card">
      <div class="spread">
        <span class="label">Lecturas en el tiempo</span>
        <button onclick={onreport}>Exportar PDF para el médico</button>
      </div>
      <Chart
        config={dispersion}
        height={340}
        plugins={[bandasGlucosa(bandas)]}
        label="Lecturas de glucosa en el tiempo"
      />
    </div>
  {/if}

  {#if d.byContext.length > 0}
    <div class="card">
      <span class="label">Promedio por contexto</span>
      <ul class="contextos">
        {#each d.byContext as c (c.context)}
          <li>
            <span class="punto" style:background={colorContexto(c.context)}></span>
            <span class="grow">{c.label}</span>
            <span class="num valor">{c.avg} mg/dL</span>
            <span class="num muted n">{c.count} {c.count === 1 ? "lectura" : "lecturas"}</span>
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  <div class="card flush">
    <div class="card-head"><span class="label">Todas las lecturas</span></div>
    <div class="tablewrap">
      <table>
        <thead>
          <tr>
            <th>Fecha</th>
            <th>Hora</th>
            <th class="right">Valor</th>
            <th>Contexto</th>
            <th>Comida vinculada</th>
            <th>Notas</th>
          </tr>
        </thead>
        <tbody>
          {#each d.readings.slice().reverse() as r (r.id)}
            <tr>
              <td>{shortDate(r.date)}</td>
              <td class="num">{r.time}</td>
              <td class="right num">
                {#if r.outOfRange}<span class="fuera"></span>{/if}{r.value}
              </td>
              <td><span class="chip">{r.contextLabel}</span></td>
              <td class="muted">{r.meal ?? "—"}</td>
              <td class="muted">{r.notes ?? "—"}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
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

  .card.flush {
    gap: 0;
  }

  .contextos {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .contextos li {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 34px;
    border-bottom: 1px solid var(--border);
  }

  .contextos li:last-child {
    border-bottom: none;
  }

  .punto {
    width: 10px;
    height: 10px;
    border-radius: 2px;
    flex: none;
  }

  .grow {
    flex: 1;
  }

  .valor {
    font-weight: 600;
  }

  .n {
    width: 90px;
    text-align: right;
    font-size: 13px;
  }

  /* El valor fuera de rango lleva punto, nunca fondo de fila. */
  .fuera {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--critical);
    margin-right: 6px;
    vertical-align: middle;
  }

  .tablewrap {
    overflow-x: auto;
    max-height: 420px;
    overflow-y: auto;
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
</style>
