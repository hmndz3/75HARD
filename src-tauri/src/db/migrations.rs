//! Migraciones versionadas con `PRAGMA user_version`.
//!
//! Cada entrada de MIGRATIONS se aplica en orden dentro de una transacción y
//! sube el `user_version`. Para agregar una nueva: crea el .sql en
//! `src-tauri/migrations/` y añádelo al final del array. Nunca edites una
//! migración ya publicada.

use rusqlite::Connection;

use crate::error::AppResult;

const MIGRATIONS: &[(&str, &str)] = &[
    ("001_init", include_str!("../../migrations/001_init.sql")),
    (
        "002_reminders",
        include_str!("../../migrations/002_reminders.sql"),
    ),
    (
        "003_reminders_custom",
        include_str!("../../migrations/003_reminders_custom.sql"),
    ),
];

pub fn run(conn: &Connection) -> AppResult<()> {
    let current: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let target = MIGRATIONS.len() as i64;

    if current >= target {
        return Ok(());
    }

    for (index, (name, sql)) in MIGRATIONS.iter().enumerate() {
        let version = index as i64 + 1;
        if version <= current {
            continue;
        }
        conn.execute_batch(&format!(
            "BEGIN; {sql} PRAGMA user_version = {version}; COMMIT;"
        ))
        .map_err(|e| crate::error::AppError::Internal(format!("migración {name} falló: {e}")))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migra_desde_cero_y_es_idempotente() {
        let conn = Connection::open_in_memory().expect("abrir memoria");
        run(&conn).expect("primera corrida");
        run(&conn).expect("segunda corrida no debe fallar");

        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .expect("leer user_version");
        assert_eq!(version, MIGRATIONS.len() as i64);
    }

    #[test]
    fn crea_todas_las_tablas_del_spec() {
        let conn = Connection::open_in_memory().expect("abrir memoria");
        run(&conn).expect("migrar");

        let esperadas = [
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
            "reminder",
            "settings",
        ];
        for tabla in esperadas {
            let n: i64 = conn
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [tabla],
                    |r| r.get(0),
                )
                .expect("consultar sqlite_master");
            assert_eq!(n, 1, "falta la tabla {tabla}");
        }
    }

    #[test]
    fn siembra_los_ajustes_por_defecto() {
        let conn = Connection::open_in_memory().expect("abrir memoria");
        run(&conn).expect("migrar");

        let tono: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key='coach_tone'",
                [],
                |r| r.get(0),
            )
            .expect("leer coach_tone");
        assert_eq!(tono, "directo");

        let corte: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key='day_cutoff_hour'",
                [],
                |r| r.get(0),
            )
            .expect("leer day_cutoff_hour");
        assert_eq!(corte, "4");
    }
}
