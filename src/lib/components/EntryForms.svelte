<script lang="ts">
  import * as api from "../api";
  import { glucoseContexts, nowClock, workoutKinds } from "../format";

  export type EntryTab = "comida" | "ejercicio" | "azucar" | "agua" | "lectura" | "trabajo";

  let {
    date = undefined as string | undefined,
    tabs = ["comida", "ejercicio", "azucar", "agua"] as EntryTab[],
    tab = $bindable("comida" as EntryTab),
    compact = false,
    onsaved,
  }: {
    date?: string;
    tabs?: EntryTab[];
    tab?: EntryTab;
    compact?: boolean;
    onsaved: (mensaje: string) => void;
  } = $props();

  const titles: Record<EntryTab, string> = {
    comida: "Comida",
    ejercicio: "Ejercicio",
    azucar: "Azúcar",
    agua: "Agua",
    lectura: "Lectura",
    trabajo: "Trabajo",
  };

  let time = $state(nowClock());
  let error = $state("");
  let saving = $state(false);

  // Comida
  let mealKind = $state<"meal" | "snack">("meal");
  let mealText = $state("");
  let mealCalories = $state("");
  let showCalories = $state(false);

  // Ejercicio
  let workoutKind = $state("gym");
  let workoutMin = $state("");
  let workoutDesc = $state("");
  let workoutOutdoor = $state(false);
  let workoutKcal = $state("");

  // Azúcar
  let glucoseValue = $state("");
  let glucoseContext = $state("fasting");
  let glucoseNotes = $state("");

  // Agua
  const waterPresets = [250, 500, 750, 1000];
  let waterMl = $state(500);

  // Lectura
  let pages = $state("");
  let book = $state("");

  // Trabajo
  let workMin = $state("");
  let workCategory = $state("Universidad");
  let workDesc = $state("");

  const optional = (v: string): number | undefined => {
    const t = v.trim();
    if (!t) return undefined;
    const n = Number(t);
    return Number.isFinite(n) ? n : undefined;
  };

  async function save() {
    if (saving) return;
    saving = true;
    error = "";
    try {
      switch (tab) {
        case "comida":
          await api.addMeal({
            date,
            time,
            kind: mealKind,
            description: mealText,
            calories: optional(mealCalories),
          });
          onsaved(mealKind === "meal" ? "Comida registrada" : "Snack registrado");
          mealText = "";
          mealCalories = "";
          break;

        case "ejercicio":
          await api.addWorkout({
            date,
            time,
            kind: workoutKind,
            description: workoutDesc.trim() || undefined,
            durationMin: Number(workoutMin),
            isOutdoor: workoutOutdoor,
            caloriesBurned: optional(workoutKcal),
          });
          onsaved(`Ejercicio registrado: ${workoutMin} min`);
          workoutMin = "";
          workoutDesc = "";
          workoutKcal = "";
          break;

        case "azucar":
          await api.addGlucose({
            date,
            time,
            valueMgdl: Number(glucoseValue),
            context: glucoseContext,
            notes: glucoseNotes.trim() || undefined,
          });
          onsaved(`Lectura registrada: ${glucoseValue} mg/dL`);
          glucoseValue = "";
          glucoseNotes = "";
          break;

        case "agua": {
          const total = await api.addWater(waterMl, date);
          onsaved(`Agua: ${(total / 1000).toFixed(1)} L en total hoy`);
          break;
        }

        case "lectura":
          await api.addReading({ date, pages: Number(pages), book: book.trim() || undefined });
          onsaved(`${pages} páginas registradas`);
          pages = "";
          break;

        case "trabajo":
          await api.addWorkSession({
            date,
            time,
            minutes: Number(workMin),
            category: workCategory,
            description: workDesc.trim() || undefined,
          });
          onsaved(`Sesión de trabajo: ${workMin} min`);
          workMin = "";
          workDesc = "";
          break;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  // Un <form> ya envía con Enter desde cualquier campo de una línea; en el área
  // de texto hace falta Ctrl+Enter. Tab, escribir, Enter.
  function onTextareaKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && e.ctrlKey) {
      e.preventDefault();
      save();
    }
  }

  function onsubmit(e: SubmitEvent) {
    e.preventDefault();
    save();
  }
</script>

<form class="forms" class:compact {onsubmit}>
  {#if tabs.length > 1}
    <div class="tabs" role="tablist">
      {#each tabs as t (t)}
        <button
          type="button"
          role="tab"
          class="tab"
          class:on={tab === t}
          aria-selected={tab === t}
          onclick={() => {
            tab = t;
            error = "";
          }}
        >
          {titles[t]}
        </button>
      {/each}
    </div>
  {/if}

  <div class="fields">
    {#if tab !== "agua" && tab !== "lectura"}
      <div class="timerow">
        <label class="label" for="hora">Hora</label>
        <input id="hora" class="time" type="time" bind:value={time} />
      </div>
    {/if}

    {#if tab === "comida"}
      <div class="row">
        <button type="button" class="chip" class:on={mealKind === "meal"} onclick={() => (mealKind = "meal")}>
          Comida
        </button>
        <button type="button" class="chip" class:on={mealKind === "snack"} onclick={() => (mealKind = "snack")}>
          Snack
        </button>
      </div>
      <!-- svelte-ignore a11y_autofocus -->
      <textarea
        autofocus
        rows={compact ? 5 : 3}
        placeholder="¿Qué comiste?"
        bind:value={mealText}
        onkeydown={onTextareaKeydown}
      ></textarea>
      {#if showCalories}
        <div class="inline">
          <label class="label" for="kcal">Calorías (opcional)</label>
          <input id="kcal" class="narrow" type="number" min="0" bind:value={mealCalories} />
        </div>
      {:else}
        <button type="button" class="ghost addcal" onclick={() => (showCalories = true)}>
          + Agregar calorías (opcional)
        </button>
      {/if}

    {:else if tab === "ejercicio"}
      <div class="pair">
        <div class="stack">
          <label class="label" for="wmin">Minutos</label>
          <!-- svelte-ignore a11y_autofocus -->
          <input id="wmin" autofocus type="number" min="1" bind:value={workoutMin} />
        </div>
        <div class="stack">
          <label class="label" for="wkind">Tipo</label>
          <select id="wkind" bind:value={workoutKind}>
            {#each workoutKinds as k (k.value)}
              <option value={k.value}>{k.label}</option>
            {/each}
          </select>
        </div>
      </div>
      <input type="text" placeholder="Descripción (opcional)" bind:value={workoutDesc} />
      <label class="check">
        <input type="checkbox" bind:checked={workoutOutdoor} />
        Al aire libre
      </label>
      <div class="inline">
        <label class="label" for="wkcal">Calorías quemadas (opcional)</label>
        <input id="wkcal" class="narrow" type="number" min="0" bind:value={workoutKcal} />
      </div>

    {:else if tab === "azucar"}
      <div class="pair">
        <div class="stack">
          <label class="label" for="gval">mg/dL</label>
          <!-- svelte-ignore a11y_autofocus -->
          <input id="gval" autofocus type="number" min="20" max="600" bind:value={glucoseValue} />
        </div>
        <div class="stack">
          <label class="label" for="gctx">Contexto</label>
          <select id="gctx" bind:value={glucoseContext}>
            {#each glucoseContexts as c (c.value)}
              <option value={c.value}>{c.label}</option>
            {/each}
          </select>
        </div>
      </div>
      <input type="text" placeholder="Notas (opcional)" bind:value={glucoseNotes} />
      <p class="hint">
        La app registra y grafica. No diagnostica ni sugiere tratamiento.
      </p>

    {:else if tab === "agua"}
      <span class="label">¿Cuánta?</span>
      <div class="row wrap">
        {#each waterPresets as ml (ml)}
          <button type="button" class="chip" class:on={waterMl === ml} onclick={() => (waterMl = ml)}>
            {ml} ml
          </button>
        {/each}
      </div>
      <div class="inline">
        <label class="label" for="wml">Otra cantidad</label>
        <input id="wml" class="narrow" type="number" min="1" bind:value={waterMl} />
      </div>

    {:else if tab === "lectura"}
      <div class="pair">
        <div class="stack">
          <label class="label" for="pag">Páginas</label>
          <!-- svelte-ignore a11y_autofocus -->
          <input id="pag" autofocus type="number" min="1" bind:value={pages} />
        </div>
        <div class="stack">
          <label class="label" for="libro">Libro (opcional)</label>
          <input id="libro" type="text" bind:value={book} />
        </div>
      </div>

    {:else if tab === "trabajo"}
      <div class="pair">
        <div class="stack">
          <label class="label" for="tmin">Minutos</label>
          <!-- svelte-ignore a11y_autofocus -->
          <input id="tmin" autofocus type="number" min="1" bind:value={workMin} />
        </div>
        <div class="stack">
          <label class="label" for="tcat">Categoría</label>
          <select id="tcat" bind:value={workCategory}>
            <option>Universidad</option>
            <option>Proyectos</option>
            <option>Trabajo</option>
          </select>
        </div>
      </div>
      <input type="text" placeholder="Descripción (opcional)" bind:value={workDesc} />
    {/if}

    {#if error}
      <p class="error" role="alert">{error}</p>
    {/if}
  </div>

  <div class="actions">
    <span class="hint">{tab === "comida" ? "Ctrl+Enter" : "Enter"} para guardar</span>
    <button type="submit" class="primary" disabled={saving}>Guardar</button>
  </div>
</form>

<style>
  .forms {
    display: flex;
    flex-direction: column;
    min-height: 0;
    flex: 1;
  }

  .tabs {
    display: flex;
    border-bottom: 1px solid var(--border);
  }

  .tab {
    flex: 1;
    height: 36px;
    border: none;
    border-radius: 0;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--ink-secondary);
    font-size: 13px;
    padding: 0 8px;
  }

  .tab.on {
    color: var(--ink-primary);
    border-bottom-color: var(--accent);
    font-weight: 500;
  }

  .fields {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px 0;
    flex: 1;
    min-height: 0;
  }

  .forms.compact .fields {
    padding: 12px;
  }

  .forms.compact .actions {
    padding: 0 12px 12px;
  }

  .timerow {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
  }

  .time {
    width: 132px;
  }

  .pair {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .stack {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .inline {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .narrow {
    width: 110px;
  }

  .check {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--ink-secondary);
  }

  .check input {
    width: auto;
  }

  .addcal {
    align-self: flex-start;
    height: 28px;
    padding: 0 8px;
    font-size: 13px;
  }

  .row.wrap {
    flex-wrap: wrap;
  }

  .row .chip {
    height: 30px;
    cursor: pointer;
  }

  .error {
    margin: 0;
    color: var(--critical);
    font-size: 13px;
  }

  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    border-top: 1px solid var(--border);
    padding-top: 12px;
  }
</style>
