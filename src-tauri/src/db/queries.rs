//! Toda la SQL vive aquí. Los comandos de Tauri solo orquestan.
//!
//! Convención de formatos: las fechas son `YYYY-MM-DD` y los instantes son ISO
//! local sin zona (`YYYY-MM-DDTHH:MM:SS`). Es una app de un solo usuario en una
//! sola máquina: guardar hora local es lo correcto y evita sorpresas al leer la
//! base con cualquier visor de SQLite.

use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, Timelike};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::coach::{self, CoachMessage, DayFacts, SleepFacts, Tone};
use crate::daycut::{self, format_minutes};
use crate::db::models::*;
use crate::error::{AppError, AppResult};

pub const DATE_FMT: &str = "%Y-%m-%d";
pub const TS_FMT: &str = "%Y-%m-%dT%H:%M:%S";

pub fn now_naive() -> NaiveDateTime {
    Local::now().naive_local()
}

pub fn now_iso() -> String {
    now_naive().format(TS_FMT).to_string()
}

pub fn parse_date(s: &str) -> AppResult<NaiveDate> {
    NaiveDate::parse_from_str(s, DATE_FMT)
        .map_err(|_| AppError::invalid(format!("fecha no válida: {s}")))
}

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

/// Etiquetas legibles de los enumerados que se guardan en inglés. Viven aquí
/// para que Rust y la UI digan exactamente lo mismo en gráficas e informes.
pub fn glucose_context_label(context: &str) -> &'static str {
    match context {
        "fasting" => "En ayunas",
        "pre_meal" => "Antes de comer",
        "post_meal_2h" => "2 h post-comida",
        "pre_workout" => "Antes de entrenar",
        _ => "Aleatoria",
    }
}

pub fn workout_kind_label(kind: &str) -> &'static str {
    match kind {
        "gym" => "Gimnasio",
        "cardio" => "Cardio",
        "outdoor" => "Al aire libre",
        "sport" => "Deporte",
        _ => "Otro",
    }
}

pub fn weekday_label(date: NaiveDate) -> String {
    let dia = match date.weekday().num_days_from_monday() {
        0 => "Lunes",
        1 => "Martes",
        2 => "Miércoles",
        3 => "Jueves",
        4 => "Viernes",
        5 => "Sábado",
        _ => "Domingo",
    };
    let mes = match date.month() {
        1 => "enero",
        2 => "febrero",
        3 => "marzo",
        4 => "abril",
        5 => "mayo",
        6 => "junio",
        7 => "julio",
        8 => "agosto",
        9 => "septiembre",
        10 => "octubre",
        11 => "noviembre",
        _ => "diciembre",
    };
    format!("{dia} {} de {mes}", date.day())
}

// ------------------------------------------------------------------ settings

pub fn get_setting(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
            r.get::<_, String>(0)
        })
        .optional()?)
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_int(conn: &Connection, key: &str, fallback: i64) -> i64 {
    get_setting(conn, key)
        .ok()
        .flatten()
        .and_then(|v| v.parse().ok())
        .unwrap_or(fallback)
}

pub fn all_settings(conn: &Connection) -> AppResult<std::collections::BTreeMap<String, String>> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings ORDER BY key")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut out = std::collections::BTreeMap::new();
    for row in rows {
        let (k, v) = row?;
        out.insert(k, v);
    }
    Ok(out)
}

pub fn cutoff_hour(conn: &Connection) -> u32 {
    get_int(conn, "day_cutoff_hour", daycut::DEFAULT_CUTOFF_HOUR as i64).clamp(0, 23) as u32
}

pub fn tone(conn: &Connection) -> Tone {
    Tone::from_setting(
        &get_setting(conn, "coach_tone")
            .ok()
            .flatten()
            .unwrap_or_else(|| "directo".into()),
    )
}

/// Fecha lógica de hoy, respetando el corte de las 4:00 AM.
pub fn today(conn: &Connection) -> NaiveDate {
    daycut::logical_date(now_naive(), cutoff_hour(conn))
}

// ----------------------------------------------------------------- challenge

pub fn active_challenge(conn: &Connection) -> AppResult<Option<Challenge>> {
    let row = conn
        .query_row(
            "SELECT id, name, start_date, target_days, rules_json, ended_at, ended_reason
             FROM challenge WHERE ended_at IS NULL
             ORDER BY start_date DESC LIMIT 1",
            [],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, Option<String>>(6)?,
                ))
            },
        )
        .optional()?;

    let Some((id, name, start_date, target_days, rules_json, ended_at, ended_reason)) = row else {
        return Ok(None);
    };

    // Si las reglas guardadas están corruptas no se tumba la app: se usan las
    // de fábrica y el usuario puede volver a guardarlas desde Ajustes.
    let rules = serde_json::from_str(&rules_json).unwrap_or_default();

    Ok(Some(Challenge {
        id,
        name,
        start_date,
        target_days,
        rules,
        ended_at,
        ended_reason,
    }))
}

pub fn challenge_count(conn: &Connection) -> AppResult<i64> {
    Ok(conn.query_row("SELECT count(*) FROM challenge", [], |r| r.get(0))?)
}

pub fn create_challenge(
    conn: &Connection,
    name: &str,
    start_date: NaiveDate,
    target_days: i64,
    rules: &Rules,
) -> AppResult<Challenge> {
    if target_days < 1 {
        return Err(AppError::invalid("el reto necesita al menos 1 día"));
    }
    let id = new_id();
    let now = now_iso();
    conn.execute(
        "INSERT INTO challenge (id, name, start_date, target_days, rules_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![
            id,
            name,
            start_date.format(DATE_FMT).to_string(),
            target_days,
            serde_json::to_string(rules)?,
            now
        ],
    )?;

    active_challenge(conn)?.ok_or_else(|| AppError::internal("el reto no quedó guardado"))
}

pub fn end_challenge(conn: &Connection, id: &str, reason: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE challenge SET ended_at = ?2, ended_reason = ?3, updated_at = ?4 WHERE id = ?1",
        params![
            id,
            today(conn).format(DATE_FMT).to_string(),
            reason,
            now_iso()
        ],
    )?;
    Ok(())
}

// ----------------------------------------------------------------------- día

/// Garantiza que exista la fila de `day`. Todo registro cuelga de ella por FK,
/// así que esto se llama antes de cualquier inserción.
pub fn ensure_day(conn: &Connection, date: NaiveDate) -> AppResult<()> {
    let key = date.format(DATE_FMT).to_string();
    let exists: bool = conn
        .query_row("SELECT 1 FROM day WHERE date = ?1", [&key], |_| Ok(true))
        .optional()?
        .unwrap_or(false);
    if exists {
        return Ok(());
    }

    let challenge = active_challenge(conn)?;
    let (challenge_id, day_number) = match &challenge {
        Some(c) => {
            let start = parse_date(&c.start_date)?;
            (Some(c.id.clone()), daycut::day_number(start, date))
        }
        None => (None, None),
    };

    conn.execute(
        "INSERT INTO day (date, challenge_id, day_number, status, updated_at)
         VALUES (?1, ?2, ?3, 'pending', ?4)",
        params![key, challenge_id, day_number, now_iso()],
    )?;
    Ok(())
}

pub fn day_status(conn: &Connection, date: NaiveDate) -> AppResult<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT status FROM day WHERE date = ?1",
            [date.format(DATE_FMT).to_string()],
            |r| r.get::<_, String>(0),
        )
        .optional()?)
}

pub fn set_day_status(conn: &Connection, date: NaiveDate, status: &str) -> AppResult<()> {
    if !matches!(status, "pending" | "complete" | "failed" | "skipped") {
        return Err(AppError::invalid(format!("estado desconocido: {status}")));
    }
    ensure_day(conn, date)?;
    conn.execute(
        "UPDATE day SET status = ?2, updated_at = ?3 WHERE date = ?1",
        params![date.format(DATE_FMT).to_string(), status, now_iso()],
    )?;
    Ok(())
}

pub fn set_day_notes(conn: &Connection, date: NaiveDate, notes: Option<&str>) -> AppResult<()> {
    ensure_day(conn, date)?;
    conn.execute(
        "UPDATE day SET notes = ?2, updated_at = ?3 WHERE date = ?1",
        params![date.format(DATE_FMT).to_string(), notes, now_iso()],
    )?;
    Ok(())
}

pub fn mark_checkin(conn: &Connection, date: NaiveDate, which: &str) -> AppResult<()> {
    ensure_day(conn, date)?;
    let column = match which {
        "morning" => "morning_checkin_at",
        "evening" => "evening_checkin_at",
        _ => return Err(AppError::invalid("check-in desconocido")),
    };
    conn.execute(
        &format!("UPDATE day SET {column} = ?2, updated_at = ?2 WHERE date = ?1"),
        params![date.format(DATE_FMT).to_string(), now_iso()],
    )?;
    Ok(())
}

// ------------------------------------------------------------------- rachas

/// Días 'complete' consecutivos hacia atrás desde `from`. Un día 'skipped' no
/// suma pero tampoco rompe (§7, modo pausa). Cualquier otra cosa corta.
pub fn current_streak(conn: &Connection, from: NaiveDate) -> AppResult<i64> {
    let mut streak = 0i64;
    let mut cursor = from;
    // Si hoy todavía está pendiente, la racha se cuenta desde ayer.
    if matches!(day_status(conn, cursor)?.as_deref(), Some("pending") | None) {
        cursor -= Duration::days(1);
    }
    loop {
        match day_status(conn, cursor)?.as_deref() {
            Some("complete") => streak += 1,
            Some("skipped") => {}
            _ => break,
        }
        cursor -= Duration::days(1);
        // Nadie tiene una racha de más de 10 años; corta el bucle por si acaso.
        if (from - cursor).num_days() > 3650 {
            break;
        }
    }
    Ok(streak)
}

pub fn longest_streak(conn: &Connection) -> AppResult<i64> {
    let mut stmt = conn.prepare("SELECT date, status FROM day ORDER BY date ASC")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;

    let mut best = 0i64;
    let mut run = 0i64;
    let mut prev: Option<NaiveDate> = None;

    for row in rows {
        let (date_s, status) = row?;
        let date = parse_date(&date_s)?;
        let contiguous = prev.map(|p| (date - p).num_days() == 1).unwrap_or(true);
        if !contiguous {
            run = 0;
        }
        match status.as_str() {
            "complete" => run += 1,
            "skipped" => {} // no suma ni rompe
            _ => run = 0,
        }
        best = best.max(run);
        prev = Some(date);
    }
    Ok(best)
}

/// Noches seguidas bajo 6 h terminando en `date` (incluida).
pub fn short_nights_streak(conn: &Connection, date: NaiveDate) -> AppResult<u32> {
    let mut n = 0u32;
    let mut cursor = date;
    loop {
        let minutes: Option<i64> = conn
            .query_row(
                "SELECT minutes FROM sleep_log WHERE date = ?1",
                [cursor.format(DATE_FMT).to_string()],
                |r| r.get(0),
            )
            .optional()?;
        match minutes {
            Some(m) if m < 360 => n += 1,
            _ => break,
        }
        cursor -= Duration::days(1);
        if n > 365 {
            break;
        }
    }
    Ok(n)
}

// --------------------------------------------------------------- agregados

pub fn get_sleep(conn: &Connection, date: NaiveDate) -> AppResult<Option<SleepLog>> {
    Ok(conn
        .query_row(
            "SELECT date, bedtime, wake_time, minutes, quality FROM sleep_log WHERE date = ?1",
            [date.format(DATE_FMT).to_string()],
            |r| {
                Ok(SleepLog {
                    date: r.get(0)?,
                    bedtime: r.get(1)?,
                    wake_time: r.get(2)?,
                    minutes: r.get(3)?,
                    quality: r.get(4)?,
                })
            },
        )
        .optional()?)
}

pub struct MealSummary {
    pub meals: i64,
    pub snacks: i64,
    pub calories: Option<i64>,
}

pub fn meal_summary(conn: &Connection, date: NaiveDate) -> AppResult<MealSummary> {
    let key = date.format(DATE_FMT).to_string();
    let (meals, snacks): (i64, i64) = conn.query_row(
        "SELECT
           coalesce(sum(kind = 'meal'), 0),
           coalesce(sum(kind = 'snack'), 0)
         FROM meal WHERE date = ?1",
        [&key],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    // Las calorías solo se muestran si hay al menos una registrada; nunca se
    // infieren ni se comparan contra una meta (§6, decisión 1).
    let calories: Option<i64> = conn.query_row(
        "SELECT sum(calories) FROM meal WHERE date = ?1 AND calories IS NOT NULL",
        [&key],
        |r| r.get(0),
    )?;
    Ok(MealSummary {
        meals,
        snacks,
        calories,
    })
}

pub struct WorkoutSummary {
    pub minutes: i64,
    pub calories: Option<i64>,
    pub has_outdoor: bool,
}

pub fn workout_summary(conn: &Connection, date: NaiveDate) -> AppResult<WorkoutSummary> {
    let key = date.format(DATE_FMT).to_string();
    let minutes: i64 = conn.query_row(
        "SELECT coalesce(sum(duration_min), 0) FROM workout WHERE date = ?1",
        [&key],
        |r| r.get(0),
    )?;
    let calories: Option<i64> = conn.query_row(
        "SELECT sum(calories_burned) FROM workout WHERE date = ?1 AND calories_burned IS NOT NULL",
        [&key],
        |r| r.get(0),
    )?;
    let outdoor: i64 = conn.query_row(
        "SELECT count(*) FROM workout WHERE date = ?1 AND is_outdoor = 1",
        [&key],
        |r| r.get(0),
    )?;
    Ok(WorkoutSummary {
        minutes,
        calories,
        has_outdoor: outdoor > 0,
    })
}

pub fn water_ml(conn: &Connection, date: NaiveDate) -> AppResult<i64> {
    Ok(conn
        .query_row(
            "SELECT ml FROM water_log WHERE date = ?1",
            [date.format(DATE_FMT).to_string()],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0))
}

pub fn reading_pages(conn: &Connection, date: NaiveDate) -> AppResult<i64> {
    Ok(conn.query_row(
        "SELECT coalesce(sum(pages), 0) FROM reading_log WHERE date = ?1",
        [date.format(DATE_FMT).to_string()],
        |r| r.get(0),
    )?)
}

pub fn glucose_count(conn: &Connection, date: NaiveDate) -> AppResult<i64> {
    Ok(conn.query_row(
        "SELECT count(*) FROM glucose_reading WHERE date = ?1",
        [date.format(DATE_FMT).to_string()],
        |r| r.get(0),
    )?)
}

pub fn work_minutes(conn: &Connection, date: NaiveDate) -> AppResult<i64> {
    Ok(conn.query_row(
        "SELECT coalesce(sum(minutes), 0) FROM work_session WHERE date = ?1",
        [date.format(DATE_FMT).to_string()],
        |r| r.get(0),
    )?)
}

pub fn weight_kg(conn: &Connection, date: NaiveDate) -> AppResult<Option<f64>> {
    Ok(conn
        .query_row(
            "SELECT kg FROM weight_log WHERE date = ?1",
            [date.format(DATE_FMT).to_string()],
            |r| r.get(0),
        )
        .optional()?)
}

pub fn get_mood(conn: &Connection, date: NaiveDate) -> AppResult<Option<MoodLog>> {
    Ok(conn
        .query_row(
            "SELECT mood, energy, stress FROM mood_log WHERE date = ?1",
            [date.format(DATE_FMT).to_string()],
            |r| {
                Ok(MoodLog {
                    mood: r.get(0)?,
                    energy: r.get(1)?,
                    stress: r.get(2)?,
                })
            },
        )
        .optional()?)
}

pub fn has_photo(conn: &Connection, date: NaiveDate) -> AppResult<bool> {
    let n: i64 = conn.query_row(
        "SELECT count(*) FROM progress_photo WHERE date = ?1",
        [date.format(DATE_FMT).to_string()],
        |r| r.get(0),
    )?;
    Ok(n > 0)
}

// -------------------------------------------------------------- escrituras

#[allow(clippy::too_many_arguments)]
pub fn save_sleep(
    conn: &Connection,
    date: NaiveDate,
    bedtime: NaiveDateTime,
    wake_time: NaiveDateTime,
    quality: Option<i64>,
) -> AppResult<i64> {
    let minutes = daycut::sleep_minutes(bedtime, wake_time);
    if minutes == 0 {
        return Err(AppError::invalid(
            "la hora de despertar tiene que ser posterior a la de dormir",
        ));
    }
    if minutes > 16 * 60 {
        return Err(AppError::invalid(
            "más de 16 horas de sueño; revisa las horas",
        ));
    }
    ensure_day(conn, date)?;
    conn.execute(
        "INSERT INTO sleep_log (date, bedtime, wake_time, minutes, quality, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(date) DO UPDATE SET
           bedtime = excluded.bedtime, wake_time = excluded.wake_time,
           minutes = excluded.minutes, quality = excluded.quality,
           updated_at = excluded.updated_at",
        params![
            date.format(DATE_FMT).to_string(),
            bedtime.format(TS_FMT).to_string(),
            wake_time.format(TS_FMT).to_string(),
            minutes,
            quality,
            now_iso()
        ],
    )?;
    Ok(minutes)
}

pub fn add_meal(
    conn: &Connection,
    date: NaiveDate,
    eaten_at: NaiveDateTime,
    kind: &str,
    description: &str,
    calories: Option<i64>,
) -> AppResult<String> {
    if !matches!(kind, "meal" | "snack") {
        return Err(AppError::invalid("tipo de comida desconocido"));
    }
    if description.trim().is_empty() {
        return Err(AppError::invalid("escribe qué comiste"));
    }
    if calories.is_some_and(|c| c < 0) {
        return Err(AppError::invalid("las calorías no pueden ser negativas"));
    }
    ensure_day(conn, date)?;
    let id = new_id();
    conn.execute(
        "INSERT INTO meal (id, date, eaten_at, kind, description, calories, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            date.format(DATE_FMT).to_string(),
            eaten_at.format(TS_FMT).to_string(),
            kind,
            description.trim(),
            calories,
            now_iso()
        ],
    )?;
    Ok(id)
}

#[allow(clippy::too_many_arguments)]
pub fn add_workout(
    conn: &Connection,
    date: NaiveDate,
    started_at: NaiveDateTime,
    kind: &str,
    description: Option<&str>,
    duration_min: i64,
    is_outdoor: bool,
    calories_burned: Option<i64>,
) -> AppResult<String> {
    if !matches!(kind, "gym" | "cardio" | "outdoor" | "sport" | "other") {
        return Err(AppError::invalid("tipo de ejercicio desconocido"));
    }
    if duration_min <= 0 {
        return Err(AppError::invalid("la duración tiene que ser mayor a 0"));
    }
    ensure_day(conn, date)?;
    let id = new_id();
    conn.execute(
        "INSERT INTO workout
           (id, date, started_at, kind, description, duration_min, is_outdoor, calories_burned, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            date.format(DATE_FMT).to_string(),
            started_at.format(TS_FMT).to_string(),
            kind,
            description,
            duration_min,
            is_outdoor as i64,
            calories_burned,
            now_iso()
        ],
    )?;
    Ok(id)
}

pub fn add_glucose(
    conn: &Connection,
    date: NaiveDate,
    measured_at: NaiveDateTime,
    value_mgdl: i64,
    context: &str,
    notes: Option<&str>,
) -> AppResult<String> {
    if !matches!(
        context,
        "fasting" | "pre_meal" | "post_meal_2h" | "random" | "pre_workout"
    ) {
        return Err(AppError::invalid("contexto de medición desconocido"));
    }
    if !(20..=600).contains(&value_mgdl) {
        return Err(AppError::invalid(
            "el valor debe estar entre 20 y 600 mg/dL",
        ));
    }
    ensure_day(conn, date)?;

    // Autocompletado del spec: se vincula la comida de las últimas 3 horas.
    let window_start = (measured_at - Duration::hours(3))
        .format(TS_FMT)
        .to_string();
    let linked: Option<String> = conn
        .query_row(
            "SELECT id FROM meal WHERE eaten_at BETWEEN ?1 AND ?2
             ORDER BY eaten_at DESC LIMIT 1",
            params![window_start, measured_at.format(TS_FMT).to_string()],
            |r| r.get(0),
        )
        .optional()?;

    let id = new_id();
    conn.execute(
        "INSERT INTO glucose_reading
           (id, date, measured_at, value_mgdl, context, notes, linked_meal_id, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            date.format(DATE_FMT).to_string(),
            measured_at.format(TS_FMT).to_string(),
            value_mgdl,
            context,
            notes,
            linked,
            now_iso()
        ],
    )?;
    Ok(id)
}

/// Suma (o resta, si `delta_ml` es negativo) agua del día. Nunca baja de 0.
pub fn add_water(conn: &Connection, date: NaiveDate, delta_ml: i64) -> AppResult<i64> {
    ensure_day(conn, date)?;
    let key = date.format(DATE_FMT).to_string();
    conn.execute(
        "INSERT INTO water_log (date, ml, updated_at) VALUES (?1, max(0, ?2), ?3)
         ON CONFLICT(date) DO UPDATE SET
           ml = max(0, water_log.ml + ?2), updated_at = excluded.updated_at",
        params![key, delta_ml, now_iso()],
    )?;
    water_ml(conn, date)
}

pub fn set_water(conn: &Connection, date: NaiveDate, ml: i64) -> AppResult<()> {
    ensure_day(conn, date)?;
    conn.execute(
        "INSERT INTO water_log (date, ml, updated_at) VALUES (?1, max(0, ?2), ?3)
         ON CONFLICT(date) DO UPDATE SET ml = max(0, ?2), updated_at = excluded.updated_at",
        params![date.format(DATE_FMT).to_string(), ml, now_iso()],
    )?;
    Ok(())
}

pub fn add_reading(
    conn: &Connection,
    date: NaiveDate,
    pages: i64,
    book: Option<&str>,
) -> AppResult<String> {
    if pages <= 0 {
        return Err(AppError::invalid("las páginas tienen que ser más de 0"));
    }
    ensure_day(conn, date)?;
    let id = new_id();
    conn.execute(
        "INSERT INTO reading_log (id, date, pages, book, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            id,
            date.format(DATE_FMT).to_string(),
            pages,
            book,
            now_iso()
        ],
    )?;
    Ok(id)
}

pub fn add_work_session(
    conn: &Connection,
    date: NaiveDate,
    started_at: NaiveDateTime,
    minutes: i64,
    category: &str,
    description: Option<&str>,
) -> AppResult<String> {
    if minutes <= 0 {
        return Err(AppError::invalid("la duración tiene que ser mayor a 0"));
    }
    if category.trim().is_empty() {
        return Err(AppError::invalid("elige una categoría"));
    }
    ensure_day(conn, date)?;
    let id = new_id();
    let ended = started_at + Duration::minutes(minutes);
    conn.execute(
        "INSERT INTO work_session
           (id, date, started_at, ended_at, minutes, category, description, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            date.format(DATE_FMT).to_string(),
            started_at.format(TS_FMT).to_string(),
            ended.format(TS_FMT).to_string(),
            minutes,
            category.trim(),
            description,
            now_iso()
        ],
    )?;
    Ok(id)
}

pub fn set_weight(conn: &Connection, date: NaiveDate, kg: f64) -> AppResult<()> {
    if !(20.0..=400.0).contains(&kg) {
        return Err(AppError::invalid("el peso debe estar entre 20 y 400 kg"));
    }
    ensure_day(conn, date)?;
    conn.execute(
        "INSERT INTO weight_log (date, kg, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(date) DO UPDATE SET kg = excluded.kg, updated_at = excluded.updated_at",
        params![date.format(DATE_FMT).to_string(), kg, now_iso()],
    )?;
    Ok(())
}

pub fn set_mood(
    conn: &Connection,
    date: NaiveDate,
    mood: i64,
    energy: i64,
    notes: Option<&str>,
) -> AppResult<()> {
    if !(1..=5).contains(&mood) || !(1..=5).contains(&energy) {
        return Err(AppError::invalid("ánimo y energía van de 1 a 5"));
    }
    ensure_day(conn, date)?;
    conn.execute(
        "INSERT INTO mood_log (date, mood, energy, notes, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(date) DO UPDATE SET
           mood = excluded.mood, energy = excluded.energy,
           notes = excluded.notes, updated_at = excluded.updated_at",
        params![
            date.format(DATE_FMT).to_string(),
            mood,
            energy,
            notes,
            now_iso()
        ],
    )?;
    Ok(())
}

/// Borrado de un registro suelto. `table` viene de una lista blanca, nunca del
/// usuario directamente.
pub fn delete_entry(conn: &Connection, table: &str, id: &str) -> AppResult<()> {
    let allowed = match table {
        "meal" => "meal",
        "workout" => "workout",
        "glucose" => "glucose_reading",
        "reading" => "reading_log",
        "work" => "work_session",
        _ => return Err(AppError::invalid("tipo de registro desconocido")),
    };
    let n = conn.execute(&format!("DELETE FROM {allowed} WHERE id = ?1"), [id])?;
    if n == 0 {
        return Err(AppError::NotFound("ese registro ya no existe".into()));
    }
    Ok(())
}

// ------------------------------------------------------------------ vistas

/// "1 comida" / "3 comidas" — el detalle de un pilar nunca dice "1 snacks".
fn plural(n: i64, singular: &str, plural: &str) -> String {
    if n == 1 {
        format!("{n} {singular}")
    } else {
        format!("{n} {plural}")
    }
}

/// Sustantivo corto para listar pilares dentro de una frase del coach.
/// "Sin registrar: ejercicio, agua y lectura" lee mucho mejor que arrastrar
/// la etiqueta completa con su meta numérica.
fn pillar_noun(key: &str, label: &str) -> String {
    match key {
        "sleep" => "sueño",
        "workout" => "ejercicio",
        "outdoor" => "ejercicio al aire libre",
        "diet" => "comidas",
        "water" => "agua",
        "reading" => "lectura",
        "glucose" => "glucosa",
        "photo" => "foto de progreso",
        _ => return label.to_lowercase(),
    }
    .to_string()
}

/// Todo lo registrado en un día, ya agregado. Se arma una vez y se reutiliza
/// para evaluar los ocho pilares.
struct DayTotals<'a> {
    sleep: Option<&'a SleepLog>,
    meals: &'a MealSummary,
    workout: &'a WorkoutSummary,
    water: i64,
    pages: i64,
    glucose: i64,
}

fn pillar_state(
    conn: &Connection,
    date: NaiveDate,
    pillar: &Pillar,
    totals: &DayTotals,
) -> AppResult<PillarState> {
    let DayTotals {
        sleep,
        meals,
        workout,
        water,
        pages,
        glucose,
    } = *totals;

    let (status, detail) = match pillar.key.as_str() {
        "sleep" => match sleep {
            Some(s) => {
                let goal = pillar.goal.unwrap_or(420);
                let done = if s.minutes >= goal { "done" } else { "partial" };
                (done, format_minutes(s.minutes))
            }
            None => ("missing", "sin registrar".to_string()),
        },
        "workout" => {
            let goal = pillar.goal.unwrap_or(45);
            if workout.minutes == 0 {
                ("missing", "sin registrar".to_string())
            } else if workout.minutes >= goal {
                ("done", format!("{} min", workout.minutes))
            } else {
                ("partial", format!("{} de {goal} min", workout.minutes))
            }
        }
        "outdoor" => {
            if workout.has_outdoor {
                ("done", "registrado".to_string())
            } else {
                ("missing", "sin registrar".to_string())
            }
        }
        "diet" => {
            if meals.meals > 0 {
                let comidas = plural(meals.meals, "comida", "comidas");
                (
                    "done",
                    match meals.snacks {
                        0 => comidas,
                        n => format!("{comidas}, {}", plural(n, "snack", "snacks")),
                    },
                )
            } else {
                ("missing", "sin registrar".to_string())
            }
        }
        "water" => {
            let goal = pillar.goal.unwrap_or(3000);
            let litros = |ml: i64| format!("{:.1} L", ml as f64 / 1000.0);
            if water >= goal {
                ("done", litros(water))
            } else if water > 0 {
                ("partial", format!("{} de {}", litros(water), litros(goal)))
            } else {
                ("missing", "sin registrar".to_string())
            }
        }
        "reading" => {
            let goal = pillar.goal.unwrap_or(10);
            if pages >= goal {
                ("done", format!("{pages} páginas"))
            } else if pages > 0 {
                ("partial", format!("{pages} de {goal} páginas"))
            } else {
                ("missing", "sin registrar".to_string())
            }
        }
        "glucose" => {
            if glucose > 0 {
                ("done", plural(glucose, "lectura", "lecturas"))
            } else {
                ("missing", "sin registrar".to_string())
            }
        }
        "photo" => {
            if has_photo(conn, date)? {
                ("done", "tomada".to_string())
            } else {
                ("missing", "sin registrar".to_string())
            }
        }
        _ => ("missing", "sin registrar".to_string()),
    };

    Ok(PillarState {
        key: pillar.key.clone(),
        label: pillar.label.clone(),
        required: pillar.required,
        status: status.to_string(),
        detail,
    })
}

pub fn today_view(conn: &Connection, date: NaiveDate) -> AppResult<TodayView> {
    ensure_day(conn, date)?;

    let challenge = active_challenge(conn)?;
    let rules = challenge
        .as_ref()
        .map(|c| c.rules.clone())
        .unwrap_or_default();
    let target_days = challenge.as_ref().map(|c| c.target_days).unwrap_or(75);
    let day_number = match &challenge {
        Some(c) => daycut::day_number(parse_date(&c.start_date)?, date),
        None => None,
    };

    let sleep = get_sleep(conn, date)?;
    let meals = meal_summary(conn, date)?;
    let workout = workout_summary(conn, date)?;
    let water = water_ml(conn, date)?;
    let pages = reading_pages(conn, date)?;
    let glucose = glucose_count(conn, date)?;
    let work = work_minutes(conn, date)?;

    let sleep_goal_min = get_int(conn, "sleep_goal_min", 420);
    let water_goal_ml = get_int(conn, "water_goal_ml", 3000);
    let reading_goal_pages = get_int(conn, "reading_goal_pages", 10);
    let tone = tone(conn);

    let sleep_verdict = match &sleep {
        Some(s) => Some(coach::sleep_verdict(
            SleepFacts {
                minutes: s.minutes,
                goal_min: sleep_goal_min,
                short_nights_streak: short_nights_streak(conn, date)?,
            },
            tone,
        )),
        None => None,
    };

    let totals = DayTotals {
        sleep: sleep.as_ref(),
        meals: &meals,
        workout: &workout,
        water,
        pages,
        glucose,
    };
    let mut pillars = Vec::with_capacity(rules.pillars.len());
    for p in &rules.pillars {
        pillars.push(pillar_state(conn, date, p, &totals)?);
    }

    let missing_pillars: Vec<String> = pillars
        .iter()
        .filter(|p| p.required && p.status != "done")
        .map(|p| pillar_noun(&p.key, &p.label))
        .collect();

    let status = day_status(conn, date)?.unwrap_or_else(|| "pending".into());
    let (morning_done, evening_done): (bool, bool) = conn.query_row(
        "SELECT morning_checkin_at IS NOT NULL, evening_checkin_at IS NOT NULL
         FROM day WHERE date = ?1",
        [date.format(DATE_FMT).to_string()],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    // Horas que quedan hasta el corte del día siguiente.
    let cutoff = cutoff_hour(conn) as i64;
    let now = now_naive();
    let end_of_day = (date + Duration::days(1))
        .and_hms_opt(cutoff as u32, 0, 0)
        .ok_or_else(|| AppError::internal("hora de corte inválida"))?;
    let hours_left = ((end_of_day - now).num_minutes().max(0) + 59) / 60;

    let streak = current_streak(conn, date)?;
    let coach_msg: CoachMessage = coach::today_message(
        &DayFacts {
            day_number: day_number.unwrap_or(0),
            target_days,
            streak,
            missing_pillars,
            hours_left,
            day_closed: status != "pending",
        },
        tone,
    );

    Ok(TodayView {
        date: date.format(DATE_FMT).to_string(),
        weekday_label: weekday_label(date),
        day_number,
        target_days,
        status,
        streak,
        longest_streak: longest_streak(conn)?,
        recent: recent_statuses(conn, date, 30)?,
        sleep,
        sleep_goal_min,
        sleep_verdict,
        meals_count: meals.meals,
        snacks_count: meals.snacks,
        calories_in: meals.calories,
        workout_min: workout.minutes,
        workout_kcal: workout.calories,
        water_ml: water,
        water_goal_ml,
        reading_pages: pages,
        reading_goal_pages,
        glucose_count: glucose,
        work_min: work,
        weight_kg: weight_kg(conn, date)?,
        mood: get_mood(conn, date)?,
        pillars,
        coach: coach_msg,
        hours_left,
        morning_done,
        evening_done,
    })
}

/// Estados de los últimos `n` días (el más antiguo primero) para la sparkline.
pub fn recent_statuses(conn: &Connection, upto: NaiveDate, n: i64) -> AppResult<Vec<String>> {
    let mut out = Vec::with_capacity(n as usize);
    for offset in (0..n).rev() {
        let d = upto - Duration::days(offset);
        out.push(day_status(conn, d)?.unwrap_or_else(|| "empty".into()));
    }
    Ok(out)
}

pub fn day_rows(conn: &Connection, limit: i64) -> AppResult<Vec<DayRow>> {
    let mut stmt = conn.prepare(
        "SELECT d.date, d.day_number, d.status,
                s.minutes,
                coalesce((SELECT sum(duration_min) FROM workout w WHERE w.date = d.date), 0),
                (SELECT count(*) FROM meal m WHERE m.date = d.date),
                coalesce((SELECT ml FROM water_log wl WHERE wl.date = d.date), 0),
                (SELECT count(*) FROM glucose_reading g WHERE g.date = d.date),
                (SELECT kg FROM weight_log wg WHERE wg.date = d.date)
         FROM day d
         LEFT JOIN sleep_log s ON s.date = d.date
         ORDER BY d.date DESC
         LIMIT ?1",
    )?;
    let rows = stmt.query_map([limit], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<i64>>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<i64>>(3)?,
            r.get::<_, i64>(4)?,
            r.get::<_, i64>(5)?,
            r.get::<_, i64>(6)?,
            r.get::<_, i64>(7)?,
            r.get::<_, Option<f64>>(8)?,
        ))
    })?;

    let mut out = Vec::new();
    for row in rows {
        let (date, day_number, status, sleep, workout, meals, water, glucose, weight) = row?;
        let parsed = parse_date(&date)?;
        out.push(DayRow {
            weekday_label: weekday_label(parsed),
            date,
            day_number,
            status,
            sleep_minutes: sleep,
            workout_min: workout,
            meals_count: meals,
            water_ml: water,
            glucose_count: glucose,
            weight_kg: weight,
        });
    }
    Ok(out)
}

pub fn history_summary(conn: &Connection) -> AppResult<HistorySummary> {
    let count = |status: &str| -> AppResult<i64> {
        Ok(conn.query_row(
            "SELECT count(*) FROM day WHERE status = ?1",
            [status],
            |r| r.get(0),
        )?)
    };
    // "Parcial" no es un estado guardado: es un día pendiente que ya tiene algo
    // registrado pero nunca se cerró.
    let partial: i64 = conn.query_row(
        "SELECT count(*) FROM day d WHERE d.status = 'pending' AND (
            EXISTS (SELECT 1 FROM sleep_log s WHERE s.date = d.date)
         OR EXISTS (SELECT 1 FROM meal m WHERE m.date = d.date)
         OR EXISTS (SELECT 1 FROM workout w WHERE w.date = d.date))",
        [],
        |r| r.get(0),
    )?;
    Ok(HistorySummary {
        complete: count("complete")?,
        partial,
        failed: count("failed")?,
        skipped: count("skipped")?,
    })
}

pub fn day_detail(conn: &Connection, date: NaiveDate) -> AppResult<DayDetail> {
    ensure_day(conn, date)?;
    let key = date.format(DATE_FMT).to_string();

    let (day_number, status, notes): (Option<i64>, String, Option<String>) = conn.query_row(
        "SELECT day_number, status, notes FROM day WHERE date = ?1",
        [&key],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;

    let mut stmt = conn.prepare(
        "SELECT id, date, eaten_at, kind, description, calories FROM meal
         WHERE date = ?1 ORDER BY eaten_at ASC",
    )?;
    let meals = stmt
        .query_map([&key], |r| {
            Ok(Meal {
                id: r.get(0)?,
                date: r.get(1)?,
                eaten_at: r.get(2)?,
                kind: r.get(3)?,
                description: r.get(4)?,
                calories: r.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT id, date, started_at, kind, description, duration_min, is_outdoor, calories_burned
         FROM workout WHERE date = ?1 ORDER BY started_at ASC",
    )?;
    let workouts = stmt
        .query_map([&key], |r| {
            Ok(Workout {
                id: r.get(0)?,
                date: r.get(1)?,
                started_at: r.get(2)?,
                kind: r.get(3)?,
                description: r.get(4)?,
                duration_min: r.get(5)?,
                is_outdoor: r.get::<_, i64>(6)? != 0,
                calories_burned: r.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT g.id, g.date, g.measured_at, g.value_mgdl, g.context, g.notes,
                g.linked_meal_id, m.description
         FROM glucose_reading g
         LEFT JOIN meal m ON m.id = g.linked_meal_id
         WHERE g.date = ?1 ORDER BY g.measured_at ASC",
    )?;
    let glucose = stmt
        .query_map([&key], |r| {
            Ok(GlucoseReading {
                id: r.get(0)?,
                date: r.get(1)?,
                measured_at: r.get(2)?,
                value_mgdl: r.get(3)?,
                context: r.get(4)?,
                notes: r.get(5)?,
                linked_meal_id: r.get(6)?,
                linked_meal_description: r.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT id, date, started_at, ended_at, minutes, category, description
         FROM work_session WHERE date = ?1 ORDER BY started_at ASC",
    )?;
    let work = stmt
        .query_map([&key], |r| {
            Ok(WorkSession {
                id: r.get(0)?,
                date: r.get(1)?,
                started_at: r.get(2)?,
                ended_at: r.get(3)?,
                minutes: r.get(4)?,
                category: r.get(5)?,
                description: r.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut stmt =
        conn.prepare("SELECT id, date, pages, book FROM reading_log WHERE date = ?1 ORDER BY id")?;
    let reading = stmt
        .query_map([&key], |r| {
            Ok(ReadingLog {
                id: r.get(0)?,
                date: r.get(1)?,
                pages: r.get(2)?,
                book: r.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(DayDetail {
        date: key,
        weekday_label: weekday_label(date),
        day_number,
        status,
        notes,
        sleep: get_sleep(conn, date)?,
        meals,
        workouts,
        glucose,
        work,
        reading,
        water_ml: water_ml(conn, date)?,
        weight_kg: weight_kg(conn, date)?,
        mood: get_mood(conn, date)?,
    })
}

/// Días calendario que quedaron sin tocar desde el último uso de la app (§7).
/// Solo cuentan los días ya cerrados (hasta ayer) que no tienen ni fila ni
/// ningún registro colgando.
pub fn missing_days(conn: &Connection, today_date: NaiveDate) -> AppResult<Vec<MissingDay>> {
    let Some(challenge) = active_challenge(conn)? else {
        return Ok(vec![]);
    };
    let start = parse_date(&challenge.start_date)?;

    let last_seen = get_setting(conn, "last_open_date")?
        .and_then(|s| NaiveDate::parse_from_str(&s, DATE_FMT).ok())
        .unwrap_or(start);

    let from = start.max(last_seen);
    let mut out = Vec::new();
    let mut cursor = from;

    while cursor < today_date {
        let key = cursor.format(DATE_FMT).to_string();
        let untouched: bool = conn.query_row(
            "SELECT NOT EXISTS (
                SELECT 1 FROM day d WHERE d.date = ?1 AND (
                    d.status <> 'pending'
                 OR d.morning_checkin_at IS NOT NULL
                 OR d.evening_checkin_at IS NOT NULL
                 OR EXISTS (SELECT 1 FROM sleep_log s WHERE s.date = d.date)
                 OR EXISTS (SELECT 1 FROM meal m WHERE m.date = d.date)
                 OR EXISTS (SELECT 1 FROM workout w WHERE w.date = d.date)
                 OR EXISTS (SELECT 1 FROM water_log wl WHERE wl.date = d.date AND wl.ml > 0)
                 OR EXISTS (SELECT 1 FROM glucose_reading g WHERE g.date = d.date)
                 OR EXISTS (SELECT 1 FROM reading_log r WHERE r.date = d.date)
                )
             )",
            [&key],
            |r| r.get(0),
        )?;

        if untouched {
            out.push(MissingDay {
                date: key,
                weekday_label: weekday_label(cursor),
                day_number: daycut::day_number(start, cursor),
            });
        }
        cursor += Duration::days(1);

        // Un reto no dura más de un año; evita un bucle infinito si la fecha
        // del sistema se fue muy atrás.
        if out.len() > 400 {
            break;
        }
    }

    Ok(out)
}

// ------------------------------------------------- fotos de progreso (F3)

pub fn add_photo(conn: &Connection, date: NaiveDate, path: &str) -> AppResult<String> {
    ensure_day(conn, date)?;
    let id = new_id();
    conn.execute(
        "INSERT INTO progress_photo (id, date, path, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, date.format(DATE_FMT).to_string(), path, now_iso()],
    )?;
    Ok(id)
}

pub fn photos(conn: &Connection) -> AppResult<Vec<ProgressPhoto>> {
    let challenge = active_challenge(conn)?;
    let start = match &challenge {
        Some(c) => Some(parse_date(&c.start_date)?),
        None => None,
    };

    let mut stmt = conn.prepare("SELECT id, date FROM progress_photo ORDER BY date ASC, id ASC")?;
    let filas = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;

    let mut out = Vec::new();
    for (id, date) in filas {
        let d = parse_date(&date)?;
        out.push(ProgressPhoto {
            id,
            weekday_label: weekday_label(d),
            day_number: start.and_then(|s| daycut::day_number(s, d)),
            date,
        });
    }
    Ok(out)
}

pub fn photo_path(conn: &Connection, id: &str) -> AppResult<String> {
    conn.query_row("SELECT path FROM progress_photo WHERE id = ?1", [id], |r| {
        r.get(0)
    })
    .optional()?
    .ok_or_else(|| AppError::NotFound("esa foto ya no existe".into()))
}

pub fn delete_photo(conn: &Connection, id: &str) -> AppResult<()> {
    let n = conn.execute("DELETE FROM progress_photo WHERE id = ?1", [id])?;
    if n == 0 {
        return Err(AppError::NotFound("esa foto ya no existe".into()));
    }
    Ok(())
}

pub fn touch_last_open(conn: &Connection, date: NaiveDate) -> AppResult<()> {
    set_setting(conn, "last_open_date", &date.format(DATE_FMT).to_string())
}

// ------------------------------------------------------- heatmap (P5)

/// Las `target_days` celdas del reto activo, del día 1 al último.
/// "future" son los días que aún no llegan; "partial" los que tienen algo
/// registrado pero nunca se cerraron.
pub fn heatmap(conn: &Connection) -> AppResult<Vec<HeatmapCell>> {
    let Some(challenge) = active_challenge(conn)? else {
        return Ok(vec![]);
    };
    let start = parse_date(&challenge.start_date)?;
    let last = start + Duration::days(challenge.target_days - 1);
    let today_date = today(conn);

    let mut stmt = conn.prepare(
        "SELECT d.date, d.status,
                (d.morning_checkin_at IS NOT NULL
                 OR EXISTS (SELECT 1 FROM sleep_log s WHERE s.date = d.date)
                 OR EXISTS (SELECT 1 FROM meal m WHERE m.date = d.date)
                 OR EXISTS (SELECT 1 FROM workout w WHERE w.date = d.date)
                 OR EXISTS (SELECT 1 FROM water_log wl WHERE wl.date = d.date AND wl.ml > 0)
                 OR EXISTS (SELECT 1 FROM reading_log r WHERE r.date = d.date)
                 OR EXISTS (SELECT 1 FROM glucose_reading g WHERE g.date = d.date))
         FROM day d WHERE d.date BETWEEN ?1 AND ?2",
    )?;
    let rows = stmt.query_map(
        params![
            start.format(DATE_FMT).to_string(),
            last.format(DATE_FMT).to_string()
        ],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, bool>(2)?,
            ))
        },
    )?;

    let mut known = std::collections::HashMap::new();
    for row in rows {
        let (date, status, activity) = row?;
        known.insert(date, (status, activity));
    }

    let mut out = Vec::with_capacity(challenge.target_days as usize);
    for offset in 0..challenge.target_days {
        let date = start + Duration::days(offset);
        let key = date.format(DATE_FMT).to_string();

        let status = if date > today_date {
            "future".to_string()
        } else {
            match known.get(&key) {
                Some((s, _)) if s != "pending" => s.clone(),
                Some((_, true)) => "partial".to_string(),
                _ => "empty".to_string(),
            }
        };

        out.push(HeatmapCell {
            date: key,
            day_number: offset + 1,
            status,
        });
    }
    Ok(out)
}

// -------------------------------------------------- recordatorios (Fase 1)

/// Etiqueta y explicación de un recordatorio, derivadas del `kind`. Viven en
/// código y no en la base para poder corregir la redacción sin migraciones.
fn reminder_copy(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "morning" => (
            "Check-in matutino",
            "A qué hora te dormiste y a qué hora despertaste.",
        ),
        "meal" => ("Recordatorio de comida", "Registra lo que comiste."),
        "workout" => (
            "Recordatorio de ejercicio",
            "Antes de que se te vaya el día.",
        ),
        "water" => ("Agua", "Cada par de horas mientras estés despierto."),
        "evening" => (
            "Check-in nocturno",
            "Cerrar el día como completo o fallido.",
        ),
        _ => ("Recordatorio", ""),
    }
}

fn is_interval_kind(kind: &str) -> bool {
    kind == "water"
}

pub fn reminders(conn: &Connection) -> AppResult<Vec<Reminder>> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, time_of_day, enabled FROM reminder
         ORDER BY CASE kind
            WHEN 'morning' THEN 1 WHEN 'meal' THEN 2 WHEN 'workout' THEN 3
            WHEN 'water' THEN 4 WHEN 'evening' THEN 5 ELSE 6 END",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, i64>(3)?,
        ))
    })?;

    let mut out = Vec::new();
    for row in rows {
        let (id, kind, time_of_day, enabled) = row?;
        let (label, description) = reminder_copy(&kind);
        out.push(Reminder {
            interval_based: is_interval_kind(&kind),
            label: label.to_string(),
            description: description.to_string(),
            id,
            kind,
            time_of_day,
            enabled: enabled != 0,
        });
    }
    Ok(out)
}

pub fn set_reminder(
    conn: &Connection,
    id: &str,
    enabled: Option<bool>,
    time_of_day: Option<&str>,
) -> AppResult<()> {
    if let Some(t) = time_of_day {
        if chrono::NaiveTime::parse_from_str(t, "%H:%M").is_err() {
            return Err(AppError::invalid(format!(
                "hora no válida: {t} (usa HH:MM)"
            )));
        }
        conn.execute(
            "UPDATE reminder SET time_of_day = ?2 WHERE id = ?1",
            params![id, t],
        )?;
    }
    if let Some(e) = enabled {
        conn.execute(
            "UPDATE reminder SET enabled = ?2 WHERE id = ?1",
            params![id, e as i64],
        )?;
    }
    Ok(())
}

/// Recordatorios que deberían dispararse en este instante. No decide el texto:
/// de eso se encarga el planificador, que sí mira los datos del día.
pub fn due_reminders(conn: &Connection, now: NaiveDateTime) -> AppResult<Vec<Reminder>> {
    if get_int(conn, "notifications", 1) == 0 {
        return Ok(vec![]);
    }

    // Horario de silencio: de `quiet_start` a `quiet_end`. Nada suena ahí.
    let hour = now.hour() as i64;
    let quiet_start = get_int(conn, "quiet_start", 22);
    let quiet_end = get_int(conn, "quiet_end", 7);
    if hour >= quiet_start || hour < quiet_end {
        return Ok(vec![]);
    }

    let every_hours = get_int(conn, "water_every_hours", 2).max(1);
    let logical_today = daycut::logical_date(now, cutoff_hour(conn));

    let mut out = Vec::new();
    for r in reminders(conn)? {
        if !r.enabled {
            continue;
        }
        let last: Option<String> = conn
            .query_row(
                "SELECT last_fired_at FROM reminder WHERE id = ?1",
                [&r.id],
                |row| row.get(0),
            )
            .optional()?
            .flatten();
        let last_dt = last
            .as_deref()
            .and_then(|s| NaiveDateTime::parse_from_str(s, TS_FMT).ok());

        let debe = if r.interval_based {
            match last_dt {
                Some(prev) => (now - prev).num_hours() >= every_hours,
                None => true,
            }
        } else {
            let Ok(hora) = chrono::NaiveTime::parse_from_str(&r.time_of_day, "%H:%M") else {
                continue;
            };
            // Ya pasó su hora hoy y no ha sonado en este día lógico.
            now.time() >= hora
                && last_dt
                    .map(|prev| daycut::logical_date(prev, cutoff_hour(conn)) < logical_today)
                    .unwrap_or(true)
        };

        if debe {
            out.push(r);
        }
    }
    Ok(out)
}

pub fn mark_reminder_fired(conn: &Connection, id: &str, now: NaiveDateTime) -> AppResult<()> {
    conn.execute(
        "UPDATE reminder SET last_fired_at = ?2 WHERE id = ?1",
        params![id, now.format(TS_FMT).to_string()],
    )?;
    Ok(())
}

// ------------------------------------------------------ racha rota (P14)

/// Cifras acumuladas del intento activo. Son lo que se le enseña a alguien que
/// acaba de romper la racha: lo que construyó, no lo que perdió.
pub fn challenge_totals(conn: &Connection) -> AppResult<ChallengeTotals> {
    let Some(challenge) = active_challenge(conn)? else {
        return Ok(ChallengeTotals {
            days_survived: 0,
            workouts: 0,
            avg_sleep_min: None,
            weight_delta_kg: None,
            work_hours: 0.0,
        });
    };
    let start = challenge.start_date.clone();
    let end = today(conn).format(DATE_FMT).to_string();

    let workouts: i64 = conn.query_row(
        "SELECT count(*) FROM workout WHERE date BETWEEN ?1 AND ?2",
        params![start, end],
        |r| r.get(0),
    )?;
    let avg_sleep: Option<f64> = conn.query_row(
        "SELECT avg(minutes) FROM sleep_log WHERE date BETWEEN ?1 AND ?2",
        params![start, end],
        |r| r.get(0),
    )?;
    let work_min: i64 = conn.query_row(
        "SELECT coalesce(sum(minutes), 0) FROM work_session WHERE date BETWEEN ?1 AND ?2",
        params![start, end],
        |r| r.get(0),
    )?;
    let days_survived: i64 = conn.query_row(
        "SELECT count(*) FROM day WHERE status = 'complete' AND date BETWEEN ?1 AND ?2",
        params![start, end],
        |r| r.get(0),
    )?;

    // El peso se compara primer registro contra último, no máximos y mínimos.
    let primero: Option<f64> = conn
        .query_row(
            "SELECT kg FROM weight_log WHERE date BETWEEN ?1 AND ?2 ORDER BY date ASC LIMIT 1",
            params![start, end],
            |r| r.get(0),
        )
        .optional()?;
    let ultimo: Option<f64> = conn
        .query_row(
            "SELECT kg FROM weight_log WHERE date BETWEEN ?1 AND ?2 ORDER BY date DESC LIMIT 1",
            params![start, end],
            |r| r.get(0),
        )
        .optional()?;

    Ok(ChallengeTotals {
        days_survived,
        workouts,
        avg_sleep_min: avg_sleep.map(|m| m.round() as i64),
        weight_delta_kg: match (primero, ultimo) {
            (Some(a), Some(b)) if (b - a).abs() > 0.05 => Some(b - a),
            _ => None,
        },
        work_hours: (work_min as f64 / 60.0 * 10.0).round() / 10.0,
    })
}

/// Reúne todo lo de la pantalla de racha rota para un día fallido concreto.
pub fn broken_streak(conn: &Connection, date: NaiveDate) -> AppResult<BrokenStreak> {
    let view = today_view(conn, date)?;
    let failed: Vec<String> = view
        .pillars
        .iter()
        .filter(|p| p.required && p.status != "done")
        .map(|p| pillar_noun(&p.key, &p.label))
        .collect();

    let day_number = view.day_number.unwrap_or(0);
    let next_attempt = challenge_count(conn)? + 1;
    let motivo = if failed.is_empty() {
        "Cerraste el día como fallido.".to_string()
    } else {
        format!("Fallaste {}.", coach::join_es(&failed))
    };

    Ok(BrokenStreak {
        day_number,
        date: view.date.clone(),
        weekday_label: view.weekday_label.clone(),
        failed_pillars: failed,
        totals: challenge_totals(conn)?,
        next_attempt,
        message: coach::streak_broken_message(day_number, next_attempt, &motivo),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    fn d(s: &str) -> NaiveDate {
        parse_date(s).expect("fecha de test")
    }
    fn dt(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").expect("instante de test")
    }

    fn setup() -> crate::db::Db {
        let db = open_in_memory().expect("abrir base en memoria");
        db.with(|c| {
            create_challenge(c, "Intento #1", d("2026-08-01"), 75, &Rules::default())?;
            Ok(())
        })
        .expect("crear reto");
        db
    }

    #[test]
    fn ensure_day_calcula_el_numero_de_dia() {
        let db = setup();
        db.with(|c| {
            ensure_day(c, d("2026-08-19"))?;
            let n: Option<i64> = c.query_row(
                "SELECT day_number FROM day WHERE date = '2026-08-19'",
                [],
                |r| r.get(0),
            )?;
            assert_eq!(n, Some(19));
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn guardar_sueno_calcula_minutos_y_rechaza_datos_absurdos() {
        let db = setup();
        db.with(|c| {
            let min = save_sleep(
                c,
                d("2026-08-19"),
                dt("2026-08-18 23:30"),
                dt("2026-08-19 06:10"),
                Some(3),
            )?;
            assert_eq!(min, 400);

            let err = save_sleep(
                c,
                d("2026-08-19"),
                dt("2026-08-19 08:00"),
                dt("2026-08-19 06:00"),
                None,
            );
            assert!(err.is_err(), "debe rechazar despertar antes de dormir");
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn las_calorias_son_opcionales_y_solo_suman_las_registradas() {
        let db = setup();
        db.with(|c| {
            let date = d("2026-08-19");
            add_meal(c, date, dt("2026-08-19 08:30"), "meal", "Avena", Some(320))?;
            add_meal(c, date, dt("2026-08-19 13:45"), "meal", "Ensalada", None)?;
            add_meal(
                c,
                date,
                dt("2026-08-19 17:00"),
                "snack",
                "Nueces",
                Some(180),
            )?;

            let s = meal_summary(c, date)?;
            assert_eq!(s.meals, 2);
            assert_eq!(s.snacks, 1);
            assert_eq!(s.calories, Some(500));
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn sin_ninguna_caloria_registrada_el_total_es_none() {
        let db = setup();
        db.with(|c| {
            let date = d("2026-08-19");
            add_meal(c, date, dt("2026-08-19 08:30"), "meal", "Avena", None)?;
            assert_eq!(meal_summary(c, date)?.calories, None);
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn el_agua_se_acumula_y_nunca_baja_de_cero() {
        let db = setup();
        db.with(|c| {
            let date = d("2026-08-19");
            assert_eq!(add_water(c, date, 500)?, 500);
            assert_eq!(add_water(c, date, 500)?, 1000);
            assert_eq!(add_water(c, date, -2000)?, 0);
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn la_glucosa_se_vincula_a_la_comida_de_las_ultimas_tres_horas() {
        let db = setup();
        db.with(|c| {
            let date = d("2026-08-19");
            let meal_id = add_meal(c, date, dt("2026-08-19 13:00"), "meal", "Pizza", None)?;
            add_glucose(c, date, dt("2026-08-19 15:00"), 138, "post_meal_2h", None)?;
            let linked: Option<String> = c.query_row(
                "SELECT linked_meal_id FROM glucose_reading WHERE date = ?1",
                [date.format(DATE_FMT).to_string()],
                |r| r.get(0),
            )?;
            assert_eq!(linked, Some(meal_id));
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn la_glucosa_no_se_vincula_a_una_comida_lejana() {
        let db = setup();
        db.with(|c| {
            let date = d("2026-08-19");
            add_meal(c, date, dt("2026-08-19 08:00"), "meal", "Avena", None)?;
            add_glucose(c, date, dt("2026-08-19 15:00"), 92, "random", None)?;
            let linked: Option<String> = c.query_row(
                "SELECT linked_meal_id FROM glucose_reading WHERE date = ?1",
                [date.format(DATE_FMT).to_string()],
                |r| r.get(0),
            )?;
            assert_eq!(linked, None);
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn un_dia_pausado_no_rompe_la_racha_pero_no_suma() {
        let db = setup();
        db.with(|c| {
            for day in 15..=17 {
                set_day_status(c, d(&format!("2026-08-{day:02}")), "complete")?;
            }
            set_day_status(c, d("2026-08-18"), "skipped")?;
            set_day_status(c, d("2026-08-19"), "complete")?;

            assert_eq!(current_streak(c, d("2026-08-19"))?, 4);
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn un_dia_fallido_corta_la_racha() {
        let db = setup();
        db.with(|c| {
            set_day_status(c, d("2026-08-17"), "complete")?;
            set_day_status(c, d("2026-08-18"), "failed")?;
            set_day_status(c, d("2026-08-19"), "complete")?;
            assert_eq!(current_streak(c, d("2026-08-19"))?, 1);
            assert_eq!(longest_streak(c)?, 1);
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn hoy_pendiente_no_borra_la_racha_de_ayer() {
        let db = setup();
        db.with(|c| {
            set_day_status(c, d("2026-08-17"), "complete")?;
            set_day_status(c, d("2026-08-18"), "complete")?;
            ensure_day(c, d("2026-08-19"))?; // queda 'pending'
            assert_eq!(current_streak(c, d("2026-08-19"))?, 2);
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn cuenta_noches_cortas_seguidas() {
        let db = setup();
        db.with(|c| {
            for (day, bed, wake) in [
                (17, "2026-08-17 01:00", "2026-08-17 06:00"), // 5h
                (18, "2026-08-18 01:30", "2026-08-18 06:00"), // 4h 30min
                (19, "2026-08-19 00:30", "2026-08-19 05:30"), // 5h
            ] {
                save_sleep(c, d(&format!("2026-08-{day}")), dt(bed), dt(wake), None)?;
            }
            assert_eq!(short_nights_streak(c, d("2026-08-19"))?, 3);
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn los_dias_sin_tocar_aparecen_como_pendientes_de_decidir() {
        let db = setup();
        db.with(|c| {
            touch_last_open(c, d("2026-08-15"))?;
            // El 16 sí se usó: tiene una comida.
            add_meal(
                c,
                d("2026-08-16"),
                dt("2026-08-16 13:00"),
                "meal",
                "Algo",
                None,
            )?;

            let faltantes = missing_days(c, d("2026-08-19"))?;
            let fechas: Vec<_> = faltantes.iter().map(|m| m.date.as_str()).collect();
            assert_eq!(fechas, vec!["2026-08-15", "2026-08-17", "2026-08-18"]);
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn la_vista_de_hoy_marca_los_pilares_pendientes() {
        let db = setup();
        db.with(|c| {
            let date = d("2026-08-19");
            save_sleep(
                c,
                date,
                dt("2026-08-18 23:30"),
                dt("2026-08-19 06:10"),
                None,
            )?;
            add_meal(c, date, dt("2026-08-19 08:30"), "meal", "Avena", None)?;
            add_water(c, date, 1800)?;

            let v = today_view(c, date)?;
            assert_eq!(v.day_number, Some(19));
            assert_eq!(v.meals_count, 1);
            assert_eq!(v.water_ml, 1800);

            let by_key = |k: &str| {
                v.pillars
                    .iter()
                    .find(|p| p.key == k)
                    .map(|p| p.status.clone())
                    .unwrap_or_default()
            };
            assert_eq!(by_key("sleep"), "partial"); // 6h40 < meta de 7h
            assert_eq!(by_key("diet"), "done");
            assert_eq!(by_key("water"), "partial");
            assert_eq!(by_key("workout"), "missing");
            assert_eq!(by_key("reading"), "missing");
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn borrar_un_registro_inexistente_da_error_legible() {
        let db = setup();
        db.with(|c| {
            let err = delete_entry(c, "meal", "no-existe").unwrap_err();
            assert!(err.to_string().contains("ya no existe"));

            let err = delete_entry(c, "tabla_rara", "x").unwrap_err();
            assert!(err.to_string().contains("desconocido"));
            Ok(())
        })
        .expect("consulta");
    }
}
