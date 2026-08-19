use std::collections::BTreeMap;

use serde::Serialize;
use tauri::State;
use tauri_plugin_autostart::ManagerExt;

use crate::coach;
use crate::db::models::{Challenge, Rules};
use crate::db::queries;
use crate::db::Db;
use crate::error::{AppError, AppResult};

/// Lo que la UI necesita en el primer render, en una sola llamada.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub needs_onboarding: bool,
    pub challenge: Option<Challenge>,
    pub settings: BTreeMap<String, String>,
    pub today: String,
    pub attempt_number: i64,
    pub default_rules: Rules,
    pub db_path: String,
}

#[tauri::command]
pub fn bootstrap(db: State<Db>) -> AppResult<Bootstrap> {
    db.with(|c| {
        let challenge = queries::active_challenge(c)?;
        Ok(Bootstrap {
            needs_onboarding: challenge.is_none(),
            challenge,
            settings: queries::all_settings(c)?,
            today: queries::today(c).format(queries::DATE_FMT).to_string(),
            attempt_number: queries::challenge_count(c)? + 1,
            default_rules: Rules::default(),
            db_path: db.path.display().to_string(),
        })
    })
}

#[tauri::command]
pub fn get_settings(db: State<Db>) -> AppResult<BTreeMap<String, String>> {
    db.with(queries::all_settings)
}

#[tauri::command]
pub fn set_settings(
    db: State<Db>,
    entries: BTreeMap<String, String>,
) -> AppResult<BTreeMap<String, String>> {
    db.tx(|tx| {
        for (key, value) in &entries {
            validate_setting(key, value)?;
            queries::set_setting(tx, key, value)?;
        }
        queries::all_settings(tx)
    })
}

fn validate_setting(key: &str, value: &str) -> AppResult<()> {
    let positive_int = |max: i64| -> AppResult<()> {
        match value.parse::<i64>() {
            Ok(n) if n > 0 && n <= max => Ok(()),
            _ => Err(AppError::invalid(format!(
                "{key}: se esperaba un número entre 1 y {max}"
            ))),
        }
    };
    match key {
        "coach_tone" => {
            if !matches!(value, "suave" | "directo" | "duro") {
                return Err(AppError::invalid("el tono es suave, directo o duro"));
            }
            Ok(())
        }
        "sleep_goal_min" => positive_int(16 * 60),
        "water_goal_ml" => positive_int(10_000),
        "workout_goal_min" => positive_int(600),
        "reading_goal_pages" => positive_int(1000),
        "day_cutoff_hour" => match value.parse::<i64>() {
            Ok(n) if (0..=12).contains(&n) => Ok(()),
            _ => Err(AppError::invalid(
                "el corte del día va de 0 a 12 (por defecto 4)",
            )),
        },
        // Claves libres (last_open_date, autostart, etc.)
        _ => Ok(()),
    }
}

#[tauri::command]
pub fn get_challenge(db: State<Db>) -> AppResult<Option<Challenge>> {
    db.with(queries::active_challenge)
}

#[tauri::command]
pub fn create_challenge(
    db: State<Db>,
    name: Option<String>,
    start_date: String,
    target_days: Option<i64>,
    rules: Option<Rules>,
) -> AppResult<Challenge> {
    db.tx(|tx| {
        let attempt = queries::challenge_count(tx)? + 1;
        let name = name
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| format!("Intento #{attempt}"));
        let start = queries::parse_date(&start_date)?;
        let challenge = queries::create_challenge(
            tx,
            &name,
            start,
            target_days.unwrap_or(75),
            &rules.unwrap_or_default(),
        )?;
        // Desde el arranque del reto no hay días "sin registrar" que reclamar.
        queries::touch_last_open(tx, start)?;
        Ok(challenge)
    })
}

/// Cierra el intento actual. Devuelve el mensaje del coach para la pantalla de
/// racha rota (P14) cuando el motivo es un fallo.
#[tauri::command]
pub fn end_challenge(
    db: State<Db>,
    reason: String,
    broken_on_day: Option<i64>,
    detail: Option<String>,
) -> AppResult<coach::CoachMessage> {
    if !matches!(reason.as_str(), "completed" | "failed" | "abandoned") {
        return Err(AppError::invalid("motivo desconocido"));
    }
    db.tx(|tx| {
        let challenge = queries::active_challenge(tx)?
            .ok_or_else(|| AppError::NotFound("no hay un reto activo".into()))?;
        queries::end_challenge(tx, &challenge.id, &reason)?;
        let attempt = queries::challenge_count(tx)? + 1;
        Ok(coach::streak_broken_message(
            broken_on_day.unwrap_or(0),
            attempt,
            detail.as_deref().unwrap_or(""),
        ))
    })
}

#[tauri::command]
pub fn set_autostart(app: tauri::AppHandle, db: State<Db>, enabled: bool) -> AppResult<bool> {
    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    result.map_err(|e| AppError::Internal(format!("no se pudo cambiar el autoarranque: {e}")))?;

    let actual = manager.is_enabled().unwrap_or(enabled);
    db.with(|c| queries::set_setting(c, "autostart", if actual { "1" } else { "0" }))?;
    Ok(actual)
}

#[tauri::command]
pub fn is_autostart_enabled(app: tauri::AppHandle) -> AppResult<bool> {
    app.autolaunch()
        .is_enabled()
        .map_err(|e| AppError::Internal(format!("no se pudo leer el autoarranque: {e}")))
}
