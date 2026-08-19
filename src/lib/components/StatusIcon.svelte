<script lang="ts">
  import type { Severity } from "../types";
  import { severityColor } from "../format";

  let {
    severity = "neutral" as Severity,
    size = 20,
    title = "",
  }: { severity?: Severity; size?: number; title?: string } = $props();
</script>

<!-- El color nunca va solo: este icono siempre acompaña a un texto. -->
<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke={severityColor[severity]}
  stroke-width="1.75"
  stroke-linecap="round"
  stroke-linejoin="round"
  role="img"
  aria-label={title}
>
  <circle cx="12" cy="12" r="9" />
  {#if severity === "good"}
    <path d="m8 12.5 2.5 2.5L16 9.5" />
  {:else if severity === "critical"}
    <path d="m9 9 6 6M15 9l-6 6" />
  {:else if severity === "warning" || severity === "serious"}
    <path d="M12 7.5v5" />
    <path d="M12 16h.01" />
  {:else}
    <path d="M12 7.5V12l3 1.5" />
  {/if}
</svg>
