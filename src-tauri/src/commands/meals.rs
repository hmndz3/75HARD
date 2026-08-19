use tauri::State;

use crate::commands::{instant_in_day, now_time, parse_time, resolve_date};
use crate::db::queries;
use crate::db::Db;
use crate::error::AppResult;

/// Registro rápido de comida o snack. Las calorías son SIEMPRE opcionales y la
/// app no emite ningún juicio sobre ellas.
#[tauri::command]
pub fn add_meal(
    db: State<Db>,
    date: Option<String>,
    time: Option<String>,
    kind: String,
    description: String,
    calories: Option<i64>,
) -> AppResult<String> {
    db.with(|c| {
        let d = resolve_date(c, date)?;
        let t = match time {
            Some(s) => parse_time(&s)?,
            None => now_time(),
        };
        let at = instant_in_day(d, t, queries::cutoff_hour(c));
        queries::add_meal(c, d, at, &kind, &description, calories)
    })
}
