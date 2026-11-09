# Réplica exacta de ByTune en ByTune-rs — índice y plan

Este conjunto de documentos describe **todo** ByTune (la app Electron,
`github.com/Jscruzgg/ByTune`, v1.0.23): interfaz (CSS), ventanas, funciones, interacciones,
animaciones y textos. Sirve para que otra sesión o agente complete ByTune-rs hasta que sea una
réplica exacta, **sin abrir el código original**: cada valor está copiado con la línea de origen
(`theme.css:123`) por si hace falta comprobarlo.

> ByTune no dibuja su propia interfaz: carga music.youtube.com y la retoca con CSS y JS. Los
> documentos describen el **resultado visual** (medidas, colores, tipografía, estados) además
> del selector, para poder dibujarlo en egui sin conocer el DOM de YouTube Music.

## Documentos

| # | Archivo | Qué cubre | Origen en ByTune |
|---|---|---|---|
| 02 | [02-tema-parte1.md](02-tema-parte1.md) | Variables, barra lateral (guía), barra superior y buscador, barra del reproductor, chips, estanterías, tarjetas, filas, menús, scrollbars; responsive 615–936 px | `theme.css` 1–950 |
| 03 | [03-tema-parte2.md](03-tema-parte2.md) | Píldora Me gusta/No me gusta, Now Playing (fondo, carátula, selector canción/vídeo), cola Material 3 (pestañas, filas, autoplay), álbum/playlist estilo Spotify, `content-visibility` | `theme.css` 948–1888 |
| 04 | [04-chrome-letras-nowplaying.md](04-chrome-letras-nowplaying.md) | **Paleta `--md-*`** (referencia de color de toda la app), barra superior arrastrable, menú de ajustes, letras sincronizadas, Now Playing, pantalla completa | `chrome.css`, `lyrics.css`, `lyrics-view.js`, `lib/lrc.js`, `lib/lyrics-timing.js` |
| 05 | [05-ventanas-funciones-menu-rapido.md](05-ventanas-funciones-menu-rapido.md) | Ventanas, `settings.json`, bandeja, thumbar, atajos, notificaciones, inicio con Windows, actualizaciones, **23 canales IPC**, **menú rápido completo**, **61 textos es/en**, lógica de letras y biblioteca, diferencias spec↔código | `main.js`, `quickmenu/*`, `lib/*`, `package.json`, `release.yml` |
| 06 | [06-pagina-interacciones-cola.md](06-pagina-interacciones-cola.md) | Bucle de la página, menú de ajustes (nodo a nodo), grabación de atajo, redimensionado de paneles, **plegado FLIP de la barra lateral**, cola de Now Playing, **arrastrar y soltar**, API de YouTube Music, portadas | `preload.js`, `page-api.js`, `covers.js`, `queue-reorder.js` |

Cada documento termina con una **checklist de réplica** numerada (en total ~430 puntos) y una
lista de dudas donde el código no deja claro un valor. **Esa checklist es la definición de
"terminado".**

## Sistema de diseño en una página

Todo esto está detallado (con origen) en los documentos; aquí, lo imprescindible para no perderse.

- **Paleta** (doc 04 §A): una sola escala. Texto y estados = blanco con opacidad:
  `#fff` / `.7` / `.56` / `.38`; estados hover `.08`, seleccionado `.14`, pulsado `.2`.
  Fondo `#000`; superficies "vidrio" `rgba(12,12,12,.6)`; barra opaca `#0e0e0e`; elevado
  `#1e1e1e` + borde `rgba(255,255,255,.1)` + sombra `0 16px 40px rgba(0,0,0,.5)`.
  **No hay color de acento: el blanco hace de acento** (botón Reproducir blanco con icono
  negro, chip seleccionado blanco con texto negro). ByTune-rs usa hoy rojo `#DF1C43` como
  acento en textos activos: **quitarlo**; el rojo solo está en el icono de la app.
- **Movimiento**: `--md-dur .35s`, `--md-ease` / `--m3-ease` = `cubic-bezier(.2,0,0,1)`;
  letras y título `cubic-bezier(.22,.8,.3,1)` 600/400 ms; plegado de la barra lateral con
  muelle `spring(0.28, 0.08)` (respaldo 350 ms). Respeta "reducir movimiento".
- **Tipografía**: `--m3-font: Roboto, "Segoe UI Variable Text", system-ui`;
  títulos `--md-font-display: "YouTube Sans", Roboto`. Para la réplica: incluir **Roboto**
  (Apache-2.0) en el ejecutable; "YouTube Sans" no tiene licencia libre → usar Roboto a peso
  equivalente y anotarlo.
- **Radios**: estándar 10 px (`--md-radius`), separación entre paneles 8 px (`--md-gap`),
  carátulas 8 (tarjetas) / 4 (filas, barra lateral) / 12 (Now Playing), píldoras = mitad del alto.

## Esqueleto de la ventana (≥ 940 px de ancho)

```
┌──────────────────────── barra superior 64 px (arrastrable) ───────────────────────┐
│ [≡] logo   ← →                  [ buscador centrado clamp(160,·,480) ]  ⚙  ─ □ ✕ │
├──────────┬──────────────────────────────────────────────────┬───────────────────┤
│ barra    │                                                  │ cola (tarjeta)    │
│ lateral  │            contenido de la página                │ top 72, bottom 100│
│ tarjeta  │                                                  │ right 8, r 10     │
│ x8 y72   │                                                  │ ancho --md-right-w│
│ 240–420  │                                                  │ o clamp(340,28vw, │
│ (72 pleg)│                                                  │ 440)              │
├──────────┴──────────────────────────────────────────────────┴───────────────────┤
│ barra del reproductor 92 px: 1fr | clamp(260px,38vw,600px) | 1fr, gap 24        │
│ [carátula título/artista]  [⇄ ⏮ (▶44) ⏭ ↻ ⋮ / ─progreso 4px─]  [👍|👎 🔊 …]     │
└──────────────────────────────────────────────────────────────────────────────────┘
```

Valores exactos y comportamiento por debajo de 940 / 936 / 820 / 760 / 680 / 615 px: doc 02 §5
y doc 03 §5. (Los iconos exactos de la barra, como aleatorio y repetir, se comprueban en doc 02
§"barra del reproductor".)

## Cómo traducir CSS a egui (convenciones para la réplica)

| En ByTune | En ByTune-rs |
|---|---|
| Colores con opacidad sobre negro | `Color32::from_white_alpha(..)` / `from_black_alpha(..)` sobre el fondo real; no aplanar a gris si hay algo detrás (fondo borroso de Now Playing) |
| `transition: X .35s cubic-bezier(.2,0,0,1)` | `ctx.animate_value_with_time(id, objetivo, 0.35)` + función de easing propia (cubic-bezier) en `theme.rs`; egui interpola lineal → aplicar la curva al progreso |
| Muelle de Motion (barra lateral) | integrador de muelle simple (rigidez/amortiguación que den ~0,28 s) o respaldo 350 ms con la curva M3 |
| `Element.animate`, `startViewTransition` ("morph" 350 ms) | animar posición/escala/opacidad de los elementos que cambian; fundido cruzado del contenido |
| `backdrop`/`blur()` del fondo de Now Playing | carátula reducida a 64×64 + desenfoque en CPU (una vez por canción) + escalar; luego degradado negro `.15→.55` |
| `clamp()`, `min()`, `vw/vh` | calcular con `ctx.content_rect()` en cada fotograma |
| `content-visibility` | `ScrollArea::show_rows` / no pintar fuera de la vista |
| `::before` (tarjeta de la barra lateral) | `painter().rect` con radio, relleno y borde antes del contenido |
| Ventana sin marco + botones nativos superpuestos | `ViewportBuilder::with_decorations(false)` + barra propia de 64 px con `StartDrag`, y botones min/max/cerrar dibujados (o `with_titlebar_shown(false)`+`with_fullsize_content_view`) |

## Estado de ByTune-rs frente a la réplica (v1, commit 9d0c054)

Leyenda: ✅ hecho y parecido · 🟡 hecho pero distinto (ajustar a la doc) · ❌ falta.

### Ventana y estructura
- 🟡 Ventana con marco nativo; ByTune no tiene marco y su barra superior de 64 px es arrastrable con botones superpuestos (doc 04 §A, doc 05 §A). Mínimo ByTune 480×400; ByTune-rs 860×560.
- 🟡 Barra superior: hoy atrás/adelante + buscador dentro del contenido; debe ocupar todo el ancho, con atrás en x=64 y adelante en x=104 (36×36), buscador centrado en la ventana con las fórmulas de doc 02 y engranaje de ajustes (doc 04 §A).
- ❌ Sugerencias del buscador (tarjeta 8 px por debajo, radio 16, `#1e1e1e`) (doc 02).
- ❌ Instancia única (ByTune: `requestSingleInstanceLock`, al abrir otra se enfoca la existente) (doc 05 §F).
- 🟡 Zoom, F5, Alt+←/→: hechos; comprobar teclas exactas en doc 05 §E.

### Barra lateral
- 🟡 Existe con nav + playlists con portada; debe ser una **tarjeta flotante** (x=8, y=72, radio 10, `rgba(12,12,12,.6)`, borde `.08`), iconos y portadas **40×40 r4 centrados en x=40**, anchos 240–420 con tope `max(240, ventana/2−164)` (doc 02, doc 06 §A).
- ❌ Plegar a 72 px con animación FLIP de muelle; textos a opacidad 0; "Nueva lista" → círculo 36 px (doc 02, doc 06).
- ❌ Redimensionar con teclado (←/→ 16 px), doble clic restablece, guardar ancho (doc 06 §A).
- ❌ Entradas de la guía de YTM tal cual (Inicio, Explorar, Biblioteca, "Nueva lista"); ByTune-rs añade "Reproduciendo" y "Ajustes" que en ByTune no están en la barra lateral (Ajustes va en el engranaje).

### Páginas
- 🟡 Inicio, Explorar, Biblioteca, búsqueda, álbum/artista/playlist: funcionan con datos reales; falta el estilo exacto:
  chips 36 px (seleccionado blanco/negro), tarjetas con hover de 8 px y radio 10, botón ▶ blanco que sube 8 px en 180 ms (doc 02);
  álbum/playlist **estilo Spotify** (contenedor máx. 1400, carátula `clamp(160px,18vw,232px)`, título `clamp(32px,5.5vw,88px)` peso 800 máx. 2 líneas, play 56 px que crece a 1.05, cabecera de lista "# Título ⏱", filas 56 px) (doc 03).
- 🟡 Filas de canción: radio 6, miniatura r4 (doc 02); hoy radio 8 y 48 px.
- ❌ Crear playlist ("Nueva lista").

### Barra del reproductor
- 🟡 Existe (96 px). Debe medir **92 px**, rejilla `1fr | clamp(260px,38vw,600px) | 1fr` gap 24; centro en 2 filas (controles `1fr auto 1fr` con "⋮" al final; progreso debajo); botones de icono 40 px (hover .08, pulsado .14); Reproducir círculo blanco 44 px con 16 px a cada lado; progreso pista 4 px con 50 px de margen, tiempos Roboto 12 tabular a 6 px; **asa vertical 4×16 que crece a 20 en 150 ms**; foco 2 px blanco con 2 px de separación (doc 02).
- ❌ Píldora **Me gusta | No me gusta** 105×40 r20 a 8 px a la izquierda del volumen (hoy solo corazón) (doc 03).
- ❌ Menú "⋮" de la canción, aleatorio y repetir (comprobar en doc 02 qué botones muestra la barra).
- 🟡 Barra de vidrio `rgba(12,12,12,.72)` en Now Playing, opaca `#0e0e0e` en el resto.

### Cola
- 🟡 Existe con arrastrar, quitar, vaciar. En ByTune (≥ 940 px) es una **tarjeta** (top 72, bottom 100, right 8, r10) con **pestañas Material 3 de 48 px** (indicador blanco 3 px), filtros fijos arriba, filas de 64 px con miniatura 40 r8 (radio de fila 12), **interruptor de reproducción automática 52×32** animado (doc 03).
- 🟡 Arrastrar: ByTune empieza tras 6 px, mueve la fila con transform y aparta las demás una altura; **no** dibuja línea roja (hoy sí) (doc 06 §C).
- ❌ Al cambiar de canción, a los 150 ms la actual se desplaza arriba (8 px bajo los filtros) con el "lock" descrito en doc 06 §A.

### Now Playing y letras
- ❌ **Fondo**: carátula 64×64 `blur(4px) saturate(1.6) brightness(.55)` escalada ×40 + degradado `.15→.55` (doc 03).
- 🟡 Carátula: radio 12 y sombra `0 24px 60px rgba(0,0,0,.55)`; tamaño `min(alto−430, ancho−guía−cola−96)` (doc 03).
- 🟡 Letras: hoy cambian de color y se centran. Deben ir **siempre blancas**, jerarquía solo por **opacidad** (futura .56, pasada .38, activa 1 con **escala 1.04**), 30/40 px negrita (24/32 estrecho), ancladas al **30 %** del alto con **600 ms `cubic-bezier(.22,.8,.3,1)`**; rueda manual que vuelve a la línea a los **4 s**; temporizador a la siguiente línea +15 ms (doc 04 §B).
- ❌ Disposición estrecha (miniatura 88 px, letra desde y=144) y secuencia escalonada al abrir/cerrar (0,2 s) (doc 04 §B).
- ❌ Estados: tres barras de carga (70/52/62 %), pedir a la fuerza a los 6 s, "sin letra" a los 15 s, créditos/fuente (doc 04 §B).
- 🟡 Botones redondos de 44 px **abajo a la derecha a 24 px**, x = 24/80/136 (pantalla completa, letras, canción/vídeo) con sus SVG exactos (doc 04); hoy están bajo el título.
- 🟡 Selector canción/vídeo: píldora 44 px (doc 03) con `visibility:hidden` si no hay vídeo; probar en la app.
- ❌ Animación del título al cambiar de canción (400 ms, sube 8 px) y "morph" 350 ms al entrar/salir de pantalla completa o cambiar canción↔vídeo (doc 04 §C).
- ❌ Pantalla completa: barras que aparecen a **150 px** de los bordes (cola solo a 16 px del derecho), ocultar todo (cursor incluido) a los **3 s** sin ratón, carátula `min(alto pantalla−448, ancho−96)`, contenido que se encoge hacia el hueco libre (doc 04 §B).

### Menú rápido
- 🟡 Existe y funciona. Ajustar a doc 05 §I: ventana **588×700** transparente con panel de **540 px a 60 px del borde superior**, centrada en el área de trabajo; apertura 0,12 s ease-out; **Intro reproduce y el menú sigue abierto**; Mayús+Intro muestra una **píldora 2,2 s**; flechas **circulares**; **F1**; clic fuera cierra; debounce **200 ms**, caché 100 consultas / 10 min, `seq`; secciones **Recientes (6) + Sugerencias** (FEmusic_home) sin texto y **Tu biblioteca (5, playlists+álbumes, sin acentos, primero prefijo) + Canciones (5)** con texto; ajuste "Desde otras apps: flotante".

### Ajustes
- 🟡 Hoy es una página. En ByTune es un **panel de 320 px** bajo el engranaje (radio 14, entra en 120 ms), alineado a la derecha 8 px por debajo; se cierra con clic fuera, resize, blur, Esc o el engranaje (doc 04 §A, doc 06 §A). Contenido exacto: cabecera plegable "Menú rápido" con atajo y chevron, fila "Atajo" con grabación, 3 grupos de radios ("Desde otras apps" atenuado salvo "En cualquier lugar"), **4 interruptores** (tema, notificaciones, cerrar a la bandeja, iniciar con Windows), pie con versión y botón "Buscar actualizaciones" con sus estados.
- 🟡 Idioma: ByTune lo toma de la página de YouTube Music (IPC `lang`), no tiene selector. Autoplay es el interruptor de la cola, no un ajuste.
- 🟡 Grabar atajo: errores `invalid` / `reserved` (3 reservados) / `taken`, traducción por código físico (tabla en doc 05 §E).

### Windows
- 🟡 Bandeja: comprobar orden y textos exactos (doc 05 §C).
- ❌ Botones de la vista previa de la barra de tareas (anterior, play/pausa, siguiente) con los iconos de `make-icon.mjs` (doc 05 §D).
- 🟡 Inicio con Windows: ByTune usa el argumento `--hidden` y repara la ruta al abrir (doc 05 §B/§F).
- 🟡 Notificaciones: comprobar condición y contenido (doc 05 §F).
- ❌ Actualizaciones desde GitHub Releases con aviso 340×110 "Actualizando ByTune a la versión X…" y `updater.log` (doc 05 §G) — requiere publicar releases de ByTune-rs.

## Orden de trabajo recomendado

1. **Tokens** en `theme.rs`: paleta exacta (doc 04 §A), radios, curvas de easing y duraciones; Roboto incluida. Quitar el rojo como acento.
2. **Ventana sin marco + barra superior** de 64 px (arrastre, botones, atrás/adelante, buscador centrado, engranaje).
3. **Barra lateral** tarjeta + plegado + redimensionado (docs 02, 06).
4. **Barra del reproductor** exacta (doc 02) + píldora me gusta/no me gusta (doc 03).
5. **Páginas**: chips, tarjetas, filas, álbum/playlist Spotify (docs 02, 03).
6. **Cola** tarjeta + pestañas + autoplay + arrastre sin línea (docs 03, 06).
7. **Now Playing y letras** (fondo, disposición, animaciones, pantalla completa) (docs 03, 04).
8. **Ajustes como panel** (docs 04, 06) y **menú rápido** exacto (doc 05).
9. **Windows**: thumbar, instancia única, inicio con Windows, actualizaciones (doc 05).
10. Repasar las cinco checklists punto por punto.

## Cómo verificar

- **ByTune original** (referencia visual): en `Documents\ByTune`, `npm start` con
  `YTM_DEBUG_PORT=9222` y `node scripts/cdp.mjs shot out.png` (ver README de ByTune). Mismo
  tamaño de ventana que ByTune-rs para comparar capturas.
- **ByTune-rs**: `BYTUNE_DEBUG_PORT=9223` abre DevTools del WebView; la interfaz egui expone
  accesibilidad (AccessKit), así que se puede pulsar por nombre con UI Automation y hacer
  capturas (script de ejemplo usado en esta sesión: buscar elementos por `Name`, `Invoke()` y
  `CopyFromScreen`). `BYTUNE_AUTOPLAY="canción"` reproduce al arrancar.
- `cargo test` y `bytune --probe "consulta"` para la parte de datos.

## Dudas abiertas (recogidas de los documentos)

- Doc 02: el comentario anuncia un hueco alrededor del asa del progreso pero el código pone
  `box-shadow: none`; transición de `box-shadow` en Reproducir sin sombra definida; tamaños que
  pone YTM y el tema no fija (carátula de la barra, ▶ de tarjetas, alto del buscador).
- Doc 03: iconos del selector canción/vídeo, color de texto del chip seleccionado, tipografía de
  filas de álbum (la de YT).
- Doc 04: los archivos no definen fondo propio para Now Playing (lo pone theme.css, doc 03).
- Doc 05: si la animación de apertura del menú rápido llega a verse; Tab sin gestionar; posible
  parpadeo con `--hidden`.
- Doc 06: duración real del muelle de Motion (no está en el código); `aria-label` de los
  tiradores sin traducir.

Para resolverlas: abrir ByTune con `YTM_DEBUG_PORT` y medir con DevTools
(`node scripts/cdp.mjs eval "getComputedStyle(...)"`).
