<script lang="ts">
  import { untrack } from "svelte";

  import * as api from "../lib/api";
  import { todayIso } from "../lib/format";
  import type { Bootstrap, Pillar } from "../lib/types";

  let { boot, ondone }: { boot: Bootstrap; ondone: () => Promise<void> } = $props();

  const steps = ["Fechas", "Reglas", "Metas"];
  let step = $state(0);
  let error = $state("");
  let saving = $state(false);

  let name = $state(untrack(() => `Intento #${boot.attemptNumber}`));
  let startDate = $state(untrack(() => boot.today || todayIso()));
  let targetDays = $state(75);

  // Copia editable de los pilares de fábrica.
  let pillars = $state<Pillar[]>(untrack(() => boot.defaultRules.pillars.map((p) => ({ ...p }))));
  let enabled = $state<Record<string, boolean>>(
    untrack(() => Object.fromEntries(boot.defaultRules.pillars.map((p) => [p.key, true])))
  );

  let sleepGoal = $state(untrack(() => Number(boot.settings.sleep_goal_min ?? 420)));
  let waterGoal = $state(untrack(() => Number(boot.settings.water_goal_ml ?? 3000)));
  let workoutGoal = $state(untrack(() => Number(boot.settings.workout_goal_min ?? 45)));
  let readingGoal = $state(untrack(() => Number(boot.settings.reading_goal_pages ?? 10)));

  async function finish() {
    if (saving) return;
    saving = true;
    error = "";
    try {
      const chosen = pillars
        .filter((p) => enabled[p.key])
        .map((p) => ({
          ...p,
          goal:
            p.key === "sleep"
              ? sleepGoal
              : p.key === "water"
                ? waterGoal
                : p.key === "workout" || p.key === "outdoor"
                  ? workoutGoal
                  : p.key === "reading"
                    ? readingGoal
                    : p.goal,
        }));

      await api.setSettings({
        sleep_goal_min: String(sleepGoal),
        water_goal_ml: String(waterGoal),
        workout_goal_min: String(workoutGoal),
        reading_goal_pages: String(readingGoal),
      });
      await api.createChallenge({
        name,
        startDate,
        targetDays,
        rules: { pillars: chosen },
      });
      await ondone();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      saving = false;
    }
  }

  const puedeSeguir = $derived(
    step === 0
      ? /^\d{4}-\d{2}-\d{2}$/.test(startDate) && targetDays >= 1
      : step === 1
        ? pillars.some((p) => enabled[p.key] && p.required)
        : true
  );
</script>

<div class="wizard">
  <div class="col">
    <div class="progress">
      {#each steps as label, i (label)}
        <div class="bar" class:on={i <= step}></div>
      {/each}
    </div>
    <div class="steplabels">
      {#each steps as label, i (label)}
        <span class:on={i === step}>{label}</span>
      {/each}
    </div>

    {#if step === 0}
      <h1 class="section-title">¿Cuándo empieza?</h1>
      <p class="muted">
        Si ya llevas días del reto anotados en otro lado, pon la fecha real de inicio: la app
        calcula el número de día sola y puedes llenar los días pasados desde Historial.
      </p>
      <div class="fields">
        <div class="stack">
          <label class="label" for="nombre">Nombre del intento</label>
          <input id="nombre" type="text" bind:value={name} />
        </div>
        <div class="pair">
          <div class="stack">
            <label class="label" for="inicio">Fecha de inicio</label>
            <input id="inicio" type="date" bind:value={startDate} />
          </div>
          <div class="stack">
            <label class="label" for="dias">Duración (días)</label>
            <input id="dias" type="number" min="1" max="365" bind:value={targetDays} />
          </div>
        </div>
      </div>

    {:else if step === 1}
      <h1 class="section-title">¿Qué cuenta como día completo?</h1>
      <p class="muted">
        Los pilares obligatorios son los que definen si el día se cerró bien. Los opcionales solo
        suman estadísticas.
      </p>
      <ul class="pillars">
        {#each pillars as p (p.key)}
          <li class:off={!enabled[p.key]}>
            <input
              type="checkbox"
              id="p-{p.key}"
              checked={enabled[p.key]}
              onchange={(e) => (enabled[p.key] = (e.currentTarget as HTMLInputElement).checked)}
            />
            <label class="grow" for="p-{p.key}">{p.label}</label>
            <div class="segmented">
              <button class="seg" class:on={p.required} onclick={() => (p.required = true)} disabled={!enabled[p.key]}>
                Obligatorio
              </button>
              <button class="seg" class:on={!p.required} onclick={() => (p.required = false)} disabled={!enabled[p.key]}>
                Opcional
              </button>
            </div>
          </li>
        {/each}
      </ul>

    {:else}
      <h1 class="section-title">Metas numéricas</h1>
      <p class="muted">Se pueden cambiar después desde Ajustes.</p>
      <div class="fields">
        <div class="pair">
          <div class="stack">
            <label class="label" for="g1">Sueño (minutos)</label>
            <input id="g1" class="num" type="number" min="60" max="960" step="15" bind:value={sleepGoal} />
          </div>
          <div class="stack">
            <label class="label" for="g2">Agua (ml)</label>
            <input id="g2" class="num" type="number" min="250" max="10000" step="250" bind:value={waterGoal} />
          </div>
        </div>
        <div class="pair">
          <div class="stack">
            <label class="label" for="g3">Ejercicio (minutos)</label>
            <input id="g3" class="num" type="number" min="5" max="600" step="5" bind:value={workoutGoal} />
          </div>
          <div class="stack">
            <label class="label" for="g4">Lectura (páginas)</label>
            <input id="g4" class="num" type="number" min="1" max="1000" bind:value={readingGoal} />
          </div>
        </div>
      </div>
      <p class="hint">
        Las calorías no tienen meta y nunca la tendrán: se registran si quieres y la app no
        opina sobre ellas.
      </p>
    {/if}

    {#if error}<p class="error" role="alert">{error}</p>{/if}

    <footer>
      <button class="ghost" onclick={() => step--} disabled={step === 0}>Atrás</button>
      {#if step < 2}
        <button class="primary" onclick={() => step++} disabled={!puedeSeguir}>Continuar</button>
      {:else}
        <button class="primary" onclick={finish} disabled={saving}>Empezar el reto</button>
      {/if}
    </footer>
  </div>
</div>

<style>
  .wizard {
    height: calc(100% - var(--titlebar-h));
    overflow-y: auto;
    display: grid;
    justify-items: center;
    padding: 64px 24px;
    background: var(--bg);
  }

  .col {
    width: 640px;
    max-width: 100%;
    display: flex;
    flex-direction: column;
    gap: 16px;
    align-content: start;
  }

  .progress {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
  }

  .bar {
    height: 4px;
    background: var(--border);
    border-radius: 2px;
  }

  .bar.on {
    background: var(--accent);
  }

  .steplabels {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
    font-size: 12px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--ink-muted);
  }

  .steplabels .on {
    color: var(--ink-primary);
  }

  h1 {
    margin: 16px 0 0;
  }

  p {
    margin: 0;
  }

  .fields {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .pair {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .stack {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .pillars {
    list-style: none;
    margin: 0;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    background: var(--surface-2);
  }

  .pillars li {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 48px;
    padding: 8px 14px;
    border-bottom: 1px solid var(--border);
  }

  .pillars li:last-child {
    border-bottom: none;
  }

  .pillars li.off {
    opacity: 0.5;
  }

  .pillars input[type="checkbox"] {
    width: 16px;
    flex: none;
  }

  .grow {
    flex: 1;
  }

  .segmented {
    display: flex;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-control);
    overflow: hidden;
    flex: none;
  }

  .seg {
    height: 28px;
    border: none;
    border-radius: 0;
    border-right: 1px solid var(--border-strong);
    background: var(--surface-2);
    color: var(--ink-secondary);
    font-size: 12px;
  }

  .seg:last-child {
    border-right: none;
  }

  .seg.on {
    background: var(--surface-sunken);
    color: var(--ink-primary);
    font-weight: 500;
  }

  footer {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    border-top: 1px solid var(--border);
    padding-top: 16px;
    margin-top: 8px;
  }

  .error {
    color: var(--critical);
  }
</style>
