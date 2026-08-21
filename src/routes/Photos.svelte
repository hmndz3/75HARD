<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";

  import Modal from "../lib/components/Modal.svelte";
  import * as api from "../lib/api";
  import { plural, shortDate, todayIso } from "../lib/format";
  import type { ProgressPhoto } from "../lib/types";

  let fotos = $state.raw<ProgressPhoto[]>([]);
  let error = $state("");
  let cargando = $state(true);
  let subiendo = $state(false);

  // Las miniaturas se leen una vez y se quedan: son las mismas imágenes que
  // luego abre el visor, y releerlas del disco en cada render no aporta nada.
  let cache = $state.raw<Record<string, string>>({});
  let abierta = $state<ProgressPhoto | null>(null);
  let confirmarBorrado = $state(false);

  async function cargar() {
    try {
      fotos = await api.listProgressPhotos();
      await Promise.all(fotos.map((f) => leer(f.id)));
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      cargando = false;
    }
  }

  async function leer(id: string): Promise<void> {
    if (cache[id]) return;
    try {
      const url = await api.readProgressPhoto(id);
      cache = { ...cache, [id]: url };
    } catch {
      // Una imagen que no se puede leer no debe tumbar la galería entera.
    }
  }

  cargar();

  /** Agrupadas por día, el más reciente primero. */
  const porDia = $derived.by(() => {
    const grupos = new Map<string, ProgressPhoto[]>();
    for (const f of fotos) {
      if (!grupos.has(f.date)) grupos.set(f.date, []);
      grupos.get(f.date)?.push(f);
    }
    return [...grupos.entries()].sort((a, b) => b[0].localeCompare(a[0]));
  });

  async function agregar() {
    error = "";
    subiendo = true;
    try {
      const elegidas = await open({
        multiple: true,
        filters: [{ name: "Imágenes", extensions: ["jpg", "jpeg", "png", "webp", "heic", "bmp"] }],
      });
      const rutas = Array.isArray(elegidas) ? elegidas : elegidas ? [elegidas] : [];
      if (rutas.length === 0) return;

      const hoy = todayIso();
      for (const ruta of rutas) {
        await api.addProgressPhoto(hoy, ruta);
      }
      await cargar();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      subiendo = false;
    }
  }

  async function borrar(id: string) {
    error = "";
    try {
      await api.deleteProgressPhoto(id);
      const { [id]: _, ...resto } = cache;
      cache = resto;
      abierta = null;
      confirmarBorrado = false;
      await cargar();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  const tituloDia = (f: ProgressPhoto) =>
    f.dayNumber ? `Día ${f.dayNumber}` : shortDate(f.date);
</script>

<div class="page">
  <div class="spread">
    <h1 class="section-title">Fotos de progreso</h1>
    <button class="primary" onclick={agregar} disabled={subiendo}>
      {subiendo ? "Copiando…" : "Agregar fotos de hoy"}
    </button>
  </div>

  {#if error}<p class="error" role="alert">{error}</p>{/if}

  {#if cargando}
    <p class="muted">Cargando…</p>
  {:else if fotos.length === 0}
    <div class="card vacio">
      <p class="muted">
        Todavía no hay fotos. Puedes subir varias de una vez, y todas quedan marcadas con el día
        en que las subiste.
      </p>
      <p class="hint">
        Se guardan dentro de la carpeta de datos de la app, no en la nube: estas imágenes no salen
        de tu máquina.
      </p>
    </div>
  {:else}
    {#each porDia as [fecha, delDia] (fecha)}
      <section>
        <div class="dia">
          <span class="label">{tituloDia(delDia[0])}</span>
          <span class="muted">{delDia[0].weekdayLabel}</span>
          <span class="hint">{plural(delDia.length, "foto", "fotos")}</span>
        </div>

        <div class="rejilla">
          {#each delDia as f (f.id)}
            <button
              class="miniatura"
              onclick={() => {
                abierta = f;
                confirmarBorrado = false;
              }}
              aria-label="Abrir foto del {f.weekdayLabel}"
            >
              {#if cache[f.id]}
                <img src={cache[f.id]} alt="Progreso del {f.weekdayLabel}" />
              {:else}
                <span class="cargando muted">…</span>
              {/if}
            </button>
          {/each}
        </div>
      </section>
    {/each}
  {/if}
</div>

{#if abierta}
  <Modal
    width={720}
    title={tituloDia(abierta)}
    subtitle={abierta.weekdayLabel}
    onclose={() => (abierta = null)}
  >
    <div class="visor">
      {#if cache[abierta.id]}
        <img src={cache[abierta.id]} alt="Progreso del {abierta.weekdayLabel}" />
      {:else}
        <p class="muted">No se pudo leer la imagen.</p>
      {/if}
    </div>

    {#snippet footer()}
      <div class="spread">
        {#if confirmarBorrado}
          <span class="hint">Se borra el archivo, no solo el registro.</span>
          <div class="row">
            <button onclick={() => (confirmarBorrado = false)}>Cancelar</button>
            <button class="danger" onclick={() => abierta && borrar(abierta.id)}>
              Sí, borrar
            </button>
          </div>
        {:else}
          <button class="ghost danger" onclick={() => (confirmarBorrado = true)}>Borrar</button>
          <button onclick={() => (abierta = null)}>Cerrar</button>
        {/if}
      </div>
    {/snippet}
  </Modal>
{/if}

<style>
  .page {
    max-width: 1200px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .dia {
    display: flex;
    align-items: baseline;
    gap: 10px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border);
  }

  .rejilla {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 12px;
  }

  .miniatura {
    padding: 0;
    /* Los botones traen alto fijo del sistema de diseño; aquí manda la
       proporción de la foto. */
    height: auto;
    width: 100%;
    aspect-ratio: 3 / 4;
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    overflow: hidden;
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .miniatura:hover {
    border-color: var(--accent);
  }

  .miniatura img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .cargando {
    font-size: 18px;
  }

  .visor {
    display: grid;
    place-items: center;
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    padding: 12px;
  }

  .visor img {
    max-width: 100%;
    max-height: 60vh;
    object-fit: contain;
  }

  .vacio {
    padding: 32px 20px;
    text-align: center;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .vacio p {
    margin: 0;
  }

  .error {
    margin: 0;
    color: var(--critical);
  }
</style>
