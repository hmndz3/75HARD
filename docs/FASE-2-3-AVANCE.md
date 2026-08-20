# Fases 2 y 3 — las gráficas y los lujos

Estado: **completas**. Con esto el mapa de las 15 pantallas del spec queda
cerrado. Nada commiteado: los cambios están en el working tree.

---

## Fase 2 — Las gráficas

### P7 · Sueño

Barras por noche en azul cuando cumplen la meta y en gris cuando no, con la
línea de promedio móvil de 7 días encima y una línea punteada de referencia en
la meta. Debajo, histograma de la hora de dormir con el cubo modal destacado, y
dispersión de sueño contra energía con su coeficiente.

Cuatro tarjetas: promedio, mejor noche, peor noche y balance acumulado. El
balance dice "déficit" o "superávit" según el signo, en vez de forzar siempre
la palabra negativa.

### P8 · Ejercicio

Columnas apiladas de minutos por semana con dos series —bajo techo y al aire
libre—, separadas por 2 px del color del panel, y línea de meta semanal.
Debajo, calorías ingeridas contra quemadas y minutos por tipo de entrenamiento
en barras horizontales con rampa secuencial.

La gráfica de calorías **solo usa los días con ambos datos**. Si no hay
ninguno, en vez de una gráfica vacía sale una frase explicando que las calorías
son opcionales y que aparecerá sola cuando haya días completos.

### P9 · Glucosa

Dispersión temporal con las bandas de referencia pintadas detrás, apenas
teñidas, y los puntos coloreados por contexto. Tooltip con fecha, hora, valor,
contexto y la comida vinculada. Debajo, promedio por contexto y la tabla
completa de lecturas, donde lo que está fuera de rango lleva un punto rojo
delante del número y nunca un fondo de fila.

Arriba del todo, la franja que no se negocia: *la app registra y grafica, no
diagnostica ni sugiere tratamiento*.

### P10 · Peso y trabajo

El peso diario va como puntitos grises **sin línea que los una**, y encima la
línea azul del promedio móvil de 7 días. Es la decisión 2 del §6 del spec y
está implementada literalmente: el dato crudo oscila ±1.5 kg por agua y sal, y
verlo solo te hace creer que fallaste un día en que no fallaste nada.

Abajo, horas de trabajo por día apiladas por categoría. Si hay más de tres
categorías, las menores se agrupan en "Otro" en lugar de inventar un cuarto
color.

### Correlaciones

Quinta pestaña. Una dispersión por cada par —sueño↔energía, sueño↔glucosa en
ayunas y ejercicio↔ánimo— con línea de tendencia, la r traducida a lenguaje
humano ("relación moderada y positiva") y el número de días con ambos datos.

Las tres se muestran siempre. **Con menos de cinco pares no se calcula la r**
—sobre cuatro puntos sería ruido con decimales— pero la dispersión sí se pinta
y la tarjeta dice cuántos días llevas.

### Export

CSV y JSON, historial completo sin recortes, con diálogo nativo para elegir el
destino. Está en la cabecera de Estadísticas y también en Ajustes → Datos.

---

## Fase 3 — Los lujos

### Informe PDF para el médico

Pantalla propia, imprimible, con rango de 30, 60 o 90 días. Lleva el descargo
por delante, el resumen de lecturas, el promedio por contexto, el contexto
general del paciente (peso, sueño, comidas, ejercicio) y la tabla completa de
lecturas con asterisco en las que quedan fuera de rango.

El PDF sale del diálogo de impresión de Windows con "Guardar como PDF": es un
PDF real con texto seleccionable y no arrastra ninguna librería. La hoja de
estilos de impresión quita la barra, los fondos y los bordes redondeados.

### Fotos de progreso

Pantalla nueva en la barra lateral. Al agregar una foto **se copia a la carpeta
de datos de la app**: si luego borras el original de tus descargas, la del reto
sigue ahí. Comparador de antes y después eligiendo cualquier par de la lista.

Las imágenes se leen como data URL en vez de exponer el protocolo de assets.
Son dos fotos a la vez, y así no hay que abrir un canal de acceso al sistema de
archivos solo para esto.

### Copias de seguridad

Dos cosas distintas, a propósito:

| | Cifrado | Para qué |
|---|---|---|
| Instantánea local | No | Vive junto a la base, misma máquina, misma protección. Se conservan las últimas 7. |
| Copia manual `.75bak` | **Sí** | Para sacar los datos de la máquina. Ahí el cifrado sí protege algo. |

Cifrar las instantáneas locales con una clave guardada al lado no añadiría
nada: quien pueda leer una puede leer la otra. Por eso solo se cifra lo que
sale de la máquina, y la frase la escribes cada vez y no se guarda en ningún
sitio.

Formato: `"75HARDBK" | versión | salt(16) | nonce(12) | AES-256-GCM(base)`, con
la clave derivada por Argon2id. La instantánea se toma con `VACUUM INTO`, no
copiando el archivo: con WAL activo una copia cruda tendría escrituras a
medias.

Al restaurar se valida que lo descifrado sea de verdad una base de 75 HARD
**antes** de tocar nada, y se guarda una instantánea de lo actual por si te
arrepientes.

### Tema claro y oscuro

Tres opciones: claro, oscuro y seguir a Windows. Los tokens de ambos ya
existían en `app.css` desde la Fase 0; faltaba el interruptor. Las gráficas
leen los tokens del CSS y se repintan al cambiar de tema, así que el color
sigue significando lo mismo en los dos modos.

### P15 · Reto completado

Cifra grande de días completos sobre el objetivo, seis tarjetas de antes y
después con el delta coloreado según la dirección que ese pilar considera
buena, el heatmap completo y tres minigráficas de peso, sueño y glucosa.

El "antes" y el "después" **comparan el primer tercio del reto contra el
último**, no el primer día contra el último: así un mal día suelto no decide el
resumen entero.

---

## Verificación

| | Resultado |
|---|---|
| `cargo test --release` | **62 pasan**, 0 fallan (eran 44) |
| `cargo clippy --all-targets` | **0 warnings** |
| `npm run check` | **0 errores, 0 warnings** |
| Las 4 pantallas de gráficas | verificadas en pantalla con datos de ejemplo |
| Tema oscuro | verificado, incluidas las gráficas repintadas |
| Informe del médico | verificado en pantalla |
| Copia cifrada | **ida y vuelta real en test**: se cifra, se cambia un dato, se restaura y vuelve el valor original |
| Fotos | copia, listado, lectura y borrado cubiertos por tests |

**RAM**, el requisito no negociable:

| Estado | Working set | Comprometido |
|---|---|---|
| Ventana abierta | 27.9 MB | 5.7 MB |
| **Cerrada, en bandeja** | **1.7 MB** | 5.7 MB |
| Tras un tick del planificador | 3.4 MB | 5.6 MB |

Binario de 4.9 MB. Sigue muy por debajo del límite de 15 MB pese a Chart.js,
las notificaciones, el atajo global, el cifrado y el planificador.

---

## Correcciones posteriores

Dos cosas que salieron al usar la app de verdad, ya arregladas:

### No se podía registrar ejercicio

`bind:value` sobre un `<input type="number">` **no guarda texto**: guarda un
`number`, o `null` si vacías el campo. El helper que leía las calorías hacía
`.trim()` como si fuera un string, y reventaba con *"v.trim is not a function"*
en cuanto escribías las calorías quemadas.

En comida no se notaba porque el campo de calorías está escondido detrás del
`+`, así que la variable seguía siendo el `""` inicial.

El arreglo es un único helper compartido, `numeroOpcional()` en
`src/lib/format.ts`, que acepta las tres formas. Se aplicó también donde iba a
pasar lo mismo sin que nadie lo hubiera notado todavía: **el peso del check-in
matutino** y los campos en línea del cierre nocturno.

> Si añades un campo numérico, léelo con `numeroOpcional()`. No con `.trim()`.

### Las correlaciones eran solo texto

Tenían razón en que así no servían: un número y una barra no dicen nada. Ahora
cada correlación es una **dispersión de verdad** con sus puntos, línea de
tendencia por mínimos cuadrados, ejes etiquetados y la r traducida a lenguaje
humano.

Y se devuelven **siempre las tres**, aunque no haya datos suficientes para
calcular la r. Antes se ocultaba la que no llegaba al mínimo de cinco pares, y
la pantalla se quedaba con un mensaje genérico que no distinguía entre "te
falta registrar" y "la app está rota". Ahora cada tarjeta dice exactamente
cuántos días con ambos datos lleva.

### Y una que resultó no ser un bug

Perseguí un rato un supuesto problema de zoom: las capturas salían con el
contenido 1.5× y recortado. No era la app. El script de capturas era
DPI-unaware y la pantalla está al 150%, así que Windows le mentía sobre el
tamaño de la ventana y `PrintWindow` recortaba la esquina superior izquierda a
resolución nativa. Llegué a tocar el zoom del WebView "para arreglarlo" y lo
revertí en cuanto lo medí. La app nunca estuvo mal.

---

## Decisiones que conviene conocer

- **Chart.js, como decía el stack.** Los genéricos de sus opciones son
  invariantes por tipo de gráfica y no dejan componer una base común con
  spread; se relajan en un único punto (`src/lib/charts.ts`) en vez de duplicar
  la configuración en cada pantalla.
- **`$state.raw` en las pantallas de estadísticas.** Chart.js redefine
  propiedades sobre los objetos que recibe y un proxy reactivo de Svelte 5 lo
  rechaza con `state_descriptors_fixed`. Costó un rato encontrarlo: la pantalla
  salía en blanco. Si añades una gráfica nueva, sus datos van en `$state.raw`.
- **La línea de meta y las bandas de glucosa son plugins de ~20 líneas**, no el
  plugin de anotaciones de Chart.js. No compensaba una dependencia entera.
- **Base64 a mano** para las fotos, por no añadir una crate por 25 líneas.
  Tiene test contra los vectores de referencia del RFC.
- **`work_goal_min`** no existía como ajuste; se usa 240 por defecto. Si algún
  día quieres meta de trabajo, ya está leída desde ajustes.

---

## Lo que sigue sin hacer

- **Sincronización con Railway.** El spec la deja como idea futura y el
  esquema ya está preparado (UUID y `updated_at` en todas las tablas), pero no
  hay nada implementado.
- **Backup automático a un servidor.** La copia cifrada es manual, a propósito:
  requiere que escribas la frase.
- **Acciones al hacer clic en una notificación.** En Tauri v2 pide registrar
  tipos de acción y un canal aparte; para lo que aporta no compensa.

---

## Commit sugerido

```bash
git add -A && git commit -m "Fases 2 y 3: graficas, export, informe medico, fotos, backup cifrado y tema oscuro"
```
