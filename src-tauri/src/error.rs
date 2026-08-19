use serde::{Serialize, Serializer};

/// Todo error que pueda llegar a la UI. Nunca se hace unwrap() en una ruta
/// que el usuario pueda tocar: el error se serializa y la pantalla lo muestra.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("No se pudo leer o escribir en la base de datos: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("No se pudo acceder al archivo de datos: {0}")]
    Io(#[from] std::io::Error),

    #[error("Dato inválido: {0}")]
    Invalid(String),

    #[error("No encontrado: {0}")]
    NotFound(String),

    #[error("Error interno: {0}")]
    Internal(String),
}

impl AppError {
    pub fn invalid(msg: impl Into<String>) -> Self {
        Self::Invalid(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(e: tauri::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
