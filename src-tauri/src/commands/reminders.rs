//! Recordatorios programados (Fase 1). El planificador vive en `scheduler.rs`;
//! aquí solo se leen y se editan.

use tauri::State;

use crate::db::models::Reminder;
use crate::db::queries;
use crate::db::Db;
use crate::error::AppResult;

#[tauri::command]
pub fn get_reminders(db: State<Db>) -> AppResult<Vec<Reminder>> {
    db.with(queries::reminders)
}

#[tauri::command]
pub fn set_reminder(
    db: State<Db>,
    id: String,
    enabled: Option<bool>,
    time_of_day: Option<String>,
) -> AppResult<Vec<Reminder>> {
    db.tx(|tx| {
        queries::set_reminder(tx, &id, enabled, time_of_day.as_deref())?;
        queries::reminders(tx)
    })
}
