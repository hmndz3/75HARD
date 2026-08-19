use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::error::AppResult;

pub const MAIN: &str = "main";
pub const QUICK: &str = "quick";

/// Abre la ventana principal, o la enfoca si ya existe.
/// Al cerrarla se DESTRUYE (no se esconde) para que WebView2 libere sus
/// procesos; por eso aquí hay que reconstruirla, no solo mostrarla.
///
/// OJO: `build()` encola la creación en el bucle de eventos y espera respuesta.
/// Si se la llama DESDE el hilo principal con el bucle ya corriendo, se bloquea
/// a sí misma: la ventana nativa aparece pero el WebView2 nunca navega y queda
/// en blanco. Todo lo que venga del bucle (menú de la bandeja, comandos
/// síncronos) tiene que pasar por las variantes `_spawned`.
pub fn open_main(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(MAIN) {
        w.unminimize().ok();
        w.show()?;
        w.set_focus()?;
        return Ok(());
    }

    WebviewWindowBuilder::new(app, MAIN, WebviewUrl::App("index.html".into()))
        .title("75 HARD")
        .inner_size(1280.0, 800.0)
        .min_inner_size(960.0, 640.0)
        // El shell dibuja su propia barra de 44px con los controles de ventana.
        .decorations(false)
        .center()
        .build()?;
    Ok(())
}

/// Ventana flotante de captura rápida (P4): 380x460, sin cromo, siempre encima.
pub fn open_quick(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(QUICK) {
        w.show()?;
        w.set_focus()?;
        return Ok(());
    }

    WebviewWindowBuilder::new(app, QUICK, WebviewUrl::App("index.html".into()))
        .title("Captura rápida")
        .inner_size(380.0, 460.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .center()
        .build()?;
    Ok(())
}

/// Variantes seguras de invocar desde el hilo principal.
pub fn open_main_spawned(app: &AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = open_main(&handle);
    });
}

pub fn open_quick_spawned(app: &AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = open_quick(&handle);
    });
}

// Los comandos son `async` a propósito: así Tauri los corre fuera del hilo
// principal y `build()` puede encolar la creación sin bloquearse.

#[tauri::command]
pub async fn open_main_window(app: AppHandle) -> AppResult<()> {
    open_main(&app)?;
    Ok(())
}

#[tauri::command]
pub async fn open_quick_entry(app: AppHandle) -> AppResult<()> {
    open_quick(&app)?;
    Ok(())
}

#[tauri::command]
pub fn close_window(window: tauri::Window) -> AppResult<()> {
    window.close()?;
    Ok(())
}
