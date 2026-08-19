use tauri::State;

use crate::commands::{instant_in_day, now_time, parse_time, resolve_date};
use crate::db::queries;
use crate::db::Db;
use crate::error::AppResult;

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn add_workout(
    db: State<Db>,
    date: Option<String>,
    time: Option<String>,
    kind: String,
    description: Option<String>,
    duration_min: i64,
    is_outdoor: Option<bool>,
    calories_burned: Option<i64>,
) -> AppResult<String> {
    db.with(|c| {
        let d = resolve_date(c, date)?;
        let t = match time {
            Some(s) => parse_time(&s)?,
            None => now_time(),
        };
        let at = instant_in_day(d, t, queries::cutoff_hour(c));
        // "outdoor" como tipo implica al aire libre aunque no marquen la casilla.
        let outdoor = is_outdoor.unwrap_or(false) || kind == "outdoor";
        queries::add_workout(
            c,
            d,
            at,
            &kind,
            description.as_deref(),
            duration_min,
            outdoor,
            calories_burned,
        )
    })
}
