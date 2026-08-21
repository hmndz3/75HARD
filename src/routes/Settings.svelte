<script lang="ts">
  import { untrack } from "svelte";
  // `save` ya existe en este componente para guardar ajustes; se renombra el
  // del diálogo para que no choquen.
  import { open as abrirArchivo, save as guardarComo } from "@tauri-apps/plugin-dialog";

  import * as api from "../lib/api";
  import Reminders from "./settings/Reminders.svelte";
  import { cambiarTema, tema, type Tema } from "../lib/theme.svelte";
  import { todayIso } from "../lib/format";
  import type { BackupFile, Bootstrap, Tone } from "../lib/types";

  let {
    boot,
    onreload,
    oncompletion,
  }: { boot: Bootstrap; onreload: () => Promise<void>; oncompletion: () => void } = $props();

  type Section =
    | "reto"
    | "coach"
    | "metas"
    | "recordatorios"
    | "apariencia"
    | "datos"
    | "sistema";
  let section = $state<Section>("coach");

  let settings = $state(untrack(() => ({ ...boot.settings })));
  let autostart = $state(untrack(() => boot.settings.autostart === "1"));
  let error = $state("");
  let saved = $state("");

  api
    .isAutostartEnabled()
    .then((v) => (autostart = v))
    .catch(() => {});

  // --- Datos: exportación y copias de seguridad
  let copias = $state.raw<BackupFile[]>([]);
  let frase = $state("");
  let ocupado = $state("");

  const recargarCopias = () =>
    api
      .listBackups()
      .then((c) => (copias = c))
      .catch(() => {});
  recargarCopias();

  async function conAviso(que: string, fn: () => Promise<string>) {
    ocupado = que;
    error = "";
    try {
      const msg = await fn();
      if (msg) flash(msg);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      ocupado = "";
      await recargarCopias();
    }
  }

  const exportar = (formato: "csv" | "json") =>
    conAviso(formato, async () => {
      const destino = await guardarComo({
        defaultPath: `75hard-${todayIso()}.${formato}`,
        filters: [{ name: formato.toUpperCase(), extensions: [formato] }],
      });
      if (!destino) return "";
      return `Exportado a ${await api.exportData(formato, destino)}`;
    });

  const instantanea = () =>
    conAviso("snapshot", async () => `Copia local creada: ${await api.backupNow()}`);

  const copiaCifrada = () =>
    conAviso("cifrar", async () => {
      const destino = await guardarComo({
        defaultPath: `75hard-${todayIso()}.75bak`,
        filters: [{ name: "Copia cifrada", extensions: ["75bak"] }],
      });
      if (!destino) return "";
      await api.createEncryptedBackup(destino, frase);
      frase = "";
      return `Copia cifrada guardada en ${destino}`;
    });

  const restaurar = () =>
    conAviso("restaurar", async () => {
      const origen = await abrirArchivo({
        multiple: false,
        filters: [{ name: "Copia cifrada", extensions: ["75bak"] }],
      });
      if (typeof origen !== "string") return "";
      const respaldo = await api.restoreEncryptedBackup(origen, frase);
      frase = "";
      await onreload();
      return `Restaurado. Lo anterior quedó guardado en ${respaldo}`;
    });

  const temas: { value: Tema; label: string; detalle: string }[] = [
    { value: "claro", label: "Claro", detalle: "Grises cálidos sobre fondo claro." },
    { value: "oscuro", label: "Oscuro", detalle: "Los mismos tokens, invertidos." },
    { value: "sistema", label: "Como Windows", detalle: "Sigue el modo del sistema." },
  ];

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
      {#each [["reto", "Reto"], ["coach", "Coach"], ["metas", "Metas"], ["recordatorios", "Recordatorios"], ["apariencia", "Apariencia"], ["datos", "Datos"], ["sistema", "Sistema"]] as [id, label] (id)}
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
          <button class="alinear" onclick={oncompletion}>Ver resumen del reto</button>
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
        <Reminders {settings} onsetting={save} />

      {:else if section === "apariencia"}
        <h2 class="section-title">Apariencia</h2>
        <p class="muted">
          Los dos temas usan la misma paleta: el color sigue significando lo mismo en ambos.
        </p>
        <div class="tones">
          {#each temas as t (t.value)}
            <button
              class="tone"
              class:on={tema.elegido === t.value}
              onclick={() => cambiarTema(t.value)}
            >
              <span class="name">{t.label}{tema.elegido === t.value ? " ✓" : ""}</span>
              <em>{t.detalle}</em>
            </button>
          {/each}
        </div>

      {:else if section === "datos"}
        <h2 class="section-title">Datos</h2>

        <ul class="rows">
          <li>
            <div class="stack grow">
              <span>Exportar todo</span>
              <span class="hint">
                Historial completo, sin recortes. CSV para abrirlo en una hoja de cálculo, JSON
                para procesarlo.
              </span>
            </div>
            <div class="row">
              <button onclick={() => exportar("csv")} disabled={ocupado === "csv"}>CSV</button>
              <button onclick={() => exportar("json")} disabled={ocupado === "json"}>JSON</button>
            </div>
          </li>
          <li>
            <div class="stack grow">
              <span>Copia local</span>
              <span class="hint">
                Instantánea junto a la base, sin cifrar. Se conservan las últimas siete.
              </span>
            </div>
            <button onclick={instantanea} disabled={ocupado === "snapshot"}>Crear ahora</button>
          </li>
        </ul>

        <div class="cifrado">
          <span class="label">Copia cifrada</span>
          <p class="hint">
            Para sacar los datos de esta máquina. La frase no se guarda en ningún sitio: si la
            pierdes, la copia no se puede recuperar. Son datos médicos, y por eso va cifrada.
          </p>
          <div class="row">
            <input
              type="password"
              placeholder="Frase de cifrado (mínimo 8 caracteres)"
              bind:value={frase}
              autocomplete="off"
            />
            <button onclick={copiaCifrada} disabled={frase.length < 8 || ocupado === "cifrar"}>
              Crear copia
            </button>
            <button onclick={restaurar} disabled={frase.length < 8 || ocupado === "restaurar"}>
              Restaurar
            </button>
          </div>
          <p class="hint">
            Restaurar sustituye todo lo que hay ahora. Antes de hacerlo se guarda una copia local
            de lo actual, por si te arrepientes.
          </p>
        </div>

        {#if copias.length > 0}
          <ul class="rows">
            {#each copias as c (c.name)}
              <li>
                <div class="stack grow">
                  <span class="num">{c.name}</span>
                  <span class="hint num">{c.sizeKb} KB</span>
                </div>
              </li>
            {/each}
          </ul>
        {/if}

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
              <span>Atajo global</span>
              <span class="hint">
                {#if boot.settings.hotkey_error}
                  <span class="s-critical">
                    No se pudo registrar: {boot.settings.hotkey_error}. Probablemente otra app ya
                    lo tiene tomado.
                  </span>
                {:else}
                  Abre la captura rápida desde cualquier lado, sin cambiar de ventana.
                {/if}
              </span>
            </div>
            <kbd class="accel">{boot.settings.hotkey_quick ?? "Ctrl+Alt+H"}</kbd>
          </li>
          <li>
            <div class="stack grow">
              <span>Ventana de captura rápida</span>
              <span class="hint">
                También está en el menú de la bandeja y en el atajo global de aquí arriba.
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






  .accel {
    font-family: inherit;
    font-size: 12px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-control);
    padding: 4px 8px;
    color: var(--ink-secondary);
    background: var(--surface-sunken);
    white-space: nowrap;
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

  .cifrado {
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .cifrado .row {
    gap: 8px;
  }

  .cifrado input {
    max-width: 320px;
  }

  .cifrado p {
    margin: 0;
  }

  .alinear {
    align-self: flex-start;
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
