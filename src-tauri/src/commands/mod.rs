//! Comandos expuestos a la UI. Aquí solo se valida la entrada y se orquesta;
//! la SQL vive en `db::queries`.

pub mod day;
pub mod glucose;
pub mod meals;
pub mod reminders;
pub mod settings;
pub mod sleep;
pub mod tracking;
pub mod window;
pub mod workouts;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use rusqlite::Connection;

use crate::db::queries;
use crate::error::{AppError, AppResult};

/// Resuelve la fecha lógica: si la UI no manda ninguna, es "hoy" con el corte
/// de las 4:00 AM aplicado.
pub fn resolve_date(conn: &Connection, date: Option<String>) -> AppResult<NaiveDate> {
    match date {
        Some(s) => queries::parse_date(&s),
        None => Ok(queries::today(conn)),
    }
}

pub fn parse_time(s: &str) -> AppResult<NaiveTime> {
    NaiveTime::parse_from_str(s, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(s, "%H:%M:%S"))
        .map_err(|_| AppError::invalid(format!("hora no válida: {s} (usa HH:MM)")))
}

/// Convierte una hora de reloj en el instante real dentro de un día lógico.
/// Como el día corta a las 4:00 AM, una hora anterior al corte pertenece a la
/// madrugada del día calendario siguiente.
pub fn instant_in_day(date: NaiveDate, time: NaiveTime, cutoff_hour: u32) -> NaiveDateTime {
    if time.hour() < cutoff_hour {
        (date + Duration::days(1)).and_time(time)
    } else {
        date.and_time(time)
    }
}

/// Hora actual de reloj, para prellenar formularios.
pub fn now_time() -> NaiveTime {
    queries::now_naive().time()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn una_comida_a_la_una_am_cae_en_la_madrugada_siguiente() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 19).expect("fecha");
        let t = NaiveTime::from_hms_opt(1, 30, 0).expect("hora");
        let dt = instant_in_day(date, t, 4);
        assert_eq!(
            dt.date(),
            NaiveDate::from_ymd_opt(2026, 8, 20).expect("fecha")
        );
    }

    #[test]
    fn una_comida_a_las_ocho_am_cae_el_mismo_dia() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 19).expect("fecha");
        let t = NaiveTime::from_hms_opt(8, 0, 0).expect("hora");
        assert_eq!(instant_in_day(date, t, 4).date(), date);
    }

    #[test]
    fn rechaza_horas_mal_escritas() {
        assert!(parse_time("25:00").is_err());
        assert!(parse_time("").is_err());
        assert!(parse_time("07:30").is_ok());
    }
}
