//! Llena una base de datos de demostración para revisar la interfaz sin tener
//! que vivir 23 días. NO toca tus datos reales: escribe en el directorio que le
//! pases por argumento.
//!
//!   cargo run --example seed_demo -- C:\ruta\a\una\carpeta
//!
//! Después se abre la app apuntando ahí:
//!
//!   APPDATA=C:\ruta\a\una\carpeta cargo run --release --features custom-protocol

use chrono::{Duration, Local, NaiveDate, NaiveTime};

use app75hard_lib::db::{self, models::Rules, queries as q};

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("uso: seed_demo <directorio donde crear 75hard/data.db>");
    let path = std::path::Path::new(&dir).join("75hard");

    // `--olvidar N` borra los últimos N días y retrasa la última apertura, para
    // poder ver la pantalla de recuperación (P13).
    let olvidar: Option<i64> = std::env::args()
        .position(|a| a == "--olvidar")
        .and_then(|i| std::env::args().nth(i + 1))
        .and_then(|v| v.parse().ok());

    let database = db::open(&path).expect("abrir la base de demostración");
    database
        .with(|c| {
            if let Some(n) = olvidar {
                return forget(c, n);
            }
            if q::active_challenge(c)?.is_some() {
                println!("ya había un reto; no se toca nada");
                return Ok(());
            }
            seed(c)
        })
        .expect("sembrar datos");

    println!("listo: {}", path.join("data.db").display());
}

fn seed(c: &rusqlite::Connection) -> app75hard_lib::error::AppResult<()> {
    let today = Local::now().date_naive();
    let start = today - Duration::days(22);
    q::create_challenge(c, "Intento #1", start, 75, &Rules::default())?;

    let t = |h: u32, m: u32| NaiveTime::from_hms_opt(h, m, 0).expect("hora válida");
    let at = |d: NaiveDate, h: u32, m: u32| d.and_time(t(h, m));

    for i in 0..23i64 {
        let day = start + Duration::days(i);
        let is_today = day == today;

        // Sueño: entre 5h10 y 8h20, con una mala racha en los días 20-22.
        let mala = (20..=22).contains(&i);
        let dormido = if mala {
            300 + (i % 3) * 20
        } else {
            400 + (i % 5) * 25
        };
        let wake = at(day, 6, 20);
        q::save_sleep(
            c,
            day,
            wake - Duration::minutes(dormido),
            wake,
            Some(3 + (i % 3)),
        )?;

        q::set_weight(c, day, 78.4 - (i as f64) * 0.12)?;
        q::set_mood(c, day, 3 + (i % 3), 2 + (i % 4), None)?;

        q::add_meal(
            c,
            day,
            at(day, 8, 30),
            "meal",
            "Avena con fruta y café",
            Some(320),
        )?;
        q::add_meal(
            c,
            day,
            at(day, 13, 45),
            "meal",
            "Pollo a la plancha con ensalada",
            Some(450),
        )?;
        if i % 3 != 0 {
            q::add_meal(c, day, at(day, 17, 0), "snack", "Nueces", None)?;
        }
        if !is_today {
            q::add_meal(
                c,
                day,
                at(day, 20, 15),
                "meal",
                "Salmón al horno con espárragos",
                Some(510),
            )?;
        }

        if i % 7 != 5 && !is_today {
            let outdoor = i % 4 == 0;
            q::add_workout(
                c,
                day,
                at(day, 18, 30),
                if outdoor { "outdoor" } else { "gym" },
                Some(if outdoor {
                    "Caminata rápida"
                } else {
                    "Pesas, tren superior"
                }),
                45 + (i % 3) * 10,
                outdoor,
                Some(320 + (i % 4) * 40),
            )?;
        }

        q::add_water(c, day, if is_today { 1800 } else { 2500 + (i % 3) * 400 })?;

        if !is_today {
            q::add_reading(c, day, 10 + (i % 8), Some("Atomic Habits"))?;
        }

        q::add_work_session(c, day, at(day, 9, 0), 180, "Universidad", None)?;
        q::add_work_session(
            c,
            day,
            at(day, 15, 0),
            120 + (i % 4) * 30,
            "Proyectos",
            None,
        )?;

        if i % 2 == 0 {
            q::add_glucose(c, day, at(day, 7, 30), 88 + (i % 12), "fasting", None)?;
        }
        if i % 3 == 0 {
            q::add_glucose(
                c,
                day,
                at(day, 15, 45),
                125 + (i % 30),
                "post_meal_2h",
                None,
            )?;
        }

        if !is_today {
            let status = match i {
                6 | 15 => "failed",
                11 => "skipped",
                _ => "complete",
            };
            q::set_day_status(c, day, status)?;
            q::mark_checkin(c, day, "morning")?;
            q::mark_checkin(c, day, "evening")?;
            if i == 15 {
                q::set_day_notes(
                    c,
                    day,
                    Some("Día largo en la universidad; no alcancé a entrenar."),
                )?;
            }
        } else {
            q::mark_checkin(c, day, "morning")?;
        }
    }

    q::touch_last_open(c, today)?;
    Ok(())
}

/// Borra los últimos `n` días completos (de ayer hacia atrás) y deja la última
/// apertura registrada justo antes, como si la app no se hubiera abierto.
fn forget(c: &rusqlite::Connection, n: i64) -> app75hard_lib::error::AppResult<()> {
    let today = Local::now().date_naive();
    for i in 1..=n {
        let day = today - Duration::days(i);
        let key = day.format("%Y-%m-%d").to_string();
        for tabla in [
            "sleep_log",
            "meal",
            "workout",
            "glucose_reading",
            "work_session",
            "water_log",
            "reading_log",
            "weight_log",
            "mood_log",
        ] {
            c.execute(&format!("DELETE FROM {tabla} WHERE date = ?1"), [&key])?;
        }
        c.execute("DELETE FROM day WHERE date = ?1", [&key])?;
    }
    q::touch_last_open(c, today - Duration::days(n))?;
    println!("olvidados los últimos {n} días");
    Ok(())
}
