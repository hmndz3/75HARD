// Chart.js atado al sistema de diseño.
//
// Reglas que vienen del §2 de docs/75HARD-DISENO.md y que aquí se hacen
// obligatorias: sin borde de gráfica, solo líneas de rejilla horizontales,
// numerales tabulares en los ejes, y como máximo tres series categóricas.

import {
  BarController,
  BarElement,
  CategoryScale,
  Chart,
  Filler,
  Legend,
  LineController,
  LineElement,
  LinearScale,
  PointElement,
  ScatterController,
  Tooltip,
  type ChartOptions,
} from "chart.js";

// Los genéricos de opciones de Chart.js son invariantes por tipo de gráfica, y
// eso impide componer una base común con spread. Se relajan aquí, en un solo
// sitio y a propósito, en vez de duplicar la configuración en cada pantalla.
type Opciones = ChartOptions<any>;

Chart.register(
  BarController,
  LineController,
  ScatterController,
  BarElement,
  LineElement,
  PointElement,
  CategoryScale,
  LinearScale,
  Filler,
  Legend,
  Tooltip
);

/** Lee un token del CSS para que las gráficas sigan el tema, no lo dupliquen. */
export function token(name: string, fallback = "#000"): string {
  if (typeof document === "undefined") return fallback;
  const v = getComputedStyle(document.documentElement).getPropertyValue(name);
  return v.trim() || fallback;
}

/** Las tres categóricas del spec. Nunca hay una cuarta. */
export const serie = {
  uno: () => token("--accent", "#3a6ea5"),
  dos: () => "#c05a30",
  tres: () => "#1f8a6d",
  apagada: () => token("--border-strong", "#c6c5bf"),
};

export const tinta = {
  eje: () => token("--ink-muted", "#7f7e78"),
  rejilla: () => token("--border", "#dcdbd7"),
  texto: () => token("--ink-primary", "#141413"),
  panel: () => token("--surface-2", "#ffffff"),
};

const FUENTE = {
  family: "Inter, 'Segoe UI Variable Text', 'Segoe UI', system-ui, sans-serif",
  size: 11,
};

/** Base común: sin cromo, ejes discretos, tooltip sobrio. */
export function base(): Opciones {
  return {
    responsive: true,
    maintainAspectRatio: false,
    animation: { duration: 0 },
    interaction: { mode: "nearest", intersect: false },
    layout: { padding: { top: 4, right: 4 } },
    plugins: {
      legend: {
        display: false,
        position: "top",
        align: "start",
        labels: {
          boxWidth: 10,
          boxHeight: 10,
          usePointStyle: false,
          color: tinta.eje(),
          font: FUENTE,
        },
      },
      tooltip: {
        backgroundColor: tinta.texto(),
        titleFont: FUENTE,
        bodyFont: FUENTE,
        padding: 10,
        cornerRadius: 4,
        displayColors: true,
        boxWidth: 8,
        boxHeight: 8,
      },
    },
    scales: {
      x: {
        grid: { display: false },
        border: { color: tinta.rejilla() },
        ticks: {
          color: tinta.eje(),
          font: FUENTE,
          maxRotation: 0,
          autoSkipPadding: 16,
        },
      },
      y: {
        beginAtZero: true,
        grid: { color: tinta.rejilla() },
        border: { display: false },
        ticks: { color: tinta.eje(), font: FUENTE, padding: 8 },
      },
    },
  };
}

/** Igual que `base` pero con la leyenda visible arriba a la izquierda. */
export function conLeyenda(): Opciones {
  const o = base();
  if (o.plugins?.legend) o.plugins.legend.display = true;
  return o;
}

/**
 * Línea de referencia punteada. Chart.js no la trae de serie sin el plugin de
 * anotaciones, y meter una dependencia entera por una línea no compensa.
 */
export function lineaMeta(valor: number, etiqueta: string) {
  return {
    id: `meta-${etiqueta}`,
    afterDatasetsDraw(chart: Chart<any>) {
      const { ctx, chartArea, scales } = chart;
      const y = scales.y?.getPixelForValue(valor);
      if (y === undefined || !chartArea) return;

      ctx.save();
      ctx.setLineDash([4, 4]);
      ctx.strokeStyle = serie.apagada();
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(chartArea.left, y);
      ctx.lineTo(chartArea.right, y);
      ctx.stroke();

      ctx.setLineDash([]);
      ctx.fillStyle = tinta.eje();
      ctx.font = `500 11px ${FUENTE.family}`;
      ctx.textAlign = "right";
      ctx.textBaseline = "bottom";
      ctx.fillText(etiqueta, chartArea.right - 2, y - 3);
      ctx.restore();
    },
  };
}

/**
 * Bandas de referencia de glucosa: rellenos horizontales muy desaturados
 * detrás de los puntos. **No son un diagnóstico**, solo sitúan la lectura.
 */
export function bandasGlucosa(bandas: { desde: number; hasta: number; color: string; label: string }[]) {
  return {
    id: "bandas-glucosa",
    beforeDatasetsDraw(chart: Chart<any>) {
      const { ctx, chartArea, scales } = chart;
      if (!chartArea || !scales.y) return;

      ctx.save();
      for (const b of bandas) {
        const y1 = scales.y.getPixelForValue(Math.min(b.hasta, scales.y.max));
        const y2 = scales.y.getPixelForValue(Math.max(b.desde, scales.y.min));
        ctx.fillStyle = b.color;
        ctx.fillRect(chartArea.left, y1, chartArea.right - chartArea.left, y2 - y1);

        ctx.fillStyle = tinta.eje();
        ctx.font = `500 10px ${FUENTE.family}`;
        ctx.textAlign = "right";
        ctx.textBaseline = "top";
        ctx.fillText(b.label, chartArea.right - 4, y1 + 3);
      }
      ctx.restore();
    },
  };
}

export { Chart };
