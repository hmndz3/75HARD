use tauri::State;

use crate::commands::{instant_in_day, now_time, parse_time, resolve_date};
use crate::db::queries;
use crate::db::Db;
use crate::error::AppResult;

/// La app registra y grafica. No diagnostica, no sugiere dosis, no interpreta.
/// Por eso aquí no hay ninguna evaluación del valor más allá de validar el rango.
#[tauri::command]
pub fn add_glucose(
    db: State<Db>,
    date: Option<String>,
    time: Option<String>,
    value_mgdl: i64,
    context: String,
    notes: Option<String>,
) -> AppResult<String> {
    db.with(|c| {
        let d = resolve_date(c, date)?;
        let t = match time {
            Some(s) => parse_time(&s)?,
            None => now_time(),
        };
        let at = instant_in_day(d, t, queries::cutoff_hour(c));
        queries::add_glucose(c, d, at, value_mgdl, &context, notes.as_deref())
    })
}
