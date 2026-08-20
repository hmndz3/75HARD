<script lang="ts">
  import type { Snippet } from "svelte";

  import TitleBar from "./TitleBar.svelte";

  let {
    route,
    subtitle = "",
    streak = 0,
    onnavigate,
    children,
  }: {
    route: string;
    subtitle?: string;
    streak?: number;
    onnavigate: (to: string) => void;
    children: Snippet;
  } = $props();

  const items = [
    { id: "hoy", label: "Hoy", icon: "M8 2v3M16 2v3M3 9h18M5 5h14a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2Z" },
    { id: "historial", label: "Historial", icon: "M3 12a9 9 0 1 0 3-6.7L3 8M3 3v5h5M12 7v5l3 2" },
    { id: "estadisticas", label: "Estadísticas", icon: "M4 20V10M10 20V4M16 20v-7M22 20H2" },
    { id: "fotos", label: "Fotos", icon: "M4 7h3l1.5-2h7L17 7h3a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V8a1 1 0 0 1 1-1ZM12 16a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Z" },
    { id: "ajustes", label: "Ajustes", icon: "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6ZM19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-2.9 1.2v.2a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-3-1.2l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0-1.2-2.9H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.2-3l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.9.3 1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 2.9 1.2l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0 1.2 2.9h.2a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1Z" },
  ];
</script>

<div class="app">
  <TitleBar {subtitle} />

  <div class="body">
    <nav class="sidebar">
      <ul>
        {#each items as item (item.id)}
          <li>
            <button
              class="nav"
              class:active={route === item.id}
              onclick={() => onnavigate(item.id)}
              aria-current={route === item.id ? "page" : undefined}
            >
              <svg viewBox="0 0 24 24"><path d={item.icon} /></svg>
              {item.label}
            </button>
          </li>
        {/each}
      </ul>

      <div class="streak">
        <span class="label">Racha actual</span>
        <span class="value num">{streak}<span class="unit">&nbsp;días</span></span>
      </div>
    </nav>

    <main class="content">
      {@render children()}
    </main>
  </div>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100%;
  }









  .body {
    flex: 1;
    display: flex;
    min-height: 0;
  }

  .sidebar {
    width: var(--sidebar-w);
    flex: none;
    background: var(--surface-1);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 8px 0;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .nav {
    width: 100%;
    height: 40px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 16px;
    border: none;
    border-radius: 0;
    border-left: 3px solid transparent;
    background: transparent;
    color: var(--ink-secondary);
    text-align: left;
  }

  .nav svg {
    width: 18px;
    height: 18px;
    flex: none;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .nav:hover {
    background: var(--surface-sunken);
    color: var(--ink-primary);
  }

  /* Ítem activo: fondo hundido + barra de 3px del acento a la izquierda. */
  .nav.active {
    background: var(--surface-sunken);
    border-left-color: var(--accent);
    color: var(--ink-primary);
    font-weight: 500;
  }

  .streak {
    border-top: 1px solid var(--border);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .streak .value {
    font-size: 24px;
    font-weight: 600;
  }

  .streak .unit {
    font-size: 13px;
    font-weight: 400;
    color: var(--ink-secondary);
  }

  .content {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    padding: 24px;
  }
</style>
