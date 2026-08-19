pub mod coach;
pub mod commands;
pub mod daycut;
pub mod db;
pub mod error;
pub mod ram;
pub mod tray;

use tauri::{Manager, RunEvent};
use tauri_plugin_autostart::MacosLauncher;

use crate::error::AppError;

/// Lee `--datos <ruta>` de la línea de comandos.
fn datos_override() -> Option<std::path::PathBuf> {
    let mut args = std::env::args();
    while let Some(a) = args.next() {
        if a == "--datos" {
            return args.next().map(std::path::PathBuf::from);
        }
    }
    None
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Una sola copia viva. Si Windows la arrancó en la bandeja y luego se
        // hace doble clic en el icono, la segunda copia solo le pide a la
        // primera que muestre su ventana y se va.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            commands::window::open_main_spawned(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            // Arranca minimizada en bandeja: no queremos una ventana en la cara
            // cada vez que se enciende la computadora.
            Some(vec!["--bandeja"]),
        ))
        .invoke_handler(tauri::generate_handler![
            commands::day::get_today,
            commands::day::get_day_detail,
            commands::day::get_history,
            commands::day::close_day,
            commands::day::reopen_day,
            commands::day::set_day_notes,
            commands::day::delete_entry,
            commands::day::get_missing_days,
            commands::day::apply_missing_days,
            commands::sleep::preview_sleep,
            commands::sleep::save_morning_checkin,
            commands::meals::add_meal,
            commands::workouts::add_workout,
            commands::glucose::add_glucose,
            commands::tracking::add_water,
            commands::tracking::set_water,
            commands::tracking::add_reading,
            commands::tracking::add_work_session,
            commands::tracking::set_weight,
            commands::tracking::set_mood,
            commands::settings::bootstrap,
            commands::settings::get_settings,
            commands::settings::set_settings,
            commands::settings::get_challenge,
            commands::settings::create_challenge,
            commands::settings::end_challenge,
            commands::settings::set_autostart,
            commands::settings::is_autostart_enabled,
            commands::window::open_main_window,
            commands::window::open_quick_entry,
            commands::window::close_window,
        ])
        .setup(|app| {
            // %APPDATA%\75hard\data.db — nunca dentro del repo.
            // `--datos <ruta>` mueve la base a otra carpeta: sirve para probar
            // con datos de ejemplo sin tocar los reales.
            let dir = match datos_override() {
                Some(ruta) => ruta,
                None => app
                    .path()
                    .data_dir()
                    .map_err(|e| AppError::internal(format!("no se encontró %APPDATA%: {e}")))?
                    .join("75hard"),
            };

            let db = db::open(&dir)?;
            app.manage(db);

            tray::build(app.handle())?;

            // Si Windows la arrancó sola, se queda en bandeja y no molesta.
            let silent = std::env::args().any(|a| a == "--bandeja");
            if !silent {
                commands::window::open_main(app.handle())?;
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("no se pudo construir la aplicación")
        .run(|_app, event| {
            // El corazón del requisito de RAM: cuando se cierra la última
            // ventana, Tauri pide salir. Se lo impedimos. La ventana ya fue
            // destruida (WebView2 liberó sus procesos) y queda vivo solo el
            // proceso Rust en la bandeja, por debajo de 15 MB.
            if let RunEvent::ExitRequested { api, code, .. } = &event {
                if code.is_none() {
                    api.prevent_exit();
                    ram::release_after_window_closed();
                }
            }
        });
}
