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
export const setReminder = (args: { id: string; enabled?: boolean; timeOfDay?: string }) =>
  call<Reminder[]>("set_reminder", args);

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

// ---------------------------------------------------------------- ventanas

export const openQuickEntry = () => call<void>("open_quick_entry");
export const closeWindow = () => call<void>("close_window");
