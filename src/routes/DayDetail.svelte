<script lang="ts">
  import Modal from "../lib/components/Modal.svelte";
  import EntryForms from "../lib/components/EntryForms.svelte";
  import type { EntryTab } from "../lib/components/EntryForms.svelte";
  import * as api from "../lib/api";
  import {
    clock,
    glucoseContextLabel,
    kg,
    litres,
    minutes,
    num,
    statusLabel,
    workoutKindLabel,
  } from "../lib/format";
  import type { DayDetail } from "../lib/types";

  let { date, onback }: { date: string; onback: () => void } = $props();

  let day = $state<DayDetail | null>(null);
  let error = $state("");
  let notes = $state("");
  let notesDirty = $state(false);
  let entryTab = $state<EntryTab | null>(null);

  async function load() {
    try {
      day = await api.getDayDetail(date);
      notes = day.notes ?? "";
      notesDirty = false;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  load();

  async function run(fn: () => Promise<unknown>) {
    error = "";
    try {
      await fn();
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  const setStatus = (status: "complete" | "failed" | "skipped") =>
    run(() => api.closeDay({ date, status }));

  const reopen = () => run(() => api.reopenDay(date));

  const remove = (kind: "meal" | "workout" | "glucose" | "reading" | "work", id: string) =>
    run(() => api.deleteEntry(kind, id));

  const saveNotes = () =>
    run(async () => {
      await api.setDayNotes(date, notes.trim() || null);
      notesDirty = false;
    });

  const DIA_MS = 24 * 60 * 60 * 1000;

  /** La pista representa "la noche": de las 18:00 de la víspera a las 18:00
      del día en que se despertó. Así la barra cae siempre en el centro. */
  const sleepBar = $derived.by(() => {
    if (!day?.sleep) return null;
    const wake = new Date(day.sleep.wakeTime);
    const anchor = new Date(day.sleep.wakeTime.slice(0, 10) + "T18:00:00");
    anchor.setDate(anchor.getDate() - 1);

    const pct = (t: number) =>
      Math.max(0, Math.min(100, ((t - anchor.getTime()) / DIA_MS) * 100));

    const from = pct(new Date(day.sleep.bedtime).getTime());
    const to = pct(wake.getTime());
    return { left: from, width: Math.max(1, to - from) };
  });

  const workByCategory = $derived.by(() => {
    const totals = new Map<string, number>();
    for (const s of day?.work ?? []) {
      totals.set(s.category, (totals.get(s.category) ?? 0) + s.minutes);
    }
    const total = [...totals.values()].reduce((a, b) => a + b, 0);
    return { total, items: [...totals.entries()] };
  });

  const categoryColors = ["var(--accent)", "#c05a30", "#1f8a6d"];
</script>

<div class="page">
  <header class="head">
    <button class="ghost back" onclick={onback} aria-label="Volver al historial">‹</button>
    <h1 class="section-title">
      {#if day}
        {day.dayNumber ? `Día ${day.dayNumber} · ` : ""}{day.weekdayLabel}
      {:else}
        Cargando…
      {/if}
    </h1>
    {#if day}
      <span class="status">{statusLabel[day.status]}</span>
      <div class="grow"></div>
      <div class="row">
        {#if day.status === "pending"}
          <button onclick={() => setStatus("skipped")}>Marcar pausado</button>
          <button class="danger" onclick={() => setStatus("failed")}>Fallido</button>
          <button class="primary" onclick={() => setStatus("complete")}>Completo</button>
        {:else}
          <button onclick={reopen}>Reabrir para editar</button>
        {/if}
      </div>
    {/if}
  </header>

  {#if error}<p class="error" role="alert">{error}</p>{/if}

  {#if day}
    <div class="masonry">
      <!-- SUEÑO -->
      <div class="card">
        <span class="label">Sueño</span>
        {#if day.sleep && sleepBar}
          <span class="card-value">{minutes(day.sleep.minutes)}</span>
          <div class="track" role="img" aria-label="Ventana de sueño">
            <div class="seg" style:left="{sleepBar.left}%" style:width="{sleepBar.width}%"></div>
          </div>
          <div class="spread hint">
            <span class="num">{clock(day.sleep.bedtime)}</span>
            <span class="num">{clock(day.sleep.wakeTime)}</span>
          </div>
        {:else}
          <p class="none">Sin registrar. Se llena desde el check-in matutino.</p>
        {/if}
      </div>

      <!-- EJERCICIO -->
      <div class="card">
        <div class="spread">
          <span class="label">Ejercicio</span>
          <button class="ghost small" onclick={() => (entryTab = "ejercicio")}>+ Agregar</button>
        </div>
        {#if day.workouts.length}
          <ul class="entries">
            {#each day.workouts as w (w.id)}
              <li>
                <span class="num time">{clock(w.startedAt)}</span>
                <span class="grow">
                  {w.description || workoutKindLabel(w.kind)}
                  <span class="muted"> · {w.durationMin} min</span>
                  {#if w.isOutdoor}<span class="chip small">Al aire libre</span>{/if}
                </span>
                <span class="muted num">{w.caloriesBurned !== null ? `${num(w.caloriesBurned)} kcal` : "—"}</span>
                <button class="ghost del" onclick={() => remove("workout", w.id)} aria-label="Borrar">✕</button>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="none">Sin registrar.</p>
        {/if}
      </div>

      <!-- COMIDAS -->
      <div class="card">
        <div class="spread">
          <span class="label">Comidas</span>
          <button class="ghost small" onclick={() => (entryTab = "comida")}>+ Agregar</button>
        </div>
        {#if day.meals.length}
          <ul class="entries">
            {#each day.meals as m (m.id)}
              <li>
                <span class="num time">{clock(m.eatenAt)}</span>
                <span class="grow">
                  {m.description}
                  {#if m.kind === "snack"}<span class="chip small">Snack</span>{/if}
                </span>
                <span class="muted num">{m.calories !== null ? `${num(m.calories)} kcal` : "—"}</span>
                <button class="ghost del" onclick={() => remove("meal", m.id)} aria-label="Borrar">✕</button>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="none">Sin registrar.</p>
        {/if}
      </div>

      <!-- AZÚCAR -->
      <div class="card">
        <div class="spread">
          <span class="label">Azúcar</span>
          <button class="ghost small" onclick={() => (entryTab = "azucar")}>+ Agregar</button>
        </div>
        {#if day.glucose.length}
          <ul class="entries">
            {#each day.glucose as g (g.id)}
              <li class="glucose">
                <span class="grow">
                  <span class="value num">{g.valueMgdl}</span>
                  <span class="muted">mg/dL</span>
                  <span class="chip small">{glucoseContextLabel(g.context)}</span>
                  <span class="hint">{clock(g.measuredAt)}</span>
                  {#if g.linkedMealDescription}
                    <span class="hint">· tras {g.linkedMealDescription}</span>
                  {/if}
                </span>
                <button class="ghost del" onclick={() => remove("glucose", g.id)} aria-label="Borrar">✕</button>
              </li>
            {/each}
          </ul>
          <p class="hint">La app registra y grafica. No diagnostica ni sugiere tratamiento.</p>
        {:else}
          <p class="none">Sin registrar.</p>
        {/if}
      </div>

      <!-- AGUA, PESO, LECTURA -->
      <div class="card">
        <span class="label">Del día</span>
        <ul class="entries plain">
          <li><span class="grow">Agua</span><span class="num">{litres(day.waterMl)} L</span></li>
          <li>
            <span class="grow">Peso</span>
            <span class="num">{kg(day.weightKg)}</span>
          </li>
          <li>
            <span class="grow">Lectura</span>
            <span class="num">
              {day.reading.reduce((a, r) => a + r.pages, 0) || "—"}
              {day.reading.length ? " páginas" : ""}
            </span>
          </li>
        </ul>
        <div class="row wrap">
          <button class="small" onclick={() => (entryTab = "agua")}>Agua</button>
          <button class="small" onclick={() => (entryTab = "lectura")}>Lectura</button>
        </div>
      </div>

      <!-- TRABAJO -->
      <div class="card">
        <div class="spread">
          <span class="label">Trabajo</span>
          <button class="ghost small" onclick={() => (entryTab = "trabajo")}>+ Agregar</button>
        </div>
        {#if workByCategory.total > 0}
          <span class="card-value">{(workByCategory.total / 60).toFixed(1)} h</span>
          <div class="stacked">
            {#each workByCategory.items as [cat, min], i (cat)}
              <div
                class="chunk"
                style:width="{(min / workByCategory.total) * 100}%"
                style:background={categoryColors[i % categoryColors.length]}
                title="{cat}: {min} min"
              ></div>
            {/each}
          </div>
          <div class="row wrap legend">
            {#each workByCategory.items as [cat, min], i (cat)}
              <span class="key">
                <i style:background={categoryColors[i % categoryColors.length]}></i>
                {cat} <span class="muted num">{minutes(min)}</span>
              </span>
            {/each}
          </div>
        {:else}
          <p class="none">Sin registrar.</p>
        {/if}
      </div>

      <!-- ÁNIMO -->
      <div class="card">
        <span class="label">Ánimo y energía</span>
        {#if day.mood}
          {#each [["Ánimo", day.mood.mood], ["Energía", day.mood.energy]] as [name, val] (name)}
            <div class="moodrow">
              <span class="muted">{name}</span>
              <div class="cells">
                {#each [1, 2, 3, 4, 5] as n (n)}
                  <span class="cell" class:on={val === n}></span>
                {/each}
              </div>
            </div>
          {/each}
        {:else}
          <p class="none">Sin registrar.</p>
        {/if}
      </div>

      <!-- NOTAS -->
      <div class="card notes">
        <span class="label">Notas</span>
        <textarea
          rows="3"
          placeholder="Notas del día (opcional)"
          bind:value={notes}
          oninput={() => (notesDirty = true)}
        ></textarea>
        {#if notesDirty}
          <div class="row">
            <button class="primary small" onclick={saveNotes}>Guardar notas</button>
            <button class="ghost small" onclick={() => { notes = day?.notes ?? ""; notesDirty = false; }}>
              Descartar
            </button>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

{#if entryTab}
  <Modal
    width={520}
    title="Agregar registro"
    subtitle={day?.weekdayLabel ?? ""}
    onclose={() => {
      entryTab = null;
      load();
    }}
  >
    <EntryForms
      {date}
      tabs={["comida", "ejercicio", "azucar", "agua", "lectura", "trabajo"]}
      bind:tab={entryTab as EntryTab}
      onsaved={() => load()}
    />
  </Modal>
{/if}

<style>
  .page {
    max-width: 1200px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .head {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .back {
    width: 28px;
    height: 28px;
    padding: 0;
    font-size: 20px;
    line-height: 1;
  }

  .status {
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-control);
    padding: 2px 8px;
    font-size: 12px;
    color: var(--ink-secondary);
  }

  .grow {
    flex: 1;
  }

  .masonry {
    columns: 2;
    column-gap: 16px;
  }

  .masonry .card {
    break-inside: avoid;
    margin-bottom: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .notes {
    column-span: all;
  }

  .track {
    height: 12px;
    background: var(--surface-sunken);
    border-radius: 3px;
    position: relative;
    overflow: hidden;
  }

  .track .seg {
    position: absolute;
    top: 0;
    bottom: 0;
    background: var(--accent);
    border-radius: 3px;
  }

  .entries {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .entries li {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 34px;
    padding: 4px 0;
    border-bottom: 1px solid var(--border);
  }

  .entries li:last-child {
    border-bottom: none;
  }

  .entries.plain li {
    justify-content: space-between;
  }

  .time {
    width: 44px;
    color: var(--ink-secondary);
    flex: none;
  }

  .glucose .value {
    font-size: 20px;
    font-weight: 600;
  }

  .chip.small {
    height: 20px;
    font-size: 11px;
    padding: 0 6px;
    margin-left: 6px;
  }

  .del {
    width: 24px;
    height: 24px;
    padding: 0;
    font-size: 11px;
    color: var(--ink-muted);
    flex: none;
  }

  .del:hover {
    color: var(--critical);
  }

  button.small {
    height: 28px;
    font-size: 13px;
  }

  .none {
    margin: 0;
    color: var(--ink-muted);
    font-size: 13px;
  }

  .stacked {
    display: flex;
    height: 12px;
    gap: 2px;
    border-radius: 3px;
    overflow: hidden;
  }

  .legend {
    flex-wrap: wrap;
    gap: 12px;
    font-size: 12px;
  }

  .key {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .key i {
    width: 10px;
    height: 10px;
    border-radius: 2px;
    display: inline-block;
  }

  .moodrow {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .moodrow .muted {
    width: 64px;
  }

  .cells {
    display: flex;
    gap: 6px;
  }

  .cells .cell {
    width: 28px;
    height: 28px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-control);
  }

  .cells .cell.on {
    background: var(--accent);
    border-color: var(--accent);
  }

  .error {
    margin: 0;
    color: var(--critical);
  }
</style>
