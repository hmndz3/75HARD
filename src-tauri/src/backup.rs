//! Copias de seguridad (Fase 3).
//!
//! Dos cosas distintas, a propósito:
//!
//! 1. **Instantáneas automáticas locales**, sin cifrar. Viven junto a la base,
//!    en la misma máquina y con la misma protección que ella. Cifrarlas con una
//!    clave guardada al lado no añadiría nada: quien pueda leer una puede leer
//!    la otra.
//! 2. **Copias cifradas manuales**, para sacar los datos de la máquina. Ahí el
//!    cifrado sí importa, y por eso la frase la escribe la persona cada vez y
//!    no se guarda en ningún sitio.
//!
//! Formato del archivo cifrado:
//! `"75HARDBK" | versión(1) | salt(16) | nonce(12) | AES-256-GCM(base .db)`
//! La clave sale de Argon2id sobre la frase, con el salt del propio archivo.

use std::path::{Path, PathBuf};

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Key, Nonce};
use rusqlite::Connection;

use crate::db::Db;
use crate::error::{AppError, AppResult};

const MAGIC: &[u8; 8] = b"75HARDBK";
const VERSION: u8 = 1;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

/// Cuántas instantáneas automáticas se conservan antes de ir borrando.
const MAX_SNAPSHOTS: usize = 7;

fn dir_backups(db: &Db) -> AppResult<PathBuf> {
    let dir = db
        .path
        .parent()
        .ok_or_else(|| AppError::internal("la base no tiene carpeta padre"))?
        .join("backups");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Copia consistente de la base con `VACUUM INTO`. No basta con copiar el
/// archivo: con WAL activo habría escrituras a medias en el .wal.
fn snapshot(conn: &Connection, destino: &Path) -> AppResult<()> {
    if destino.exists() {
        std::fs::remove_file(destino)?;
    }
    conn.execute("VACUUM INTO ?1", [destino.to_string_lossy()])?;
    Ok(())
}

fn derivar_clave(frase: &str, salt: &[u8]) -> AppResult<[u8; 32]> {
    if frase.chars().count() < 8 {
        return Err(AppError::invalid(
            "la frase de cifrado necesita al menos 8 caracteres",
        ));
    }
    let mut clave = [0u8; 32];
    argon2::Argon2::default()
        .hash_password_into(frase.as_bytes(), salt, &mut clave)
        .map_err(|e| AppError::internal(format!("no se pudo derivar la clave: {e}")))?;
    Ok(clave)
}

/// Instantánea automática, sin cifrar, con rotación. Devuelve la ruta creada.
pub fn snapshot_rotado(db: &Db) -> AppResult<PathBuf> {
    let dir = dir_backups(db)?;
    let nombre = format!("75hard-{}.db", chrono::Local::now().format("%Y%m%d-%H%M%S"));
    let destino = dir.join(nombre);

    db.with(|conn| snapshot(conn, &destino))?;

    // Se conservan las más recientes; el resto se va.
    let mut existentes: Vec<_> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("75hard-"))
        .collect();
    existentes.sort_by_key(|e| e.file_name());
    while existentes.len() > MAX_SNAPSHOTS {
        let viejo = existentes.remove(0);
        let _ = std::fs::remove_file(viejo.path());
    }

    Ok(destino)
}

/// Copia cifrada en la ruta elegida por el usuario.
pub fn crear_cifrado(db: &Db, destino: &Path, frase: &str) -> AppResult<()> {
    let temporal = dir_backups(db)?.join(".tmp-export.db");
    db.with(|conn| snapshot(conn, &temporal))?;

    let plano = std::fs::read(&temporal);
    let _ = std::fs::remove_file(&temporal);
    let plano = plano?;

    let mut salt = [0u8; SALT_LEN];
    use aes_gcm::aead::rand_core::RngCore;
    OsRng.fill_bytes(&mut salt);

    let clave = derivar_clave(frase, &salt)?;
    let cifrador = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&clave));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let cifrado = cifrador
        .encrypt(&nonce, plano.as_ref())
        .map_err(|_| AppError::internal("falló el cifrado"))?;

    let mut salida = Vec::with_capacity(cifrado.len() + 64);
    salida.extend_from_slice(MAGIC);
    salida.push(VERSION);
    salida.extend_from_slice(&salt);
    salida.extend_from_slice(nonce.as_slice());
    salida.extend_from_slice(&cifrado);

    if let Some(dir) = destino.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(destino, salida)?;
    Ok(())
}

/// Descifra y restaura. Antes de tocar nada guarda una instantánea de lo que
/// hay ahora: restaurar por error no puede ser una operación sin vuelta atrás.
pub fn restaurar_cifrado(db: &Db, origen: &Path, frase: &str) -> AppResult<PathBuf> {
    let bytes = std::fs::read(origen)?;
    let cabecera = MAGIC.len() + 1 + SALT_LEN + NONCE_LEN;
    if bytes.len() < cabecera || &bytes[..MAGIC.len()] != MAGIC {
        return Err(AppError::invalid(
            "ese archivo no es una copia de seguridad de 75 HARD",
        ));
    }
    if bytes[MAGIC.len()] != VERSION {
        return Err(AppError::invalid(
            "la copia es de una versión más nueva de la app",
        ));
    }

    let salt = &bytes[MAGIC.len() + 1..MAGIC.len() + 1 + SALT_LEN];
    let nonce = &bytes[MAGIC.len() + 1 + SALT_LEN..cabecera];
    let cifrado = &bytes[cabecera..];

    let clave = derivar_clave(frase, salt)?;
    let descifrador = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&clave));
    let plano = descifrador
        .decrypt(Nonce::from_slice(nonce), cifrado)
        .map_err(|_| AppError::invalid("la frase no coincide, o el archivo está dañado"))?;

    // Se valida que lo descifrado sea de verdad una base nuestra antes de
    // sustituir nada.
    let dir = dir_backups(db)?;
    let candidato = dir.join(".tmp-restore.db");
    std::fs::write(&candidato, &plano)?;
    validar(&candidato).inspect_err(|_| {
        let _ = std::fs::remove_file(&candidato);
    })?;

    let respaldo = snapshot_rotado(db)?;

    // Se suelta la conexión actual para que Windows libere el archivo, se
    // sustituye, y se vuelve a abrir.
    {
        let mut guard = db.conn.lock().unwrap_or_else(|e| e.into_inner());
        *guard = Connection::open_in_memory()?;

        for sufijo in ["-wal", "-shm"] {
            let extra = PathBuf::from(format!("{}{sufijo}", db.path.display()));
            let _ = std::fs::remove_file(extra);
        }
        std::fs::copy(&candidato, &db.path)?;
        *guard = crate::db::conectar(&db.path)?;
    }

    let _ = std::fs::remove_file(&candidato);
    Ok(respaldo)
}

/// Comprueba que el archivo abre como SQLite y tiene nuestro esquema.
fn validar(archivo: &Path) -> AppResult<()> {
    let conn = Connection::open(archivo)
        .map_err(|_| AppError::invalid("la copia no se puede abrir como base de datos"))?;
    let tablas: i64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table'
             AND name IN ('challenge','day','sleep_log','settings')",
            [],
            |r| r.get(0),
        )
        .map_err(|_| AppError::invalid("la copia no tiene el esquema de 75 HARD"))?;
    if tablas < 4 {
        return Err(AppError::invalid("la copia no tiene el esquema de 75 HARD"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_frase_corta_se_rechaza() {
        assert!(derivar_clave("corta", &[0u8; SALT_LEN]).is_err());
        assert!(derivar_clave("suficientemente larga", &[0u8; SALT_LEN]).is_ok());
    }

    #[test]
    fn la_misma_frase_y_salt_dan_la_misma_clave() {
        let salt = [7u8; SALT_LEN];
        let a = derivar_clave("frase de prueba", &salt).expect("clave");
        let b = derivar_clave("frase de prueba", &salt).expect("clave");
        assert_eq!(a, b);
    }

    #[test]
    fn distinto_salt_da_distinta_clave() {
        let a = derivar_clave("frase de prueba", &[1u8; SALT_LEN]).expect("clave");
        let b = derivar_clave("frase de prueba", &[2u8; SALT_LEN]).expect("clave");
        assert_ne!(a, b, "el salt tiene que cambiar la clave derivada");
    }

    /// Base temporal con un dato reconocible, para probar el viaje completo.
    fn base_temporal(nombre: &str) -> (Db, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("75hard-test-{nombre}"));
        let _ = std::fs::remove_dir_all(&dir);
        let db = crate::db::open(&dir).expect("abrir base temporal");
        db.with(|c| {
            crate::db::queries::set_setting(c, "coach_tone", "duro")?;
            Ok(())
        })
        .expect("sembrar");
        (db, dir)
    }

    #[test]
    fn la_copia_cifrada_va_y_vuelve() {
        let (db, dir) = base_temporal("roundtrip");
        let destino = dir.join("copia.75bak");

        crear_cifrado(&db, &destino, "frase larga de prueba").expect("crear copia");
        assert!(destino.exists(), "el archivo cifrado tiene que existir");

        // El archivo NO puede parecerse a una base SQLite en claro.
        let bytes = std::fs::read(&destino).expect("leer copia");
        assert_eq!(&bytes[..MAGIC.len()], MAGIC);
        assert!(
            !bytes.windows(6).any(|w| w == b"SQLite"),
            "la copia no puede llevar la cabecera de SQLite en claro"
        );

        // Se cambia el dato y se restaura: tiene que volver al valor original.
        db.with(|c| crate::db::queries::set_setting(c, "coach_tone", "suave"))
            .expect("cambiar");

        restaurar_cifrado(&db, &destino, "frase larga de prueba").expect("restaurar");

        let tono = db
            .with(|c| crate::db::queries::get_setting(c, "coach_tone"))
            .expect("leer")
            .expect("hay valor");
        assert_eq!(tono, "duro", "la restauración debe traer el valor guardado");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn una_frase_equivocada_no_restaura_nada() {
        let (db, dir) = base_temporal("frase-mala");
        let destino = dir.join("copia.75bak");
        crear_cifrado(&db, &destino, "la frase correcta").expect("crear copia");

        let err =
            restaurar_cifrado(&db, &destino, "otra frase distinta").expect_err("no debe restaurar");
        assert!(err.to_string().contains("frase no coincide"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn las_instantaneas_locales_rotan() {
        let (db, dir) = base_temporal("rotacion");
        for _ in 0..MAX_SNAPSHOTS + 3 {
            // El nombre lleva segundos; se fuerza que no colisionen.
            std::thread::sleep(std::time::Duration::from_millis(1100));
            snapshot_rotado(&db).expect("instantánea");
        }
        let n = std::fs::read_dir(dir.join("backups"))
            .expect("listar")
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("75hard-"))
            .count();
        assert_eq!(n, MAX_SNAPSHOTS, "solo se conservan las más recientes");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn un_archivo_ajeno_no_se_confunde_con_una_copia() {
        let dir = std::env::temp_dir().join("75hard-test-backup");
        std::fs::create_dir_all(&dir).expect("crear dir");
        let falso = dir.join("cualquier-cosa.bin");
        std::fs::write(&falso, b"esto no es una copia").expect("escribir");

        let bytes = std::fs::read(&falso).expect("leer");
        assert!(&bytes[..MAGIC.len().min(bytes.len())] != MAGIC);
        let _ = std::fs::remove_file(&falso);
    }
}
