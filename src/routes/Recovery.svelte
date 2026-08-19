<script lang="ts">
  import { untrack } from "svelte";

  import Modal from "../lib/components/Modal.svelte";
  import * as api from "../lib/api";
  import { shortDate } from "../lib/format";
  import type { MissingAction, MissingDay } from "../lib/types";

  let {
    days,
    onclose,
    ondone,
  }: {
    days: MissingDay[];
    onclose: () => void;
    ondone: (aLlenar: string[]) => Promise<void>;
  } = $props();

  const acciones: { value: MissingAction; label: string }[] = [
    { value: "fill", label: "Llenar ahora" },
    { value: "failed", label: "Marcar fallido" },
    { value: "empty", label: "Dejar vacío" },
  ];

  let choices = $state<Record<string, MissingAction>>(
    untrack(() => Object.fromEntries(days.map((d) => [d.date, "empty" as MissingAction])))
  );
  let error = $state("");
  let saving = $state(false);

  const rango = $derived(
    days.length === 1
      ? `El ${shortDate(days[0].date)}.`
      : `Del ${shortDate(days[0].date)} al ${shortDate(days[days.length - 1].date)}.`
  );

  function todos(action: MissingAction) {
    for (const d of days) choices[d.date] = action;
  }

  async function apply() {
    if (saving) return;
    saving = true;
    error = "";
    try {
      const aLlenar = await api.applyMissingDays(
        days.map((d) => ({ date: d.date, action: choices[d.date] }))
      );
      await ondone(aLlenar);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      saving = false;
    }
  }
</script>

<Modal
  width={600}
  title="Tienes {days.length} {days.length === 1 ? 'día' : 'días'} sin registrar"
  subtitle="{rango} Decide qué hacer con cada uno."
  {onclose}
>
  <div class="body">
    <div class="bulk">
      <span class="hint">Para todos:</span>
      {#each acciones as a (a.value)}
        <button class="ghost small" onclick={() => todos(a.value)}>{a.label}</button>
      {/each}
    </div>

    <ul class="rows">
      {#each days as d (d.date)}
        <li>
          <div class="stack grow">
            <span class="name">{d.weekdayLabel}</span>
            <span class="hint">{d.dayNumber ? `Día ${d.dayNumber}` : "Fuera del reto"}</span>
          </div>
          <div class="segmented">
            {#each acciones as a (a.value)}
              <button
                class="seg"
                class:on={choices[d.date] === a.value}
                onclick={() => (choices[d.date] = a.value)}
              >
                {a.label}
              </button>
            {/each}
          </div>
        </li>
      {/each}
    </ul>

    <p class="note">
      Los días vacíos no cuentan para tu racha ni la rompen.
    </p>

    {#if error}<p class="error" role="alert">{error}</p>{/if}
  </div>

  {#snippet footer()}
    <div class="spread">
      <button class="ghost" onclick={onclose}>Decidir después</button>
      <button class="primary" onclick={apply} disabled={saving}>Aplicar</button>
    </div>
  {/snippet}
</Modal>

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .bulk {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  button.small {
    height: 26px;
    font-size: 12px;
    padding: 0 8px;
  }

  .rows {
    list-style: none;
    margin: 0;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    overflow: hidden;
    max-height: 320px;
    overflow-y: auto;
  }

  .rows li {
    display: flex;
    align-items: center;
    gap: 16px;
    min-height: 52px;
    padding: 6px 14px;
    border-bottom: 1px solid var(--border);
  }

  .rows li:last-child {
    border-bottom: none;
  }

  .rows li:nth-child(even) {
    background: var(--surface-1);
  }

  .stack {
    display: flex;
    flex-direction: column;
  }

  .grow {
    flex: 1;
  }

  .name {
    font-weight: 500;
  }

  .segmented {
    display: flex;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-control);
    overflow: hidden;
    flex: none;
  }

  .seg {
    height: 30px;
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

  .note {
    margin: 0;
    padding: 10px 14px;
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    font-size: 13px;
    color: var(--ink-secondary);
  }

  .error {
    margin: 0;
    color: var(--critical);
  }
</style>
