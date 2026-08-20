# 75 HARD

Tracker personal de hábitos para un reto de 75 días. App de escritorio para
Windows. **Un solo usuario, sin cuentas, sin servidor, sin internet.**

Los datos viven en `%APPDATA%\75hard\data.db` (SQLite con WAL). Nunca salen de
la máquina: cero telemetría, cero analítica, cero red.

El diseño completo está en [`docs/75HARD-SPEC.md`](docs/75HARD-SPEC.md) (modelo
de datos, fases y decisiones) y [`docs/75HARD-DISENO.md`](docs/75HARD-DISENO.md)
(sistema visual y mapa de pantallas). Son la fuente de verdad.

---

## Estado: Fases 0 y 1 completas

Lo que ya funciona:

- Icono en bandeja con menú **Abrir / Captura rápida / Salir**, y autoarranque
  con Windows activable desde Ajustes.
- **Hoy** (P1): día del reto, racha, cuatro métricas, checklist de pilares,
  panel del coach y registro rápido.
- **Check-in matutino** (P2) con veredicto de sueño en vivo.
- **Check-in nocturno** (P3) con campos en línea para lo que falta.
- **Captura rápida** (P4): ventana flotante de 380×460.
- **Historial** (P5) editable hacia atrás y **Detalle del día** (P6).
- **Onboarding** de 3 pasos (P12) y **días sin registrar** (P13).
- **Ajustes** (P11): tono del coach, metas, autoarranque.
- Registro de comida, snack, ejercicio, glucosa, agua, lectura, trabajo, peso,
  ánimo y energía. Escritura inmediata en cada acción.
- **Heatmap de 75 días** en Historial, con los cinco estados y leyenda.
- **Notificaciones programadas** que miran los datos del día antes de molestar:
  si ya registraste el ejercicio, el aviso de las 17:00 no suena. Configurables
  desde Ajustes, con interruptor general y horario de silencio.
- **Atajo global `Ctrl+Alt+H`** para la captura rápida desde cualquier app.
- **Pantalla de racha rota** (P14) al cerrar un día como fallido.

El detalle de la Fase 1 está en [`docs/FASE-1-AVANCE.md`](docs/FASE-1-AVANCE.md).

Pendiente por fase, tal y como está en el spec:

| Fase | Qué falta |
|---|---|
| 2 | Las gráficas (P7–P10), export CSV/JSON |
| 3 | PDF para el médico, correlaciones, fotos de progreso, backup cifrado, tema oscuro, reto completado (P15) |

El tema oscuro ya tiene sus tokens definidos en `src/app.css`; solo falta el
interruptor.

---

## Cómo correrlo

Necesitas Rust (MSVC), Node 20+ y WebView2 (ya viene con Windows 11).

```bash
npm install
```

Desarrollo, con recarga en caliente del frontend:

```bash
npm run tauri dev
```

Compilar el binario de producción:

```bash
npm run tauri build
```

> En producción el frontend va **embebido en el binario**. Si cambias algo de
> `src/`, tienes que recompilar el ejecutable; `npm run build` por sí solo no
> actualiza la app ya compilada. En `dev` no aplica: ahí se sirve desde Vite.

Comprobaciones:

```bash
npm run check
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

---

## Consumo de memoria

Requisito del proyecto: en reposo, por debajo de 15 MB. Medido en Windows 11
sobre el binario de release:

| Estado | Working set | Comprometido |
|---|---|---|
| Ventana abierta | ~27 MB (+ procesos de WebView2) | ~6 MB |
| **Cerrada, en bandeja** | **~1.7 MB** | **~5.6 MB** |

Al cerrar la ventana se **destruye**, no se esconde: WebView2 termina sus
procesos y el proceso Rust devuelve su working set al sistema
(`src-tauri/src/ram.rs`). Reabrir tarda unos 400 ms.

---

## Estructura

```
src/                    UI — Svelte 5 + TypeScript, CSS puro con variables
  app.css               tokens del sistema de diseño
  lib/api.ts            envoltorios tipados de invoke()
  lib/components/       Shell, TitleBar, Modal, EntryForms, CoachPanel…
  routes/               Today, History, DayDetail, Settings, Onboarding…
src-tauri/
  src/coach.rs          reglas de regaño/felicitación (puras, con tests)
  src/daycut.rs         el día corta a las 4:00 AM (con tests)
  src/scheduler.rs      tick de 60 s que dispara los recordatorios
  src/ram.rs            devolver memoria al cerrar la ventana
  src/tray.rs           icono de bandeja
  src/db/               conexión, migraciones, modelos y toda la SQL
  src/commands/         comandos expuestos a la UI
  migrations/           001_init.sql, 002_reminders.sql
  examples/seed_demo.rs base de ejemplo para revisar la interfaz
docs/                   spec y sistema de diseño
```

### Banderas de línea de comandos

| Bandera | Para qué |
|---|---|
| `--bandeja` | Arranca sin ventana, solo en la bandeja. La usa el autoarranque. |
| `--datos <ruta>` | Usa otra carpeta para la base de datos en vez de `%APPDATA%\75hard`. |

### Datos de ejemplo

Para revisar la interfaz con 23 días de historia sin tocar tus datos reales:

```bash
cargo run --manifest-path src-tauri/Cargo.toml --release --example seed_demo -- C:\temp\demo75
```

Y luego abrir la app con `--datos C:\temp\demo75\75hard`. Añadiendo
`--olvidar 3` al comando anterior se borran los últimos 3 días para probar la
pantalla de recuperación.

---

## Decisiones que conviene no romper

- **El día corta a las 4:00 AM**, no a medianoche. El sueño se atribuye al día
  en que despiertas.
- **Las calorías son siempre opcionales y la app nunca opina sobre ellas.** Sin
  metas, sin semáforos, sin "te pasaste".
- **El coach le habla al hábito, nunca a la persona**, ni en el tono duro. Hay
  un test que lo verifica.
- **La glucosa se registra y se grafica; no se diagnostica.** El valor está en
  llegar a la cita médica con datos, no en que la app interprete.
- **Un color de estado nunca va solo**: siempre con icono o texto.
- Nada de sombras: la profundidad se hace con hairlines de 1px y saltos de
  superficie.
