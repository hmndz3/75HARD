//! Icono de bandeja. La app vive aquí: la ventana es una visita, no la casa.

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle,
};

use crate::commands::window::{open_main_spawned, open_quick_spawned};
use crate::error::{AppError, AppResult};

pub fn build(app: &AppHandle) -> AppResult<()> {
    let abrir = MenuItem::with_id(app, "abrir", "Abrir", true, None::<&str>)?;
    let captura = MenuItem::with_id(app, "captura", "Captura rápida", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let salir = MenuItem::with_id(app, "salir", "Salir", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&abrir, &captura, &sep, &salir])?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| AppError::internal("la app no tiene icono configurado"))?;

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip("75 HARD")
        .menu(&menu)
        // El menú debe salir solo con clic derecho; el izquierdo abre la app.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            // El menú corre en el hilo principal: crear la ventana ahí mismo
            // bloquearía el bucle de eventos.
            "abrir" => open_main_spawned(app),
            "captura" => open_quick_spawned(app),
            "salir" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                open_main_spawned(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}
