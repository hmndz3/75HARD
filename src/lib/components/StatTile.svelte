<script lang="ts">
  import type { Severity } from "../types";
  import { severityClass } from "../format";

  let {
    label,
    value,
    detail = "",
    severity = "neutral" as Severity,
  }: { label: string; value: string; detail?: string; severity?: Severity } = $props();
</script>

<div class="card tile">
  <span class="label">{label}</span>
  <span class="card-value">{value}</span>
  {#if detail}
    <span class="detalle {severity === 'neutral' ? 'muted' : severityClass[severity]}">
      {detail}
    </span>
  {/if}
</div>

<style>
  .tile {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-height: 104px;
  }

  .detalle {
    font-size: 13px;
  }
</style>
