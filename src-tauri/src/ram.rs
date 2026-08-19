//! Devolver la RAM al sistema cuando la ventana se cierra.
//!
//! Cerrar la ventana destruye el WebView2 y sus procesos hijo mueren, pero el
//! proceso Rust se queda con un working set inflado por las DLL que llegó a
//! cargar: ~27 MB en vez de los ~13 MB con los que arrancó. Windows no lo
//! recorta solo mientras haya memoria libre.
//!
//! `SetProcessWorkingSetSize(proceso, -1, -1)` le dice al sistema que puede
//! sacar esas páginas a la lista de standby. No se pierde nada: si hacen falta
//! otra vez se recargan. Es lo que hace cualquier app de bandeja seria.

/// Recorta el working set del proceso actual, dos veces: una de inmediato y
/// otra unos segundos después, cuando WebView2 ya terminó de desmontarse.
pub fn release_after_window_closed() {
    trim();
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(3));
        trim();
    });
}

#[cfg(windows)]
fn trim() {
    unsafe extern "system" {
        fn GetCurrentProcess() -> isize;
        fn SetProcessWorkingSetSize(handle: isize, min: usize, max: usize) -> i32;
    }

    // usize::MAX es (SIZE_T)-1: "recorta todo lo que puedas".
    unsafe {
        SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
    }
}

#[cfg(not(windows))]
fn trim() {}
