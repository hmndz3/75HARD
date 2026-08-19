//! Registros sueltos: agua, lectura, trabajo, peso, ánimo.

use tauri::State;

use crate::commands::{instant_in_day, now_time, parse_time, resolve_date};
use crate::db::queries;
use crate::db::Db;
use crate::error::AppResult;

#[tauri::command]
pub fn add_water(db: State<Db>, date: Option<String>, ml: i64) -> AppResult<i64> {
    db.with(|c| {
        let d = resolve_date(c, date)?;
        queries::add_water(c, d, ml)
    })
}

#[tauri::command]
pub fn set_water(db: State<Db>, date: Option<String>, ml: i64) -> AppResult<()> {
    db.with(|c| {
        let d = resolve_date(c, date)?;
        queries::set_water(c, d, ml)
    })
}

#[tauri::command]
pub fn add_reading(
    db: State<Db>,
    date: Option<String>,
    pages: i64,
    book: Option<String>,
) -> AppResult<String> {
    db.with(|c| {
        let d = resolve_date(c, date)?;
        queries::add_reading(c, d, pages, book.as_deref())
    })
}

#[tauri::command]
pub fn add_work_session(
    db: State<Db>,
    date: Option<String>,
    time: Option<String>,
    minutes: i64,
    category: String,
    description: Option<String>,
) -> AppResult<String> {
    db.with(|c| {
        let d = resolve_date(c, date)?;
        let t = match time {
            Some(s) => parse_time(&s)?,
            None => now_time(),
        };
        let at = instant_in_day(d, t, queries::cutoff_hour(c));
        queries::add_work_session(c, d, at, minutes, &category, description.as_deref())
    })
}

#[tauri::command]
pub fn set_weight(db: State<Db>, date: Option<String>, kg: f64) -> AppResult<()> {
    db.with(|c| {
        let d = resolve_date(c, date)?;
        queries::set_weight(c, d, kg)
    })
}

#[tauri::command]
pub fn set_mood(
    db: State<Db>,
    date: Option<String>,
    mood: i64,
    energy: i64,
    notes: Option<String>,
) -> AppResult<()> {
    db.with(|c| {
        let d = resolve_date(c, date)?;
        queries::set_mood(c, d, mood, energy, notes.as_deref())
    })
}
