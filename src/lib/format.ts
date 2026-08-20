import type { CellStatus, DayStatus, PillarStatus, Severity } from "./types";

/** "6h 40min" — mismo formato que daycut::format_minutes en Rust. */
export function minutes(total: number): string {
  const h = Math.floor(total / 60);
  const m = total % 60;
  if (h === 0) return `${m}min`;
  if (m === 0) return `${h}h`;
  return `${h}h ${m}min`;
}

/** Minutos como "6h 40" compacto, para tablas. */
export function minutesShort(total: number | null): string {
  if (total === null) return "—";
  const h = Math.floor(total / 60);
  const m = total % 60;
  return `${h}h ${String(m).padStart(2, "0")}m`;
}

export function litres(ml: number): string {
  return (ml / 1000).toFixed(1);
}

/** Peso siempre a un decimal: la báscula no mide más y el float no debe verse. */
export function kg(value: number | null): string {
  return value === null ? "—" : `${value.toFixed(1)} kg`;
}

/** "a, b y c" — misma redacción que coach::join_es en Rust. */
export function joinEs(items: string[]): string {
  if (items.length === 0) return "";
  if (items.length === 1) return items[0];
  return `${items.slice(0, -1).join(", ")} y ${items[items.length - 1]}`;
}

/** "1 parcial" / "3 parciales" */
export function plural(n: number, singular: string, plural: string): string {
  return `${n} ${n === 1 ? singular : plural}`;
}

/** "2026-08-19T23:40:00" → "23:40" */
export function clock(iso: string): string {
  return iso.slice(11, 16);
}

/** "2026-08-19" → "19 ago" */
export function shortDate(date: string): string {
  const meses = ["ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic"];
  const [, m, d] = date.split("-");
  return `${Number(d)} ${meses[Number(m) - 1]}`;
}

export function todayIso(): string {
  const now = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

export function nowClock(): string {
  const now = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(now.getHours())}:${pad(now.getMinutes())}`;
}

export const statusLabel: Record<DayStatus, string> = {
  complete: "Completo",
  failed: "Fallido",
  skipped: "Pausado",
  pending: "Pendiente",
  empty: "Sin registrar",
};

/** Color de estado. Nunca se usa solo: siempre acompañado de icono o texto. */
export const statusColor: Record<DayStatus, string> = {
  complete: "var(--good)",
  failed: "var(--critical)",
  skipped: "var(--border-strong)",
  pending: "var(--warning)",
  empty: "var(--surface-sunken)",
};

/** Los cinco estados del heatmap de 75 días (P5). */
export const cellLabel: Record<CellStatus, string> = {
  complete: "Completo",
  partial: "Parcial",
  failed: "Fallido",
  skipped: "Pausado",
  pending: "Sin cerrar",
  empty: "Sin registrar",
  future: "Por venir",
};

export const cellColor: Record<CellStatus, string> = {
  complete: "var(--good)",
  partial: "var(--warning)",
  failed: "var(--critical)",
  skipped: "var(--border-strong)",
  pending: "var(--warning)",
  empty: "var(--surface-sunken)",
  future: "var(--surface-sunken)",
};

export const severityClass: Record<Severity, string> = {
  good: "s-good",
  neutral: "s-neutral",
  warning: "s-warning",
  serious: "s-serious",
  critical: "s-critical",
};

export const severityColor: Record<Severity, string> = {
  good: "var(--good)",
  neutral: "var(--border-strong)",
  warning: "var(--warning)",
  serious: "var(--serious)",
  critical: "var(--critical)",
};

export const pillarSeverity: Record<PillarStatus, Severity> = {
  done: "good",
  partial: "warning",
  missing: "critical",
};

export const workoutKinds: { value: string; label: string }[] = [
  { value: "gym", label: "Gimnasio" },
  { value: "cardio", label: "Cardio" },
  { value: "outdoor", label: "Al aire libre" },
  { value: "sport", label: "Deporte" },
  { value: "other", label: "Otro" },
];

export const glucoseContexts: { value: string; label: string }[] = [
  { value: "fasting", label: "En ayunas" },
  { value: "pre_meal", label: "Antes de comer" },
  { value: "post_meal_2h", label: "2 h post-comida" },
  { value: "pre_workout", label: "Antes de entrenar" },
  { value: "random", label: "Aleatoria" },
];

export function glucoseContextLabel(value: string): string {
  return glucoseContexts.find((c) => c.value === value)?.label ?? value;
}

export function workoutKindLabel(value: string): string {
  return workoutKinds.find((k) => k.value === value)?.label ?? value;
}

/** Número con separador de miles en español. */
export function num(n: number): string {
  return n.toLocaleString("es-MX");
}
