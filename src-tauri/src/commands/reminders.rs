//! Recordatorios programados. El planificador vive en `scheduler.rs`; aquí
//! solo se leen, se editan, se crean y se borran.

use tauri::State;

use crate::db::models::Reminder;
use crate::db::queries::{self, ReminderPatch};
use crate::db::Db;
use crate::error::AppResult;

#[tauri::command]
pub fn get_reminders(db: State<Db>) -> AppResult<Vec<Reminder>> {
    db.with(queries::reminders)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn set_reminder(
    db: State<Db>,
    id: String,
    enabled: Option<bool>,
    time_of_day: Option<String>,
    days_mask: Option<i64>,
    interval_days: Option<i64>,
    title: Option<String>,
    message: Option<String>,
) -> AppResult<Vec<Reminder>> {
    db.tx(|tx| {
        queries::set_reminder(
            tx,
            &id,
            ReminderPatch {
                enabled,
                time_of_day: time_of_day.as_deref(),
                days_mask,
                interval_days,
                title: title.as_deref(),
                message: message.as_deref(),
            },
        )?;
        queries::reminders(tx)
    })
}

/// Crea un recordatorio propio, además de los cinco de fábrica.
#[tauri::command]
pub fn add_reminder(
    db: State<Db>,
    title: String,
    message: Option<String>,
    time_of_day: String,
    days_mask: i64,
    interval_days: Option<i64>,
) -> AppResult<Vec<Reminder>> {
    db.tx(|tx| {
        queries::add_reminder(
            tx,
            &title,
            message.as_deref(),
            &time_of_day,
            days_mask,
            interval_days.unwrap_or(0),
        )?;
        queries::reminders(tx)
    })
}

#[tauri::command]
pub fn delete_reminder(db: State<Db>, id: String) -> AppResult<Vec<Reminder>> {
    db.tx(|tx| {
        queries::delete_reminder(tx, &id)?;
        queries::reminders(tx)
    })
}
