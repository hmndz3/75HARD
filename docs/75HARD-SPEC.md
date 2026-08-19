# 75 HARD — Spec de diseño (app de escritorio, Windows)

> Documento de diseño previo a programar. Versión 1.0 — 19 ago 2026.
> Decisiones tomadas: reto **híbrido** (checklist core + trackers propios), datos **100% locales**, **solo escritorio**, prioridad absoluta en **bajo consumo de RAM**.

---

## 1. Qué es esto realmente

No es una app de fitness. Es un **diario cuantificado de 75 días con un capataz encima**. Tres cosas tienen que ser ciertas o el proyecto fracasa:

1. **Registrar algo toma menos de 10 segundos.** Si tarda más, en el día 12 dejas de usarla.
2. **La app te busca a ti**, no tú a ella. Vive en la bandeja del sistema y te interrumpe.
3. **Nunca pierde un dato.** Ni por reinicio, ni por apagón, ni por un día que no abriste la compu.

Todo lo demás (gráficas bonitas, correlaciones, temas) es secundario y va en fases posteriores.

---

## 2. Stack recomendado

### Veredicto: **Tauri v2 + Rust + SQLite**

Dijiste "solo escritorio, mínima RAM posible". Analicé las opciones con números reales en Windows 11:

| Opción | RAM en reposo (bandeja) | RAM con ventana abierta | Esfuerzo para gráficas |
|---|---|---|---|
| **Tauri v2** (destroy-on-close) | **~8–14 MB** | ~90–170 MB | Trivial (Chart.js/ECharts) |
| egui / eframe (Rust nativo) | ~35–60 MB | ~45–90 MB | Medio (egui_plot) |
| Slint (Rust nativo) | ~20–40 MB | ~25–50 MB | Alto (dibujar a mano) |
| Electron | ~120 MB | ~250–400 MB | Trivial |

**El razonamiento que importa:** la app va a estar en reposo el 99 % del tiempo. Vas a abrir la ventana 2–3 veces al día por un minuto. Entonces lo que hay que optimizar es el **estado en reposo**, no el pico.

Y ahí Tauri gana si aplicas un truco: al cerrar la ventana **no la escondes, la destruyes** (`window.close()` con el proceso principal vivo). WebView2 libera sus procesos y te quedas solo con el binario Rust: ~10 MB residentes, menos que egui. Cuando reabres, tarda ~400 ms en levantar la ventana — un precio que ni notas 3 veces al día.

Ventajas adicionales de Tauri aquí:
- WebView2 ya viene con Windows 11 → tu instalador pesa **~4–8 MB**, no 90 MB como Electron.
- Notificaciones nativas de Windows, bandeja, autostart y hotkeys globales son plugins oficiales (`tauri-plugin-notification`, `-autostart`, `-global-shortcut`).
- Las gráficas — que son el 40 % de lo que quieres — son HTML/JS, o sea gratis. En egui te costarían semanas.

**Cuándo NO usar Tauri:** si te obsesionas con que el pico también sea bajo. En ese caso la alternativa es un **daemon Rust puro (~8 MB siempre) que sirve la UI en `127.0.0.1:7575` y la abres en el Chrome que ya tienes abierto**. Consumo marginal casi cero, gráficas igual de fáciles. La contra es que no se siente como app: es una pestaña más. Mi recomendación sigue siendo Tauri, pero esta opción es legítima y la dejo anotada.

### Detalle del stack

```
Backend      Rust 2021 (edición 2024 si tu toolchain la soporta)
Framework    Tauri v2
UI           Svelte 5 + TypeScript  (bundle chico, sin Virtual DOM, ~15 KB)
             Alternativa: HTML/JS vanilla si quieres cero build step
Estilos      CSS puro con variables, o UnoCSS. NO Tailwind full (peso innecesario)
Gráficas     ECharts (más completo) o Chart.js (más liviano, ~60 KB)
DB           SQLite vía rusqlite (feature "bundled" → sin dependencias externas)
Migraciones  refinery, o manual con PRAGMA user_version
Fechas       chrono / time
Scheduler    tokio + un tick de 60 s que evalúa recordatorios pendientes
Logs         tracing → archivo rotado, nivel INFO
```

### Por qué SQLite y no Railway (por ahora)

Tu volumen total de datos en 75 días: **menos de 5 000 filas, ~2 MB**. Eso cabe en la RAM de una calculadora. Un Postgres en Railway para esto es como rentar un camión para llevar una pizza — además te obliga a tener internet para registrar que dormiste 6 horas.

Railway sí sirve para algo, después: como **destino de backup** (un endpoint que recibe tu `.db` cifrado una vez al día) y, si algún día quieres celular, como capa de sync. El esquema que propongo abajo ya está diseñado para eso: todas las tablas llevan `id` UUID + `updated_at`, así que agregar sincronización luego no requiere rehacer nada.

---

## 3. Modelo de datos

Nota de diseño importante: separo **`challenge`** (el intento) de los **logs diarios**. Si rompes la racha en el día 40 y reinicias, tu historial de peso, sueño y azúcar **sigue siendo continuo** — solo empieza un `challenge` nuevo. Sin esto, cada reinicio te borra visualmente meses de datos.

```sql
-- El intento del reto. Puedes tener varios a lo largo del tiempo.
challenge (
  id TEXT PK, name TEXT, start_date DATE, target_days INT DEFAULT 75,
  rules_json TEXT,           -- qué pilares cuentan como "core" en este intento
  ended_at DATE NULL, ended_reason TEXT NULL,  -- 'completed' | 'failed' | 'abandoned'
  updated_at TIMESTAMP
)

-- Una fila por día calendario. El ancla de todo.
day (
  date DATE PK, challenge_id TEXT, day_number INT,
  status TEXT,               -- 'pending' | 'complete' | 'failed' | 'skipped'
  morning_checkin_at TIMESTAMP NULL, evening_checkin_at TIMESTAMP NULL,
  notes TEXT, updated_at TIMESTAMP
)

sleep_log (
  date DATE PK,              -- el día en que DESPERTASTE (ver §7, regla del corte)
  bedtime TIMESTAMP, wake_time TIMESTAMP, minutes INT,
  quality INT NULL,          -- 1..5
  woke_up_during_night INT NULL, updated_at TIMESTAMP
)

meal (
  id TEXT PK, date DATE, eaten_at TIMESTAMP,
  kind TEXT,                 -- 'meal' | 'snack'
  description TEXT,          -- texto libre, multilínea
  calories INT NULL,         -- SIEMPRE opcional
  photo_path TEXT NULL, updated_at TIMESTAMP
)

workout (
  id TEXT PK, date DATE, started_at TIMESTAMP,
  kind TEXT,                 -- 'gym' | 'cardio' | 'outdoor' | 'sport' | 'other'
  description TEXT, duration_min INT,
  is_outdoor BOOLEAN, calories_burned INT NULL, updated_at TIMESTAMP
)

glucose_reading (
  id TEXT PK, measured_at TIMESTAMP, value_mgdl INT,
  context TEXT,              -- 'fasting' | 'pre_meal' | 'post_meal_2h' | 'random' | 'pre_workout'
  notes TEXT NULL,           -- "había comido pizza hace 2h"
  linked_meal_id TEXT NULL,  -- autocompletado: la comida de las últimas 3h
  updated_at TIMESTAMP
)

work_session (id TEXT PK, date DATE, started_at, ended_at, minutes INT,
              category TEXT, description TEXT, updated_at TIMESTAMP)

water_log   (date DATE PK, ml INT, updated_at TIMESTAMP)
reading_log (id TEXT PK, date DATE, pages INT, book TEXT, updated_at TIMESTAMP)
weight_log  (date DATE PK, kg REAL, body_fat_pct REAL NULL, updated_at TIMESTAMP)
mood_log    (date DATE PK, mood INT, energy INT, stress INT NULL,
             notes TEXT, updated_at TIMESTAMP)
progress_photo (id TEXT PK, date DATE, path TEXT, updated_at TIMESTAMP)

reminder (id TEXT PK, kind TEXT, time_of_day TIME, days_mask INT,
          enabled BOOLEAN, last_fired_at TIMESTAMP NULL)

settings (key TEXT PK, value TEXT)
```

**Configuración de SQLite:** `PRAGMA journal_mode=WAL; synchronous=NORMAL; foreign_keys=ON;`
**Ubicación:** `%APPDATA%\75hard\data.db` (nunca dentro del repo — el repo es código, no tus datos).
**`.gitignore` obligatorio:** `*.db`, `*.db-wal`, `*.db-shm`, `/photos/`, `/backups/`.

---

## 4. Funcionalidades, por fase

### Fase 0 — MVP usable (meta: 1 semana de trabajo)
Objetivo: que puedas **empezar el reto** con esto. No es bonito, es funcional.

- [ ] Icono en bandeja + arranque automático con Windows
- [ ] Ventana principal con "Hoy": qué falta registrar
- [ ] **Check-in matutino**: a qué hora te dormiste, a qué hora despertaste → cálculo + veredicto
- [ ] **Check-in nocturno**: comidas, snacks, ejercicio, agua, lectura, ánimo → marcar día
- [ ] Registro de comida / ejercicio / azúcar en cualquier momento
- [ ] Vista de historial: tabla de días hacia atrás, editable
- [ ] SQLite con escritura inmediata en cada acción

### Fase 1 — Que te persiga
- [ ] Notificaciones nativas programadas (mañana, agua, ejercicio, noche)
- [ ] **Hotkey global** (ej. `Ctrl+Alt+H`) → ventana mini de captura rápida, 5 segundos, Enter y se cierra
- [ ] **Heatmap de 75 días** estilo GitHub (verde / amarillo / rojo / gris) — el visual estrella
- [ ] Sistema de mensajes: regaño / felicitación según reglas (ver §6)
- [ ] Racha actual + racha más larga

### Fase 2 — Las gráficas
- [ ] Sueño: barras diarias + línea de meta + promedio móvil de 7 días
- [ ] Ejercicio: minutos por semana, split indoor/outdoor
- [ ] Calorías: ingeridas vs quemadas (solo cuando hay datos, sin inventar)
- [ ] Azúcar: dispersión temporal con bandas de referencia por contexto
- [ ] Peso: línea con promedio móvil de 7 días (**no** el dato crudo — ver §6)
- [ ] Trabajo: horas por día y por categoría
- [ ] Export CSV / JSON completo

### Fase 3 — Lujos
- [ ] **Reporte PDF para el doctor**: azúcar + comidas + peso del último mes, listo para imprimir
- [ ] Correlaciones: sueño ↔ energía, sueño ↔ azúcar en ayunas, ejercicio ↔ ánimo
- [ ] Fotos de progreso con comparador antes/después
- [ ] Backup cifrado automático (local, y opcionalmente a tu Railway)
- [ ] Temas claro/oscuro

---

## 5. El flujo diario (lo más importante del documento)

```
07:00  🔔 "Buenos días. ¿A qué hora te dormiste y a qué hora despertaste?"
       → 2 inputs de hora. Peso (opcional). Ánimo y energía 1-5.
       → Veredicto inmediato: "6h 10min. Vas 3 días seguidos bajo 7h."

12:30  🔔 "¿Ya almorzaste? Registra la comida."   [Registrar] [Ya lo hice] [Luego]

17:00  🔔 "Aún no registras ejercicio hoy. Quedan 7 horas."

21:30  🔔 Check-in nocturno → resumen del día:
       ✅ Sueño registrado    ❌ Ejercicio: falta
       ✅ 3 comidas, 2 snacks ⚠️ Agua: 1.8L de 3L
       "¿Cierro el día como completo o fallido?"

En cualquier momento: Ctrl+Alt+H → ventana mini
       [Comida] [Ejercicio] [Azúcar] [Agua +500ml] [Nota]
```

**Regla de oro de UX:** cada pregunta debe poder responderse con teclado puro. Tab, número, Enter. Si necesitas el mouse, perdiste 5 segundos y la próxima vez no lo haces.

---

## 6. El sistema de "regañarme o felicitarme"

Esto lo pediste explícitamente y es donde hay que tener cuidado. Diseño propuesto:

**Reglas duras, no IA.** Umbrales evaluados en Rust, cero costo, cero latencia, cero dependencia de internet:

| Condición | Mensaje |
|---|---|
| Sueño ≥ 7h | "Dormiste 7h 40. Así se ve un día bien empezado." |
| Sueño < 6h | "5h 20min. Hoy vas a rendir menos y lo sabes." |
| 3 días seguidos < 6h | "Tercer día seguido con poco sueño. Esto ya no es un mal día, es un patrón." |
| Día completo | "Día 23 de 75. Racha intacta." |
| Racha rota | "Se rompió la racha en el día 31. Los datos de esos 31 días siguen aquí. Empezamos el intento #2." |

**Tono configurable en 3 niveles** (suave / directo / duro) en Settings.

Tres decisiones de diseño que te recomiendo fuerte, aunque no las pediste:

1. **Las calorías siempre opcionales, nunca con meta ni semáforo.** Tú ya lo planteaste así con el `+` opcional y está bien. Que la app no te diga nunca "te pasaste". Contar calorías con un sistema que te regaña es la receta exacta para una relación fea con la comida.
2. **El peso se grafica con promedio móvil de 7 días, no el dato diario.** El peso diario oscila ±1.5 kg por agua y sal. Ver el dato crudo te hace sentir que fallaste un día en que no fallaste nada.
3. **El regaño va al hábito, nunca a ti.** "Dormiste poco" ≠ "eres un desastre". Que el nivel "duro" siga siendo sobre la conducta. Un tracker que te insulta se deja de abrir en dos semanas — y peor, deja huella.

---

## 7. Decisiones finas que hay que cerrar antes de programar

**El corte del día.** Si te duermes a la 1:00 AM del martes, ¿ese sueño es del lunes o del martes?
→ Propuesta: **el sueño se atribuye al día en que despiertas**. Y el "día" para el resto de registros corta a las **4:00 AM**, no a medianoche. Así, si registras algo a la 1 AM, cuenta para el día anterior — que es como funciona tu cabeza.

**Días sin abrir la compu.** Si no la prendes el sábado, al abrirla el domingo:
→ "Tienes 1 día sin registrar. ¿Lo llenas ahora, lo marcas como fallido, o lo dejas en blanco?" Sin esto, un fin de semana te destruye la data.

**Modo pausa.** ¿Enfermedad, viaje, emergencia familiar? En 75 Hard estricto no existe. En tu versión híbrida propongo un `status = 'skipped'` que no rompe la racha pero se ve distinto en el heatmap (gris rayado). Tú decides al usarlo si fue legítimo.

**Privacidad.** Esto son datos médicos (glucosa, peso, fotos). Todo local, cero telemetría, cero analytics, cero cuentas. Si algún día haces backup a Railway, va cifrado con una passphrase tuya.

**Disclaimer de glucosa.** La app registra y grafica; no diagnostica, no sugiere dosis, no interpreta. Su valor real es que llegues a la cita médica con un PDF de 30 días de lecturas en vez de "pues a veces me sale alto". Eso vale muchísimo y es una feature, no un adorno.

---

## 8. Estructura de repo propuesta

```
75HARD/
├─ src-tauri/
│  ├─ src/
│  │  ├─ main.rs
│  │  ├─ db/            mod.rs, migrations.rs, models.rs, queries.rs
│  │  ├─ commands/      day.rs, sleep.rs, meals.rs, workouts.rs, glucose.rs, stats.rs
│  │  ├─ scheduler.rs   tick de 60s, dispara recordatorios
│  │  ├─ coach.rs       las reglas de regaño/felicitación
│  │  ├─ tray.rs
│  │  └─ backup.rs
│  ├─ migrations/       001_init.sql, 002_...
│  ├─ Cargo.toml
│  └─ tauri.conf.json
├─ src/                 UI (Svelte)
│  ├─ routes/           Today, History, Stats, Settings
│  ├─ lib/components/   Heatmap, SleepChart, GlucoseChart, QuickEntry
│  └─ lib/api.ts        wrappers de invoke()
├─ docs/                este documento
├─ .gitignore
└─ README.md
```

---

## 9. Consejo final sobre el reto en sí

No esperes a que la app esté lista para empezar los 75 días. **Arranca el reto ya**, anota en un archivo de texto o en notas del celular, y cuando la Fase 0 esté lista importas esos días a mano en 20 minutos. Construir la app antes de empezar es la forma más elegante de procrastinar el reto — y tú lo que quieres es el reto, no el software.

Regla sana: **máximo 1 hora al día en la app.** Si el proyecto empieza a comerse el tiempo de gym y de tus clases, el proyecto va perdiendo.
