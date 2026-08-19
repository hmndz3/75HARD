# 75 HARD — Sistema de diseño y mapa de pantallas

> Para diseñar en **Stitch (Google)**. Estética: formal, gris, sobria. Los colores solo aparecen donde significan algo.
> Versión 1.0 — 19 ago 2026.

---

## 1. La idea visual en una frase

**Todo es gris; el color es información.** Nada de acentos decorativos, nada de degradados, nada de tarjetas de colores. La interfaz entera vive en una escala de grises cálidos, y cuando aparece verde, ámbar o rojo, es porque un dato lo está diciendo. Es la lógica de un terminal financiero o un panel médico: aburrido a propósito, para que lo importante grite solo.

Los "shades diferentes" que pediste se juegan en tres niveles de superficie (fondo → panel → tarjeta), cada uno un paso más claro, separados por hairlines de 1px en vez de sombras.

---

## 2. Paleta

### Grises (la base — el 95% de la pantalla)

| Token | Light | Dark | Uso |
|---|---|---|---|
| `--bg` | `#f4f4f2` | `#141413` | Fondo de la app |
| `--surface-1` | `#fbfbfa` | `#1f1f1d` | Paneles, tarjetas |
| `--surface-2` | `#ffffff` | `#2e2d2a` | Tarjeta elevada, input enfocado |
| `--surface-sunken` | `#eaeae7` | `#0d0d0c` | Zonas hundidas, tabla rayada |
| `--border` | `#dcdbd7` | `#2e2d2a` | Hairline 1px |
| `--border-strong` | `#c6c5bf` | `#45443f` | Borde de input, divisor fuerte |
| `--ink-primary` | `#141413` | `#fbfbfa` | Texto principal, números grandes |
| `--ink-secondary` | `#5e5d58` | `#a5a49d` | Texto de apoyo |
| `--ink-muted` | `#7f7e78` | `#7f7e78` | Labels de ejes, placeholders |

Escala completa por si necesitas pasos intermedios:
`#ffffff · #fbfbfa · #f4f4f2 · #eaeae7 · #dcdbd7 · #c6c5bf · #a5a49d · #7f7e78 · #5e5d58 · #45443f · #2e2d2a · #1f1f1d · #141413`

### Acento — azul pizarra

Un solo acento en toda la app. Es el color del botón primario, del foco de teclado y de la serie principal de casi todas las gráficas.

| Paso | Hex | Uso |
|---|---|---|
| 100 | `#e4ecf4` | Fondo de selección |
| 200 | `#c8d9ea` | |
| 300 | `#a7c0dc` | |
| 400 | `#84a4c9` | |
| 500 | `#5f86b2` | Acento en modo oscuro |
| **600** | **`#3a6ea5`** | **Acento base — botón primario, serie 1** |
| 700 | `#2e5786` | Hover del botón primario |
| 800 | `#234269` | |
| 900 | `#192e4b` | |

Rampa monotónica verificada. Es también la rampa **secuencial** para heatmaps y magnitudes.

### Estado — los únicos colores saturados de la app

| Rol | Hex | Significado en la app |
|---|---|---|
| good | `#0ca30c` | Día completo, meta alcanzada, glucosa en rango |
| warning | `#fab219` | Día parcial, meta cerca, glucosa límite |
| serious | `#ec835a` | Advertencia acumulada (3 días seguidos mal) |
| critical | `#d03b3b` | Día fallido, racha rota, glucosa fuera de rango |

**Regla obligatoria:** un color de estado nunca va solo. Siempre acompañado de icono y/o texto. Alguien con daltonismo tiene que poder leer tu heatmap.

### Categóricas — máximo 3 series

Solo para las pocas gráficas con series distintas (indoor vs outdoor, ingeridas vs quemadas). Validadas con el verificador de contraste y separación para daltonismo, en ambos modos:

| Slot | Light | Dark |
|---|---|---|
| 1 | `#3a6ea5` azul pizarra | `#5b93cc` |
| 2 | `#c05a30` terracota | `#d1703f` |
| 3 | `#1f8a6d` verde pizarra | `#3aab8c` |

**Nunca pases de 3.** Si necesitas más, la respuesta correcta es partir en gráficas pequeñas lado a lado (*small multiples*), no inventar un cuarto color. Y para "esta serie es la importante, las demás son contexto", la forma correcta es **énfasis**: una serie en azul, el resto en `#c6c5bf`.

### Tipografía

`Inter` o el `system-ui` de Windows (Segoe UI Variable). Sin serif, sin display, sin fuente decorativa en ningún lado.

| Rol | Tamaño / peso |
|---|---|
| Cifra héroe ("Día 23") | 56px / 600 |
| Valor de tarjeta | 32px / 600 |
| Título de sección | 18px / 600 |
| Cuerpo | 14px / 400 |
| Label / eje | 12px / 500, `--ink-muted`, mayúsculas con `letter-spacing: .04em` |

Números en columnas y ejes: `font-variant-numeric: tabular-nums`.

### Reglas de forma

- Radio: **6px** en tarjetas e inputs, **4px** en botones y chips. Nada de píldoras ni de 16px.
- **Sin sombras.** La separación se hace con hairlines de 1px y cambio de superficie.
- Grid de 8px. Padding de tarjeta: 20px. Gap entre tarjetas: 16px.
- Foco de teclado: anillo de 2px `#3a6ea5` con 2px de offset. Visible siempre, en todo.

---

## 3. El shell de la aplicación

```
┌──────────────────────────────────────────────────────────────┐
│  75 HARD · Día 23 de 75            [buscar]      [⚙] [— □ ×]│  ← 44px, --surface-1
├────────┬─────────────────────────────────────────────────────┤
│        │                                                     │
│  Hoy   │                                                     │
│  Hist. │           Área de contenido                         │
│  Estad.│           (--bg, scroll propio)                     │
│  Ajust.│                                                     │
│        │                                                     │
│ ────── │                                                     │
│ Racha  │                                                     │
│  23 🔥 │                                                     │
└────────┴─────────────────────────────────────────────────────┘
   200px            resto
```

Barra lateral de 200px, colapsable a 60px (solo iconos). Fondo `--surface-1`, hairline a la derecha. Ítem activo: fondo `--surface-sunken` + barra de 3px `#3a6ea5` a la izquierda. Ventana por defecto 1280×800, mínimo 960×640.

---

## 4. Mapa de pantallas

### Prioridad 1 — sin esto no hay app (6 pantallas)

| # | Pantalla | Qué contiene |
|---|---|---|
| **P1** | **Hoy** | Cifra héroe del día, racha, checklist de pilares, fila de 4 tarjetas de métricas, mensaje del coach, botones de registro rápido |
| **P2** | **Check-in matutino** | Dos selectores de hora grandes, horas dormidas calculadas en vivo, peso opcional, ánimo y energía 1–5, veredicto |
| **P3** | **Check-in nocturno** | Resumen del día con checks, campos faltantes en línea, botón de cerrar día |
| **P4** | **Captura rápida** (ventana flotante 380×460) | Tabs: Comida · Ejercicio · Azúcar · Agua. Un campo enfocado al abrir, Enter guarda y cierra |
| **P5** | **Historial** | Heatmap de 75 días arriba, lista de días abajo con filtros |
| **P6** | **Detalle del día** | Todo lo registrado ese día, en secciones editables |

### Prioridad 2 — las gráficas (4 pantallas, o 1 con tabs)

| # | Pantalla | Gráficas |
|---|---|---|
| **P7** | **Estadísticas · Sueño** | Barras diarias con línea de meta, promedio móvil de 7 días, histograma de hora de dormir, tarjetas de promedio/déficit |
| **P8** | **Estadísticas · Ejercicio** | Columnas de minutos por semana (2 series: indoor / outdoor), calorías ingeridas vs quemadas, distribución por tipo |
| **P9** | **Estadísticas · Glucosa** | Dispersión temporal con bandas de referencia, promedio por contexto, tabla de lecturas, botón de exportar PDF |
| **P10** | **Estadísticas · Peso y trabajo** | Peso con promedio móvil de 7 días, horas de trabajo por día apiladas por categoría |

### Prioridad 3 — el resto (5 pantallas)

| # | Pantalla | Qué contiene |
|---|---|---|
| **P11** | **Ajustes** | Secciones: Reto · Recordatorios · Coach (tono) · Datos y backups · Apariencia · Sistema |
| **P12** | **Nuevo reto / Onboarding** | Wizard de 3 pasos: fechas, qué pilares cuentan, metas numéricas |
| **P13** | **Días sin registrar** | Estado de recuperación: lista de días pendientes con acción por cada uno |
| **P14** | **Racha rota** | Pantalla a página completa: qué se rompió, los datos siguen ahí, empezar intento #2 |
| **P15** | **Reto completado** | Resumen de los 75 días: antes/después, todas las métricas, exportar |

---

## 5. Prompts para Stitch

**Cómo usarlos:** Stitch responde mucho mejor en inglés, así que los prompts van en inglés con el texto de la interfaz especificado en español. Pega el **preámbulo** al inicio de *cada* prompt — es lo que mantiene las 15 pantallas coherentes entre sí.

### Preámbulo (pégalo siempre)

```
Desktop application UI, 1440x900 canvas, Windows desktop app (not a website —
no marketing hero, no footer, no nav links).

DESIGN SYSTEM — follow exactly:
- Formal, restrained, data-dense. Think Bloomberg terminal or a medical
  dashboard, not a consumer fitness app. Boring on purpose.
- Warm neutral greyscale only. Background #f4f4f2, panels #fbfbfa, elevated
  #ffffff, sunken #eaeae7. Borders are 1px hairlines #dcdbd7. NO drop shadows
  anywhere — depth comes from surface steps and hairlines only.
- Text: #141413 primary, #5e5d58 secondary, #7f7e78 muted labels.
- ONE accent color: slate blue #3a6ea5. Used only for the primary button,
  keyboard focus rings, and the main data series. Nothing else is colored.
- Status colors appear ONLY on data: green #0ca30c, amber #fab219,
  red #d03b3b. Always paired with an icon or a text label, never color alone.
- Typography: Inter. 56px/600 hero numbers, 32px/600 card values,
  18px/600 section titles, 14px/400 body, 12px/500 uppercase muted labels
  with 0.04em letter-spacing. Tabular numerals in tables and axes.
- Corner radius 6px on cards and inputs, 4px on buttons and chips. Never pills.
- 8px spacing grid. 20px card padding, 16px gap between cards.
- App shell: 200px left sidebar on #fbfbfa with a hairline right border,
  44px title bar on top. Active nav item has a sunken background and a 3px
  #3a6ea5 bar on its left edge.
- All interface copy in Spanish.
```

### P1 — Hoy

```
[PREÁMBULO]

Screen: "Hoy" — the main daily dashboard.

Top: a hero block with "DÍA 23" at 56px/600 and "de 75" at 18px in muted grey
beside it. To its right, a streak card: "RACHA ACTUAL" label, "23 días" value,
and a tiny 30-cell sparkline strip of the recent days in green/amber/red squares.

Below: a row of four equal metric cards, each with an uppercase muted label,
a large value, and a small secondary line:
- SUEÑO ANOCHE — "6h 40min" — "-20 min vs. tu meta" in amber
- EJERCICIO HOY — "0 min" — "sin registrar" in red
- COMIDAS — "2 + 1 snack" — "1,340 kcal registradas"
- AGUA — "1.8 / 3.0 L" — with a thin horizontal progress meter in slate blue

Below that, two columns:
LEFT (60%) — "PILARES DE HOY", a vertical checklist of 6 rows. Each row: a
24px status icon (check / clock / x), the pillar name in 14px, and a right-
aligned muted detail. Rows separated by hairlines, no card borders between them.
Items: Dormir 7h ✓, Ejercicio 45min ✗, Dieta limpia ✓, Agua 3L (en progreso),
Leer 10 páginas ✗, Foto de progreso ✓.

RIGHT (40%) — a "coach" panel on #eaeae7 sunken background with a 3px amber
left border, containing a short message: "Tercer día seguido bajo 7 horas de
sueño. Esto ya no es un mal día, es un patrón." Below it, four secondary
buttons in a 2x2 grid: Registrar comida, Registrar ejercicio, Medir azúcar,
Agregar agua. All outlined grey buttons except none — one primary slate blue
button spans the bottom: "Cerrar el día".
```

### P2 — Check-in matutino

```
[PREÁMBULO]

Screen: morning check-in, shown as a centered modal 560px wide over a dimmed app.

Header: "Buenos días, Harry" 18px/600, and below in muted 14px:
"Miércoles 19 de agosto · Día 23".

Body, generous vertical spacing:
1. Two large time inputs side by side, each 220px wide with a 12px uppercase
   label above: "¿A QUÉ HORA TE DORMISTE?" and "¿A QUÉ HORA DESPERTASTE?".
   Inputs are 48px tall, 24px tabular numerals, hairline border, white fill.
2. Immediately below, spanning both, a computed result strip on sunken
   background: "6 h 40 min" at 32px/600 with an amber dot and the label
   "20 minutos bajo tu meta de 7 h" in 14px secondary.
3. "PESO (OPCIONAL)" — a narrow 120px numeric input with a "kg" suffix.
4. "¿CÓMO AMANECISTE?" — two rows of five square 40px selectable chips
   numbered 1 to 5, labeled "Ánimo" and "Energía". Selected chip is filled
   slate blue with white numeral; unselected is hairline outline.

Footer: hairline divider, then a ghost "Después" button on the left and a
primary slate blue "Guardar" button on the right. Small muted hint under the
footer: "Enter para guardar".
```

### P3 — Check-in nocturno

```
[PREÁMBULO]

Screen: evening check-in modal, 640px wide, centered.

Header: "Cierre del día · Día 23" with the date in muted grey below.

Body: a summary list of 7 rows on alternating surface tones. Each row has a
20px status icon on the left, the item name, a value in the middle, and a
ghost "Editar" link on the right. Completed rows use a green check, missing
rows use a red x AND show an inline compact input right in the row so it can
be filled without leaving the screen.
Rows: Sueño 6h40 ✓ · Comidas 3 registradas ✓ · Snacks 1 ✓ · Ejercicio
(missing, inline input for minutes + type) · Agua 2.4/3L amber · Lectura
(missing, inline page count input) · Ánimo 4/5 ✓.

Below the list, a full-width text area with the placeholder "Notas del día
(opcional)".

Footer: two large buttons side by side, equal width — an outlined red-bordered
"Marcar como fallido" and a primary slate blue "Día completo". Above them a
muted line: "Vas a cerrar el día 23. Después podrás editarlo desde Historial."
```

### P4 — Captura rápida

```
[PREÁMBULO]

Screen: a small always-on-top floating utility window, exactly 380x460,
opened by a global hotkey. Compact and dense — this must be fillable in five
seconds.

Top: a 4-tab segmented control filling the width: Comida · Ejercicio · Azúcar ·
Agua. The "Comida" tab is active with a slate blue underline.

Body for the active Comida tab:
- A time field pre-filled with the current time, small, top-right aligned.
- Two chips: "Comida" (selected, filled slate blue) and "Snack" (outlined).
- A multiline text area, autofocused, with a visible focus ring, placeholder
  "¿Qué comiste?" — takes most of the space.
- A collapsed row: a small "+" outlined button next to muted text
  "Agregar calorías (opcional)".

Footer: hairline, then a ghost "Cancelar" and a primary "Guardar" button, with
"Ctrl+Enter" shown as a keyboard hint chip inside the primary button area.

The window has no sidebar and no title bar chrome beyond a thin 32px drag
strip reading "Captura rápida" with a close x.
```

### P5 — Historial

```
[PREÁMBULO]

Screen: "Historial", full app shell with sidebar.

Top section: a card titled "75 DÍAS" containing a GitHub-style contribution
heatmap — 75 cells in a grid of 15 columns x 5 rows, each cell 22px with 4px
gaps and 3px corner radius. Cell colors: completed days green #0ca30c,
partial amber #fab219, failed red #d03b3b, skipped a diagonally hatched grey,
and future days empty #eaeae7. Day numbers in tiny 10px muted text inside each
cell. Below the grid a legend row with a small square plus a text label for
each of the five states. Top-right of the card: three summary figures inline —
"18 completos · 3 parciales · 2 fallidos".

Below: a filter row — a segmented control (Todos · Completos · Fallidos), a
date range dropdown, and a search input, all 36px tall.

Below that: a dense data table with tabular numerals and hairline row
separators, no vertical borders, alternating row backgrounds. Columns: Día,
Fecha, Estado (a small colored dot plus text label), Sueño, Ejercicio, Comidas,
Agua, Azúcar, Peso. Rows are clickable; show one row in a hover state with a
sunken background. Around 12 rows visible.
```

### P6 — Detalle del día

```
[PREÁMBULO]

Screen: detail view for a single day, full app shell.

Header row: a back chevron, "Día 23 · Miércoles 19 de agosto" at 18px/600, a
green "Completo" status pill with a check icon, and on the right a ghost
"Editar día" button.

Body: a two-column masonry of cards, 16px gap.
- SUEÑO card: a horizontal timeline bar showing the sleep window from 23:40
  to 06:20 rendered as a slate blue rounded segment against a 24-hour grey
  track, with "6 h 40 min" at 32px beside it.
- COMIDAS card: a vertical timeline, each entry a row with a time in tabular
  numerals, the description, and a right-aligned calorie value in muted grey
  (some entries show "—" for missing calories). A ghost "+ Agregar" row at the
  bottom.
- EJERCICIO card: type, duration, an "Al aire libre" outlined chip, calories
  burned.
- AZÚCAR card: two readings listed with value at 24px, a context chip
  ("En ayunas", "2 h post-comida"), and a small colored dot for in/out of range.
- TRABAJO card: total hours and a thin stacked horizontal bar split by category.
- ÁNIMO Y ENERGÍA card: two rows of five small squares with the selected value
  filled slate blue.
- NOTAS card: plain paragraph text, spanning full width at the bottom.
```

### P7 — Estadísticas · Sueño

```
[PREÁMBULO]

Screen: "Estadísticas", full app shell, with a tab bar at the top:
Sueño · Ejercicio · Glucosa · Peso y trabajo. The "Sueño" tab is active.

Under the tabs, a time range segmented control: 7 días · 30 días · Todo.

Row of four stat tiles: PROMEDIO 6h 52min (with "-8 min vs meta" in amber),
MEJOR NOCHE 8h 15min, PEOR NOCHE 4h 40min, DÉFICIT ACUMULADO -4h 20min in red.
Each tile has an uppercase muted label, a 32px value, and a small secondary line.

Main chart card, full width, 320px tall: a vertical bar chart of hours slept
per day for 30 days. Bars are slate blue #3a6ea5, thin, with 4px rounded tops
and 2px gaps, anchored on the baseline. Bars below the goal are grey #c6c5bf
instead of blue. A dashed horizontal reference line at 7h labeled "Meta 7h" in
muted text at the right end. Over the bars, a smooth 2px darker line showing
the 7-day moving average. Y axis in tabular numerals, hairline gridlines
#dcdbd7, no vertical gridlines, no chart border. A legend at the top-left of
the card with two small swatches: "Horas dormidas" and "Promedio 7 días".

Below, two half-width cards side by side:
- "HORA DE DORMIR" — a histogram of bedtimes in 30-minute buckets, all bars
  grey except the modal bucket in slate blue.
- "SUEÑO VS ENERGÍA" — a scatter plot, slate blue dots 8px, with a faint
  trend line, x axis hours slept, y axis energy 1-5. A muted caption below:
  "Correlación observada, no causalidad."
```

### P8 — Estadísticas · Ejercicio

```
[PREÁMBULO]

Screen: "Estadísticas" with the "Ejercicio" tab active. Same tab bar and range
control as the sleep screen.

Four stat tiles: SESIONES 34, MINUTOS TOTALES 1,530, PROMEDIO SEMANAL
382 min, DÍAS SIN ENTRENAR 6.

Main card: a stacked column chart of minutes per week, 320px tall. Exactly two
series with a 2px surface-colored gap between the stacked segments: "Bajo
techo" in slate blue #3a6ea5 and "Al aire libre" in terracotta #c05a30. A
legend at the top-left with two swatches. A dashed reference line at the
weekly goal.

Below, two half-width cards:
- "CALORÍAS: INGERIDAS VS QUEMADAS" — a grouped bar chart over 14 days, two
  series, slate blue and terracotta, legend present, with a muted caption
  "Solo se grafican los días con ambos datos registrados."
- "TIPOS DE ENTRENAMIENTO" — a horizontal bar chart, categories on the left in
  14px text with values right-aligned. Single sequential slate blue ramp, the
  largest bar darkest. No pie chart.
```

### P9 — Estadísticas · Glucosa

```
[PREÁMBULO]

Screen: "Estadísticas" with the "Glucosa" tab active. This screen must read as
clinical and trustworthy.

A discreet note strip at the top on sunken background with an info icon:
"Esta app registra y grafica. No diagnostica ni sugiere tratamiento."

Four stat tiles: PROMEDIO EN AYUNAS 96 mg/dL (green dot), PROMEDIO
POST-COMIDA 138 mg/dL (amber dot), LECTURAS 42, FUERA DE RANGO 5.

Main card, 340px tall: a time-series scatter plot. X axis is dates over 30
days, Y axis mg/dL from 60 to 220. Horizontal reference BANDS painted as very
pale full-width fills behind the dots: a green-tinted band 70–99, an
amber-tinted band 100–125, a red-tinted band above 126 — all extremely
desaturated, barely tinted greys with a hint of hue, with their labels in muted
text at the right edge. Dots are 9px, colored by context using the three
categorical colors: slate blue "En ayunas", terracotta "Post-comida 2h",
green-slate "Random". Legend at the top-left. One dot shown in a hover state
with a tooltip card: date, time, value, context, and the linked meal.

Below, a dense table of readings: Fecha, Hora, Valor (tabular, right-aligned),
Contexto (an outlined chip), Comida vinculada, Notas. Out-of-range values have
a small colored dot before the number, never a colored row background.

Top-right of the screen: an outlined button "Exportar PDF para el doctor" with
a document icon.
```

### P10 — Estadísticas · Peso y trabajo

```
[PREÁMBULO]

Screen: "Estadísticas" with the "Peso y trabajo" tab active.

Top half — weight:
Three stat tiles: PESO ACTUAL 78.4 kg, CAMBIO EN 75 DÍAS -3.2 kg (green),
PROMEDIO SEMANAL -0.3 kg.
A full-width line chart card 280px tall: raw daily weight points shown as tiny
6px grey #c6c5bf dots with NO connecting line, and over them a smooth 2px
slate blue line for the 7-day moving average. Legend with two entries. A muted
caption below the chart: "El peso diario oscila por agua y sal. La línea azul
es lo que realmente importa."

Bottom half — work:
Two stat tiles: HORAS ESTA SEMANA 31.5, PROMEDIO DIARIO 4.5 h.
A stacked column chart of hours per day over 30 days, exactly three categories
using the three categorical colors with 2px gaps between segments: Universidad,
Proyectos, Trabajo. Legend at top-left. A dashed reference line at the daily goal.
```

### P11 — Ajustes

```
[PREÁMBULO]

Screen: "Ajustes", full app shell.

A 220px secondary nav column inside the content area listing sections: Reto ·
Recordatorios · Coach · Datos · Apariencia · Sistema. "Recordatorios" is
active, marked with a sunken background.

Right side, the Recordatorios panel: a list of reminder rows separated by
hairlines. Each row has a toggle switch on the left (small, rectangular with
6px radius — not a pill), the reminder name in 14px, a muted description
below it, and a time input on the right in tabular numerals.
Rows: Check-in matutino 07:00 · Recordatorio de comida 12:30 · Recordatorio de
ejercicio 17:00 · Agua cada 2 horas · Check-in nocturno 21:30.

Below the list, a subsection "TONO DEL COACH" with three large radio cards
side by side: Suave, Directo, Duro. Each card contains the name and a real
example message in italic muted text. "Directo" is selected with a 2px slate
blue border and a small check in its corner.

Bottom: a "Zona de riesgo" section on sunken background with hairline red-bordered
outlined buttons: "Exportar todos mis datos", "Reiniciar el reto".
```

### P12 — Nuevo reto (onboarding)

```
[PREÁMBULO]

Screen: a full-window setup wizard, no sidebar. Centered 640px column.

Top: a 3-step progress indicator — three thin 4px bars, the first filled slate
blue, the rest #dcdbd7 — with the labels "Fechas · Reglas · Metas" below them.

Step 2 "Reglas" is shown. Title: "¿Qué cuenta como día completo?" at 18px/600,
with a muted subtitle: "Si fallas un pilar marcado como obligatorio, la racha
se reinicia. Los opcionales solo suman estadísticas."

Body: a list of 8 pillar rows, each with a checkbox on the left, the pillar
name, a muted description, and a segmented "Obligatorio / Opcional" control
on the right. Pillars: Dormir 7h, Ejercicio 45 min, Segundo ejercicio al aire
libre, Dieta limpia, 3 L de agua, Leer 10 páginas, Foto de progreso, Registrar
glucosa.

Footer: hairline divider, a ghost "Atrás" on the left, a primary slate blue
"Continuar" on the right.
```

### P13 — Días sin registrar

```
[PREÁMBULO]

Screen: a recovery modal, 600px wide, shown on app launch.

Header with a 24px amber clock icon: "Tienes 3 días sin registrar" at 18px/600,
and muted below: "Del 15 al 17 de agosto. Decide qué hacer con cada uno."

Body: three rows on alternating surfaces, each with the date and weekday on
the left and a three-option segmented control on the right: "Llenar ahora ·
Marcar fallido · Dejar vacío". One row has "Llenar ahora" selected.

Below, a muted note: "Los días vacíos no cuentan para tu racha ni la rompen."

Footer: ghost "Decidir después" and primary slate blue "Aplicar".
```

### P14 — Racha rota

```
[PREÁMBULO]

Screen: full-window state screen, no sidebar, centered 520px column, generous
whitespace, sober and calm — NOT dramatic.

A 40px red x-circle outline icon at the top, then "Se rompió la racha en el
día 31" at 32px/600, then muted body text: "Fallaste el pilar de ejercicio el
18 de agosto."

Below, a card on sunken background titled "LO QUE CONSTRUISTE EN 31 DÍAS"
containing four inline figures with labels: 28 entrenamientos · 7h 12min de
sueño promedio · -2.8 kg · 214h de trabajo. Under it, a muted line: "Nada de
esto se borra. Tus gráficas siguen siendo continuas."

Footer: a ghost "Ver el historial completo" and a primary slate blue
"Empezar intento #2".
```

### P15 — Reto completado

```
[PREÁMBULO]

Screen: full-window completion summary, no sidebar. Still restrained — a
formal report, not a celebration page. No confetti, no illustrations.

Top: "75 DE 75" at 56px/600 with a green check icon beside it, and muted
below: "Del 5 de junio al 19 de agosto de 2026".

A six-tile grid of before/after figures. Each tile: uppercase muted label, the
"antes → después" pair with the arrow in muted grey, and the delta in green or
red below. Tiles: Peso, Sueño promedio, Glucosa en ayunas, Minutos de
ejercicio semanales, Horas de trabajo, Energía promedio.

Below, the full 75-cell heatmap, same style as the Historial screen.

Below that, three small sparkline cards side by side titled Peso, Sueño,
Glucosa — each a thin 2px slate blue line with no axes and just the start and
end values labeled.

Footer: an outlined "Exportar informe PDF" and a primary slate blue
"Empezar un reto nuevo".
```

---

## 6. Notas prácticas sobre Stitch

- **Diseña primero P1, P5 y P7.** Esas tres fijan el lenguaje visual (dashboard, tabla densa, gráfica). El resto sale más consistente si las haces después.
- **El código que exporta Stitch es referencia visual, no código para pegar.** Sale con Tailwind y estructura de web; nosotros vamos con Svelte y CSS puro. Úsalo para sacar espaciados, jerarquía y colores exactos, no para copiar clases.
- **Stitch tiende a "webificar" todo** — te va a meter secciones tipo landing, botones redondeados grandes y sombras. Por eso el preámbulo dice explícitamente *no shadows*, *never pills*, *not a website*. Si aun así lo hace, repite la instrucción en el turno siguiente: "remove all shadows, this is a native desktop app".
- **Las gráficas que dibuje Stitch son decorativas.** Sirven para ver la composición; los datos reales los renderiza Chart.js con esta misma paleta.
- Exporta a Figma para poder medir espaciados exactos antes de traducir a CSS.
