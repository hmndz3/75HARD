//! El "día" del reto no corta a medianoche sino a las 4:00 AM (§7 del spec).
//! Si registras algo a la 1 AM del martes, cuenta para el lunes — que es como
//! funciona la cabeza de quien lo está usando.
//!
//! El sueño se atribuye siempre al día en que la persona DESPIERTA.

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

pub const DEFAULT_CUTOFF_HOUR: u32 = 4;

/// Fecha lógica del reto para un instante dado.
pub fn logical_date(now: NaiveDateTime, cutoff_hour: u32) -> NaiveDate {
    let cutoff = cutoff_hour.min(23);
    if now.hour() < cutoff {
        now.date() - Duration::days(1)
    } else {
        now.date()
    }
}

/// Dadas la fecha en que se despertó y las dos horas de reloj, reconstruye los
/// dos instantes completos. Si la hora de dormir es posterior a la de despertar,
/// se durmió el día anterior.
pub fn resolve_sleep_window(
    wake_date: NaiveDate,
    bedtime: NaiveTime,
    wake_time: NaiveTime,
) -> (NaiveDateTime, NaiveDateTime) {
    let wake_dt = wake_date.and_time(wake_time);
    let bed_date = if bedtime > wake_time {
        wake_date - Duration::days(1)
    } else {
        wake_date
    };
    (bed_date.and_time(bedtime), wake_dt)
}

/// Minutos dormidos. Nunca negativo: si los datos vienen invertidos devuelve 0
/// y la capa de comandos lo rechaza con un mensaje legible.
pub fn sleep_minutes(bedtime: NaiveDateTime, wake_time: NaiveDateTime) -> i64 {
    (wake_time - bedtime).num_minutes().max(0)
}

/// Número de día dentro del reto (1-based). None si la fecha es anterior al inicio.
pub fn day_number(start_date: NaiveDate, date: NaiveDate) -> Option<i64> {
    let n = (date - start_date).num_days();
    if n < 0 {
        None
    } else {
        Some(n + 1)
    }
}

/// "6h 40min" — el formato que se usa en toda la UI.
pub fn format_minutes(total: i64) -> String {
    let h = total / 60;
    let m = total % 60;
    if h == 0 {
        format!("{m}min")
    } else if m == 0 {
        format!("{h}h")
    } else {
        format!("{h}h {m}min")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dt(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").expect("fecha de test válida")
    }
    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").expect("fecha de test válida")
    }
    fn t(s: &str) -> NaiveTime {
        NaiveTime::parse_from_str(s, "%H:%M").expect("hora de test válida")
    }

    #[test]
    fn la_una_am_todavia_es_el_dia_anterior() {
        assert_eq!(logical_date(dt("2026-08-19 01:00"), 4), d("2026-08-18"));
        assert_eq!(logical_date(dt("2026-08-19 03:59"), 4), d("2026-08-18"));
    }

    #[test]
    fn a_partir_del_corte_ya_es_el_dia_nuevo() {
        assert_eq!(logical_date(dt("2026-08-19 04:00"), 4), d("2026-08-19"));
        assert_eq!(logical_date(dt("2026-08-19 23:59"), 4), d("2026-08-19"));
    }

    #[test]
    fn dormirse_antes_de_medianoche_pone_la_cama_el_dia_previo() {
        let (bed, wake) = resolve_sleep_window(d("2026-08-19"), t("23:30"), t("06:10"));
        assert_eq!(bed, dt("2026-08-18 23:30"));
        assert_eq!(wake, dt("2026-08-19 06:10"));
        assert_eq!(sleep_minutes(bed, wake), 400); // 6h 40min
    }

    #[test]
    fn dormirse_despues_de_medianoche_deja_todo_el_mismo_dia() {
        let (bed, wake) = resolve_sleep_window(d("2026-08-19"), t("01:15"), t("08:00"));
        assert_eq!(bed, dt("2026-08-19 01:15"));
        assert_eq!(sleep_minutes(bed, wake), 405);
    }

    #[test]
    fn numero_de_dia_es_uno_based() {
        assert_eq!(day_number(d("2026-08-19"), d("2026-08-19")), Some(1));
        assert_eq!(day_number(d("2026-08-19"), d("2026-09-11")), Some(24));
        assert_eq!(day_number(d("2026-08-19"), d("2026-08-18")), None);
    }

    #[test]
    fn formato_de_minutos() {
        assert_eq!(format_minutes(400), "6h 40min");
        assert_eq!(format_minutes(420), "7h");
        assert_eq!(format_minutes(45), "45min");
    }
}
