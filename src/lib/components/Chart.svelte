<script lang="ts">
  import { Chart } from "../charts";
  import { tema } from "../theme.svelte";
  import type { ChartConfiguration, Plugin } from "chart.js";

  let {
    config,
    height = 320,
    plugins = [] as Plugin[],
    label,
  }: {
    config: ChartConfiguration<any>;
    height?: number;
    plugins?: Plugin<any>[];
    label: string;
  } = $props();

  let canvas = $state<HTMLCanvasElement | undefined>();

  // Se recrea la gráfica cuando cambia la configuración. Con series de 75
  // puntos como mucho, reconstruir es más simple y más fiable que parchear los
  // datasets a mano, y no se nota.
  $effect(() => {
    const cfg = config;
    // Leer el tema aquí hace que la gráfica se reconstruya al cambiarlo: los
    // colores salen de los tokens del CSS y hay que volver a consultarlos.
    void tema.aplicado;
    if (!canvas) return;

    const chart = new Chart(canvas, {
      ...cfg,
      plugins: [...(cfg.plugins ?? []), ...plugins],
    });
    return () => chart.destroy();
  });
</script>

<div class="lienzo" style:height="{height}px" role="img" aria-label={label}>
  <canvas bind:this={canvas}></canvas>
</div>

<style>
  .lienzo {
    position: relative;
    width: 100%;
  }
</style>
