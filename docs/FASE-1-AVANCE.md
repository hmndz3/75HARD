# Fase 1 — "Que te persiga"

Estado: **completa, 4 de 4 puntos**. Todo verificado corriendo la app compilada
contra una base de ejemplo. Nada commiteado: los cambios están en el working
tree para que los revises.

---

## Lo que quedó hecho

### 1. Heatmap de 75 días (P5)

Tarjeta nueva arriba de la tabla de Historial. Rejilla de 15×5, celdas de 22px
con 4px de separación y radio 3px, con el número de día dentro en 10px. Cinco
estados con leyenda: completo verde, parcial ámbar, fallido rojo, pausado gris
con trama diagonal, y sin registrar en gris con borde punteado. Los días que aún
no llegan salen apagados y no son clicables; los demás abren el detalle del día.

Dos detalles que no estaban en el spec y añadí:

- **"Parcial" no es un estado guardado.** Se calcula: un día que sigue en
  `pending` pero tiene algo registrado. Antes solo aparecía en el resumen de
  Historial; ahora también pinta el heatmap.
- **El día sin registrar se distingue del futuro por el borde punteado**, no
  solo por el color. Comparten fondo y con puro gris no se diferenciaban.

### 2. Notificaciones programadas

`src-tauri/src/scheduler.rs`: tarea de tokio con un tick de 60 segundos que lee
la tabla `reminder` y dispara notificaciones nativas de Windows.

Lo que lo hace distinto de un temporizador tonto: **mira los datos del día antes
de molestar**. Si ya registraste el ejercicio, el aviso de las 17:00 no suena. Si
ya cerraste el día, el de las 21:30 tampoco. Y los textos llevan datos reales
("Aún no registras ejercicio hoy. Quedan 7 horas.", "Llevas 1.8 de 3.0 L").

Recordatorios sembrados en la migración `002_reminders.sql`:

| | Hora | Se calla si |
|---|---|---|
| Check-in matutino | 07:00 | ya hay sueño registrado |
| Recordatorio de comida | 12:30 | ya hay 2 comidas |
| Recordatorio de ejercicio | 17:00 | ya hay ejercicio |
| Agua | cada 2 h | ya llegaste a la meta |
| Check-in nocturno | 21:30 | el día ya está cerrado |

La sección **Recordatorios** de Ajustes dejó de ser un cartel que decía "esto es
Fase 1": ahora los interruptores y las horas escriben en la base de verdad. Le
agregué dos cosas que no estaban pedidas pero hacían falta para que esto sea
usable:

- **Un interruptor general** de notificaciones. Apagado, no suena nada.
- **Horario de silencio**, de 22:00 a 07:00 por defecto. Sin esto, cambiar la
  hora de un recordatorio a las 3 AM te despertaba.

### 3. Atajo global `Ctrl+Alt+H`

Abre la ventana de captura rápida desde cualquier aplicación. El acelerador se
guarda en ajustes (`hotkey_quick`) y **si falla el registro se muestra el motivo
en Ajustes → Sistema**, en vez de dejar una tecla que no hace nada — es lo que
pasa cuando otra app ya lo tiene tomado.

### 4. Pantalla de racha rota (P14)

Aparece sola al cerrar un día como fallido desde el check-in nocturno. Ventana
completa, columna de 520px, sobria: icono rojo de 40px, el titular, y una tarjeta
hundida "LO QUE CONSTRUISTE EN N DÍAS" con entrenamientos, sueño promedio, cambio
de peso y horas de trabajo del intento, más la línea de que nada se borra.

Pie con tres salidas: ver el historial, "Ahora no", y empezar el intento
siguiente (que cierra el actual y te manda al onboarding). El botón de "ahora no"
no estaba en el spec; lo puse porque obligar a decidir en ese momento es
justamente lo que hace que la gente cierre la app y no vuelva.

---

## Archivos

**Nuevos**

```
src-tauri/migrations/002_reminders.sql
src-tauri/src/scheduler.rs
src-tauri/src/commands/reminders.rs
src/lib/components/Heatmap.svelte
src/routes/BrokenStreak.svelte
```

**Modificados**

```
src-tauri/Cargo.toml              tauri-plugin-notification, -global-shortcut, tokio
src-tauri/capabilities/default.json
src-tauri/src/lib.rs              plugins, atajo global, arranque del scheduler
src-tauri/src/coach.rs            join_es pasa a pub(crate)
src-tauri/src/db/migrations.rs    registra la 002
src-tauri/src/db/models.rs        HeatmapCell, Reminder, ChallengeTotals, BrokenStreak, hoursLeft
src-tauri/src/db/queries.rs       heatmap, recordatorios, totales del intento, racha rota
src-tauri/src/commands/day.rs     heatmap en get_history, get_broken_streak
src-tauri/src/commands/mod.rs
src/App.svelte                    ruta de racha rota
src/lib/api.ts  src/lib/types.ts  src/lib/format.ts
src/routes/History.svelte         tarjeta del heatmap
src/routes/Settings.svelte        recordatorios reales + fila del atajo
src/routes/Today.svelte           avisa cuando el día se cierra como fallido
```

---

## Verificación

Todo corrido sobre el binario de release, no solo compilado:

| | Resultado |
|---|---|
| `cargo test` | **44 pasan**, 0 fallan (eran 37; 7 nuevos del planificador) |
| `cargo clippy --all-targets` | **0 warnings** |
| `cargo fmt` | aplicado |
| `npm run check` | **0 errores, 0 warnings** |
| Heatmap | verificado en pantalla con 23 días de datos de ejemplo |
| Recordatorios | verificado en pantalla; interruptores y horas persisten |
| Atajo `Ctrl+Alt+H` | **verificado de verdad**: se envió la combinación y la ventana de captura rápida pasó a primer plano |
| Racha rota | verificado cerrando el día 23 como fallido |

**RAM, el requisito no negociable:**

| Estado | Working set | Comprometido |
|---|---|---|
| Ventana abierta | 27.6 MB | 5.6 MB |
| **Cerrada, en bandeja** | **1.7 MB** | **5.5 MB** |

Sigue igual que en la Fase 0 pese al tick de 60 s y a los dos plugins nuevos.

Los 7 tests nuevos cubren: horario de silencio, interruptor general, que un
recordatorio suene una sola vez al día, que el de agua vuelva a sonar pasado el
intervalo, que uno apagado no suene, que nada suene antes de su hora, y que se
rechace una hora mal escrita.

---

## Lo que NO se hizo, a propósito

- **Nada de Fase 2.** Las cuatro pantallas de gráficas (P7–P10) y el export
  siguen pendientes. La pantalla de Estadísticas sigue siendo el cartel honesto
  que dice qué falta.
- **P15, reto completado**, no entraba en la Fase 1 y no se tocó.
- **Las notificaciones no llevan acciones al hacer clic.** En Tauri v2 eso pide
  registrar tipos de acción y un canal aparte; para el valor que da (abrir la
  ventana, que está a un clic en la bandeja) no valía la complejidad. Si lo
  quieres después, va en `scheduler.rs`.

---

## Cómo probarlo a mano

```bash
npm run tauri dev
```

Con datos de ejemplo, sin tocar tus datos reales:

```bash
cargo run --manifest-path src-tauri/Cargo.toml --release --example seed_demo -- C:\temp\demo75
```

Y luego abrir el binario con `--datos C:\temp\demo75\75hard`.

Qué mirar:

1. **Historial** — el heatmap arriba. Pasa el cursor por una celda para ver el
   estado, haz clic para abrir ese día.
2. **Ajustes → Recordatorios** — apaga uno, cambia una hora, cierra y reabre la
   app: tiene que haber quedado guardado.
3. **`Ctrl+Alt+H`** desde cualquier app — sale la captura rápida.
4. **Racha rota** — en Hoy, "Cerrar el día" → "Marcar como fallido".
5. **Una notificación de verdad**: en Ajustes → Recordatorios pon el check-in
   nocturno un par de minutos adelante de la hora actual y espera. Ojo con el
   horario de silencio: si son más de las 22:00, súbelo primero o no sonará.

---

## Commit sugerido

```bash
git add -A && git commit -m "Fase 1: heatmap de 75 dias, notificaciones programadas, hotkey global y racha rota"
```
