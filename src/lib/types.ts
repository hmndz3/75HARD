// Espejo en TypeScript de los tipos de src-tauri/src/db/models.rs.
// Si cambias uno, cambia el otro: no hay generación automática a propósito,
// son pocos tipos y la duplicación se paga sola en claridad.

export type Severity = "good" | "neutral" | "warning" | "serious" | "critical";
export type DayStatus = "pending" | "complete" | "failed" | "skipped" | "empty";
export type PillarStatus = "done" | "partial" | "missing";
export type Tone = "suave" | "directo" | "duro";

export interface CoachMessage {
  severity: Severity;
  text: string;
}

export interface Pillar {
  key: string;
  label: string;
  required: boolean;
  goal: number | null;
}

export interface Rules {
  pillars: Pillar[];
}

export interface Challenge {
  id: string;
  name: string;
  startDate: string;
  targetDays: number;
  rules: Rules;
  endedAt: string | null;
  endedReason: string | null;
}

export interface SleepLog {
  date: string;
  bedtime: string;
  wakeTime: string;
  minutes: number;
  quality: number | null;
}

export interface Meal {
  id: string;
  date: string;
  eatenAt: string;
  kind: "meal" | "snack";
  description: string;
  calories: number | null;
}

export interface Workout {
  id: string;
  date: string;
  startedAt: string;
  kind: string;
  description: string | null;
  durationMin: number;
  isOutdoor: boolean;
  caloriesBurned: number | null;
}

export interface GlucoseReading {
  id: string;
  date: string;
  measuredAt: string;
  valueMgdl: number;
  context: string;
  notes: string | null;
  linkedMealId: string | null;
  linkedMealDescription: string | null;
}

export interface WorkSession {
  id: string;
  date: string;
  startedAt: string;
  endedAt: string | null;
  minutes: number;
  category: string;
  description: string | null;
}

export interface ReadingLog {
  id: string;
  date: string;
  pages: number;
  book: string | null;
}

export interface MoodLog {
  mood: number;
  energy: number;
  stress: number | null;
}

export interface PillarState {
  key: string;
  label: string;
  required: boolean;
  status: PillarStatus;
  detail: string;
}

export interface TodayView {
  date: string;
  weekdayLabel: string;
  dayNumber: number | null;
  targetDays: number;
  status: DayStatus;
  streak: number;
  longestStreak: number;
  recent: DayStatus[];

  sleep: SleepLog | null;
  sleepGoalMin: number;
  sleepVerdict: CoachMessage | null;

  mealsCount: number;
  snacksCount: number;
  caloriesIn: number | null;
  workoutMin: number;
  workoutKcal: number | null;
  waterMl: number;
  waterGoalMl: number;
  readingPages: number;
  readingGoalPages: number;
  glucoseCount: number;
  workMin: number;
  weightKg: number | null;
  mood: MoodLog | null;

  pillars: PillarState[];
  coach: CoachMessage;
  morningDone: boolean;
  eveningDone: boolean;
}

export interface DayRow {
  date: string;
  dayNumber: number | null;
  weekdayLabel: string;
  status: DayStatus;
  sleepMinutes: number | null;
  workoutMin: number;
  mealsCount: number;
  waterMl: number;
  glucoseCount: number;
  weightKg: number | null;
}

export interface HistorySummary {
  complete: number;
  partial: number;
  failed: number;
  skipped: number;
}

export interface History {
  rows: DayRow[];
  summary: HistorySummary;
}

export interface DayDetail {
  date: string;
  weekdayLabel: string;
  dayNumber: number | null;
  status: DayStatus;
  notes: string | null;
  sleep: SleepLog | null;
  meals: Meal[];
  workouts: Workout[];
  glucose: GlucoseReading[];
  work: WorkSession[];
  reading: ReadingLog[];
  waterMl: number;
  weightKg: number | null;
  mood: MoodLog | null;
}

export interface MissingDay {
  date: string;
  weekdayLabel: string;
  dayNumber: number | null;
}

export interface SleepPreview {
  minutes: number;
  label: string;
  verdict: CoachMessage;
  goalMin: number;
}

export interface Bootstrap {
  needsOnboarding: boolean;
  challenge: Challenge | null;
  settings: Record<string, string>;
  today: string;
  attemptNumber: number;
  defaultRules: Rules;
  dbPath: string;
}

export type MissingAction = "fill" | "failed" | "empty";
