//! Exportación de datos (Fase 2) y copias de seguridad cifradas (Fase 3).
//!
//! La ruta de destino la elige el usuario con el diálogo nativo desde la UI;
//! aquí solo se escribe. Nunca se abre un diálogo desde Rust: eso corre en el
//! hilo del bucle de eventos y lo bloquearía.

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::State;

use crate::backup;
use crate::db::models::ProgressPhoto;
use crate::db::Db;
use crate::db::{queries, stats};
use crate::error::{AppError, AppResult};

/// Escribe todo el historial en la ruta indicada. `format` es "csv" o "json".
#[tauri::command]
pub fn export_data(db: State<Db>, format: String, path: String) -> AppResult<String> {
    if !matches!(format.as_str(), "csv" | "json") {
        return Err(AppError::invalid("el formato es csv o json"));
    }
    let contenido = db.with(|c| stats::export_all(c, &format))?;
    let destino = PathBuf::from(&path);
    if let Some(dir) = destino.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&destino, contenido)?;
    Ok(destino.display().to_string())
}

// -------------------------------------------------- fotos de progreso (F3)

/// Extensiones que aceptamos. Nada de copiar cualquier archivo a la carpeta de
/// datos solo porque el diálogo lo devolvió.
const IMAGENES: &[&str] = &["jpg", "jpeg", "png", "webp", "heic", "bmp"];

fn carpeta_fotos(db: &Db) -> AppResult<PathBuf> {
    let dir = db
        .path
        .parent()
        .ok_or_else(|| AppError::internal("la base no tiene carpeta padre"))?
        .join("photos");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Copia la imagen a la carpeta de datos y la registra. El original no se toca:
/// si luego borras la foto de tu carpeta de descargas, la del reto sigue ahí.
#[tauri::command]
pub fn add_progress_photo(db: State<Db>, date: String, source: String) -> AppResult<String> {
    guardar_foto(&db, &date, Path::new(&source))
}

/// Separado del comando para poder probarlo sin levantar Tauri.
pub fn guardar_foto(db: &Db, date: &str, origen: &Path) -> AppResult<String> {
    let ext = origen
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if !IMAGENES.contains(&ext.as_str()) {
        return Err(AppError::invalid(
            "solo se aceptan imágenes: jpg, png, webp, heic o bmp",
        ));
    }
    if !origen.is_file() {
        return Err(AppError::NotFound("no se encontró esa imagen".into()));
    }

    let dir = carpeta_fotos(db)?;
    let nombre = format!("{}.{ext}", queries::new_id());
    let destino = dir.join(&nombre);
    std::fs::copy(origen, &destino)?;

    let resultado = db.with(|c| {
        let d = queries::parse_date(date)?;
        queries::add_photo(c, d, &destino.to_string_lossy())
    });

    // Si la fila no entró, no dejamos el archivo huérfano.
    if resultado.is_err() {
        let _ = std::fs::remove_file(&destino);
    }
    resultado
}

#[tauri::command]
pub fn list_progress_photos(db: State<Db>) -> AppResult<Vec<ProgressPhoto>> {
    db.with(queries::photos)
}

/// Devuelve la imagen como data URL. Son dos fotos a la vez en el comparador,
/// así que evita configurar el protocolo de assets solo para esto.
#[tauri::command]
pub fn read_progress_photo(db: State<Db>, id: String) -> AppResult<String> {
    let ruta = db.with(|c| queries::photo_path(c, &id))?;
    let bytes = std::fs::read(&ruta)?;
    let ext = Path::new(&ruta)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpeg")
        .to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "heic" => "image/heic",
        _ => "image/jpeg",
    };
    Ok(format!("data:{mime};base64,{}", base64(&bytes)))
}

#[tauri::command]
pub fn delete_progress_photo(db: State<Db>, id: String) -> AppResult<()> {
    let ruta = db.with(|c| queries::photo_path(c, &id))?;
    db.with(|c| queries::delete_photo(c, &id))?;
    let _ = std::fs::remove_file(ruta);
    Ok(())
}

/// Base64 estándar. Son unas pocas fotos y evita una dependencia entera.
fn base64(bytes: &[u8]) -> String {
    const TABLA: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for trozo in bytes.chunks(3) {
        let b = [
            trozo[0],
            *trozo.get(1).unwrap_or(&0),
            *trozo.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(TABLA[(n >> 18) as usize & 63] as char);
        out.push(TABLA[(n >> 12) as usize & 63] as char);
        out.push(if trozo.len() > 1 {
            TABLA[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if trozo.len() > 2 {
            TABLA[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

// ------------------------------------------------------- copias de seguridad

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFile {
    pub name: String,
    pub path: String,
    pub size_kb: u64,
}

/// Instantánea local automática, sin cifrar y con rotación.
#[tauri::command]
pub fn backup_now(db: State<Db>) -> AppResult<String> {
    Ok(backup::snapshot_rotado(&db)?.display().to_string())
}

#[tauri::command]
pub fn list_backups(db: State<Db>) -> AppResult<Vec<BackupFile>> {
    let dir = match db.path.parent() {
        Some(p) => p.join("backups"),
        None => return Ok(vec![]),
    };
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut out: Vec<BackupFile> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("75hard-"))
        .map(|e| BackupFile {
            name: e.file_name().to_string_lossy().to_string(),
            path: e.path().display().to_string(),
            size_kb: e.metadata().map(|m| m.len() / 1024).unwrap_or(0),
        })
        .collect();
    out.sort_by(|a, b| b.name.cmp(&a.name));
    Ok(out)
}

/// Copia cifrada con una frase que la persona escribe y no se guarda.
#[tauri::command]
pub fn create_encrypted_backup(
    db: State<Db>,
    path: String,
    passphrase: String,
) -> AppResult<String> {
    backup::crear_cifrado(&db, Path::new(&path), &passphrase)?;
    Ok(path)
}

/// Restaura desde una copia cifrada. Devuelve la ruta de la instantánea de
/// seguridad que se guardó antes de sustituir nada.
#[tauri::command]
pub fn restore_encrypted_backup(
    db: State<Db>,
    path: String,
    passphrase: String,
) -> AppResult<String> {
    let respaldo = backup::restaurar_cifrado(&db, Path::new(&path), &passphrase)?;
    Ok(respaldo.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(nombre: &str) -> (Db, PathBuf) {
        let dir = std::env::temp_dir().join(format!("75hard-test-{nombre}"));
        let _ = std::fs::remove_dir_all(&dir);
        let db = crate::db::open(&dir).expect("abrir base temporal");
        (db, dir)
    }

    #[test]
    fn base64_coincide_con_la_referencia() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
        // Bytes no ASCII, que es lo que de verdad lleva una imagen.
        assert_eq!(base64(&[0xff, 0xd8, 0xff]), "/9j/");
    }

    #[test]
    fn una_foto_se_copia_se_lista_y_se_borra() {
        let (db, dir) = base("fotos");
        let origen = dir.join("original.png");
        // PNG mínimo válido: basta con que exista y tenga la extensión.
        std::fs::write(&origen, [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a])
            .expect("escribir");

        let id = guardar_foto(&db, "2026-08-19", &origen).expect("guardar foto");

        let listado = db.with(queries::photos).expect("listar");
        assert_eq!(listado.len(), 1);
        assert_eq!(listado[0].id, id);
        assert_eq!(listado[0].date, "2026-08-19");

        // El archivo vive en la carpeta de datos, no donde estaba el original.
        let ruta = db.with(|c| queries::photo_path(c, &id)).expect("ruta");
        assert!(Path::new(&ruta).is_file());
        assert!(ruta.contains("photos"));
        assert!(origen.is_file(), "el original no se toca");

        db.with(|c| queries::delete_photo(c, &id)).expect("borrar");
        assert!(db.with(queries::photos).expect("listar").is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn no_se_acepta_cualquier_archivo() {
        let (db, dir) = base("fotos-malas");
        let doc = dir.join("cosas.txt");
        std::fs::write(&doc, b"no soy una imagen").expect("escribir");

        let err = guardar_foto(&db, "2026-08-19", &doc).expect_err("debe rechazar");
        assert!(err.to_string().contains("solo se aceptan imágenes"));

        let inexistente = dir.join("no-existe.png");
        let err = guardar_foto(&db, "2026-08-19", &inexistente).expect_err("debe rechazar");
        assert!(err.to_string().contains("no se encontró"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn el_export_csv_lleva_encabezados_de_todas_las_tablas() {
        let (db, dir) = base("export");
        let csv = db.with(|c| stats::export_all(c, "csv")).expect("csv");
        for tabla in ["challenge", "day", "sleep_log", "meal", "glucose_reading"] {
            assert!(csv.contains(&format!("# {tabla}")), "falta {tabla}");
        }

        let json = db.with(|c| stats::export_all(c, "json")).expect("json");
        let v: serde_json::Value = serde_json::from_str(&json).expect("json válido");
        assert!(v.get("day").is_some(), "el JSON debe traer la tabla day");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
