<script lang="ts">
  import { untrack } from "svelte";

  import * as api from "../lib/api";
  import type { Bootstrap, Tone } from "../lib/types";

  let { boot, onreload }: { boot: Bootstrap; onreload: () => Promise<void> } = $props();

  type Section = "reto" | "coach" | "metas" | "recordatorios" | "sistema";
  let section = $state<Section>("coach");

  let settings = $state(untrack(() => ({ ...boot.settings })));
  let autostart = $state(untrack(() => boot.settings.autostart === "1"));
  let error = $state("");
  let saved = $state("");

  api
    .isAutostartEnabled()
    .then((v) => (autostart = v))
    .catch(() => {});

  function flash(msg: string) {
    saved = msg;
    setTimeout(() => (saved = ""), 2000);
  }

  async function save(entries: Record<string, string>) {
    error = "";
    try {
      settings = await api.setSettings(entries);
      flash("Guardado");
      await onreload();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function toggleAutostart() {
    error = "";
    try {
      autostart = await api.setAutostart(!autostart);
      flash(autostart ? "75 HARD arrancará con Windows" : "Autoarranque desactivado");
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  const tones: { value: Tone; label: string; example: string }[] = [
    { value: "suave", label: "Suave", example: "5h 20min. Es poco; intenta acostarte antes hoy." },
    { value: "directo", label: "Directo", example: "5h 20min. Hoy vas a rendir menos y lo sabes." },
    {
      value: "duro",
      label: "Duro",
      example: "5h 20min. El día empieza en déficit y eso se decidió anoche.",
    },
  ];

  const metas: { key: string; label: string; hint: string; min: number; max: number; step?: number }[] = [
    { key: "sleep_goal_min", label: "Meta de sueño (minutos)", hint: "420 = 7 horas", min: 60, max: 960, step: 15 },
    { key: "water_goal_ml", label: "Meta de agua (ml)", hint: "3000 = 3 litros", min: 250, max: 10000, step: 250 },
    { key: "workout_goal_min", label: "Meta de ejercicio (minutos)", hint: "45 minutos", min: 5, max: 600, step: 5 },
    { key: "reading_goal_pages", label: "Meta de lectura (páginas)", hint: "10 páginas", min: 1, max: 1000 },
    { key: "day_cutoff_hour", label: "Hora de corte del día", hint: "4 = el día cambia a las 4:00 AM", min: 0, max: 12 },
  ];

  let confirmReset = $state(false);

  async function resetChallenge() {
    error = "";
    try {
      await api.endChallenge({ reason: "abandoned" });
      await onreload();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<div class="page">
  <h1 class="section-title">Ajustes</h1>

  <div class="layout">
    <nav class="subnav">
      {#each [["reto", "Reto"], ["coach", "Coach"], ["metas", "Metas"], ["recordatorios", "Recordatorios"], ["sistema", "Sistema"]] as [id, label] (id)}
        <button class="item" class:on={section === id} onclick={() => (section = id as Section)}>
          {label}
        </button>
      {/each}
    </nav>

    <div class="panel card">
      {#if error}<p class="error" role="alert">{error}</p>{/if}
      {#if saved}<p class="ok" role="status">{saved}</p>{/if}

      {#if section === "reto"}
        <h2 class="section-title">Reto</h2>
        {#if boot.challenge}
          <dl class="facts">
            <dt>Nombre</dt><dd>{boot.challenge.name}</dd>
            <dt>Inicio</dt><dd class="num">{boot.challenge.startDate}</dd>
            <dt>Duración</dt><dd class="num">{boot.challenge.targetDays} días</dd>
            <dt>Pilares obligatorios</dt>
            <dd>{boot.challenge.rules.pillars.filter((p) => p.required).length} de {boot.challenge.rules.pillars.length}</dd>
          </dl>

          <ul class="pillars">
            {#each boot.challenge.rules.pillars as p (p.key)}
              <li>
                <span class="grow">{p.label}</span>
                <span class="chip">{p.required ? "Obligatorio" : "Opcional"}</span>
              </li>
            {/each}
          </ul>
          <p class="hint">
            Los pilares se definen al crear el reto. Para cambiarlos hay que empezar un intento
            nuevo — así el historial de este intento sigue siendo comparable consigo mismo.
          </p>
        {:else}
          <p class="muted">No hay un reto activo.</p>
        {/if}

      {:else if section === "coach"}
        <h2 class="section-title">Tono del coach</h2>
        <p class="muted">
          Cambia cómo te habla la app. En los tres niveles el mensaje va al hábito, nunca a ti.
        </p>
        <div class="tones">
          {#each tones as t (t.value)}
            <button
              class="tone"
              class:on={settings.coach_tone === t.value}
              onclick={() => save({ coach_tone: t.value })}
            >
              <span class="name">{t.label}{settings.coach_tone === t.value ? " ✓" : ""}</span>
              <em>{t.example}</em>
            </button>
          {/each}
        </div>

      {:else if section === "metas"}
        <h2 class="section-title">Metas</h2>
        <ul class="rows">
          {#each metas as m (m.key)}
            <li>
              <div class="stack grow">
                <span>{m.label}</span>
                <span class="hint">{m.hint}</span>
              </div>
              <input
                class="narrow num"
                type="number"
                min={m.min}
                max={m.max}
                step={m.step ?? 1}
                value={settings[m.key]}
                onchange={(e) => save({ [m.key]: (e.currentTarget as HTMLInputElement).value })}
              />
            </li>
          {/each}
        </ul>

      {:else if section === "recordatorios"}
        <h2 class="section-title">Recordatorios</h2>
        <p class="muted">
          Las notificaciones programadas y el atajo global de captura rápida son la Fase 1.
          Todavía no están conectados, y prefiero decírtelo aquí antes que dejarte interruptores
          que no hacen nada.
        </p>
        <ul class="rows disabled">
          {#each [["Check-in matutino", "07:00"], ["Recordatorio de comida", "12:30"], ["Recordatorio de ejercicio", "17:00"], ["Agua cada 2 horas", "—"], ["Check-in nocturno", "21:30"]] as [name, time] (name)}
            <li>
              <span class="grow">{name}</span>
              <span class="num muted">{time}</span>
            </li>
          {/each}
        </ul>

      {:else}
        <h2 class="section-title">Sistema</h2>
        <ul class="rows">
          <li>
            <div class="stack grow">
              <span>Arrancar con Windows</span>
              <span class="hint">Se abre directo en la bandeja, sin ventana.</span>
            </div>
            <button class:primary={autostart} onclick={toggleAutostart}>
              {autostart ? "Activado" : "Desactivado"}
            </button>
          </li>
          <li>
            <div class="stack grow">
              <span>Ventana de captura rápida</span>
              <span class="hint">
                También está en el menú de la bandeja. El atajo global de teclado llega en la Fase 1.
              </span>
            </div>
            <button onclick={() => api.openQuickEntry()}>Abrir</button>
          </li>
          <li>
            <div class="stack grow">
              <span>Base de datos</span>
              <span class="hint num">{boot.dbPath}</span>
            </div>
          </li>
          <li>
            <div class="stack grow">
              <span>Privacidad</span>
              <span class="hint">
                Todo local. Sin cuentas, sin telemetría, sin red. Estos son datos médicos y no
                salen de esta máquina.
              </span>
            </div>
          </li>
        </ul>

        <div class="danger-zone">
          <span class="label">Zona de riesgo</span>
          {#if !confirmReset}
            <button class="danger" onclick={() => (confirmReset = true)}>Reiniciar el reto</button>
            <p class="hint">
              Cierra el intento actual y empieza uno nuevo. Ningún dato se borra: tus gráficas
              siguen siendo continuas.
            </p>
          {:else}
            <p>¿Seguro? Se cierra el intento actual y arrancas el #{boot.attemptNumber}.</p>
            <div class="row">
              <button onclick={() => (confirmReset = false)}>Cancelar</button>
              <button class="danger" onclick={resetChallenge}>Sí, reiniciar</button>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .page {
    max-width: 1000px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .layout {
    display: grid;
    grid-template-columns: 220px 1fr;
    gap: 16px;
    align-items: start;
  }

  .subnav {
    display: flex;
    flex-direction: column;
    background: var(--surface-1);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    overflow: hidden;
  }

  .item {
    height: 38px;
    border: none;
    border-radius: 0;
    border-bottom: 1px solid var(--border);
    background: transparent;
    text-align: left;
    color: var(--ink-secondary);
  }

  .item:last-child {
    border-bottom: none;
  }

  .item.on {
    background: var(--surface-sunken);
    color: var(--ink-primary);
    font-weight: 500;
  }

  .panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .facts {
    display: grid;
    grid-template-columns: 180px 1fr;
    gap: 8px 16px;
    margin: 0;
  }

  .facts dt {
    color: var(--ink-muted);
    font-size: 13px;
  }

  .facts dd {
    margin: 0;
  }

  .rows,
  .pillars {
    list-style: none;
    margin: 0;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
  }

  .rows li,
  .pillars li {
    display: flex;
    align-items: center;
    gap: 16px;
    min-height: 48px;
    padding: 8px 14px;
    border-bottom: 1px solid var(--border);
  }

  .rows li:last-child,
  .pillars li:last-child {
    border-bottom: none;
  }

  .rows.disabled {
    opacity: 0.55;
  }

  .grow {
    flex: 1;
  }

  .stack {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .narrow {
    width: 120px;
  }

  .tones {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
  }

  .tone {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    height: auto;
    padding: 14px;
    text-align: left;
    border-radius: var(--radius-card);
    white-space: normal;
  }

  .tone.on {
    border: 2px solid var(--accent);
    padding: 13px;
  }

  .tone .name {
    font-weight: 600;
  }

  .tone em {
    color: var(--ink-muted);
    font-size: 13px;
    line-height: 18px;
  }

  .danger-zone {
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    background: var(--surface-sunken);
    padding: 16px;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }

  .danger-zone p {
    margin: 0;
  }

  .error {
    margin: 0;
    color: var(--critical);
  }

  .ok {
    margin: 0;
    color: var(--good);
  }
</style>
