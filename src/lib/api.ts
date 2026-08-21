// Envoltorios tipados de invoke(). Todo error del backend llega aquí como
// string legible en español; la UI lo muestra tal cual.

import { invoke } from "@tauri-apps/api/core";
import type {
  Bootstrap,
  BrokenStreak,
  Challenge,
  CoachMessage,
  DayDetail,
  History,
  Reminder,
  MissingAction,
  MissingDay,
  Rules,
  SleepPreview,
  TodayView,
  BodyStats,
  Completion,
  Correlation,
  DoctorReport,
  GlucoseStats,
  Range,
  SleepStats,
  WorkoutStats,
  BackupFile,
  ProgressPhoto,
} from "./types";

export class ApiError extends Error {}

async function call<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    throw new ApiError(typeof e === "string" ? e : "Ocurrió un error inesperado.");
  }
}

// ------------------------------------------------------------------ arranque

export const bootstrap = () => call<Bootstrap>("bootstrap");

export const createChallenge = (args: {
  name?: string;
  startDate: string;
  targetDays?: number;
  rules?: Rules;
}) => call<Challenge>("create_challenge", args);

export const endChallenge = (args: {
  reason: "completed" | "failed" | "abandoned";
  brokenOnDay?: number;
  detail?: string;
}) => call<CoachMessage>("end_challenge", args);

// ---------------------------------------------------------------------- día

export const getToday = (date?: string) => call<TodayView>("get_today", { date });
export const getDayDetail = (date: string) => call<DayDetail>("get_day_detail", { date });
export const getHistory = (limit?: number) => call<History>("get_history", { limit });

export const closeDay = (args: {
  date?: string;
  status: "complete" | "failed" | "skipped";
  notes?: string;
}) => call<TodayView>("close_day", args);

export const reopenDay = (date: string) => call<DayDetail>("reopen_day", { date });
export const setDayNotes = (date: string, notes: string | null) =>
  call<void>("set_day_notes", { date, notes });

export const deleteEntry = (kind: "meal" | "workout" | "glucose" | "reading" | "work", id: string) =>
  call<void>("delete_entry", { kind, id });

// ------------------------------------------------------- racha rota (P14)

export const getBrokenStreak = (date?: string) =>
  call<BrokenStreak>("get_broken_streak", { date });

// ------------------------------------------------------------ recordatorios

export const getReminders = () => call<Reminder[]>("get_reminders");

export const setReminder = (args: {
  id: string;
  enabled?: boolean;
  timeOfDay?: string;
  daysMask?: number;
  intervalDays?: number;
  title?: string;
  message?: string;
}) => call<Reminder[]>("set_reminder", args);

export const addReminder = (args: {
  title: string;
  message?: string;
  timeOfDay: string;
  daysMask: number;
  intervalDays?: number;
}) => call<Reminder[]>("add_reminder", args);

export const deleteReminder = (id: string) => call<Reminder[]>("delete_reminder", { id });

// --------------------------------------------------------- días sin registrar

export const getMissingDays = () => call<MissingDay[]>("get_missing_days");
export const applyMissingDays = (decisions: { date: string; action: MissingAction }[]) =>
  call<string[]>("apply_missing_days", { decisions });

// -------------------------------------------------------------------- sueño

export const previewSleep = (args: { date?: string; bedtime: string; wakeTime: string }) =>
  call<SleepPreview>("preview_sleep", args);

export const saveMorningCheckin = (args: {
  date?: string;
  bedtime: string;
  wakeTime: string;
  quality?: number;
  weightKg?: number;
  mood?: number;
  energy?: number;
}) => call<TodayView>("save_morning_checkin", args);

// ---------------------------------------------------------------- registros

export const addMeal = (args: {
  date?: string;
  time?: string;
  kind: "meal" | "snack";
  description: string;
  calories?: number;
}) => call<string>("add_meal", args);

export const addWorkout = (args: {
  date?: string;
  time?: string;
  kind: string;
  description?: string;
  durationMin: number;
  isOutdoor?: boolean;
  caloriesBurned?: number;
}) => call<string>("add_workout", args);

export const addGlucose = (args: {
  date?: string;
  time?: string;
  valueMgdl: number;
  context: string;
  notes?: string;
}) => call<string>("add_glucose", args);

export const addWater = (ml: number, date?: string) => call<number>("add_water", { date, ml });
export const setWater = (ml: number, date?: string) => call<void>("set_water", { date, ml });

export const addReading = (args: { date?: string; pages: number; book?: string }) =>
  call<string>("add_reading", args);

export const addWorkSession = (args: {
  date?: string;
  time?: string;
  minutes: number;
  category: string;
  description?: string;
}) => call<string>("add_work_session", args);

export const setWeight = (kg: number, date?: string) => call<void>("set_weight", { date, kg });
export const setMood = (args: { date?: string; mood: number; energy: number; notes?: string }) =>
  call<void>("set_mood", args);

// ----------------------------------------------------------------- ajustes

export const getSettings = () => call<Record<string, string>>("get_settings");
export const setSettings = (entries: Record<string, string>) =>
  call<Record<string, string>>("set_settings", { entries });

export const setAutostart = (enabled: boolean) => call<boolean>("set_autostart", { enabled });
export const isAutostartEnabled = () => call<boolean>("is_autostart_enabled");

// ----------------------------------------------------------- estadísticas

export const getSleepStats = (range: Range) => call<SleepStats>("get_sleep_stats", { range });
export const getWorkoutStats = (range: Range) =>
  call<WorkoutStats>("get_workout_stats", { range });
export const getGlucoseStats = (range: Range) =>
  call<GlucoseStats>("get_glucose_stats", { range });
export const getBodyStats = (range: Range) => call<BodyStats>("get_body_stats", { range });
export const getCorrelations = () => call<Correlation[]>("get_correlations");
export const getCompletion = () => call<Completion | null>("get_completion");
export const getDoctorReport = (days?: number) =>
  call<DoctorReport>("get_doctor_report", { days });

export const exportData = (format: "csv" | "json", path: string) =>
  call<string>("export_data", { format, path });

// ------------------------------------------------- fotos de progreso (F3)

export const addProgressPhoto = (date: string, source: string) =>
  call<string>("add_progress_photo", { date, source });
export const listProgressPhotos = () => call<ProgressPhoto[]>("list_progress_photos");
export const readProgressPhoto = (id: string) => call<string>("read_progress_photo", { id });
export const deleteProgressPhoto = (id: string) => call<void>("delete_progress_photo", { id });

// ----------------------------------------------------- copias de seguridad

export const backupNow = () => call<string>("backup_now");
export const listBackups = () => call<BackupFile[]>("list_backups");
export const createEncryptedBackup = (path: string, passphrase: string) =>
  call<string>("create_encrypted_backup", { path, passphrase });
export const restoreEncryptedBackup = (path: string, passphrase: string) =>
  call<string>("restore_encrypted_backup", { path, passphrase });

// ---------------------------------------------------------------- ventanas

export const openQuickEntry = () => call<void>("open_quick_entry");
export const closeWindow = () => call<void>("close_window");
