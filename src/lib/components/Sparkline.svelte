<script lang="ts">
  import type { DayStatus } from "../types";
  import { statusColor, statusLabel } from "../format";

  let { days }: { days: DayStatus[] } = $props();
</script>

<div class="strip" aria-label="Últimos {days.length} días">
  {#each days as status, i (i)}
    <span
      class="cell"
      class:hatched={status === "skipped"}
      style:background={statusColor[status]}
      title={statusLabel[status]}
    ></span>
  {/each}
</div>

<style>
  .strip {
    display: flex;
    gap: 2px;
  }

  .cell {
    flex: 1;
    height: 14px;
    min-width: 4px;
    border-radius: 2px;
    border: 1px solid rgb(20 20 19 / 6%);
  }

  /* El día pausado se distingue por trama, no solo por color. */
  .cell.hatched {
    background-image: repeating-linear-gradient(
      45deg,
      transparent 0 2px,
      var(--surface-2) 2px 4px
    );
  }
</style>
