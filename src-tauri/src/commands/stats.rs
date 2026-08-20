//! Comandos de las pantallas de estadísticas (Fase 2) y de los extras de la
//! Fase 3. Toda la agregación vive en `db::stats`.

use tauri::State;

use crate::db::stats::{
    self, BodyStats, Completion, Correlation, DoctorReport, GlucoseStats, SleepStats, WorkoutStats,
};
use crate::db::Db;
use crate::error::AppResult;

/// Rango del eje temporal: "7", "30" o "all".
fn rango(v: Option<String>) -> String {
    v.unwrap_or_else(|| "30".to_string())
}

#[tauri::command]
pub fn get_sleep_stats(db: State<Db>, range: Option<String>) -> AppResult<SleepStats> {
    db.with(|c| stats::sleep_stats(c, &rango(range)))
}

#[tauri::command]
pub fn get_workout_stats(db: State<Db>, range: Option<String>) -> AppResult<WorkoutStats> {
    db.with(|c| stats::workout_stats(c, &rango(range)))
}

#[tauri::command]
pub fn get_glucose_stats(db: State<Db>, range: Option<String>) -> AppResult<GlucoseStats> {
    db.with(|c| stats::glucose_stats(c, &rango(range)))
}

#[tauri::command]
pub fn get_body_stats(db: State<Db>, range: Option<String>) -> AppResult<BodyStats> {
    db.with(|c| stats::body_stats(c, &rango(range)))
}

#[tauri::command]
pub fn get_correlations(db: State<Db>) -> AppResult<Vec<Correlation>> {
    db.with(stats::correlations)
}

#[tauri::command]
pub fn get_completion(db: State<Db>) -> AppResult<Option<Completion>> {
    db.with(stats::completion)
}

#[tauri::command]
pub fn get_doctor_report(db: State<Db>, days: Option<i64>) -> AppResult<DoctorReport> {
    db.with(|c| stats::doctor_report(c, days.unwrap_or(30)))
}
