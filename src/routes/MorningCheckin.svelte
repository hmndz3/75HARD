<script lang="ts">
  import { untrack } from "svelte";

  import Modal from "../lib/components/Modal.svelte";
  import Scale15 from "../lib/components/Scale15.svelte";
  import StatusIcon from "../lib/components/StatusIcon.svelte";
  import * as api from "../lib/api";
  import { numeroOpcional, severityClass } from "../lib/format";
  import type { SleepPreview, TodayView } from "../lib/types";

  let {
    view,
    onclose,
    onsaved,
  }: { view: TodayView; onclose: () => void; onsaved: (v: TodayView) => void } = $props();

  // Valores iniciales: se leen una sola vez al abrir el modal.
  let bedtime = $state(untrack(() => view.sleep?.bedtime.slice(11, 16) ?? "23:30"));
  let wakeTime = $state(untrack(() => view.sleep?.wakeTime.slice(11, 16) ?? "06:30"));
  let weight = $state<string | number | null>(
    untrack(() => (view.weightKg !== null ? view.weightKg : ""))
  );
  let mood = $state<number | null>(untrack(() => view.mood?.mood ?? null));
  let energy = $state<number | null>(untrack(() => view.mood?.energy ?? null));

  let preview = $state<SleepPreview | null>(null);
  let error = $state("");
  let saving = $state(false);

  // Veredicto en vivo: se recalcula en cuanto ambas horas son válidas.
  $effect(() => {
    const b = bedtime;
    const w = wakeTime;
    if (!/^\d{2}:\d{2}$/.test(b) || !/^\d{2}:\d{2}$/.test(w)) {
      preview = null;
      return;
    }
    let cancelled = false;
    api
      .previewSleep({ date: view.date, bedtime: b, wakeTime: w })
      .then((p) => {
        if (!cancelled) preview = p;
      })
      .catch(() => {
        if (!cancelled) preview = null;
      });
    return () => {
      cancelled = true;
    };
  });

  async function save() {
    if (saving) return;
    saving = true;
    error = "";
    try {
      const updated = await api.saveMorningCheckin({
        date: view.date,
        bedtime,
        wakeTime,
        weightKg: numeroOpcional(weight),
        mood: mood ?? undefined,
        energy: energy ?? undefined,
      });
      onsaved(updated);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  function onsubmit(e: SubmitEvent) {
    e.preventDefault();
    save();
  }
</script>

<Modal
  width={560}
  title="Buenos días"
  subtitle="{view.weekdayLabel}{view.dayNumber ? ` · Día ${view.dayNumber}` : ''}"
  {onclose}
>
  <form class="body" {onsubmit}>
    <div class="times">
      <div class="stack">
        <label class="label" for="bed">¿A qué hora te dormiste?</label>
        <input id="bed" class="big" type="time" bind:value={bedtime} />
      </div>
      <div class="stack">
        <label class="label" for="wake">¿A qué hora despertaste?</label>
        <input id="wake" class="big" type="time" bind:value={wakeTime} />
      </div>
    </div>

    <div class="verdict sunken">
      {#if preview}
        <span class="card-value">{preview.label}</span>
        <div class="row">
          <StatusIcon severity={preview.verdict.severity} size={16} title="" />
          <span class={severityClass[preview.verdict.severity]}>{preview.verdict.text}</span>
        </div>
      {:else}
        <span class="card-value dim">—</span>
        <span class="muted">Escribe las dos horas para ver el cálculo.</span>
      {/if}
    </div>

    <div class="stack narrow">
      <label class="label" for="peso">Peso (opcional)</label>
      <div class="suffix">
        <input id="peso" type="number" step="0.1" min="20" max="400" bind:value={weight} />
        <span class="dim">kg</span>
      </div>
    </div>

    <div class="stack scales">
      <span class="label">¿Cómo amaneciste?</span>
      <Scale15 label="Ánimo" bind:value={mood} />
      <Scale15 label="Energía" bind:value={energy} />
    </div>

    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <button type="submit" hidden aria-hidden="true"></button>
  </form>

  {#snippet footer()}
    <div class="spread">
      <span class="hint">Enter para guardar</span>
      <div class="row">
        <button class="ghost" onclick={onclose}>Después</button>
        <button class="primary" onclick={save} disabled={saving}>Guardar</button>
      </div>
    </div>
  {/snippet}
</Modal>

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .times {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .stack {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .big {
    height: 48px;
    font-size: 24px;
    font-variant-numeric: tabular-nums;
  }

  .verdict {
    display: flex;
    align-items: center;
    gap: 20px;
    white-space: nowrap;
    padding: 16px 20px;
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
  }

  .verdict :global(.card-value) {
    flex: none;
  }

  .verdict .row {
    white-space: normal;
  }

  .narrow {
    max-width: 160px;
  }

  .suffix {
    position: relative;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .suffix span {
    position: absolute;
    right: 10px;
    pointer-events: none;
  }

  .scales {
    gap: 12px;
  }

  .error {
    margin: 0;
    color: var(--critical);
  }
</style>
