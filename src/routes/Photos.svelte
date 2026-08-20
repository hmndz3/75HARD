<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";

  import * as api from "../lib/api";
  import { shortDate, todayIso } from "../lib/format";
  import type { ProgressPhoto } from "../lib/types";

  let fotos = $state.raw<ProgressPhoto[]>([]);
  let error = $state("");
  let cargando = $state(true);

  // Se guardan las imágenes ya decodificadas para no releerlas en cada cambio
  // del comparador. Son pocas y pesan lo que pesan una vez.
  let cache = $state.raw<Record<string, string>>({});
  let antesId = $state<string | null>(null);
  let despuesId = $state<string | null>(null);
  let confirmarBorrado = $state<string | null>(null);

  async function cargar() {
    try {
      fotos = await api.listProgressPhotos();
      if (fotos.length > 0) {
        antesId ??= fotos[0].id;
        despuesId ??= fotos[fotos.length - 1].id;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      cargando = false;
    }
  }

  cargar();

  async function ver(id: string | null): Promise<string | null> {
    if (!id) return null;
    if (cache[id]) return cache[id];
    const url = await api.readProgressPhoto(id);
    cache = { ...cache, [id]: url };
    return url;
  }

  let antesUrl = $state<string | null>(null);
  let despuesUrl = $state<string | null>(null);

  $effect(() => {
    const id = antesId;
    ver(id)
      .then((u) => (antesUrl = u))
      .catch(() => (antesUrl = null));
  });

  $effect(() => {
    const id = despuesId;
    ver(id)
      .then((u) => (despuesUrl = u))
      .catch(() => (despuesUrl = null));
  });

  async function agregar() {
    error = "";
    try {
      const elegida = await open({
        multiple: false,
        filters: [{ name: "Imágenes", extensions: ["jpg", "jpeg", "png", "webp", "heic", "bmp"] }],
      });
      if (typeof elegida !== "string") return;

      await api.addProgressPhoto(todayIso(), elegida);
      await cargar();
      despuesId = fotos.at(-1)?.id ?? null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function borrar(id: string) {
    error = "";
    try {
      await api.deleteProgressPhoto(id);
      confirmarBorrado = null;
      if (antesId === id) antesId = null;
      if (despuesId === id) despuesId = null;
      const { [id]: _, ...resto } = cache;
      cache = resto;
      await cargar();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  const etiqueta = (id: string | null) => {
    const f = fotos.find((x) => x.id === id);
    if (!f) return "—";
    return f.dayNumber ? `Día ${f.dayNumber} · ${shortDate(f.date)}` : shortDate(f.date);
  };
</script>

<div class="page">
  <div class="spread">
    <h1 class="section-title">Fotos de progreso</h1>
    <button class="primary" onclick={agregar}>Agregar foto de hoy</button>
  </div>

  {#if error}<p class="error" role="alert">{error}</p>{/if}

  {#if cargando}
    <p class="muted">Cargando…</p>
  {:else if fotos.length === 0}
    <div class="card vacio">
      <p class="muted">
        Todavía no hay fotos. Se guardan dentro de la carpeta de datos de la app, no en la nube:
        estas imágenes no salen de tu máquina.
      </p>
    </div>
  {:else}
    <div class="card comparador">
      <div class="card-head">
        <span class="label">Antes y después</span>
        <span class="hint">{fotos.length} {fotos.length === 1 ? "foto" : "fotos"}</span>
      </div>

      <div class="lado-a-lado">
        {#each [{ url: antesUrl, id: antesId, nombre: "Antes" }, { url: despuesUrl, id: despuesId, nombre: "Después" }] as lado (lado.nombre)}
          <figure>
            <div class="marco">
              {#if lado.url}
                <img src={lado.url} alt="{lado.nombre}: {etiqueta(lado.id)}" />
              {:else}
                <span class="muted">Elige una foto abajo</span>
              {/if}
            </div>
            <figcaption>
              <span class="label">{lado.nombre}</span>
              <span class="num">{etiqueta(lado.id)}</span>
            </figcaption>
          </figure>
        {/each}
      </div>
    </div>

    <div class="card flush">
      <div class="card-head"><span class="label">Todas las fotos</span></div>
      <ul class="tira">
        {#each fotos as f (f.id)}
          <li>
            <div class="fila">
              <div class="stack grow">
                <span class="nombre">
                  {f.dayNumber ? `Día ${f.dayNumber}` : "Fuera del reto"}
                </span>
                <span class="hint">{f.weekdayLabel}</span>
              </div>
              <div class="row">
                <button
                  class="chip"
                  class:on={antesId === f.id}
                  onclick={() => (antesId = f.id)}
                >
                  Antes
                </button>
                <button
                  class="chip"
                  class:on={despuesId === f.id}
                  onclick={() => (despuesId = f.id)}
                >
                  Después
                </button>
                {#if confirmarBorrado === f.id}
                  <button class="ghost small" onclick={() => (confirmarBorrado = null)}>
                    Cancelar
                  </button>
                  <button class="danger small" onclick={() => borrar(f.id)}>Sí, borrar</button>
                {:else}
                  <button class="ghost small" onclick={() => (confirmarBorrado = f.id)}>
                    Borrar
                  </button>
                {/if}
              </div>
            </div>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</div>

<style>
  .page {
    max-width: 1200px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .comparador {
    padding: 0;
  }

  .lado-a-lado {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    padding: 20px;
  }

  figure {
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .marco {
    aspect-ratio: 3 / 4;
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    display: grid;
    place-items: center;
    overflow: hidden;
  }

  .marco img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  figcaption {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
  }

  .tira {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 320px;
    overflow-y: auto;
  }

  .fila {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 20px;
    border-bottom: 1px solid var(--border);
  }

  .tira li:last-child .fila {
    border-bottom: none;
  }

  .stack {
    display: flex;
    flex-direction: column;
  }

  .grow {
    flex: 1;
  }

  .nombre {
    font-weight: 500;
  }

  .chip {
    cursor: pointer;
    height: 26px;
  }

  button.small {
    height: 26px;
    font-size: 12px;
    padding: 0 8px;
  }

  .vacio {
    padding: 32px 20px;
    text-align: center;
  }

  .vacio p {
    margin: 0;
  }

  .error {
    margin: 0;
    color: var(--critical);
  }
</style>
