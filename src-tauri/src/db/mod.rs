pub mod migrations;
pub mod models;
pub mod queries;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::AppResult;

/// Conexión única a SQLite, gestionada por Tauri como estado global.
/// Un solo usuario, un solo proceso: un Mutex es suficiente y evita el peso de
/// un pool. Cada comando escribe inmediatamente; no hay estado en memoria que
/// se pueda perder si se va la luz.
pub struct Db {
    pub conn: Mutex<Connection>,
    pub path: PathBuf,
}

impl Db {
    /// Ejecuta una operación con la conexión bloqueada.
    /// Si el Mutex quedó envenenado por un panic previo, se recupera en vez de
    /// tumbar la app: los datos en disco siguen íntegros gracias a WAL.
    pub fn with<T>(&self, f: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let guard = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        f(&guard)
    }

    /// Igual que `with`, pero envuelto en una transacción.
    pub fn tx<T>(&self, f: impl FnOnce(&rusqlite::Transaction) -> AppResult<T>) -> AppResult<T> {
        let mut guard = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let tx = guard.transaction()?;
        let out = f(&tx)?;
        tx.commit()?;
        Ok(out)
    }
}

/// Abre (o crea) la base en `dir/data.db`, aplica los PRAGMA y corre migraciones.
pub fn open(dir: &Path) -> AppResult<Db> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join("data.db");
    let conn = Connection::open(&path)?;

    // WAL: sobrevive a un apagón sin corromper la base y no bloquea lecturas.
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;

    migrations::run(&conn)?;

    Ok(Db {
        conn: Mutex::new(conn),
        path,
    })
}

#[cfg(test)]
pub fn open_in_memory() -> AppResult<Db> {
    let conn = Connection::open_in_memory()?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrations::run(&conn)?;
    Ok(Db {
        conn: Mutex::new(conn),
        path: PathBuf::from(":memory:"),
    })
}
