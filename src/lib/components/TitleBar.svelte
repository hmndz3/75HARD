<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  // La ventana principal no tiene decoración del sistema: esta barra de 44px
  // es la que se arrastra y la que cierra. Va en TODAS las pantallas de
  // ventana completa, incluido el onboarding, o la ventana queda atrapada.
  let { subtitle = "" }: { subtitle?: string } = $props();

  const win = getCurrentWindow();
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <strong>75&nbsp;HARD</strong>
    {#if subtitle}<span class="dim">·</span><span class="muted">{subtitle}</span>{/if}
  </div>
  <div class="controls">
    <button class="wbtn" onclick={() => win.minimize()} aria-label="Minimizar">
      <svg viewBox="0 0 12 12"><path d="M2 6h8" /></svg>
    </button>
    <button class="wbtn" onclick={() => win.toggleMaximize()} aria-label="Maximizar">
      <svg viewBox="0 0 12 12"><rect x="2.5" y="2.5" width="7" height="7" /></svg>
    </button>
    <button class="wbtn close" onclick={() => win.close()} aria-label="Cerrar a la bandeja">
      <svg viewBox="0 0 12 12"><path d="m3 3 6 6M9 3l-6 6" /></svg>
    </button>
  </div>
</header>

<style>
  .titlebar {
    height: var(--titlebar-h);
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--surface-1);
    border-bottom: 1px solid var(--border);
    padding-left: 16px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    letter-spacing: 0.02em;
  }

  .controls {
    display: flex;
    height: 100%;
  }

  .wbtn {
    width: 46px;
    height: 100%;
    border: none;
    border-radius: 0;
    background: transparent;
    display: grid;
    place-items: center;
    padding: 0;
  }

  .wbtn svg {
    width: 12px;
    height: 12px;
    fill: none;
    stroke: var(--ink-secondary);
    stroke-width: 1.2;
  }

  .wbtn:hover {
    background: var(--surface-sunken);
  }

  .wbtn.close:hover {
    background: var(--critical);
  }

  .wbtn.close:hover svg {
    stroke: #ffffff;
  }
</style>
