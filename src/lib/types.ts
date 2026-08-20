// Espejo en TypeScript de los tipos de src-tauri/src/db/models.rs.
// Si cambias uno, cambia el otro: no hay generación automática a propósito,
// son pocos tipos y la duplicación se paga sola en claridad.

export type Severity = "good" | "neutral" | "warning" | "serious" | "critical";
export type DayStatus = "pending" | "complete" | "failed" | "skipped" | "empty";
export type CellStatus = DayStatus | "partial" | "future";
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
  hoursLeft: number;
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

export interface HeatmapCell {
  date: string;
  dayNumber: number;
  status: CellStatus;
}

export interface History {
  rows: DayRow[];
  summary: HistorySummary;
  heatmap: HeatmapCell[];
}

export interface Reminder {
  id: string;
  kind: string;
  label: string;
  description: string;
  timeOfDay: string;
  enabled: boolean;
  intervalBased: boolean;
}

export interface ChallengeTotals {
  daysSurvived: number;
  workouts: number;
  avgSleepMin: number | null;
  weightDeltaKg: number | null;
  workHours: number;
}

export interface BrokenStreak {
  dayNumber: number;
  date: string;
  weekdayLabel: string;
  failedPillars: string[];
  totals: ChallengeTotals;
  nextAttempt: number;
  message: CoachMessage;
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

// ---------------------------------------------------- estadísticas (Fase 2)

export type Range = "7" | "30" | "all";

export interface SleepPoint {
  date: string;
  label: string;
  minutes: number | null;
}

export interface HistBucket {
  label: string;
  count: number;
}

export interface XY {
  x: number;
  y: number;
}

export interface SleepStats {
  goalMin: number;
  avgMin: number | null;
  bestMin: number | null;
  worstMin: number | null;
  balanceMin: number;
  nights: number;
  daily: SleepPoint[];
  movingAvg: (number | null)[];
  bedtimes: HistBucket[];
  modalBedtime: string | null;
  sleepVsEnergy: XY[];
  correlation: number | null;
}

export interface WeekBar {
  label: string;
  indoorMin: number;
  outdoorMin: number;
}

export interface CaloriesDay {
  date: string;
  label: string;
  intake: number;
  burned: number;
}

export interface KindBar {
  label: string;
  minutes: number;
}

export interface WorkoutStats {
  sessions: number;
  totalMin: number;
  weeklyAvgMin: number;
  daysWithout: number;
  weeklyGoalMin: number;
  weekly: WeekBar[];
  calories: CaloriesDay[];
  byKind: KindBar[];
}

export interface GlucosePoint {
  id: string;
  date: string;
  time: string;
  value: number;
  context: string;
  contextLabel: string;
  meal: string | null;
  notes: string | null;
  outOfRange: boolean;
}

export interface ContextAvg {
  context: string;
  label: string;
  avg: number;
  count: number;
}

export interface GlucoseStats {
  readings: GlucosePoint[];
  byContext: ContextAvg[];
  total: number;
  outOfRange: number;
  avgFasting: number | null;
  avgPostMeal: number | null;
}

export interface WeightPoint {
  date: string;
  label: string;
  kg: number | null;
}

export interface WorkDay {
  date: string;
  label: string;
  values: number[];
}

export interface BodyStats {
  currentKg: number | null;
  deltaKg: number | null;
  weeklyDeltaKg: number | null;
  points: WeightPoint[];
  movingAvg: (number | null)[];
  categories: string[];
  workDaily: WorkDay[];
  workHoursWeek: number;
  workAvgDailyH: number;
  workGoalMin: number;
}

export interface Correlation {
  key: string;
  label: string;
  caption: string;
  xLabel: string;
  yLabel: string;
  r: number | null;
  n: number;
  points: XY[];
}

// -------------------------------------------------------- Fase 3

export interface BeforeAfter {
  label: string;
  before: number | null;
  after: number | null;
  unit: string;
  better: "up" | "down" | "none";
}

export interface Completion {
  name: string;
  startDate: string;
  endDate: string;
  targetDays: number;
  completeDays: number;
  finished: boolean;
  tiles: BeforeAfter[];
  heatmap: HeatmapCell[];
  weight: (number | null)[];
  sleep: (number | null)[];
  glucose: (number | null)[];
}

export interface ProgressPhoto {
  id: string;
  date: string;
  weekdayLabel: string;
  dayNumber: number | null;
}

export interface BackupFile {
  name: string;
  path: string;
  sizeKb: number;
}

export interface DoctorReport {
  generatedAt: string;
  from: string;
  to: string;
  fromLabel: string;
  toLabel: string;
  glucose: GlucoseStats;
  weightStart: number | null;
  weightEnd: number | null;
  avgSleepMin: number | null;
  mealsPerDay: number;
  workouts: number;
  days: number;
}
