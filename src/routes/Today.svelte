<script lang="ts">
  import CoachPanel from "../lib/components/CoachPanel.svelte";
  import Sparkline from "../lib/components/Sparkline.svelte";
  import StatusIcon from "../lib/components/StatusIcon.svelte";
  import Modal from "../lib/components/Modal.svelte";
  import EntryForms from "../lib/components/EntryForms.svelte";
  import type { EntryTab } from "../lib/components/EntryForms.svelte";
  import MorningCheckin from "./MorningCheckin.svelte";
  import EveningCheckin from "./EveningCheckin.svelte";
  import * as api from "../lib/api";
  import { litres, minutes, num, pillarSeverity, statusLabel } from "../lib/format";
  import type { TodayView } from "../lib/types";

  let {
    view,
    onupdate,
  }: { view: TodayView; onupdate: (v: TodayView) => void } = $props();

  let modal = $state<"morning" | "evening" | "entry" | null>(null);
  let entryTab = $state<EntryTab>("comida");
  let toast = $state("");
  let toastTimer: ReturnType<typeof setTimeout> | undefined;

  function flash(mensaje: string) {
    toast = mensaje;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = ""), 2600);
  }

  async function refresh() {
    onupdate(await api.getToday(view.date));
  }

  function openEntry(tab: EntryTab) {
    entryTab = tab;
    modal = "entry";
  }

  async function quickWater() {
    const total = await api.addWater(500, view.date);
    flash(`Agua: ${litres(total)} L en total hoy`);
    await refresh();
  }

  // Atajos de teclado de la pantalla principal.
  function onkeydown(e: KeyboardEvent) {
    if (modal || e.ctrlKey || e.altKey || e.metaKey) return;
    const tag = (e.target as HTMLElement)?.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;
    const map: Record<string, () => void> = {
      c: () => openEntry("comida"),
      e: () => openEntry("ejercicio"),
      a: () => openEntry("azucar"),
      g: () => quickWater(),
      m: () => (modal = "morning"),
    };
    const action = map[e.key.toLowerCase()];
    if (action) {
      e.preventDefault();
      action();
    }
  }

  const sleepDelta = $derived(
    view.sleep ? view.sleep.minutes - view.sleepGoalMin : null
  );
  const waterPct = $derived(Math.min(100, (view.waterMl / view.waterGoalMl) * 100));
</script>

<svelte:window on:keydown={onkeydown} />

<div class="page">
  <section class="hero">
    <div class="numbers">
      {#if view.dayNumber !== null}
        <span class="hero-num">DÍA {view.dayNumber}</span>
        <span class="of">de {view.targetDays}</span>
      {:else}
        <span class="hero-num">—</span>
        <span class="of">el reto todavía no empieza</span>
      {/if}
      {#if view.status !== "pending"}
        <span class="pill">{statusLabel[view.status]}</span>
      {/if}
    </div>

    <div class="card streak">
      <div class="spread">
        <span class="label">Racha actual</span>
        <span class="value num">{view.streak} días</span>
      </div>
      <Sparkline days={view.recent} />
      <span class="hint">Racha más larga: {view.longestStreak} días</span>
    </div>
  </section>

  <section class="metrics">
    <div class="card metric">
      <span class="label">Sueño anoche</span>
      <span class="card-value">{view.sleep ? minutes(view.sleep.minutes) : "—"}</span>
      {#if view.sleep && sleepDelta !== null}
        <span class={sleepDelta >= 0 ? "s-good sub" : "s-warning sub"}>
          {sleepDelta >= 0 ? "+" : "−"}{minutes(Math.abs(sleepDelta))} vs. tu meta
        </span>
      {:else}
        <button class="ghost sub link" onclick={() => (modal = "morning")}>
          sin registrar · check-in matutino
        </button>
      {/if}
    </div>

    <div class="card metric">
      <span class="label">Ejercicio hoy</span>
      <span class="card-value">{view.workoutMin} min</span>
      <span class="sub {view.workoutMin > 0 ? 'muted' : 's-critical'}">
        {view.workoutMin > 0
          ? view.workoutKcal
            ? `${num(view.workoutKcal)} kcal quemadas`
            : "registrado"
          : "sin registrar"}
      </span>
    </div>

    <div class="card metric">
      <span class="label">Comidas</span>
      <span class="card-value">
        {view.mealsCount}{view.snacksCount > 0
          ? ` + ${view.snacksCount} ${view.snacksCount === 1 ? "snack" : "snacks"}`
          : ""}
      </span>
      <!-- Las calorías se muestran solo si existen, y sin juicio de ningún tipo. -->
      <span class="sub muted">
        {view.caloriesIn !== null ? `${num(view.caloriesIn)} kcal registradas` : "sin calorías registradas"}
      </span>
    </div>

    <div class="card metric">
      <span class="label">Agua</span>
      <span class="card-value"
        >{litres(view.waterMl)}<span class="of-small">&nbsp;/ {litres(view.waterGoalMl)} L</span
        ></span
      >
      <div class="meter" role="img" aria-label="{litres(view.waterMl)} de {litres(view.waterGoalMl)} litros">
        <div class="fill" style:width="{waterPct}%"></div>
      </div>
    </div>
  </section>

  <section class="columns">
    <div class="card flush pillars">
      <div class="card-head">
        <span class="label">Pilares de hoy</span>
        <span class="hint">
          {view.pillars.filter((p) => p.status === "done").length} de {view.pillars.length}
        </span>
      </div>
      <ul>
        {#each view.pillars as p (p.key)}
          <li class:optional={!p.required}>
            <StatusIcon severity={pillarSeverity[p.status]} size={20} title={p.status} />
            <span class="name">
              {p.label}
              {#if !p.required}<span class="opt">opcional</span>{/if}
            </span>
            <span class="detail muted">{p.detail}</span>
          </li>
        {/each}
      </ul>
    </div>

    <div class="side">
      <CoachPanel message={view.coach} label="Coach" />

      <div class="card flush">
        <div class="card-head"><span class="label">Acciones rápidas</span></div>
        <div class="actions">
          <button onclick={() => openEntry("comida")}>Registrar comida <kbd>C</kbd></button>
          <button onclick={() => openEntry("ejercicio")}>Registrar ejercicio <kbd>E</kbd></button>
          <button onclick={() => openEntry("azucar")}>Medir azúcar <kbd>A</kbd></button>
          <button onclick={quickWater}>Agregar 500 ml <kbd>G</kbd></button>
          <button onclick={() => openEntry("lectura")}>Registrar lectura</button>
          <button onclick={() => openEntry("trabajo")}>Sesión de trabajo</button>
        </div>
        <div class="closer">
          {#if view.status === "pending"}
            <button class="primary wide" onclick={() => (modal = "evening")}>Cerrar el día</button>
          {:else}
            <div class="spread">
              <span class="muted">Día cerrado como {statusLabel[view.status].toLowerCase()}.</span>
              <button onclick={() => (modal = "evening")}>Editar cierre</button>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </section>
</div>

{#if toast}
  <div class="toast" role="status">{toast}</div>
{/if}

{#if modal === "morning"}
  <MorningCheckin
    {view}
    onclose={() => (modal = null)}
    onsaved={(v) => {
      onupdate(v);
      modal = null;
      flash("Check-in matutino guardado");
    }}
  />
{:else if modal === "evening"}
  <EveningCheckin
    {view}
    onclose={() => {
      modal = null;
      refresh();
    }}
    onsaved={(v) => {
      onupdate(v);
      modal = null;
      flash("Día cerrado");
    }}
  />
{:else if modal === "entry"}
  <Modal
    width={520}
    title="Registro rápido"
    subtitle={view.weekdayLabel}
    onclose={() => {
      modal = null;
      refresh();
    }}
  >
    <EntryForms
      date={view.date}
      tabs={["comida", "ejercicio", "azucar", "agua", "lectura", "trabajo"]}
      bind:tab={entryTab}
      onsaved={(m) => {
        flash(m);
        refresh();
      }}
    />
  </Modal>
{/if}

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 1200px;
  }

  .hero {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
  }

  .numbers {
    display: flex;
    align-items: baseline;
    gap: 12px;
  }

  .of {
    font-size: 18px;
    color: var(--ink-muted);
  }

  .pill {
    align-self: center;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-control);
    padding: 2px 8px;
    font-size: 12px;
    color: var(--ink-secondary);
  }

  .streak {
    width: 320px;
    flex: none;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px 20px;
  }

  .streak .value {
    font-size: 20px;
    font-weight: 600;
  }

  .metrics {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 16px;
  }

  .metric {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-height: 118px;
  }

  .sub {
    font-size: 13px;
  }

  .link {
    align-self: flex-start;
    height: 22px;
    padding: 0;
    font-size: 13px;
    text-decoration: underline;
    text-underline-offset: 3px;
    color: var(--critical);
  }

  .of-small {
    font-size: 16px;
    font-weight: 400;
    color: var(--ink-muted);
  }

  .meter {
    height: 6px;
    background: var(--surface-sunken);
    border-radius: 3px;
    overflow: hidden;
    margin-top: 6px;
  }

  .meter .fill {
    height: 100%;
    background: var(--accent);
  }

  .columns {
    display: grid;
    grid-template-columns: 3fr 2fr;
    gap: 16px;
    align-items: start;
  }

  .pillars ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .pillars li {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 52px;
    padding: 0 20px;
    border-bottom: 1px solid var(--border);
  }

  .pillars li:last-child {
    border-bottom: none;
  }

  .pillars li.optional .name {
    color: var(--ink-secondary);
  }

  .name {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .opt {
    font-size: 11px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--ink-muted);
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    padding: 0 5px;
  }

  .detail {
    font-size: 13px;
  }

  .side {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    padding: 16px 20px;
  }

  .actions button {
    justify-content: space-between;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 38px;
    font-size: 13px;
    text-align: left;
  }

  kbd {
    font-family: inherit;
    font-size: 10px;
    color: var(--ink-muted);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 0 4px;
    line-height: 15px;
  }

  .closer {
    padding: 0 20px 16px;
  }

  .wide {
    width: 100%;
    height: 40px;
  }

  .toast {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--ink-primary);
    color: var(--bg);
    border-radius: var(--radius-control);
    padding: 8px 16px;
    font-size: 13px;
    z-index: 60;
  }
</style>
