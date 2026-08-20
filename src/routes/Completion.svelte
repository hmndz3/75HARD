<script lang="ts">
  import Heatmap from "../lib/components/Heatmap.svelte";
  import * as api from "../lib/api";
  import { shortDate } from "../lib/format";
  import type { BeforeAfter, Completion } from "../lib/types";

  let {
    onback,
    onnew,
    onexport,
  }: { onback: () => void; onnew: () => Promise<void>; onexport: () => void } = $props();

  let d = $state.raw<Completion | null>(null);
  let error = $state("");
  let saving = $state(false);

  api
    .getCompletion()
    .then((v) => (d = v))
    .catch((e) => (error = e instanceof Error ? e.message : String(e)));

  async function empezarNuevo() {
    if (saving) return;
    saving = true;
    error = "";
    try {
      await api.endChallenge({ reason: "completed" });
      await onnew();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      saving = false;
    }
  }

  const fmt = (v: number | null, unit: string) => (v === null ? "—" : `${v}${unit ? ` ${unit}` : ""}`);

  /** Verde si el cambio va en la dirección que el pilar considera buena. */
  function claseDelta(t: BeforeAfter): string {
    if (t.before === null || t.after === null || t.better === "none") return "muted";
    const delta = t.after - t.before;
    if (Math.abs(delta) < 0.05) return "muted";
    const mejora = t.better === "up" ? delta > 0 : delta < 0;
    return mejora ? "s-good" : "s-warning";
  }

  function delta(t: BeforeAfter): string {
    if (t.before === null || t.after === null) return "";
    const v = t.after - t.before;
    // Redondear a un decimal y luego imprimir el signo produce cosas como
    // "−0 h". Si el cambio no llega al decimal, no hubo cambio.
    if (Math.abs(v) < 0.05) return "sin cambio";
    const signo = v > 0 ? "+" : "−";
    return `${signo}${Math.abs(Math.round(v * 10) / 10)}${t.unit ? ` ${t.unit}` : ""}`;
  }

  /** Minigráfica de línea, sin ejes: solo la forma y los extremos. */
  function ruta(valores: (number | null)[], ancho: number, alto: number): string {
    const v = valores.filter((x): x is number => x !== null);
    if (v.length < 2) return "";
    const min = Math.min(...v);
    const max = Math.max(...v);
    const rango = max - min || 1;
    return v
      .map((y, i) => {
        const px = (i / (v.length - 1)) * ancho;
        const py = alto - ((y - min) / rango) * alto;
        return `${i === 0 ? "M" : "L"}${px.toFixed(1)},${py.toFixed(1)}`;
      })
      .join(" ");
  }

  const chispas = $derived.by(() => {
    if (!d) return [];
    return [
      { nombre: "Peso", datos: d.weight, unidad: "kg" },
      { nombre: "Sueño", datos: d.sleep, unidad: "h" },
      { nombre: "Glucosa en ayunas", datos: d.glucose, unidad: "mg/dL" },
    ].filter((s) => s.datos.filter((x) => x !== null).length >= 2);
  });
</script>

<div class="pantalla">
  <div class="col">
    {#if error}
      <p class="error" role="alert">{error}</p>
      <button onclick={onback}>Volver</button>
    {:else if !d}
      <p class="muted">Cargando…</p>
    {:else}
      <header>
        <div class="titular">
          <span class="hero-num">{d.completeDays} de {d.targetDays}</span>
          {#if d.finished}
            <svg class="check" viewBox="0 0 24 24" aria-hidden="true">
              <circle cx="12" cy="12" r="9.5" />
              <path d="m8 12.5 2.5 2.5L16 9.5" />
            </svg>
          {/if}
        </div>
        <p class="muted">
          {d.name} · del {shortDate(d.startDate)} al {shortDate(d.endDate)}
        </p>
      </header>

      <div class="rejilla">
        {#each d.tiles as t (t.label)}
          <div class="card tile">
            <span class="label">{t.label}</span>
            <div class="par">
              <span class="num antes">{fmt(t.before, t.unit)}</span>
              <span class="flecha dim">→</span>
              <span class="num despues">{fmt(t.after, t.unit)}</span>
            </div>
            <span class="delta {claseDelta(t)}">{delta(t) || "sin datos suficientes"}</span>
          </div>
        {/each}
      </div>

      <div class="card">
        <span class="label">Los {d.targetDays} días</span>
        <Heatmap cells={d.heatmap} onopen={() => {}} />
      </div>

      {#if chispas.length > 0}
        <div class="chispas">
          {#each chispas as s (s.nombre)}
            {@const v = s.datos.filter((x): x is number => x !== null)}
            <div class="card">
              <span class="label">{s.nombre}</span>
              <svg viewBox="0 0 200 48" preserveAspectRatio="none" aria-hidden="true">
                <path d={ruta(s.datos, 200, 44)} />
              </svg>
              <div class="spread hint num">
                <span>{v[0]} {s.unidad}</span>
                <span>{v[v.length - 1]} {s.unidad}</span>
              </div>
            </div>
          {/each}
        </div>
      {/if}

      <p class="cierre muted">
        Los datos de estos {d.targetDays} días no se van a ningún lado. Si empiezas otro reto, tus
        gráficas siguen siendo continuas.
      </p>

      <footer>
        <button class="ghost" onclick={onback}>Volver</button>
        <div class="row">
          <button onclick={onexport}>Exportar informe PDF</button>
          <button class="primary" onclick={empezarNuevo} disabled={saving}>
            Empezar un reto nuevo
          </button>
        </div>
      </footer>
    {/if}
  </div>
</div>

<style>
  .pantalla {
    height: calc(100% - var(--titlebar-h));
    overflow-y: auto;
    display: grid;
    justify-items: center;
    padding: 48px 24px;
    background: var(--bg);
  }

  .col {
    width: 860px;
    max-width: 100%;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .titular {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .check {
    width: 40px;
    height: 40px;
    fill: none;
    stroke: var(--good);
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  header p {
    margin: 4px 0 0;
  }

  .rejilla {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 16px;
  }

  .tile {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .par {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .antes {
    font-size: 18px;
    color: var(--ink-secondary);
  }

  .despues {
    font-size: 24px;
    font-weight: 600;
  }

  .flecha {
    font-size: 14px;
  }

  .delta {
    font-size: 13px;
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .chispas {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 16px;
  }

  .chispas svg {
    width: 100%;
    height: 48px;
    fill: none;
    stroke: var(--accent);
    stroke-width: 2;
    vector-effect: non-scaling-stroke;
  }

  .cierre {
    margin: 0;
  }

  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    border-top: 1px solid var(--border);
    padding-top: 20px;
  }

  .error {
    color: var(--critical);
  }
</style>
