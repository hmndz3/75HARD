//! Tipos que cruzan la frontera Rust → UI. Todo sale en camelCase para que el
//! lado TypeScript no tenga que renombrar nada.

use serde::{Deserialize, Serialize};

use crate::coach::CoachMessage;

// ---------------------------------------------------------------- reto

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pillar {
    pub key: String,
    pub label: String,
    /// Si falla un pilar obligatorio, el día no puede cerrarse como completo.
    /// Los opcionales solo suman estadísticas.
    pub required: bool,
    /// Meta numérica del pilar cuando aplica (minutos, ml, páginas).
    pub goal: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rules {
    pub pillars: Vec<Pillar>,
}

impl Default for Rules {
    fn default() -> Self {
        let p = |key: &str, label: &str, required: bool, goal: Option<i64>| Pillar {
            key: key.to_string(),
            label: label.to_string(),
            required,
            goal,
        };
        Self {
            pillars: vec![
                p("sleep", "Dormir 7 h", true, Some(420)),
                p("workout", "Ejercicio 45 min", true, Some(45)),
                p(
                    "outdoor",
                    "Segundo ejercicio al aire libre",
                    false,
                    Some(45),
                ),
                p("diet", "Dieta limpia", true, None),
                p("water", "3 L de agua", true, Some(3000)),
                p("reading", "Leer 10 páginas", true, Some(10)),
                p("glucose", "Registrar glucosa", false, Some(1)),
                p("photo", "Foto de progreso", false, None),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Challenge {
    pub id: String,
    pub name: String,
    pub start_date: String,
    pub target_days: i64,
    pub rules: Rules,
    pub ended_at: Option<String>,
    pub ended_reason: Option<String>,
}

// ---------------------------------------------------------------- registros

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SleepLog {
    pub date: String,
    pub bedtime: String,
    pub wake_time: String,
    pub minutes: i64,
    pub quality: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meal {
    pub id: String,
    pub date: String,
    pub eaten_at: String,
    pub kind: String,
    pub description: String,
    pub calories: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workout {
    pub id: String,
    pub date: String,
    pub started_at: String,
    pub kind: String,
    pub description: Option<String>,
    pub duration_min: i64,
    pub is_outdoor: bool,
    pub calories_burned: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlucoseReading {
    pub id: String,
    pub date: String,
    pub measured_at: String,
    pub value_mgdl: i64,
    pub context: String,
    pub notes: Option<String>,
    pub linked_meal_id: Option<String>,
    pub linked_meal_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkSession {
    pub id: String,
    pub date: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub minutes: i64,
    pub category: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingLog {
    pub id: String,
    pub date: String,
    pub pages: i64,
    pub book: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoodLog {
    pub mood: i64,
    pub energy: i64,
    pub stress: Option<i64>,
}

// ---------------------------------------------------------------- vistas

/// Estado de un pilar en un día concreto. La UI lo pinta con icono + texto,
/// nunca con color solo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PillarState {
    pub key: String,
    pub label: String,
    pub required: bool,
    /// "done" | "partial" | "missing"
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayView {
    pub date: String,
    pub weekday_label: String,
    pub day_number: Option<i64>,
    pub target_days: i64,
    pub status: String,
    pub streak: i64,
    pub longest_streak: i64,
    /// Últimos 30 días como "complete" | "failed" | "skipped" | "pending" | "empty",
    /// para la tira sparkline de la tarjeta de racha.
    pub recent: Vec<String>,

    pub sleep: Option<SleepLog>,
    pub sleep_goal_min: i64,
    pub sleep_verdict: Option<CoachMessage>,

    pub meals_count: i64,
    pub snacks_count: i64,
    pub calories_in: Option<i64>,
    pub workout_min: i64,
    pub workout_kcal: Option<i64>,
    pub water_ml: i64,
    pub water_goal_ml: i64,
    pub reading_pages: i64,
    pub reading_goal_pages: i64,
    pub glucose_count: i64,
    pub work_min: i64,
    pub weight_kg: Option<f64>,
    pub mood: Option<MoodLog>,

    pub pillars: Vec<PillarState>,
    pub coach: CoachMessage,
    /// Horas hasta el corte del día siguiente.
    pub hours_left: i64,
    pub morning_done: bool,
    pub evening_done: bool,
}

/// Una fila de la tabla de Historial.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayRow {
    pub date: String,
    pub day_number: Option<i64>,
    pub weekday_label: String,
    pub status: String,
    pub sleep_minutes: Option<i64>,
    pub workout_min: i64,
    pub meals_count: i64,
    pub water_ml: i64,
    pub glucose_count: i64,
    pub weight_kg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistorySummary {
    pub complete: i64,
    pub partial: i64,
    pub failed: i64,
    pub skipped: i64,
}

/// Todo lo registrado en un día (P6 — Detalle del día).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayDetail {
    pub date: String,
    pub weekday_label: String,
    pub day_number: Option<i64>,
    pub status: String,
    pub notes: Option<String>,
    pub sleep: Option<SleepLog>,
    pub meals: Vec<Meal>,
    pub workouts: Vec<Workout>,
    pub glucose: Vec<GlucoseReading>,
    pub work: Vec<WorkSession>,
    pub reading: Vec<ReadingLog>,
    pub water_ml: i64,
    pub weight_kg: Option<f64>,
    pub mood: Option<MoodLog>,
}

/// Una celda del heatmap de 75 días (P5). El estado incluye dos valores que no
/// existen en la tabla `day`: "future" para los días que aún no llegan y
/// "partial" para los que tienen registros pero nunca se cerraron.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatmapCell {
    pub date: String,
    pub day_number: i64,
    pub status: String,
}

/// Un recordatorio programado (Fase 1). `kind` manda: de él salen la etiqueta,
/// el texto de la notificación y la condición para no molestar de más.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub description: String,
    pub time_of_day: String,
    pub enabled: bool,
    /// Los de intervalo por horas (agua) no se programan a una hora fija.
    pub interval_based: bool,
    /// Los creados por el usuario se pueden renombrar y borrar.
    pub custom: bool,
    /// Un bit por día de la semana, bit 0 = lunes. 127 son todos.
    pub days_mask: i64,
    /// Cada cuántos días suena. 0 o 1 = todos los que permita la máscara.
    pub interval_days: i64,
    /// Resumen para la UI: "daily" | "alternate" | "weekdays".
    pub repeat: String,
}

/// Las cifras del intento para la pantalla de racha rota (P14).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeTotals {
    pub days_survived: i64,
    pub workouts: i64,
    pub avg_sleep_min: Option<i64>,
    pub weight_delta_kg: Option<f64>,
    pub work_hours: f64,
}

/// Todo lo que necesita la pantalla de racha rota (P14).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokenStreak {
    pub day_number: i64,
    pub date: String,
    pub weekday_label: String,
    /// Pilares obligatorios que se fallaron ese día, ya legibles.
    pub failed_pillars: Vec<String>,
    pub totals: ChallengeTotals,
    pub next_attempt: i64,
    pub message: CoachMessage,
}

/// Una foto de progreso (Fase 3). El archivo vive fuera de la base, en
/// `%APPDATA%\75hard\photos`; aquí solo va la referencia.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressPhoto {
    pub id: String,
    pub date: String,
    pub weekday_label: String,
    pub day_number: Option<i64>,
}

/// Un día calendario que pasó sin que se abriera la app (P13).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissingDay {
    pub date: String,
    pub weekday_label: String,
    pub day_number: Option<i64>,
}
