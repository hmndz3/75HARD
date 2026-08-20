<script lang="ts">
  import { untrack } from "svelte";

  import * as api from "../lib/api";
  import { joinEs, minutes } from "../lib/format";
  import type { BrokenStreak } from "../lib/types";

  let {
    date,
    onhistory,
    onrestart,
    ondismiss,
  }: {
    date: string;
    onhistory: () => void;
    onrestart: () => Promise<void>;
    ondismiss: () => void;
  } = $props();

  let data = $state<BrokenStreak | null>(null);
  let error = $state("");
  let saving = $state(false);

  api
    .getBrokenStreak(untrack(() => date))
    .then((d) => (data = d))
    .catch((e) => (error = e instanceof Error ? e.message : String(e)));

  async function empezarDeNuevo() {
    if (saving || !data) return;
    saving = true;
    error = "";
    try {
      await api.endChallenge({
        reason: "failed",
        brokenOnDay: data.dayNumber,
        detail: data.failedPillars.length ? `Fallaste ${joinEs(data.failedPillars)}.` : "",
      });
      await onrestart();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      saving = false;
    }
  }

  const cifras = $derived.by(() => {
    if (!data) return [];
    const t = data.totals;
    const out: { label: string; value: string }[] = [
      { label: "Entrenamientos", value: String(t.workouts) },
      {
        label: "Sueño promedio",
        value: t.avgSleepMin !== null ? minutes(t.avgSleepMin) : "—",
      },
      {
        label: "Cambio de peso",
        value:
          t.weightDeltaKg !== null
            ? `${t.weightDeltaKg > 0 ? "+" : "−"}${Math.abs(t.weightDeltaKg).toFixed(1)} kg`
            : "—",
      },
      { label: "Horas de trabajo", value: `${t.workHours} h` },
    ];
    return out;
  });
</script>

<div class="pantalla">
  <div class="col">
    {#if error}
      <p class="error" role="alert">{error}</p>
      <button onclick={ondismiss}>Volver</button>
    {:else if data}
      <svg class="icono" viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="12" cy="12" r="9.5" />
        <path d="m8.5 8.5 7 7M15.5 8.5l-7 7" />
      </svg>

      <h1>Se rompió la racha en el día {data.dayNumber}</h1>
      <p class="muted sub">
        {#if data.failedPillars.length}
          Fallaste {joinEs(data.failedPillars)} el {data.weekdayLabel.toLowerCase()}.
        {:else}
          Cerraste como fallido el {data.weekdayLabel.toLowerCase()}.
        {/if}
      </p>

      <div class="construido">
        <span class="label">
          Lo que construiste en {data.dayNumber}
          {data.dayNumber === 1 ? "día" : "días"}
        </span>
        <div class="cifras">
          {#each cifras as c (c.label)}
            <div class="cifra">
              <span class="valor num">{c.value}</span>
              <span class="hint">{c.label}</span>
            </div>
          {/each}
        </div>
        <p class="hint nota">
          Nada de esto se borra. Tus gráficas siguen siendo continuas.
        </p>
      </div>

      <footer>
        <button class="ghost" onclick={onhistory}>Ver el historial completo</button>
        <div class="row">
          <button onclick={ondismiss}>Ahora no</button>
          <button class="primary" onclick={empezarDeNuevo} disabled={saving}>
            Empezar intento #{data.nextAttempt}
          </button>
        </div>
      </footer>
    {:else}
      <p class="muted">Cargando…</p>
    {/if}
  </div>
</div>

<style>
  .pantalla {
    height: calc(100% - var(--titlebar-h));
    overflow-y: auto;
    display: grid;
    justify-items: center;
    padding: 72px 24px;
    background: var(--bg);
  }

  .col {
    width: 520px;
    max-width: 100%;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  /* Sobrio a propósito: es un mal momento, no un evento. */
  .icono {
    width: 40px;
    height: 40px;
    fill: none;
    stroke: var(--critical);
    stroke-width: 1.5;
    stroke-linecap: round;
  }

  h1 {
    font-size: 32px;
    font-weight: 600;
    line-height: 40px;
    margin: 0;
  }

  .sub {
    margin: -12px 0 0;
  }

  .construido {
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    padding: 20px;
  }

  .cifras {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
    margin-top: 16px;
  }

  .cifra {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .valor {
    font-size: 24px;
    font-weight: 600;
    line-height: 30px;
  }

  .nota {
    margin: 16px 0 0;
  }

  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    border-top: 1px solid var(--border);
    padding-top: 20px;
  }

  .error {
    margin: 0;
    color: var(--critical);
  }
</style>
