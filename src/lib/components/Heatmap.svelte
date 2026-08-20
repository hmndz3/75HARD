<script lang="ts">
  import { cellColor, cellLabel } from "../format";
  import type { CellStatus, HeatmapCell } from "../types";

  let {
    cells,
    onopen,
  }: { cells: HeatmapCell[]; onopen: (date: string) => void } = $props();

  // Los cinco estados que se explican en la leyenda. "pending" no aparece:
  // se pinta igual que "partial" y significa lo mismo para quien lo mira.
  const leyenda: CellStatus[] = ["complete", "partial", "failed", "skipped", "empty"];

  // El texto dentro de la celda tiene que leerse sobre su propio fondo.
  const oscuro = (s: CellStatus) => s === "empty" || s === "future" || s === "partial";
</script>

<div class="grid" role="group" aria-label="Calendario del reto">
  {#each cells as c (c.date)}
    <button
      type="button"
      class="cell {c.status}"
      style:background={cellColor[c.status]}
      class:dark-text={oscuro(c.status)}
      disabled={c.status === "future"}
      title="Día {c.dayNumber} · {cellLabel[c.status]}"
      aria-label="Día {c.dayNumber}, {cellLabel[c.status]}"
      onclick={() => onopen(c.date)}
    >
      {c.dayNumber}
    </button>
  {/each}
</div>

<div class="legend">
  {#each leyenda as s (s)}
    <span class="key">
      <i class={s} style:background={cellColor[s]}></i>
      {cellLabel[s]}
    </span>
  {/each}
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(15, 22px);
    gap: 4px;
  }

  .cell {
    width: 22px;
    height: 22px;
    padding: 0;
    border: 1px solid rgb(20 20 19 / 8%);
    border-radius: 3px;
    font-size: 10px;
    font-variant-numeric: tabular-nums;
    color: #ffffff;
    line-height: 1;
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .cell.dark-text {
    color: var(--ink-muted);
  }

  .cell:disabled {
    cursor: default;
    color: var(--border-strong);
  }

  .cell:hover:not(:disabled) {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  /* Un día sin registrar se distingue del futuro por el borde punteado, no
     solo por el color: los dos comparten fondo. */
  .cell.empty {
    border-style: dashed;
    border-color: var(--border-strong);
  }

  /* El día pausado lleva trama además de color, para que se lea sin depender
     de distinguir grises. */
  .cell.skipped,
  i.skipped {
    background-image: repeating-linear-gradient(
      45deg,
      transparent 0 3px,
      var(--surface-2) 3px 5px
    );
  }

  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    margin-top: 14px;
    font-size: 12px;
    color: var(--ink-secondary);
  }

  .key {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .key i {
    width: 12px;
    height: 12px;
    border-radius: 3px;
    border: 1px solid rgb(20 20 19 / 8%);
    display: inline-block;
  }

  .key i.empty {
    border-style: dashed;
    border-color: var(--border-strong);
  }
</style>
