//! Agregaciones para las pantallas de estadísticas (Fase 2) y el informe.
//!
//! Todo lo que sale de aquí son números crudos: el formato lo pone la UI con
//! los mismos helpers que usa el resto de la app. Así una cifra se ve igual en
//! una tarjeta, en un eje y en el PDF.
//!
//! Regla del §6 que se respeta aquí: el peso se expone crudo Y con promedio
//! móvil de 7 días, y la UI grafica la línea del promedio. El dato diario
//! oscila ±1.5 kg por agua y sal; verlo solo te hace creer que fallaste.

use chrono::{Datelike, Duration, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db::models::HeatmapCell;
use crate::db::queries::{
    self, active_challenge, glucose_context_label, parse_date, today, weekday_label, DATE_FMT,
};
use crate::error::AppResult;

// --------------------------------------------------------------- utilidades

/// Rango de fechas a graficar. "7" y "30" son días hacia atrás desde hoy;
/// "all" abarca el reto activo completo.
pub fn range_bounds(conn: &Connection, range: &str) -> AppResult<(NaiveDate, NaiveDate)> {
    let end = today(conn);
    let start = match range {
        "7" => end - Duration::days(6),
        "30" => end - Duration::days(29),
        _ => match active_challenge(conn)? {
            Some(c) => parse_date(&c.start_date)?,
            None => end - Duration::days(29),
        },
    };
    Ok((start.min(end), end))
}

/// Promedio móvil de `window` puntos. Devuelve `None` mientras no haya
/// suficientes datos: inventar el arranque de la curva es mentir.
fn moving_average(values: &[Option<f64>], window: usize) -> Vec<Option<f64>> {
    values
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let from = i + 1;
            if from < window {
                return None;
            }
            let slice = &values[from - window..from];
            let presentes: Vec<f64> = slice.iter().filter_map(|v| *v).collect();
            // Con menos de la mitad de la ventana el promedio no dice nada.
            if presentes.len() * 2 < window {
                None
            } else {
                Some(presentes.iter().sum::<f64>() / presentes.len() as f64)
            }
        })
        .collect()
}

/// Coeficiente de Pearson. `None` si hay menos de 5 pares o no hay varianza.
fn pearson(pairs: &[(f64, f64)]) -> Option<f64> {
    if pairs.len() < 5 {
        return None;
    }
    let n = pairs.len() as f64;
    let mx = pairs.iter().map(|p| p.0).sum::<f64>() / n;
    let my = pairs.iter().map(|p| p.1).sum::<f64>() / n;

    let mut num = 0.0;
    let mut dx = 0.0;
    let mut dy = 0.0;
    for (x, y) in pairs {
        let a = x - mx;
        let b = y - my;
        num += a * b;
        dx += a * a;
        dy += b * b;
    }
    if dx <= f64::EPSILON || dy <= f64::EPSILON {
        return None;
    }
    Some(num / (dx * dy).sqrt())
}

fn dates_between(start: NaiveDate, end: NaiveDate) -> Vec<NaiveDate> {
    let mut out = Vec::new();
    let mut cursor = start;
    while cursor <= end {
        out.push(cursor);
        cursor += Duration::days(1);
    }
    out
}

// ------------------------------------------------------------- sueño (P7)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SleepPoint {
    pub date: String,
    pub label: String,
    pub minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistBucket {
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XY {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SleepStats {
    pub goal_min: i64,
    pub avg_min: Option<i64>,
    pub best_min: Option<i64>,
    pub worst_min: Option<i64>,
    /// Suma de (dormido − meta) sobre los días con dato. Negativo es déficit.
    pub balance_min: i64,
    pub nights: i64,
    pub daily: Vec<SleepPoint>,
    pub moving_avg: Vec<Option<f64>>,
    pub bedtimes: Vec<HistBucket>,
    pub modal_bedtime: Option<String>,
    pub sleep_vs_energy: Vec<XY>,
    pub correlation: Option<f64>,
}

pub fn sleep_stats(conn: &Connection, range: &str) -> AppResult<SleepStats> {
    let (start, end) = range_bounds(conn, range)?;
    let goal_min = queries::get_int(conn, "sleep_goal_min", 420);

    let mut stmt = conn.prepare(
        "SELECT date, minutes, bedtime FROM sleep_log
         WHERE date BETWEEN ?1 AND ?2 ORDER BY date ASC",
    )?;
    let rows = stmt.query_map(
        params![
            start.format(DATE_FMT).to_string(),
            end.format(DATE_FMT).to_string()
        ],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
            ))
        },
    )?;

    let mut por_fecha = std::collections::HashMap::new();
    let mut bedtime_buckets = std::collections::BTreeMap::new();
    for row in rows {
        let (date, minutes, bedtime) = row?;
        por_fecha.insert(date, minutes);

        // Cubos de media hora, del reloj de la hora de dormir.
        if bedtime.len() >= 16 {
            let h: u32 = bedtime[11..13].parse().unwrap_or(0);
            let m: u32 = bedtime[14..16].parse().unwrap_or(0);
            let etiqueta = format!("{:02}:{}", h, if m < 30 { "00" } else { "30" });
            *bedtime_buckets.entry(etiqueta).or_insert(0i64) += 1;
        }
    }

    let mut daily = Vec::new();
    let mut serie = Vec::new();
    for d in dates_between(start, end) {
        let key = d.format(DATE_FMT).to_string();
        let minutes = por_fecha.get(&key).copied();
        serie.push(minutes.map(|m| m as f64));
        daily.push(SleepPoint {
            label: format!("{}/{}", d.day(), d.month()),
            date: key,
            minutes,
        });
    }

    let con_dato: Vec<i64> = daily.iter().filter_map(|p| p.minutes).collect();
    let nights = con_dato.len() as i64;

    // La hora de dormir se ordena empezando a las 18:00 para que la madrugada
    // quede a la derecha y no parta el histograma por la mitad.
    let mut bedtimes: Vec<HistBucket> = bedtime_buckets
        .into_iter()
        .map(|(label, count)| HistBucket { label, count })
        .collect();
    bedtimes.sort_by_key(|b| {
        let h: i32 = b.label[0..2].parse().unwrap_or(0);
        if h < 12 {
            h + 24
        } else {
            h
        }
    });
    let modal_bedtime = bedtimes
        .iter()
        .max_by_key(|b| b.count)
        .map(|b| b.label.clone());

    let mut stmt = conn.prepare(
        "SELECT s.minutes, m.energy FROM sleep_log s
         JOIN mood_log m ON m.date = s.date
         WHERE s.date BETWEEN ?1 AND ?2",
    )?;
    let pares = stmt
        .query_map(
            params![
                start.format(DATE_FMT).to_string(),
                end.format(DATE_FMT).to_string()
            ],
            |r| {
                Ok((
                    r.get::<_, i64>(0)? as f64 / 60.0,
                    r.get::<_, i64>(1)? as f64,
                ))
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(SleepStats {
        goal_min,
        avg_min: (!con_dato.is_empty())
            .then(|| con_dato.iter().sum::<i64>() / con_dato.len() as i64),
        best_min: con_dato.iter().max().copied(),
        worst_min: con_dato.iter().min().copied(),
        balance_min: con_dato.iter().map(|m| m - goal_min).sum(),
        nights,
        moving_avg: moving_average(&serie, 7),
        daily,
        bedtimes,
        modal_bedtime,
        sleep_vs_energy: pares.iter().map(|(x, y)| XY { x: *x, y: *y }).collect(),
        correlation: pearson(&pares),
    })
}

// --------------------------------------------------------- ejercicio (P8)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeekBar {
    pub label: String,
    pub indoor_min: i64,
    pub outdoor_min: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaloriesDay {
    pub date: String,
    pub label: String,
    pub intake: i64,
    pub burned: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KindBar {
    pub label: String,
    pub minutes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkoutStats {
    pub sessions: i64,
    pub total_min: i64,
    pub weekly_avg_min: i64,
    pub days_without: i64,
    pub weekly_goal_min: i64,
    pub weekly: Vec<WeekBar>,
    pub calories: Vec<CaloriesDay>,
    pub by_kind: Vec<KindBar>,
}

pub fn workout_stats(conn: &Connection, range: &str) -> AppResult<WorkoutStats> {
    let (start, end) = range_bounds(conn, range)?;
    let s = start.format(DATE_FMT).to_string();
    let e = end.format(DATE_FMT).to_string();

    let (sessions, total_min): (i64, i64) = conn.query_row(
        "SELECT count(*), coalesce(sum(duration_min), 0) FROM workout WHERE date BETWEEN ?1 AND ?2",
        params![s, e],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    // Semanas ISO, etiquetadas por el lunes que las abre.
    let mut stmt = conn.prepare(
        "SELECT date, duration_min, is_outdoor FROM workout WHERE date BETWEEN ?1 AND ?2",
    )?;
    let rows = stmt.query_map(params![s, e], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)? != 0,
        ))
    })?;

    let mut semanas: std::collections::BTreeMap<NaiveDate, (i64, i64)> =
        std::collections::BTreeMap::new();
    let mut por_dia: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for row in rows {
        let (date, min, outdoor) = row?;
        let d = parse_date(&date)?;
        let lunes = d - Duration::days(d.weekday().num_days_from_monday() as i64);
        let entry = semanas.entry(lunes).or_insert((0, 0));
        if outdoor {
            entry.1 += min;
        } else {
            entry.0 += min;
        }
        *por_dia.entry(date).or_insert(0) += min;
    }

    let weekly: Vec<WeekBar> = semanas
        .iter()
        .map(|(lunes, (indoor, outdoor))| WeekBar {
            label: format!("{}/{}", lunes.day(), lunes.month()),
            indoor_min: *indoor,
            outdoor_min: *outdoor,
        })
        .collect();

    let dias = dates_between(start, end);
    let days_without = dias
        .iter()
        .filter(|d| !por_dia.contains_key(&d.format(DATE_FMT).to_string()))
        .count() as i64;

    let weekly_avg_min = if weekly.is_empty() {
        0
    } else {
        weekly
            .iter()
            .map(|w| w.indoor_min + w.outdoor_min)
            .sum::<i64>()
            / weekly.len() as i64
    };

    // Calorías: solo los días que tienen AMBOS datos. Mezclar un lado con el
    // otro vacío produce una gráfica que miente.
    let mut stmt = conn.prepare(
        "SELECT d.date,
                (SELECT sum(calories) FROM meal m WHERE m.date = d.date AND m.calories IS NOT NULL),
                (SELECT sum(calories_burned) FROM workout w
                   WHERE w.date = d.date AND w.calories_burned IS NOT NULL)
         FROM day d WHERE d.date BETWEEN ?1 AND ?2 ORDER BY d.date ASC",
    )?;
    let calories = stmt
        .query_map(params![s, e], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<i64>>(1)?,
                r.get::<_, Option<i64>>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter_map(|(date, intake, burned)| match (intake, burned) {
            (Some(i), Some(b)) => {
                let d = parse_date(&date).ok()?;
                Some(CaloriesDay {
                    label: format!("{}/{}", d.day(), d.month()),
                    date,
                    intake: i,
                    burned: b,
                })
            }
            _ => None,
        })
        .collect();

    let mut stmt = conn.prepare(
        "SELECT kind, sum(duration_min) FROM workout WHERE date BETWEEN ?1 AND ?2
         GROUP BY kind ORDER BY sum(duration_min) DESC",
    )?;
    let by_kind = stmt
        .query_map(params![s, e], |r| {
            Ok(KindBar {
                label: queries::workout_kind_label(&r.get::<_, String>(0)?).to_string(),
                minutes: r.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(WorkoutStats {
        sessions,
        total_min,
        weekly_avg_min,
        days_without,
        weekly_goal_min: queries::get_int(conn, "workout_goal_min", 45) * 7,
        weekly,
        calories,
        by_kind,
    })
}

// ----------------------------------------------------------- glucosa (P9)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlucosePoint {
    pub id: String,
    pub date: String,
    pub time: String,
    pub value: i64,
    pub context: String,
    pub context_label: String,
    pub meal: Option<String>,
    pub notes: Option<String>,
    pub out_of_range: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextAvg {
    pub context: String,
    pub label: String,
    pub avg: i64,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlucoseStats {
    pub readings: Vec<GlucosePoint>,
    pub by_context: Vec<ContextAvg>,
    pub total: i64,
    pub out_of_range: i64,
    pub avg_fasting: Option<i64>,
    pub avg_post_meal: Option<i64>,
}

/// Rango de referencia por contexto. **Esto no es un diagnóstico**: son las
/// bandas que usa cualquier glucómetro para pintar el fondo, y sirven solo
/// para situar el punto en la gráfica.
fn glucose_in_range(context: &str, value: i64) -> bool {
    match context {
        "fasting" | "pre_meal" | "pre_workout" => (70..=99).contains(&value),
        "post_meal_2h" => value < 140,
        _ => (70..=140).contains(&value),
    }
}

pub fn glucose_stats(conn: &Connection, range: &str) -> AppResult<GlucoseStats> {
    let (start, end) = range_bounds(conn, range)?;
    let s = start.format(DATE_FMT).to_string();
    let e = end.format(DATE_FMT).to_string();

    let mut stmt = conn.prepare(
        "SELECT g.id, g.date, g.measured_at, g.value_mgdl, g.context, g.notes, m.description
         FROM glucose_reading g
         LEFT JOIN meal m ON m.id = g.linked_meal_id
         WHERE g.date BETWEEN ?1 AND ?2
         ORDER BY g.measured_at ASC",
    )?;
    let readings: Vec<GlucosePoint> = stmt
        .query_map(params![s, e], |r| {
            let context: String = r.get(4)?;
            let value: i64 = r.get(3)?;
            let measured: String = r.get(2)?;
            Ok(GlucosePoint {
                id: r.get(0)?,
                date: r.get(1)?,
                time: measured.chars().skip(11).take(5).collect(),
                value,
                context_label: glucose_context_label(&context).to_string(),
                out_of_range: !glucose_in_range(&context, value),
                context,
                notes: r.get(5)?,
                meal: r.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT context, round(avg(value_mgdl)), count(*) FROM glucose_reading
         WHERE date BETWEEN ?1 AND ?2 GROUP BY context ORDER BY count(*) DESC",
    )?;
    let by_context = stmt
        .query_map(params![s, e], |r| {
            let context: String = r.get(0)?;
            Ok(ContextAvg {
                label: glucose_context_label(&context).to_string(),
                context,
                avg: r.get::<_, f64>(1)? as i64,
                count: r.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let avg_de =
        |ctx: &str| -> Option<i64> { by_context.iter().find(|c| c.context == ctx).map(|c| c.avg) };

    Ok(GlucoseStats {
        total: readings.len() as i64,
        out_of_range: readings.iter().filter(|r| r.out_of_range).count() as i64,
        avg_fasting: avg_de("fasting"),
        avg_post_meal: avg_de("post_meal_2h"),
        readings,
        by_context,
    })
}

// -------------------------------------------------- peso y trabajo (P10)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeightPoint {
    pub date: String,
    pub label: String,
    pub kg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkDay {
    pub date: String,
    pub label: String,
    /// Minutos por categoría, alineado con `categories`.
    pub values: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyStats {
    pub current_kg: Option<f64>,
    pub delta_kg: Option<f64>,
    pub weekly_delta_kg: Option<f64>,
    pub points: Vec<WeightPoint>,
    pub moving_avg: Vec<Option<f64>>,
    pub categories: Vec<String>,
    pub work_daily: Vec<WorkDay>,
    pub work_hours_week: f64,
    pub work_avg_daily_h: f64,
    pub work_goal_min: i64,
}

pub fn body_stats(conn: &Connection, range: &str) -> AppResult<BodyStats> {
    let (start, end) = range_bounds(conn, range)?;
    let s = start.format(DATE_FMT).to_string();
    let e = end.format(DATE_FMT).to_string();

    let mut stmt =
        conn.prepare("SELECT date, kg FROM weight_log WHERE date BETWEEN ?1 AND ?2 ORDER BY date")?;
    let pesos: std::collections::HashMap<String, f64> = stmt
        .query_map(params![s, e], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))
        })?
        .collect::<Result<_, _>>()?;

    let mut points = Vec::new();
    let mut serie = Vec::new();
    for d in dates_between(start, end) {
        let key = d.format(DATE_FMT).to_string();
        let kg = pesos.get(&key).copied();
        serie.push(kg);
        points.push(WeightPoint {
            label: format!("{}/{}", d.day(), d.month()),
            date: key,
            kg,
        });
    }
    let moving_avg = moving_average(&serie, 7);

    let con_dato: Vec<f64> = points.iter().filter_map(|p| p.kg).collect();
    let current_kg = con_dato.last().copied();
    let delta_kg = match (con_dato.first(), con_dato.last()) {
        (Some(a), Some(b)) if con_dato.len() > 1 => Some(b - a),
        _ => None,
    };
    let semanas = ((end - start).num_days() as f64 / 7.0).max(1.0);
    let weekly_delta_kg = delta_kg.map(|d| (d / semanas * 100.0).round() / 100.0);

    // Trabajo apilado por categoría. Máximo tres series (§ paleta): si hay más,
    // las menores se agrupan en "Otro" en vez de inventar un cuarto color.
    let mut stmt = conn.prepare(
        "SELECT category, sum(minutes) FROM work_session WHERE date BETWEEN ?1 AND ?2
         GROUP BY category ORDER BY sum(minutes) DESC",
    )?;
    let mut categories: Vec<String> = stmt
        .query_map(params![s, e], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    let sobran = categories.len() > 3;
    if sobran {
        categories.truncate(3);
    }

    let mut stmt = conn.prepare(
        "SELECT date, category, sum(minutes) FROM work_session
         WHERE date BETWEEN ?1 AND ?2 GROUP BY date, category",
    )?;
    let filas = stmt
        .query_map(params![s, e], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut etiquetas = categories.clone();
    if sobran {
        etiquetas.push("Otro".to_string());
    }

    let mut por_dia: std::collections::HashMap<String, Vec<i64>> = std::collections::HashMap::new();
    for (date, category, minutes) in filas {
        let fila = por_dia
            .entry(date)
            .or_insert_with(|| vec![0; etiquetas.len()]);
        match categories.iter().position(|c| *c == category) {
            Some(i) => fila[i] += minutes,
            None if sobran => {
                let ultimo = etiquetas.len() - 1;
                fila[ultimo] += minutes;
            }
            None => {}
        }
    }

    let mut work_daily = Vec::new();
    let mut total_min = 0i64;
    for d in dates_between(start, end) {
        let key = d.format(DATE_FMT).to_string();
        let values = por_dia
            .get(&key)
            .cloned()
            .unwrap_or_else(|| vec![0; etiquetas.len()]);
        total_min += values.iter().sum::<i64>();
        work_daily.push(WorkDay {
            label: format!("{}/{}", d.day(), d.month()),
            date: key,
            values,
        });
    }

    let dias = work_daily.len().max(1) as f64;
    let ultimos_7: i64 = work_daily
        .iter()
        .rev()
        .take(7)
        .map(|w| w.values.iter().sum::<i64>())
        .sum();

    Ok(BodyStats {
        current_kg,
        delta_kg,
        weekly_delta_kg,
        points,
        moving_avg,
        categories: etiquetas,
        work_daily,
        work_hours_week: (ultimos_7 as f64 / 60.0 * 10.0).round() / 10.0,
        work_avg_daily_h: (total_min as f64 / dias / 60.0 * 10.0).round() / 10.0,
        work_goal_min: queries::get_int(conn, "work_goal_min", 240),
    })
}

// ----------------------------------------------------------- correlaciones

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Correlation {
    pub label: String,
    pub caption: String,
    pub r: f64,
    pub n: i64,
}

/// Las tres correlaciones del §4 fase 3. Solo se devuelven las que tienen
/// suficientes pares; una r calculada sobre cuatro puntos es ruido con
/// decimales.
pub fn correlations(conn: &Connection) -> AppResult<Vec<Correlation>> {
    let consultas: [(&str, &str, &str); 3] = [
        (
            "Sueño y energía",
            "Más horas dormidas, ¿más energía al despertar?",
            "SELECT s.minutes / 60.0, m.energy FROM sleep_log s
             JOIN mood_log m ON m.date = s.date",
        ),
        (
            "Sueño y glucosa en ayunas",
            "Horas dormidas contra la lectura en ayunas del día siguiente.",
            "SELECT s.minutes / 60.0, g.value_mgdl FROM sleep_log s
             JOIN glucose_reading g ON g.date = s.date AND g.context = 'fasting'",
        ),
        (
            "Ejercicio y ánimo",
            "Minutos de ejercicio contra el ánimo de ese día.",
            "SELECT sum(w.duration_min), m.mood FROM workout w
             JOIN mood_log m ON m.date = w.date GROUP BY w.date, m.mood",
        ),
    ];

    let mut out = Vec::new();
    for (label, caption, sql) in consultas {
        let mut stmt = conn.prepare(sql)?;
        let pares = stmt
            .query_map([], |r| Ok((r.get::<_, f64>(0)?, r.get::<_, f64>(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        if let Some(r) = pearson(&pares) {
            out.push(Correlation {
                label: label.to_string(),
                caption: caption.to_string(),
                r: (r * 100.0).round() / 100.0,
                n: pares.len() as i64,
            });
        }
    }
    Ok(out)
}

// ------------------------------------------------- reto completado (P15)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeforeAfter {
    pub label: String,
    pub before: Option<f64>,
    pub after: Option<f64>,
    pub unit: String,
    /// "up" si subir es mejor, "down" si bajar es mejor, "none" si no aplica.
    pub better: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Completion {
    pub name: String,
    pub start_date: String,
    pub end_date: String,
    pub target_days: i64,
    pub complete_days: i64,
    pub finished: bool,
    pub tiles: Vec<BeforeAfter>,
    pub heatmap: Vec<HeatmapCell>,
    pub weight: Vec<Option<f64>>,
    pub sleep: Vec<Option<f64>>,
    pub glucose: Vec<Option<f64>>,
}

/// Compara el primer tercio del reto contra el último. Usar solo el primer y el
/// último día haría que un mal día cualquiera decidiera todo el resumen.
fn antes_despues(valores: &[f64]) -> (Option<f64>, Option<f64>) {
    if valores.len() < 2 {
        return (valores.first().copied(), valores.last().copied());
    }
    let tercio = (valores.len() / 3).max(1);
    let media = |s: &[f64]| s.iter().sum::<f64>() / s.len() as f64;
    (
        Some(media(&valores[..tercio])),
        Some(media(&valores[valores.len() - tercio..])),
    )
}

pub fn completion(conn: &Connection) -> AppResult<Option<Completion>> {
    let Some(challenge) = active_challenge(conn)? else {
        return Ok(None);
    };
    let start = parse_date(&challenge.start_date)?;
    let end = (start + Duration::days(challenge.target_days - 1)).min(today(conn));
    let s = start.format(DATE_FMT).to_string();
    let e = end.format(DATE_FMT).to_string();

    let complete_days: i64 = conn.query_row(
        "SELECT count(*) FROM day WHERE status = 'complete' AND date BETWEEN ?1 AND ?2",
        params![s, e],
        |r| r.get(0),
    )?;

    let serie = |sql: &str| -> AppResult<Vec<f64>> {
        let mut stmt = conn.prepare(sql)?;
        let filas = stmt
            .query_map(params![&s, &e], |r| r.get::<_, f64>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(filas)
    };

    let pesos = serie("SELECT kg FROM weight_log WHERE date BETWEEN ?1 AND ?2 ORDER BY date")?;
    let suenos =
        serie("SELECT minutes / 60.0 FROM sleep_log WHERE date BETWEEN ?1 AND ?2 ORDER BY date")?;
    let ayunas = serie(
        "SELECT value_mgdl * 1.0 FROM glucose_reading
         WHERE context = 'fasting' AND date BETWEEN ?1 AND ?2 ORDER BY measured_at",
    )?;
    let energias =
        serie("SELECT energy * 1.0 FROM mood_log WHERE date BETWEEN ?1 AND ?2 ORDER BY date")?;

    let semanal = |sql: &str| -> AppResult<Vec<f64>> {
        let mut stmt = conn.prepare(sql)?;
        let filas = stmt
            .query_map(params![&s, &e], |r| r.get::<_, f64>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(filas)
    };
    let ejercicio = semanal(
        "SELECT sum(duration_min) * 1.0 FROM workout WHERE date BETWEEN ?1 AND ?2 GROUP BY date",
    )?;
    let trabajo = semanal(
        "SELECT sum(minutes) / 60.0 FROM work_session WHERE date BETWEEN ?1 AND ?2 GROUP BY date",
    )?;

    let tile = |label: &str, datos: &[f64], unit: &str, better: &str| {
        let (before, after) = antes_despues(datos);
        BeforeAfter {
            label: label.to_string(),
            before: before.map(|v| (v * 10.0).round() / 10.0),
            after: after.map(|v| (v * 10.0).round() / 10.0),
            unit: unit.to_string(),
            better: better.to_string(),
        }
    };

    Ok(Some(Completion {
        name: challenge.name.clone(),
        start_date: s.clone(),
        end_date: e.clone(),
        target_days: challenge.target_days,
        complete_days,
        finished: complete_days >= challenge.target_days,
        tiles: vec![
            tile("Peso", &pesos, "kg", "down"),
            tile("Sueño promedio", &suenos, "h", "up"),
            tile("Glucosa en ayunas", &ayunas, "mg/dL", "down"),
            tile("Ejercicio por día", &ejercicio, "min", "up"),
            tile("Trabajo por día", &trabajo, "h", "none"),
            tile("Energía", &energias, "/5", "up"),
        ],
        heatmap: queries::heatmap(conn)?,
        weight: pesos.into_iter().map(Some).collect(),
        sleep: suenos.into_iter().map(Some).collect(),
        glucose: ayunas.into_iter().map(Some).collect(),
    }))
}

// ------------------------------------------------------------- exportación

/// Vuelca todo a CSV o JSON. No hay filtros ni rangos: un export parcial que
/// parece completo es peor que no tenerlo.
pub fn export_all(conn: &Connection, format: &str) -> AppResult<String> {
    if format == "json" {
        return export_json(conn);
    }
    export_csv(conn)
}

fn escape_csv(v: &str) -> String {
    if v.contains(',') || v.contains('"') || v.contains('\n') {
        format!("\"{}\"", v.replace('"', "\"\""))
    } else {
        v.to_string()
    }
}

fn tabla_csv(conn: &Connection, tabla: &str, out: &mut String) -> AppResult<()> {
    let mut stmt = conn.prepare(&format!("SELECT * FROM {tabla}"))?;
    let columnas: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();

    out.push_str(&format!("\n# {tabla}\n"));
    out.push_str(&columnas.join(","));
    out.push('\n');

    let n = columnas.len();
    let mut filas = stmt.query([])?;
    while let Some(fila) = filas.next()? {
        let campos: Vec<String> = (0..n)
            .map(|i| match fila.get_ref(i) {
                Ok(rusqlite::types::ValueRef::Null) => String::new(),
                Ok(rusqlite::types::ValueRef::Integer(v)) => v.to_string(),
                Ok(rusqlite::types::ValueRef::Real(v)) => v.to_string(),
                Ok(rusqlite::types::ValueRef::Text(v)) => escape_csv(&String::from_utf8_lossy(v)),
                _ => String::new(),
            })
            .collect();
        out.push_str(&campos.join(","));
        out.push('\n');
    }
    Ok(())
}

const TABLAS: &[&str] = &[
    "challenge",
    "day",
    "sleep_log",
    "meal",
    "workout",
    "glucose_reading",
    "work_session",
    "water_log",
    "reading_log",
    "weight_log",
    "mood_log",
    "progress_photo",
];

fn export_csv(conn: &Connection) -> AppResult<String> {
    let mut out = String::from("# 75 HARD — export completo\n");
    for tabla in TABLAS {
        tabla_csv(conn, tabla, &mut out)?;
    }
    Ok(out)
}

fn export_json(conn: &Connection) -> AppResult<String> {
    let mut raiz = serde_json::Map::new();
    for tabla in TABLAS {
        let mut stmt = conn.prepare(&format!("SELECT * FROM {tabla}"))?;
        let columnas: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();
        let n = columnas.len();

        let mut filas_json = Vec::new();
        let mut filas = stmt.query([])?;
        while let Some(fila) = filas.next()? {
            let mut obj = serde_json::Map::new();
            for (i, col) in columnas.iter().enumerate().take(n) {
                let v = match fila.get_ref(i) {
                    Ok(rusqlite::types::ValueRef::Null) => serde_json::Value::Null,
                    Ok(rusqlite::types::ValueRef::Integer(v)) => serde_json::json!(v),
                    Ok(rusqlite::types::ValueRef::Real(v)) => serde_json::json!(v),
                    Ok(rusqlite::types::ValueRef::Text(v)) => {
                        serde_json::json!(String::from_utf8_lossy(v))
                    }
                    _ => serde_json::Value::Null,
                };
                obj.insert(col.clone(), v);
            }
            filas_json.push(serde_json::Value::Object(obj));
        }
        raiz.insert(tabla.to_string(), serde_json::Value::Array(filas_json));
    }
    Ok(serde_json::to_string_pretty(&serde_json::Value::Object(
        raiz,
    ))?)
}

// ---------------------------------------------------- informe para el médico

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorReport {
    pub generated_at: String,
    pub from: String,
    pub to: String,
    pub from_label: String,
    pub to_label: String,
    pub glucose: GlucoseStats,
    pub weight_start: Option<f64>,
    pub weight_end: Option<f64>,
    pub avg_sleep_min: Option<i64>,
    pub meals_per_day: f64,
    pub workouts: i64,
    pub days: i64,
}

/// Los datos del informe imprimible. La app registra y grafica: **no
/// diagnostica**. El informe existe para llegar a la consulta con 30 días de
/// lecturas en vez de "pues a veces me sale alto".
pub fn doctor_report(conn: &Connection, days: i64) -> AppResult<DoctorReport> {
    let end = today(conn);
    let start = end - Duration::days(days.clamp(7, 365) - 1);
    let s = start.format(DATE_FMT).to_string();
    let e = end.format(DATE_FMT).to_string();

    let glucose = {
        // Se reutiliza el mismo agregado que la pantalla, pero con rango propio.
        let (gs, ge) = (s.clone(), e.clone());
        let mut stmt = conn.prepare(
            "SELECT g.id, g.date, g.measured_at, g.value_mgdl, g.context, g.notes, m.description
             FROM glucose_reading g
             LEFT JOIN meal m ON m.id = g.linked_meal_id
             WHERE g.date BETWEEN ?1 AND ?2 ORDER BY g.measured_at ASC",
        )?;
        let readings: Vec<GlucosePoint> = stmt
            .query_map(params![gs, ge], |r| {
                let context: String = r.get(4)?;
                let value: i64 = r.get(3)?;
                let measured: String = r.get(2)?;
                Ok(GlucosePoint {
                    id: r.get(0)?,
                    date: r.get(1)?,
                    time: measured.chars().skip(11).take(5).collect(),
                    value,
                    context_label: glucose_context_label(&context).to_string(),
                    out_of_range: !glucose_in_range(&context, value),
                    context,
                    notes: r.get(5)?,
                    meal: r.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut stmt = conn.prepare(
            "SELECT context, round(avg(value_mgdl)), count(*) FROM glucose_reading
             WHERE date BETWEEN ?1 AND ?2 GROUP BY context ORDER BY count(*) DESC",
        )?;
        let by_context = stmt
            .query_map(params![s, e], |r| {
                let context: String = r.get(0)?;
                Ok(ContextAvg {
                    label: glucose_context_label(&context).to_string(),
                    context,
                    avg: r.get::<_, f64>(1)? as i64,
                    count: r.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let avg_de = |ctx: &str| by_context.iter().find(|c| c.context == ctx).map(|c| c.avg);
        GlucoseStats {
            total: readings.len() as i64,
            out_of_range: readings.iter().filter(|r| r.out_of_range).count() as i64,
            avg_fasting: avg_de("fasting"),
            avg_post_meal: avg_de("post_meal_2h"),
            readings,
            by_context,
        }
    };

    let peso = |orden: &str| -> AppResult<Option<f64>> {
        Ok(conn
            .query_row(
                &format!(
                    "SELECT kg FROM weight_log WHERE date BETWEEN ?1 AND ?2 ORDER BY date {orden} LIMIT 1"
                ),
                params![&s, &e],
                |r| r.get(0),
            )
            .optional()?)
    };
    let weight_start = peso("ASC")?;
    let weight_end = peso("DESC")?;

    let avg_sleep: Option<f64> = conn.query_row(
        "SELECT avg(minutes) FROM sleep_log WHERE date BETWEEN ?1 AND ?2",
        params![s, e],
        |r| r.get(0),
    )?;
    let comidas: i64 = conn.query_row(
        "SELECT count(*) FROM meal WHERE date BETWEEN ?1 AND ?2",
        params![s, e],
        |r| r.get(0),
    )?;
    let workouts: i64 = conn.query_row(
        "SELECT count(*) FROM workout WHERE date BETWEEN ?1 AND ?2",
        params![s, e],
        |r| r.get(0),
    )?;

    let dias = (end - start).num_days() + 1;
    Ok(DoctorReport {
        generated_at: queries::now_iso(),
        from_label: weekday_label(start),
        to_label: weekday_label(end),
        from: s,
        to: e,
        glucose,
        weight_start,
        weight_end,
        avg_sleep_min: avg_sleep.map(|m| m.round() as i64),
        meals_per_day: (comidas as f64 / dias as f64 * 10.0).round() / 10.0,
        workouts,
        days: dias,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_promedio_movil_espera_a_tener_ventana() {
        let v: Vec<Option<f64>> = (1..=10).map(|i| Some(i as f64)).collect();
        let ma = moving_average(&v, 7);
        assert!(
            ma[..6].iter().all(|x| x.is_none()),
            "los primeros 6 van vacíos"
        );
        assert_eq!(ma[6], Some(4.0)); // media de 1..7
        assert_eq!(ma[9], Some(7.0)); // media de 4..10
    }

    #[test]
    fn el_promedio_movil_tolera_huecos_pero_no_demasiados() {
        let mut v: Vec<Option<f64>> = (1..=7).map(|i| Some(i as f64)).collect();
        v[0] = None;
        assert!(moving_average(&v, 7)[6].is_some(), "un hueco se tolera");

        let casi_vacio: Vec<Option<f64>> = vec![Some(1.0), None, None, None, None, None, None];
        assert!(
            moving_average(&casi_vacio, 7)[6].is_none(),
            "seis huecos no"
        );
    }

    #[test]
    fn pearson_detecta_relacion_perfecta_y_ninguna() {
        let sube: Vec<(f64, f64)> = (1..=8).map(|i| (i as f64, i as f64 * 2.0)).collect();
        assert!((pearson(&sube).expect("hay r") - 1.0).abs() < 1e-9);

        let baja: Vec<(f64, f64)> = (1..=8).map(|i| (i as f64, -(i as f64))).collect();
        assert!((pearson(&baja).expect("hay r") + 1.0).abs() < 1e-9);

        let plano: Vec<(f64, f64)> = (1..=8).map(|i| (i as f64, 5.0)).collect();
        assert!(pearson(&plano).is_none(), "sin varianza no hay correlación");
    }

    #[test]
    fn pearson_no_opina_con_pocos_datos() {
        let pocos: Vec<(f64, f64)> = (1..=4).map(|i| (i as f64, i as f64)).collect();
        assert!(pearson(&pocos).is_none());
    }

    #[test]
    fn el_rango_de_glucosa_depende_del_contexto() {
        assert!(glucose_in_range("fasting", 92));
        assert!(!glucose_in_range("fasting", 130));
        // 130 dos horas después de comer es normal; en ayunas no lo sería.
        assert!(glucose_in_range("post_meal_2h", 130));
        assert!(!glucose_in_range("post_meal_2h", 180));
    }

    #[test]
    fn antes_y_despues_usan_tercios_no_extremos() {
        // Un pico aislado al final no debe decidir el resumen.
        let v = vec![80.0, 80.0, 80.0, 70.0, 70.0, 70.0];
        let (antes, despues) = antes_despues(&v);
        assert_eq!(antes, Some(80.0));
        assert_eq!(despues, Some(70.0));
    }

    #[test]
    fn el_csv_escapa_comas_y_comillas() {
        assert_eq!(escape_csv("simple"), "simple");
        assert_eq!(escape_csv("con,coma"), "\"con,coma\"");
        assert_eq!(escape_csv("con\"comilla"), "\"con\"\"comilla\"");
    }
}
