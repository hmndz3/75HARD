use serde::Serialize;
use tauri::State;

use crate::coach::{self, CoachMessage, SleepFacts};
use crate::commands::{parse_time, resolve_date};
use crate::daycut;
use crate::db::models::TodayView;
use crate::db::queries;
use crate::db::Db;
use crate::error::AppResult;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SleepPreview {
    pub minutes: i64,
    pub label: String,
    pub verdict: CoachMessage,
    pub goal_min: i64,
}

/// Cálculo en vivo mientras se escriben las horas en el check-in matutino.
/// No escribe nada; solo devuelve el veredicto para pintarlo bajo los inputs.
#[tauri::command]
pub fn preview_sleep(
    db: State<Db>,
    date: Option<String>,
    bedtime: String,
    wake_time: String,
) -> AppResult<SleepPreview> {
    db.with(|c| {
        let d = resolve_date(c, date)?;
        let (bed, wake) =
            daycut::resolve_sleep_window(d, parse_time(&bedtime)?, parse_time(&wake_time)?);
        let minutes = daycut::sleep_minutes(bed, wake);
        let goal_min = queries::get_int(c, "sleep_goal_min", 420);

        // La noche de anoche todavía no está guardada: se cuenta como parte de
        // la racha de noches cortas si lo es.
        let previous = queries::short_nights_streak(c, d - chrono::Duration::days(1))?;
        let short_nights_streak = if minutes < 360 { previous + 1 } else { 0 };

        Ok(SleepPreview {
            minutes,
            label: daycut::format_minutes(minutes),
            verdict: coach::sleep_verdict(
                SleepFacts {
                    minutes,
                    goal_min,
                    short_nights_streak,
                },
                queries::tone(c),
            ),
            goal_min,
        })
    })
}

/// Check-in matutino completo (P2): sueño + peso opcional + ánimo y energía.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn save_morning_checkin(
    db: State<Db>,
    date: Option<String>,
    bedtime: String,
    wake_time: String,
    quality: Option<i64>,
    weight_kg: Option<f64>,
    mood: Option<i64>,
    energy: Option<i64>,
) -> AppResult<TodayView> {
    db.tx(|tx| {
        let d = resolve_date(tx, date)?;
        let (bed, wake) =
            daycut::resolve_sleep_window(d, parse_time(&bedtime)?, parse_time(&wake_time)?);

        queries::save_sleep(tx, d, bed, wake, quality)?;

        if let Some(kg) = weight_kg {
            queries::set_weight(tx, d, kg)?;
        }
        if let (Some(m), Some(e)) = (mood, energy) {
            queries::set_mood(tx, d, m, e, None)?;
        }
        queries::mark_checkin(tx, d, "morning")?;
        queries::today_view(tx, d)
    })
}
