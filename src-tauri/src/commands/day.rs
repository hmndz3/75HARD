use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::resolve_date;
use crate::db::models::{
    BrokenStreak, DayDetail, DayRow, HeatmapCell, HistorySummary, MissingDay, TodayView,
};
use crate::db::queries;
use crate::db::Db;
use crate::error::{AppError, AppResult};

#[tauri::command]
pub fn get_today(db: State<Db>, date: Option<String>) -> AppResult<TodayView> {
    db.with(|c| {
        let d = resolve_date(c, date)?;
        queries::today_view(c, d)
    })
}

#[tauri::command]
pub fn get_day_detail(db: State<Db>, date: String) -> AppResult<DayDetail> {
    db.with(|c| {
        let d = queries::parse_date(&date)?;
        queries::day_detail(c, d)
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct History {
    pub rows: Vec<DayRow>,
    pub summary: HistorySummary,
    /// Las celdas del reto completo, para el heatmap de arriba (P5).
    pub heatmap: Vec<HeatmapCell>,
}

#[tauri::command]
pub fn get_history(db: State<Db>, limit: Option<i64>) -> AppResult<History> {
    db.with(|c| {
        Ok(History {
            rows: queries::day_rows(c, limit.unwrap_or(120).clamp(1, 1000))?,
            summary: queries::history_summary(c)?,
            heatmap: queries::heatmap(c)?,
        })
    })
}

/// Cierra el día como completo o fallido (P3). Devuelve la vista actualizada
/// para que la pantalla "Hoy" se repinte sin una segunda llamada.
#[tauri::command]
pub fn close_day(
    db: State<Db>,
    date: Option<String>,
    status: String,
    notes: Option<String>,
) -> AppResult<TodayView> {
    if !matches!(status.as_str(), "complete" | "failed" | "skipped") {
        return Err(AppError::invalid(
            "un día solo se puede cerrar como completo, fallido o pausado",
        ));
    }
    db.with(|c| {
        let d = resolve_date(c, date)?;
        queries::set_day_status(c, d, &status)?;
        if let Some(n) = notes.as_deref() {
            let trimmed = n.trim();
            queries::set_day_notes(c, d, (!trimmed.is_empty()).then_some(trimmed))?;
        }
        queries::mark_checkin(c, d, "evening")?;
        queries::today_view(c, d)
    })
}

/// Reabre un día ya cerrado para editarlo desde Historial.
#[tauri::command]
pub fn reopen_day(db: State<Db>, date: String) -> AppResult<DayDetail> {
    db.with(|c| {
        let d = queries::parse_date(&date)?;
        queries::set_day_status(c, d, "pending")?;
        queries::day_detail(c, d)
    })
}

#[tauri::command]
pub fn set_day_notes(db: State<Db>, date: String, notes: Option<String>) -> AppResult<()> {
    db.with(|c| {
        let d = queries::parse_date(&date)?;
        let trimmed = notes.as_deref().map(str::trim).filter(|s| !s.is_empty());
        queries::set_day_notes(c, d, trimmed)
    })
}

#[tauri::command]
pub fn delete_entry(db: State<Db>, kind: String, id: String) -> AppResult<()> {
    db.with(|c| queries::delete_entry(c, &kind, &id))
}

// ------------------------------------------------- días sin registrar (P13)

#[tauri::command]
pub fn get_missing_days(db: State<Db>) -> AppResult<Vec<MissingDay>> {
    db.with(|c| {
        let today = queries::today(c);
        let missing = queries::missing_days(c, today)?;
        // Si no hay nada que decidir, el arranque de hoy queda registrado y la
        // próxima vez solo se preguntará por días realmente nuevos.
        if missing.is_empty() {
            queries::touch_last_open(c, today)?;
        }
        Ok(missing)
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissingDayDecision {
    pub date: String,
    /// "fill" (llenar ahora) · "failed" (marcar fallido) · "empty" (dejar vacío)
    pub action: String,
}

#[tauri::command]
pub fn apply_missing_days(
    db: State<Db>,
    decisions: Vec<MissingDayDecision>,
) -> AppResult<Vec<String>> {
    db.tx(|tx| {
        let mut to_fill = Vec::new();
        for decision in &decisions {
            let d = queries::parse_date(&decision.date)?;
            match decision.action.as_str() {
                // "Llenar ahora" solo crea la fila; el usuario la edita en
                // Historial y ahí decide si el día fue completo o fallido.
                "fill" => {
                    queries::ensure_day(tx, d)?;
                    to_fill.push(decision.date.clone());
                }
                "failed" => queries::set_day_status(tx, d, "failed")?,
                // Un día vacío no cuenta para la racha ni la rompe.
                "empty" => queries::set_day_status(tx, d, "skipped")?,
                other => {
                    return Err(AppError::invalid(format!(
                        "acción desconocida para {}: {other}",
                        decision.date
                    )))
                }
            }
        }
        let today = queries::today(tx);
        queries::touch_last_open(tx, today)?;
        Ok(to_fill)
    })
}

/// Datos de la pantalla de racha rota (P14) para un día concreto.
#[tauri::command]
pub fn get_broken_streak(db: State<Db>, date: Option<String>) -> AppResult<BrokenStreak> {
    db.with(|c| {
        let d = resolve_date(c, date)?;
        queries::broken_streak(c, d)
    })
}
