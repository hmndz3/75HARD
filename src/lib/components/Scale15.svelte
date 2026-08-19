<script lang="ts">
  // Selector 1..5 completamente navegable con teclado: flechas o los números.
  let {
    label,
    value = $bindable(null as number | null),
  }: { label: string; value: number | null } = $props();

  function onkeydown(e: KeyboardEvent) {
    const n = Number(e.key);
    if (n >= 1 && n <= 5) {
      value = n;
      e.preventDefault();
      return;
    }
    if (e.key === "ArrowRight" || e.key === "ArrowUp") {
      value = Math.min(5, (value ?? 0) + 1);
      e.preventDefault();
    }
    if (e.key === "ArrowLeft" || e.key === "ArrowDown") {
      value = Math.max(1, (value ?? 6) - 1);
      e.preventDefault();
    }
  }
</script>

<div class="scale" role="radiogroup" aria-label={label} tabindex="0" {onkeydown}>
  <span class="name">{label}</span>
  <div class="cells">
    {#each [1, 2, 3, 4, 5] as n (n)}
      <button
        type="button"
        class="cell"
        class:on={value === n}
        role="radio"
        aria-checked={value === n}
        aria-label="{label} {n} de 5"
        onclick={() => (value = n)}
      >
        {n}
      </button>
    {/each}
  </div>
</div>

<style>
  .scale {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .scale:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 4px;
  }

  .name {
    width: 72px;
    color: var(--ink-secondary);
    flex: none;
  }

  .cells {
    display: flex;
    gap: 8px;
  }

  .cell {
    width: 40px;
    height: 40px;
    padding: 0;
    font-variant-numeric: tabular-nums;
    background: var(--surface-2);
    border: 1px solid var(--border-strong);
    color: var(--ink-secondary);
  }

  .cell.on {
    background: var(--accent);
    border-color: var(--accent);
    color: #ffffff;
    font-weight: 600;
  }
</style>
