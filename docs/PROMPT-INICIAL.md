# Prompt inicial para empezar a programar

Cuando quieras arrancar la implementación, pega esto en una sesión nueva (con el repo `75HARD` conectado). Está escrito para producir la **Fase 0** completa y funcional, no un esqueleto vacío.

---

```
Vas a construir una app de escritorio para Windows llamada "75HARD" en el repo
que tienes conectado (está vacío, solo tiene .git). Es un tracker personal de
hábitos para un reto de 75 días. Un solo usuario, sin cuentas, sin servidor,
sin internet. Lee docs/75HARD-SPEC.md antes de escribir código: ahí está el
diseño completo y es la fuente de verdad.

## Stack (no lo cambies sin avisarme primero)
- Tauri v2 + Rust (edición 2021)
- UI: Svelte 5 + TypeScript + Vite
- DB: SQLite vía rusqlite con feature "bundled", en %APPDATA%\75hard\data.db
- Migraciones versionadas con PRAGMA user_version, en src-tauri/migrations/
- Gráficas: Chart.js (fase posterior, todavía no)
- CSS puro con variables. Nada de Tailwind ni librerías de componentes.

## Requisito no negociable: RAM
La app vive en la bandeja del sistema. Al cerrar la ventana principal debe
DESTRUIRSE (no ocultarse), de modo que WebView2 libere sus procesos y en reposo
solo quede el proceso Rust por debajo de 15 MB. Al reabrir desde la bandeja se
recrea la ventana. Verifica esto con el Administrador de tareas antes de decir
que está listo.

## Alcance de esta entrega (Fase 0, nada más)
1. Proyecto Tauri v2 que compila y corre en Windows.
2. Icono en bandeja con menú: Abrir / Captura rápida / Salir.
3. Autostart con Windows vía tauri-plugin-autostart, activable en Settings.
4. Esquema SQLite completo del §3 del spec, con la migración 001_init.sql.
   WAL activado. Escritura inmediata en cada acción, sin estado en memoria que
   se pueda perder.
5. Pantalla "Hoy": muestra el día N del reto y qué falta registrar.
6. Check-in matutino: hora en que se durmió, hora en que despertó, peso
   (opcional), ánimo y energía 1-5. Calcula las horas dormidas y muestra un
   veredicto usando las reglas de coach.rs.
7. Check-in nocturno: resumen del día y marcar el día como completo o fallido.
8. Formularios de registro rápido para: comida (con calorías OPCIONALES),
   snack, ejercicio (con calorías quemadas opcionales), lectura de glucosa
   (con contexto: ayunas / pre-comida / post-comida 2h / random), agua, sesión
   de trabajo.
9. Historial: tabla de los días pasados, editable retroactivamente.
10. Al abrir la app, si hay días calendario sin registrar desde el último uso,
    preguntar qué hacer con ellos (llenar ahora / marcar fallido / dejar vacío).

## Reglas de implementación
- El "día" corta a las 4:00 AM, no a medianoche. El sueño se atribuye al día
  en que la persona despierta.
- Todo formulario debe completarse solo con teclado: Tab, escribir, Enter.
  Sin campos obligatorios más allá del mínimo indispensable.
- Las calorías son siempre opcionales y la app nunca muestra un juicio sobre
  ellas (nada de "te pasaste", ni semáforos, ni metas de calorías).
- coach.rs contiene las reglas de mensajes como funciones puras de Rust sobre
  los datos. Sin IA, sin red. Con tests unitarios.
- Nada de unwrap() en rutas que el usuario pueda tocar: maneja errores y
  muéstralos en la UI.
- .gitignore debe excluir *.db, *.db-wal, *.db-shm, /photos/, /backups/,
  target/ y node_modules/.

## Lo que NO quiero en esta entrega
Notificaciones programadas, hotkey global, gráficas, heatmap, export, PDF,
backups. Todo eso es Fase 1 y 2. Si te sobra tiempo, mejora los tests, no
agregues features.

## Cómo trabajar
Primero muéstrame el plan de archivos y el SQL de la migración inicial, y
espera mi visto bueno. Después implementa por bloques, empezando por la capa
de datos, luego los comandos de Tauri, luego la UI. Al terminar dime
exactamente qué comandos correr para compilar y probar, y qué debería ver.
```

---

## Notas sobre este prompt

- **Le pide plan antes de código.** En un proyecto desde cero eso te ahorra rehacer la capa de datos.
- **Acota el alcance con una lista de "lo que NO quiero".** Sin eso, cualquier modelo te entrega 40 archivos y gráficas a medias en vez de una Fase 0 sólida.
- **El requisito de RAM está redactado como verificable** ("compruébalo en el Administrador de tareas"), no como un deseo vago.
- Sustituye `docs/75HARD-SPEC.md` por la ruta real donde dejes el spec.
