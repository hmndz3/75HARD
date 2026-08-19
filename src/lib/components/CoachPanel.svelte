<script lang="ts">
  import type { CoachMessage } from "../types";
  import { severityColor } from "../format";
  import StatusIcon from "./StatusIcon.svelte";

  let { message, label = "Coach" }: { message: CoachMessage; label?: string } = $props();
</script>

<!-- Fondo hundido y barra de 3px del color de la severidad. Sin sombra. -->
<div class="coach" style:border-left-color={severityColor[message.severity]}>
  <StatusIcon severity={message.severity} size={18} title={label} />
  <div class="stack">
    <span class="label">{label}</span>
    <p>{message.text}</p>
  </div>
</div>

<style>
  .coach {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    background: var(--surface-sunken);
    border: 1px solid var(--border);
    border-left: 3px solid var(--border-strong);
    border-radius: var(--radius-card);
    padding: 16px 20px;
  }

  p {
    margin: 6px 0 0;
    color: var(--ink-primary);
  }
</style>
