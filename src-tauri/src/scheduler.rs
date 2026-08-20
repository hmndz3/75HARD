//! El tick que hace que la app te busque a ti (§5 del spec).
//!
//! Cada 60 segundos revisa qué recordatorios tocan y dispara una notificación
//! nativa. La gracia está en que mira los datos del día antes de molestar: si
//! ya registraste el ejercicio, el recordatorio de las 17:00 no suena.
//!
//! Todo el trabajo contra SQLite se hace en un bloque síncrono y se sale de él
//! con la lista ya armada, para no sostener el Mutex de la conexión mientras se
//! muestran las notificaciones.

use std::time::Duration as StdDuration;

use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::daycut::format_minutes;
use crate::db::queries;
use crate::db::Db;
use crate::error::AppResult;

const TICK_SECS: u64 = 60;

/// Un aviso ya resuelto y listo para mostrarse.
struct Aviso {
    title: String,
    body: String,
}

pub fn start(app: &AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(StdDuration::from_secs(TICK_SECS)).await;

            let avisos = match evaluar(&handle) {
                Ok(a) => a,
                // Un fallo del planificador no puede tumbar la app ni el bucle:
                // se salta este tick y se vuelve a intentar en un minuto.
                Err(_) => continue,
            };

            for aviso in avisos {
                let _ = handle
                    .notification()
                    .builder()
                    .title(&aviso.title)
                    .body(&aviso.body)
                    .show();
            }
        }
    });
}

/// Decide qué avisos tocan y marca los recordatorios como disparados.
/// Devuelve solo los que de verdad hay que mostrar.
fn evaluar(app: &AppHandle) -> AppResult<Vec<Aviso>> {
    let db = app.state::<Db>();
    db.with(|conn| {
        let now = queries::now_naive();
        let pendientes = queries::due_reminders(conn, now)?;
        if pendientes.is_empty() {
            return Ok(vec![]);
        }

        let hoy = queries::today(conn);
        let vista = queries::today_view(conn, hoy)?;

        let mut avisos = Vec::new();
        for r in pendientes {
            // Se marca siempre, se muestre o no: si el pilar ya está cubierto
            // no hay nada que decir, pero tampoco hay que volver a revisarlo
            // dentro de un minuto.
            queries::mark_reminder_fired(conn, &r.id, now)?;

            if let Some(aviso) = redactar(&r.kind, &vista) {
                avisos.push(aviso);
            }
        }
        Ok(avisos)
    })
}

/// El texto de cada aviso. Devuelve `None` cuando el pilar ya está cubierto:
/// una app que te felicita por algo que ya hiciste se vuelve ruido.
fn redactar(kind: &str, v: &crate::db::models::TodayView) -> Option<Aviso> {
    let dia = v
        .day_number
        .map(|n| format!("Día {n}"))
        .unwrap_or_else(|| "75 HARD".to_string());

    let (title, body) = match kind {
        "morning" => {
            if v.sleep.is_some() {
                return None;
            }
            (
                "Buenos días".to_string(),
                "¿A qué hora te dormiste y a qué hora despertaste?".to_string(),
            )
        }

        "meal" => {
            if v.meals_count >= 2 {
                return None;
            }
            (
                dia,
                "¿Ya comiste? Registra la comida, son diez segundos.".to_string(),
            )
        }

        "workout" => {
            if v.workout_min > 0 {
                return None;
            }
            (
                dia,
                format!(
                    "Aún no registras ejercicio hoy. Quedan {} horas.",
                    v.hours_left
                ),
            )
        }

        "water" => {
            if v.water_ml >= v.water_goal_ml {
                return None;
            }
            (
                "Agua".to_string(),
                format!(
                    "Llevas {:.1} de {:.1} L.",
                    v.water_ml as f64 / 1000.0,
                    v.water_goal_ml as f64 / 1000.0
                ),
            )
        }

        "evening" => {
            if v.status != "pending" {
                return None;
            }
            let faltan: Vec<String> = v
                .pillars
                .iter()
                .filter(|p| p.required && p.status != "done")
                .map(|p| p.label.clone())
                .collect();

            let cuerpo = if faltan.is_empty() {
                match &v.sleep {
                    Some(s) => format!(
                        "Todo cubierto: dormiste {}. Falta cerrar el día.",
                        format_minutes(s.minutes)
                    ),
                    None => "Todo cubierto. Falta cerrar el día.".to_string(),
                }
            } else {
                format!("Sin registrar: {}.", crate::coach::join_es(&faltan))
            };
            (format!("Cierre del {}", dia.to_lowercase()), cuerpo)
        }

        _ => return None,
    };

    Some(Aviso { title, body })
}

#[cfg(test)]
mod tests {
    use crate::db::open_in_memory;
    use crate::db::queries as q;
    use chrono::NaiveDateTime;

    fn dt(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").expect("instante de test")
    }

    #[test]
    fn el_horario_de_silencio_calla_todo() {
        let db = open_in_memory().expect("base en memoria");
        db.with(|c| {
            // 23:30 cae dentro del silencio por defecto (22:00 a 07:00).
            assert!(q::due_reminders(c, dt("2026-08-19 23:30"))?.is_empty());
            assert!(q::due_reminders(c, dt("2026-08-19 03:00"))?.is_empty());
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn apagar_las_notificaciones_calla_todo() {
        let db = open_in_memory().expect("base en memoria");
        db.with(|c| {
            q::set_setting(c, "notifications", "0")?;
            assert!(q::due_reminders(c, dt("2026-08-19 12:45"))?.is_empty());
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn un_recordatorio_suena_una_vez_al_dia() {
        let db = open_in_memory().expect("base en memoria");
        db.with(|c| {
            let momento = dt("2026-08-19 12:45");
            let ids: Vec<_> = q::due_reminders(c, momento)?
                .into_iter()
                .map(|r| r.id)
                .collect();
            assert!(
                ids.contains(&"meal".to_string()),
                "esperaba 'meal' en {ids:?}"
            );

            q::mark_reminder_fired(c, "meal", momento)?;

            let ids: Vec<_> = q::due_reminders(c, dt("2026-08-19 13:30"))?
                .into_iter()
                .map(|r| r.id)
                .collect();
            assert!(!ids.contains(&"meal".to_string()), "no debe repetirse hoy");
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn el_agua_vuelve_a_sonar_pasado_el_intervalo() {
        let db = open_in_memory().expect("base en memoria");
        db.with(|c| {
            q::mark_reminder_fired(c, "water", dt("2026-08-19 09:00"))?;

            let a_la_hora: Vec<_> = q::due_reminders(c, dt("2026-08-19 10:00"))?
                .into_iter()
                .map(|r| r.id)
                .collect();
            assert!(!a_la_hora.contains(&"water".to_string()));

            let a_las_dos: Vec<_> = q::due_reminders(c, dt("2026-08-19 11:05"))?
                .into_iter()
                .map(|r| r.id)
                .collect();
            assert!(a_las_dos.contains(&"water".to_string()));
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn un_recordatorio_apagado_no_suena() {
        let db = open_in_memory().expect("base en memoria");
        db.with(|c| {
            q::set_reminder(c, "meal", Some(false), None)?;
            let ids: Vec<_> = q::due_reminders(c, dt("2026-08-19 12:45"))?
                .into_iter()
                .map(|r| r.id)
                .collect();
            assert!(!ids.contains(&"meal".to_string()));
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn no_suena_antes_de_su_hora() {
        let db = open_in_memory().expect("base en memoria");
        db.with(|c| {
            let ids: Vec<_> = q::due_reminders(c, dt("2026-08-19 08:00"))?
                .into_iter()
                .map(|r| r.id)
                .collect();
            assert!(!ids.contains(&"evening".to_string()));
            assert!(!ids.contains(&"workout".to_string()));
            Ok(())
        })
        .expect("consulta");
    }

    #[test]
    fn rechaza_una_hora_mal_escrita() {
        let db = open_in_memory().expect("base en memoria");
        db.with(|c| {
            assert!(q::set_reminder(c, "meal", None, Some("25:99")).is_err());
            assert!(q::set_reminder(c, "meal", None, Some("13:15")).is_ok());
            Ok(())
        })
        .expect("consulta");
    }
}
