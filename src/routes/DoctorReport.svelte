<script lang="ts">
  import * as api from "../lib/api";
  import { kg, minutes, num, shortDate } from "../lib/format";
  import type { DoctorReport } from "../lib/types";

  let { onback }: { onback: () => void } = $props();

  let dias = $state(30);
  let d = $state<DoctorReport | null>(null);
  let error = $state("");

  $effect(() => {
    const n = dias;
    let vivo = true;
    api
      .getDoctorReport(n)
      .then((v) => vivo && (d = v))
      .catch((e) => vivo && (error = e instanceof Error ? e.message : String(e)));
    return () => {
      vivo = false;
    };
  });

  // El PDF sale del diálogo de impresión de Windows con "Guardar como PDF".
  // Es un PDF real, con texto seleccionable, y no arrastra ninguna librería.
  function imprimir() {
    window.print();
  }
</script>

<div class="hoja">
  <div class="barra no-print">
    <button class="ghost" onclick={onback}>‹ Volver</button>
    <div class="row">
      <label class="label" for="dias">Últimos</label>
      <select id="dias" bind:value={dias}>
        <option value={30}>30 días</option>
        <option value={60}>60 días</option>
        <option value={90}>90 días</option>
      </select>
      <button class="primary" onclick={imprimir} disabled={!d}>Guardar como PDF</button>
    </div>
  </div>

  {#if error}
    <p class="error" role="alert">{error}</p>
  {:else if !d}
    <p class="muted">Preparando el informe…</p>
  {:else}
    <article class="informe">
      <header>
        <div>
          <h1>Registro de glucosa</h1>
          <p class="muted">
            Del {shortDate(d.from)} al {shortDate(d.to)} · {d.days} días
          </p>
        </div>
        <div class="marca">
          <strong>75 HARD</strong>
          <span class="hint">Generado el {d.generatedAt.slice(0, 10)}</span>
        </div>
      </header>

      <p class="descargo">
        Este documento es un registro de mediciones hechas por el paciente con su propio
        glucómetro. <strong>No es un diagnóstico ni una interpretación clínica.</strong> Los rangos
        de referencia que aparecen son los de uso general y se incluyen solo para situar cada
        lectura.
      </p>

      <section class="resumen">
        <div><span class="label">Lecturas</span><span class="cifra num">{num(d.glucose.total)}</span></div>
        <div>
          <span class="label">Promedio en ayunas</span>
          <span class="cifra num">{d.glucose.avgFasting ?? "—"}<small> mg/dL</small></span>
        </div>
        <div>
          <span class="label">Promedio post-comida</span>
          <span class="cifra num">{d.glucose.avgPostMeal ?? "—"}<small> mg/dL</small></span>
        </div>
        <div>
          <span class="label">Fuera de rango</span>
          <span class="cifra num">{num(d.glucose.outOfRange)}</span>
        </div>
      </section>

      {#if d.glucose.byContext.length > 0}
        <h2>Promedio por contexto</h2>
        <table class="compacta">
          <thead>
            <tr><th>Contexto</th><th class="right">Promedio</th><th class="right">Lecturas</th></tr>
          </thead>
          <tbody>
            {#each d.glucose.byContext as c (c.context)}
              <tr>
                <td>{c.label}</td>
                <td class="right num">{c.avg} mg/dL</td>
                <td class="right num">{c.count}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}

      <h2>Contexto general del paciente</h2>
      <table class="compacta">
        <tbody>
          <tr>
            <td>Peso al inicio y al final del periodo</td>
            <td class="right num">{kg(d.weightStart)} → {kg(d.weightEnd)}</td>
          </tr>
          <tr>
            <td>Sueño promedio por noche</td>
            <td class="right num">{d.avgSleepMin !== null ? minutes(d.avgSleepMin) : "—"}</td>
          </tr>
          <tr>
            <td>Comidas registradas por día</td>
            <td class="right num">{d.mealsPerDay}</td>
          </tr>
          <tr>
            <td>Sesiones de ejercicio en el periodo</td>
            <td class="right num">{num(d.workouts)}</td>
          </tr>
        </tbody>
      </table>

      <h2>Todas las lecturas</h2>
      {#if d.glucose.readings.length === 0}
        <p class="muted">Sin lecturas en el periodo.</p>
      {:else}
        <table class="compacta">
          <thead>
            <tr>
              <th>Fecha</th>
              <th>Hora</th>
              <th class="right">mg/dL</th>
              <th>Contexto</th>
              <th>Comida vinculada</th>
              <th>Notas</th>
            </tr>
          </thead>
          <tbody>
            {#each d.glucose.readings as r (r.id)}
              <tr>
                <td class="num">{shortDate(r.date)}</td>
                <td class="num">{r.time}</td>
                <td class="right num">
                  {r.value}{#if r.outOfRange}<span class="asterisco">*</span>{/if}
                </td>
                <td>{r.contextLabel}</td>
                <td>{r.meal ?? "—"}</td>
                <td>{r.notes ?? "—"}</td>
              </tr>
            {/each}
          </tbody>
        </table>
        <p class="hint pie">
          * Fuera del rango de referencia para ese contexto: 70–99 mg/dL en ayunas y antes de
          comer, menos de 140 mg/dL dos horas después de comer.
        </p>
      {/if}
    </article>
  {/if}
</div>

<style>
  .hoja {
    height: calc(100% - var(--titlebar-h));
    overflow-y: auto;
    background: var(--bg);
    padding: 24px;
  }

  .barra {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    max-width: 860px;
    margin: 0 auto 16px;
  }

  .barra select {
    width: 120px;
  }

  .informe {
    max-width: 860px;
    margin: 0 auto;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    padding: 40px;
  }

  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 24px;
    border-bottom: 1px solid var(--border);
    padding-bottom: 16px;
  }

  h1 {
    font-size: 22px;
    font-weight: 600;
    margin: 0 0 4px;
  }

  h2 {
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--ink-muted);
    margin: 28px 0 8px;
  }

  header p {
    margin: 0;
  }

  .marca {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
  }

  .descargo {
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    padding: 12px 16px;
    font-size: 13px;
    color: var(--ink-secondary);
    margin: 16px 0 0;
  }

  .resumen {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 16px;
    margin-top: 20px;
  }

  .resumen div {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .cifra {
    font-size: 22px;
    font-weight: 600;
  }

  .cifra small {
    font-size: 12px;
    font-weight: 400;
    color: var(--ink-muted);
  }

  .compacta tbody td,
  .compacta thead th {
    height: 28px;
    padding: 0 8px;
    font-size: 13px;
  }

  .asterisco {
    color: var(--critical);
    font-weight: 600;
  }

  .pie {
    margin-top: 10px;
  }

  .error {
    color: var(--critical);
  }

  /* Al imprimir queda solo el documento: sin barra, sin fondos, sin bordes
     redondeados. Tinta negra sobre papel blanco. */
  @media print {
    .hoja {
      height: auto;
      overflow: visible;
      padding: 0;
      background: #ffffff;
    }

    .no-print {
      display: none !important;
    }

    .informe {
      max-width: none;
      border: none;
      border-radius: 0;
      padding: 0;
      background: #ffffff;
      color: #000000;
    }

    .descargo {
      background: #ffffff;
      border: 1px solid #999999;
    }

    table {
      page-break-inside: auto;
    }

    tr {
      page-break-inside: avoid;
    }

    thead {
      display: table-header-group;
    }
  }
</style>
