<script lang="ts">
  import * as api from "../../lib/api";
  import type { Reminder, Repeat } from "../../lib/types";

  let {
    settings,
    onsetting,
  }: {
    settings: Record<string, string>;
    onsetting: (entries: Record<string, string>) => Promise<void>;
  } = $props();

  const DIAS = [
    { bit: 0, corto: "L", largo: "lunes" },
    { bit: 1, corto: "M", largo: "martes" },
    { bit: 2, corto: "X", largo: "miércoles" },
    { bit: 3, corto: "J", largo: "jueves" },
    { bit: 4, corto: "V", largo: "viernes" },
    { bit: 5, corto: "S", largo: "sábado" },
    { bit: 6, corto: "D", largo: "domingo" },
  ];
  const TODOS = 127;

  const repeticiones: [Repeat, string][] = [
    ["daily", "Todos los días"],
    ["alternate", "Día de por medio"],
    ["weekdays", "Elegir días"],
  ];

  let lista = $state.raw<Reminder[]>([]);
  let error = $state("");
  let aviso = $state("");
  let ocupado = $state("");

  // Formulario de recordatorio propio
  let creando = $state(false);
  let nuevoNombre = $state("");
  let nuevoMensaje = $state("");
  let nuevaHora = $state("09:00");
  let nuevaRepeticion = $state<Repeat>("daily");
  let nuevosDias = $state(TODOS);

  api
    .getReminders()
    .then((r) => (lista = r))
    .catch((e) => (error = e instanceof Error ? e.message : String(e)));

  function flash(msg: string) {
    aviso = msg;
    setTimeout(() => (aviso = ""), 2500);
  }

  async function actuar(que: string, fn: () => Promise<Reminder[]>) {
    ocupado = que;
    error = "";
    try {
      lista = await fn();
      flash("Guardado");
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      ocupado = "";
    }
  }

  type Cambio = Omit<Parameters<typeof api.setReminder>[0], "id">;

  const editar = (id: string, cambio: Cambio) =>
    actuar(id, () => api.setReminder({ ...cambio, id }));

  /**
   * Traduce la elección de la UI a lo que guarda la base. "Todos los días" y
   * "elegir días" son una máscara; "día de por medio" es un intervalo, que una
   * máscara semanal no puede expresar porque no cae siempre en los mismos días.
   */
  function cambiarRepeticion(r: Reminder, repeat: Repeat) {
    modo = { ...modo, [r.id]: repeat };
    if (repeat === "daily") return editar(r.id, { daysMask: TODOS, intervalDays: 0 });
    if (repeat === "alternate") return editar(r.id, { daysMask: TODOS, intervalDays: 2 });
    // Al pasar a "elegir" se arranca de lo que ya había, para no perder nada.
    const mascara = r.daysMask & TODOS ? r.daysMask : TODOS;
    return editar(r.id, { daysMask: mascara, intervalDays: 0 });
  }

  function alternarDia(r: Reminder, bit: number) {
    const nueva = r.daysMask ^ (1 << bit);
    if ((nueva & TODOS) === 0) {
      error = "Deja al menos un día marcado.";
      return;
    }
    editar(r.id, { daysMask: nueva, intervalDays: 0 });
  }

  async function crear() {
    ocupado = "nuevo";
    error = "";
    try {
      lista = await api.addReminder({
        title: nuevoNombre,
        message: nuevoMensaje.trim() || undefined,
        timeOfDay: nuevaHora,
        daysMask: nuevaRepeticion === "weekdays" ? nuevosDias : TODOS,
        intervalDays: nuevaRepeticion === "alternate" ? 2 : 0,
      });
      nuevoNombre = "";
      nuevoMensaje = "";
      nuevaHora = "09:00";
      nuevaRepeticion = "daily";
      nuevosDias = TODOS;
      creando = false;
      flash("Recordatorio creado");
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      ocupado = "";
    }
  }

  let confirmar = $state<string | null>(null);

  /**
   * "Elegir días" con los siete marcados guarda exactamente lo mismo que
   * "todos los días", así que el modo no se puede deducir de lo guardado: al
   * pulsarlo rebotaba al primero. Esto recuerda la elección mientras la
   * pantalla está abierta; al volver se muestra lo que de verdad hace.
   */
  let modo = $state<Record<string, Repeat>>({});
  const repeatDe = (r: Reminder): Repeat => modo[r.id] ?? r.repeat;

  const borrar = (id: string) =>
    actuar(id, async () => {
      const r = await api.deleteReminder(id);
      confirmar = null;
      return r;
    });

  const notificacionesOn = $derived(settings.notifications !== "0");

  /** "Lunes, miércoles y viernes" a partir de la máscara. */
  function resumenDias(r: Reminder): string {
    if (r.intervalDays > 1) return `Cada ${r.intervalDays} días`;
    if ((r.daysMask & TODOS) === TODOS) return "Todos los días";
    const activos = DIAS.filter((d) => r.daysMask & (1 << d.bit)).map((d) => d.largo);
    if (activos.length === 0) return "Ningún día";
    if (activos.length === 1) return activos[0][0].toUpperCase() + activos[0].slice(1);
    const texto = `${activos.slice(0, -1).join(", ")} y ${activos.at(-1)}`;
    return texto[0].toUpperCase() + texto.slice(1);
  }
</script>

<h2 class="section-title">Recordatorios</h2>
<p class="muted">
  La app te busca a ti. Antes de avisar mira si el pilar ya está cubierto: si registraste el
  ejercicio, el recordatorio de la tarde no suena.
</p>

{#if error}<p class="error" role="alert">{error}</p>{/if}
{#if aviso}<p class="ok" role="status">{aviso}</p>{/if}

<ul class="rows">
  <li>
    <div class="stack grow">
      <span>Notificaciones</span>
      <span class="hint">Interruptor general. Apagado, no suena nada.</span>
    </div>
    <button
      class:primary={notificacionesOn}
      onclick={() => onsetting({ notifications: notificacionesOn ? "0" : "1" })}
    >
      {notificacionesOn ? "Activadas" : "Desactivadas"}
    </button>
  </li>
</ul>

<div class="lista" class:apagada={!notificacionesOn}>
  {#each lista as r (r.id)}
    <div class="recordatorio">
      <div class="cabecera">
        <button
          class="toggle"
          class:on={r.enabled}
          aria-pressed={r.enabled}
          aria-label="{r.enabled ? 'Desactivar' : 'Activar'} {r.label}"
          onclick={() => editar(r.id, { enabled: !r.enabled })}
        >
          <span class="knob"></span>
        </button>

        <div class="stack grow">
          <span class="nombre">
            {r.label}
            {#if r.custom}<span class="etiqueta">tuyo</span>{/if}
          </span>
          <span class="hint">{r.description}</span>
        </div>

        {#if r.intervalBased}
          <span class="num muted">cada {settings.water_every_hours ?? "2"} h</span>
        {:else}
          <input
            class="hora num"
            type="time"
            value={r.timeOfDay}
            disabled={!r.enabled}
            onchange={(e) =>
              editar(r.id, { timeOfDay: (e.currentTarget as HTMLInputElement).value })}
          />
        {/if}

        {#if r.custom}
          {#if confirmar === r.id}
            <button class="ghost chico" onclick={() => (confirmar = null)}>Cancelar</button>
            <button class="danger chico" onclick={() => borrar(r.id)}>Sí, borrar</button>
          {:else}
            <button class="ghost chico" onclick={() => (confirmar = r.id)}>Borrar</button>
          {/if}
        {/if}
      </div>

      {#if !r.intervalBased}
        <div class="repeticion">
          <div class="segmented" role="group" aria-label="Días de {r.label}">
            {#each repeticiones as [valor, texto] (valor)}
              <button
                class="seg"
                class:on={repeatDe(r) === valor}
                disabled={!r.enabled || ocupado === r.id}
                onclick={() => cambiarRepeticion(r, valor)}
              >
                {texto}
              </button>
            {/each}
          </div>

          {#if repeatDe(r) === "weekdays"}
            <div class="dias" role="group" aria-label="Elegir días">
              {#each DIAS as d (d.bit)}
                <button
                  class="dia"
                  class:on={(r.daysMask & (1 << d.bit)) !== 0}
                  aria-pressed={(r.daysMask & (1 << d.bit)) !== 0}
                  aria-label={d.largo}
                  disabled={!r.enabled}
                  onclick={() => alternarDia(r, d.bit)}
                >
                  {d.corto}
                </button>
              {/each}
            </div>
          {:else}
            <span class="hint resumen">{resumenDias(r)}</span>
          {/if}
        </div>
      {/if}
    </div>
  {/each}
</div>

{#if creando}
  <div class="nuevo">
    <span class="label">Recordatorio nuevo</span>
    <div class="campos">
      <div class="stack">
        <label class="label" for="rn-nombre">Nombre</label>
        <!-- svelte-ignore a11y_autofocus -->
        <input
          id="rn-nombre"
          autofocus
          type="text"
          placeholder="Tomar la pastilla"
          bind:value={nuevoNombre}
        />
      </div>
      <div class="stack corto">
        <label class="label" for="rn-hora">Hora</label>
        <input id="rn-hora" class="num" type="time" bind:value={nuevaHora} />
      </div>
    </div>

    <div class="stack">
      <label class="label" for="rn-mensaje">Mensaje (opcional)</label>
      <input
        id="rn-mensaje"
        type="text"
        placeholder="Lo que quieres leer en la notificación"
        bind:value={nuevoMensaje}
      />
    </div>

    <div class="repeticion">
      <div class="segmented" role="group" aria-label="Días del recordatorio nuevo">
        {#each repeticiones as [valor, texto] (valor)}
          <button class="seg" class:on={nuevaRepeticion === valor} onclick={() => (nuevaRepeticion = valor)}>
            {texto}
          </button>
        {/each}
      </div>

      {#if nuevaRepeticion === "weekdays"}
        <div class="dias" role="group" aria-label="Elegir días">
          {#each DIAS as d (d.bit)}
            <button
              class="dia"
              class:on={(nuevosDias & (1 << d.bit)) !== 0}
              aria-pressed={(nuevosDias & (1 << d.bit)) !== 0}
              aria-label={d.largo}
              onclick={() => (nuevosDias = nuevosDias ^ (1 << d.bit))}
            >
              {d.corto}
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <div class="spread">
      <span class="hint">
        Los recordatorios propios avisan y ya: la app no sabe si cumpliste, así que no se callan
        solos como los de fábrica.
      </span>
      <div class="row">
        <button class="ghost" onclick={() => (creando = false)}>Cancelar</button>
        <button
          class="primary"
          onclick={crear}
          disabled={!nuevoNombre.trim() || ocupado === "nuevo"}
        >
          Crear
        </button>
      </div>
    </div>
  </div>
{:else}
  <button class="agregar" onclick={() => (creando = true)}>+ Agregar recordatorio</button>
{/if}

<ul class="rows">
  <li>
    <div class="stack grow">
      <span>Horario de silencio</span>
      <span class="hint">
        Entre estas horas no suena nada, aunque toque. Por defecto de 22:00 a 07:00.
      </span>
    </div>
    <div class="row">
      <input
        class="corta num"
        type="number"
        min="0"
        max="23"
        value={settings.quiet_start}
        onchange={(e) => onsetting({ quiet_start: (e.currentTarget as HTMLInputElement).value })}
      />
      <span class="muted">a</span>
      <input
        class="corta num"
        type="number"
        min="0"
        max="23"
        value={settings.quiet_end}
        onchange={(e) => onsetting({ quiet_end: (e.currentTarget as HTMLInputElement).value })}
      />
    </div>
  </li>
</ul>

<style>
  .rows {
    list-style: none;
    margin: 0;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
  }

  .rows li {
    display: flex;
    align-items: center;
    gap: 16px;
    min-height: 48px;
    padding: 8px 14px;
  }

  .lista {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
  }

  .lista.apagada {
    opacity: 0.55;
  }

  .recordatorio {
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
  }

  .recordatorio:last-child {
    border-bottom: none;
  }

  .cabecera {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .stack {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .grow {
    flex: 1;
    min-width: 0;
  }

  .nombre {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .etiqueta {
    font-size: 11px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--ink-muted);
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    padding: 0 5px;
  }

  .hora {
    width: 120px;
  }

  .corta {
    width: 72px;
  }

  .corto {
    width: 130px;
    flex: none;
  }

  /* Interruptor rectangular con radio 6px, nunca una píldora. */
  .toggle {
    width: 38px;
    height: 22px;
    flex: none;
    padding: 2px;
    border-radius: 6px;
    background: var(--surface-sunken);
    border: 1px solid var(--border-strong);
    display: flex;
    justify-content: flex-start;
  }

  .toggle.on {
    background: var(--accent);
    border-color: var(--accent);
    justify-content: flex-end;
  }

  .knob {
    width: 16px;
    height: 16px;
    border-radius: 4px;
    background: var(--surface-2);
    display: block;
  }

  .repeticion {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
    margin: 10px 0 0 50px;
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

  .dias {
    display: flex;
    gap: 4px;
  }

  .dia {
    width: 28px;
    height: 28px;
    padding: 0;
    font-size: 12px;
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    color: var(--ink-secondary);
  }

  .dia.on {
    background: var(--accent);
    border-color: var(--accent);
    color: #ffffff;
    font-weight: 600;
  }

  .resumen {
    margin: 0;
  }

  .agregar {
    align-self: flex-start;
  }

  .nuevo {
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    background: var(--surface-sunken);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .nuevo .repeticion {
    margin-left: 0;
  }

  .campos {
    display: flex;
    gap: 12px;
  }

  .campos .stack:first-child {
    flex: 1;
  }

  button.chico {
    height: 26px;
    font-size: 12px;
    padding: 0 8px;
  }

  .error {
    margin: 0;
    color: var(--critical);
  }

  .ok {
    margin: 0;
    color: var(--good);
  }

  .hint {
    margin: 0;
  }
</style>
