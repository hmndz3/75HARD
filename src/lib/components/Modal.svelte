<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    width = 560,
    title,
    subtitle = "",
    onclose,
    closable = true,
    children,
    footer,
  }: {
    width?: number;
    title: string;
    subtitle?: string;
    onclose?: () => void;
    closable?: boolean;
    children: Snippet;
    footer?: Snippet;
  } = $props();

  let panel: HTMLDivElement | undefined = $state();

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && closable && onclose) {
      e.preventDefault();
      onclose();
    }
  }

  // El campo enfocado al abrir: se puede llenar sin tocar el mouse. Gana el
  // que pida autofoco explícito; si no hay, el primer control del panel.
  $effect(() => {
    const preferred = panel?.querySelector<HTMLElement>("[autofocus]");
    const fallback = panel?.querySelector<HTMLElement>(
      "input:not([type=hidden]):not([type=time]), textarea, select, button.primary"
    );
    (preferred ?? fallback)?.focus();
  });
</script>

<svelte:window on:keydown={onkeydown} />

<div class="scrim">
  <div class="panel" style:width="{width}px" bind:this={panel} role="dialog" aria-modal="true">
    <header>
      <div class="stack">
        <h2 class="section-title">{title}</h2>
        {#if subtitle}<span class="subtitle muted">{subtitle}</span>{/if}
      </div>
      {#if closable && onclose}
        <button class="ghost close" onclick={onclose} aria-label="Cerrar">✕</button>
      {/if}
    </header>

    <div class="body">
      {@render children()}
    </div>

    {#if footer}
      <footer>{@render footer()}</footer>
    {/if}
  </div>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: rgb(20 20 19 / 45%);
    display: grid;
    place-items: center;
    z-index: 50;
    padding: 24px;
  }

  .panel {
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    max-width: 100%;
    max-height: 100%;
    display: flex;
    flex-direction: column;
  }

  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    padding: 20px;
    border-bottom: 1px solid var(--border);
  }

  .subtitle {
    margin-top: 4px;
  }

  .body {
    padding: 20px;
    overflow-y: auto;
  }

  footer {
    padding: 16px 20px;
    border-top: 1px solid var(--border);
  }

  .close {
    height: 28px;
    width: 28px;
    padding: 0;
    font-size: 13px;
    flex: none;
  }
</style>
