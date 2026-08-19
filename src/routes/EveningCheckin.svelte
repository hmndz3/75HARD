<script lang="ts">
  import { untrack } from "svelte";

  import Modal from "../lib/components/Modal.svelte";
  import StatusIcon from "../lib/components/StatusIcon.svelte";
  import * as api from "../lib/api";
  import { litres, minutes, pillarSeverity } from "../lib/format";
  import type { TodayView } from "../lib/types";

  let {
    view: initial,
    onclose,
    onsaved,
  }: { view: TodayView; onclose: () => void; onsaved: (v: TodayView) => void } = $props();

  // Copia local: el modal refresca sus propios datos cuando se llena un campo
  // en línea, sin obligar al padre a repintarse en cada tecla.
  let view = $state(untrack(() => initial));

  let notes = $state("");
  let error = $state("");
  let saving = $state(false);

  // Campos en línea para lo que falta: se llena sin salir de la pantalla.
  let workoutMin = $state("");
  let workoutKind = $state("gym");
  let pages = $state("");
  let busy = $state("");

  const resumen = $derived([
    {
      key: "sleep",
      label: "Sueño",
      status: view.sleep ? (view.sleep.minutes >= view.sleepGoalMin ? "done" : "partial") : "missing",
      value: view.sleep ? minutes(view.sleep.minutes) : "sin registrar",
    },
    {
      key: "meals",
      label: "Comidas",
      status: view.mealsCount > 0 ? "done" : "missing",
      value: `${view.mealsCount} registradas`,
    },
    {
      key: "snacks",
      label: "Snacks",
      status: "done",
      value: `${view.snacksCount}`,
    },
    {
      key: "workout",
      label: "Ejercicio",
      status: view.workoutMin > 0 ? "done" : "missing",
      value: view.workoutMin > 0 ? `${view.workoutMin} min` : "",
    },
    {
      key: "water",
      label: "Agua",
      status:
        view.waterMl >= view.waterGoalMl ? "done" : view.waterMl > 0 ? "partial" : "missing",
      value: `${litres(view.waterMl)} / ${litres(view.waterGoalMl)} L`,
    },
    {
      key: "reading",
      label: "Lectura",
      status:
        view.readingPages >= view.readingGoalPages
          ? "done"
          : view.readingPages > 0
            ? "partial"
            : "missing",
      value: view.readingPages > 0 ? `${view.readingPages} páginas` : "",
    },
    {
      key: "mood",
      label: "Ánimo",
      status: view.mood ? "done" : "missing",
      value: view.mood ? `${view.mood.mood}/5` : "",
    },
  ] as const);

  async function run(what: string, fn: () => Promise<unknown>) {
    busy = what;
    error = "";
    try {
      await fn();
      view = await api.getToday(view.date);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = "";
    }
  }

  const addWorkout = () =>
    run("workout", async () => {
      await api.addWorkout({
        date: view.date,
        kind: workoutKind,
        durationMin: Number(workoutMin),
      });
      workoutMin = "";
    });

  const addReading = () =>
    run("reading", async () => {
      await api.addReading({ date: view.date, pages: Number(pages) });
      pages = "";
    });

  const addWater = () => run("water", () => api.addWater(500, view.date));

  async function close(status: "complete" | "failed") {
    if (saving) return;
    saving = true;
    error = "";
    try {
      const updated = await api.closeDay({
        date: view.date,
        status,
        notes: notes.trim() || undefined,
      });
      onsaved(updated);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  const faltanObligatorios = $derived(
    view.pillars.filter((p) => p.required && p.status !== "done").length
  );
</script>

<Modal
  width={640}
  title="Cierre del día{view.dayNumber ? ` · Día ${view.dayNumber}` : ''}"
  subtitle={view.weekdayLabel}
  {onclose}
>
  <div class="body">
    <ul class="rows">
      {#each resumen as row (row.key)}
        <li>
          <StatusIcon severity={pillarSeverity[row.status]} size={18} title={row.status} />
          <span class="name">{row.label}</span>

          {#if row.key === "workout" && row.status === "missing"}
            <div class="inline">
              <input
                type="number"
                min="1"
                placeholder="Minutos"
                bind:value={workoutMin}
                onkeydown={(e) => e.key === "Enter" && workoutMin && addWorkout()}
              />
              <select bind:value={workoutKind}>
                <option value="gym">Gimnasio</option>
                <option value="cardio">Cardio</option>
                <option value="outdoor">Al aire libre</option>
                <option value="sport">Deporte</option>
                <option value="other">Otro</option>
              </select>
              <button onclick={addWorkout} disabled={!workoutMin || busy === "workout"}>
                Guardar
              </button>
            </div>
          {:else if row.key === "reading" && row.status !== "done"}
            <div class="inline">
              <input
                type="number"
                min="1"
                placeholder="Páginas"
                bind:value={pages}
                onkeydown={(e) => e.key === "Enter" && pages && addReading()}
              />
              <button onclick={addReading} disabled={!pages || busy === "reading"}>Guardar</button>
            </div>
          {:else if row.key === "water" && row.status !== "done"}
            <div class="inline">
              <span class="num muted">{row.value}</span>
              <button onclick={addWater} disabled={busy === "water"}>+500 ml</button>
            </div>
          {:else}
            <span class="num muted value">{row.value}</span>
          {/if}
        </li>
      {/each}
    </ul>

    <textarea rows="3" placeholder="Notas del día (opcional)" bind:value={notes}></textarea>

    {#if error}<p class="error" role="alert">{error}</p>{/if}
  </div>

  {#snippet footer()}
    <div class="stack footer">
      <span class="hint">
        {#if faltanObligatorios > 0}
          Faltan {faltanObligatorios} pilares obligatorios. Puedes cerrarlo igual: la decisión es tuya.
        {:else}
          Vas a cerrar el día{view.dayNumber ? ` ${view.dayNumber}` : ""}. Después podrás editarlo desde Historial.
        {/if}
      </span>
      <div class="buttons">
        <button class="danger" onclick={() => close("failed")} disabled={saving}>
          Marcar como fallido
        </button>
        <button class="primary" onclick={() => close("complete")} disabled={saving}>
          Día completo
        </button>
      </div>
    </div>
  {/snippet}
</Modal>

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .rows {
    list-style: none;
    margin: 0;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    overflow: hidden;
  }

  li {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 44px;
    padding: 0 14px;
    border-bottom: 1px solid var(--border);
  }

  li:last-child {
    border-bottom: none;
  }

  li:nth-child(even) {
    background: var(--surface-1);
  }

  .name {
    flex: 1;
  }

  .value {
    color: var(--ink-secondary);
  }

  .inline {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .inline input {
    width: 96px;
    height: 30px;
  }

  .inline select {
    width: 130px;
    height: 30px;
  }

  .inline button {
    height: 30px;
  }

  .footer {
    gap: 12px;
  }

  .buttons {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .buttons button {
    height: 40px;
  }

  .error {
    margin: 0;
    color: var(--critical);
  }
</style>
