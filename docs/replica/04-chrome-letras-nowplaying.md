# 04 — Chrome (paleta, barra superior, menú de ajustes), Letras y Now Playing

Fuentes leídas enteras: `src/chrome.css` (274 líneas), `src/lyrics.css` (560), `src/lyrics-view.js` (795),
`src/lib/lrc.js` (40), `src/lib/lyrics-timing.js` (33). Consultados puntualmente para la forma del IPC:
`src/preload.js:382-396`, `src/main.js:96-146` y `src/main.js:699-740`, `src/lib/i18n.js:50-58,124-132`,
`src/lib/lyrics-source.js` (líneas 38-80).

Convención: `archivo:línea`. "YT" = YouTube Music. Las medidas son px CSS (en egui, puntos lógicos).
Todo lo que en el original son atributos HTML (`data-md-*`) es, en la réplica, estado booleano/enumerado.

> **Motion (`vendor/motion.js`)**: `lyrics-view.js` **no usa Motion en absoluto** (no hay ninguna
> referencia a `Motion`, `animate` de Motion ni muelles). Las únicas animaciones JS de la vista son
> `Element.animate()` nativo (lyrics-view.js:439) y `document.startViewTransition` (lyrics-view.js:611).
> Motion lo inyecta main.js en el mundo aislado 999 del preload solo para "plegar la barra lateral"
> (main.js:131-140); no forma parte de este documento.

---

## A. chrome.css

Se inyecta **siempre**, con o sin tema (chrome.css:3-4, :32; main.js:108), con `cssOrigin: 'user'`
(main.js:103). Por ser hoja de origen *user*, los botones necesitan `appearance: none` o Chromium los pinta
como botón nativo gris claro (chrome.css:261).

### A.1 Paleta (`:root`, chrome.css:1-28) — referencia para toda la app

Intención del autor (chrome.css:1-5): **una sola escala**. Texto, iconos, estados y bordes = blanco con
opacidad (sobre cualquier carátula se tiñen igual y conservan jerarquía); superficies = negro con
opacidad. Contrastes medidos sobre fondos reales (negro, tarjetas, carátulas de color):
`--md-text-2 ≥ 6,5:1`, `--md-text-3 ≥ 4,7:1`.

| Variable | Valor exacto | RGBA (0-255, alfa 0-1) | Uso (comentario del autor) | Línea |
|---|---|---|---|---|
| `--md-bg` | `#000` | 0,0,0,1 | fondo | 7 |
| `--md-text` | `#fff` | 255,255,255,1 | títulos, activo, anillo de foco | 8 |
| `--md-text-2` | `rgba(255,255,255,.7)` | 255,255,255,0.70 | artistas, subtítulos, iconos en reposo | 9 |
| `--md-text-3` | `rgba(255,255,255,.56)` | 255,255,255,0.56 | notas, separadores con texto, duraciones | 10 |
| `--md-text-off` | `rgba(255,255,255,.38)` | 255,255,255,0.38 | desactivado | 11 |
| `--md-on-fill` | `#000` | 0,0,0,1 | texto e icono sobre un relleno blanco | 12 |
| `--md-glass` | `rgba(12,12,12,.6)` | 12,12,12,0.60 | tarjetas sobre la página: barra lateral, cola | 13 |
| `--md-glass-bar` | `rgba(12,12,12,.72)` | 12,12,12,0.72 | barra del reproductor en Now Playing (lleva controles) | 14 |
| `--md-bar` | `#0e0e0e` | 14,14,14,1 | la misma barra, opaca: en las páginas que pasan por debajo | 15 |
| `--md-raised` | `#1e1e1e` | 30,30,30,1 | lo que se abre encima: menús, volumen, sugerencias | 16 |
| `--md-raised-line` | `rgba(255,255,255,.1)` | 255,255,255,0.10 | borde de lo elevado | 17 |
| `--md-raised-shadow` | `0 16px 40px rgba(0,0,0,.5)` | sombra: offset y=16, blur 40, negro 0.5 | sombra de lo elevado | 18 |
| `--md-field` | `rgba(255,255,255,.08)` | 255,255,255,0.08 | campos (buscador) | 19 |
| `--md-state-hover` | `rgba(255,255,255,.08)` | 255,255,255,0.08 | hover | 20 |
| `--md-state-on` | `rgba(255,255,255,.14)` | 255,255,255,0.14 | seleccionado; fondo de los botones secundarios | 21 |
| `--md-state-press` | `rgba(255,255,255,.2)` | 255,255,255,0.20 | pulsado | 22 |
| `--md-state-ink` | `rgba(0,0,0,.1)` | 0,0,0,0.10 | hover sobre un relleno blanco | 23 |
| `--md-state-ink-press` | `rgba(0,0,0,.16)` | 0,0,0,0.16 | pulsado sobre un relleno blanco | 24 |
| `--md-line` | `rgba(255,255,255,.08)` | 255,255,255,0.08 | bordes de tarjeta, separadores | 25 |
| `--md-line-strong` | `rgba(255,255,255,.24)` | 255,255,255,0.24 | contornos que deben verse (radios, pista) | 26 |
| `--md-danger` | `#ff8b8b` | 255,139,139,1 | errores | 27 |

Variables globales adicionales definidas en lyrics.css (también de toda la app, lyrics.css:12-17):
`--md-dur: .35s` y `--md-ease: cubic-bezier(.2, 0, 0, 1)` — "duración y curva de todo lo que se mueve a la
vez (paneles, carátula, barra lateral): con las mismas, nada va por delante ni por detrás. preload.js usa
los mismos valores".

### A.2 Marco de ventana: barra superior arrastrable (chrome.css:30-45, 137-157)

- Ventana **sin barra de título de Windows**; la barra superior de YT (`ytmusic-nav-bar`) hace de barra de
  título: se arrastra y deja sitio a minimizar/maximizar/cerrar (Window Controls Overlay) (chrome.css:30-32).
- `ytmusic-nav-bar { -webkit-app-region: drag }` (chrome.css:34) → toda la barra es zona de arrastre.
- Hueco para los botones de ventana (chrome.css:35):
  `--md-wco-space = 100vw − env(titlebar-area-x, 0) − env(titlebar-area-width, 100vw) + 8px`
  = ancho de la zona reservada a los botones nativos a la derecha + 8 px.
  - Ventana estrecha (avatar en flujo normal): `padding-right: var(--md-wco-space)`, `box-sizing: border-box`
    (chrome.css:36-38).
  - Ventana ancha (avatar en posición absoluta, el padding no le afecta): `#right-content { left: auto;
    right: var(--md-wco-space) }` (chrome.css:137-142). "Cada regla actúa en su caso".
- Alto de la barra: no lo fija chrome.css; en lyrics.css se usa `var(--ytmusic-nav-bar-height, 64px)`
  (lyrics.css:402, 410, 462) → **64 px**. La franja de los botones de ventana se cita como "franja de 64 px"
  (chrome.css:97-98) y "botones de la ventana (64 px)" (lyrics.css:342).
- **Con foco dentro (búsqueda abierta) deja de arrastrar**: `ytmusic-nav-bar:focus-within { -webkit-app-region:
  no-drag }` (chrome.css:43-45). Motivo: Windows se queda los clics de una zona de arrastre y la página no se
  enteraba, la búsqueda no se cerraba al pulsar la barra. Al salir de la búsqueda vuelve a arrastrar.
- Zonas **no-drag** (clicables) dentro de la barra (chrome.css:143-157): `a`, `button`, `input`,
  `yt-icon-button`, `tp-yt-paper-icon-button`, `ytmusic-search-box`, `ytmusic-logo`, `ytmusic-cast-button`,
  `ytmusic-settings-button`, todo hijo directo de `#right-content`, y además (en cualquier sitio)
  `tp-yt-iron-dropdown`, `ytmusic-popup-container`, `tp-yt-paper-dialog` (menús/diálogos).
  Réplica: arrastrar la ventana solo desde el fondo vacío de la barra superior.
- En pantalla completa la barra superior **no arrastra** (lyrics.css:483-486, ver B.9).

### A.3 Botones Atrás / Adelante (chrome.css:46-94)

Los crea preload.js; flechas entre el menú (☰, que ocupa x 16–56 px) y el logo, "como en Spotify"; el círculo
solo aparece al pasar el ratón.
- Logo desplazado: `ytmusic-logo { margin-left: 80px }` (chrome.css:49).
- `.md-back` / `.md-forward` (chrome.css:50-71): `position: absolute; top: 50%; translate: 0 -50%` (centrado
  vertical en la barra); `left: 64px` (atrás) y `left: 104px` (adelante, chrome.css:72); `z-index: 2`;
  **36×36**, `border-radius: 50%`, sin borde, fondo transparente, color `--md-text`, contenido centrado,
  `cursor: pointer`, no-drag. Transición `background-color .15s linear, opacity .15s linear`.
- Icono (`::before`, chrome.css:73-81): 22×22, color `currentColor` aplicado como máscara SVG
  (Material Symbols `chevron_left`, viewBox `0 -960 960 960`, path
  `M560-240 320-480l240-240 56 56-184 184 184 184-56 56Z`). Adelante = el mismo espejado: `scale: -1 1`
  (chrome.css:83).
- Hover (no desactivado): fondo `--md-state-on` (chrome.css:84).
- Desactivado: `opacity: .35`, `cursor: default` (chrome.css:85).
- Foco teclado: `outline: 2px solid --md-text; outline-offset: 2px` (chrome.css:86).
- **≤ 615 px** (diseño móvil de YT) (chrome.css:87-94): logo solo con su icono (`width: 26px; overflow:
  hidden`), logo con `margin-left: 40px`; se ocultan el botón de transmitir (`ytmusic-cast-button`) y
  `.md-forward` (solo queda Atrás).

### A.4 Botón "volver a YouTube Music" (`body > .md-home`, chrome.css:96-135)

Lo crea preload.js **solo fuera de music.youtube.com** (p. ej. inicio de sesión de Google). Píldora oscura
con flecha, centrada arriba en la franja de 64 px de los botones de ventana (Google no pone nada ahí; no se
mueve la página de Google porque fija su alto al de la ventana y apartarla dejaba una banda).
- `position: fixed; top: 14px; left: 50%; translate: -50% 0; z-index: 2147483647`.
- Alto 36 px, `padding: 0 14px 0 6px`, `border-radius: 18px`, sin borde, fondo `rgba(0,0,0,.55)`,
  color `--md-text`, fuente `500 14px/1 Roboto, Arial, sans-serif`, flex centrado vertical con `gap: 4px`,
  no-drag. Transición `background-color .15s linear`.
- Icono `::before`: chevron_left 22×22 (mismo path que A.3).
- Hover: fondo `rgba(0,0,0,.75)` (chrome.css:133).
- Foco: `outline: 2px solid --md-on-fill (#000)` offset 2 — negro porque vive sobre páginas blancas de
  Google (chrome.css:134-135).
- El texto del botón no está en este CSS (lo pone preload.js).

### A.5 Engranaje de ajustes (`.md-gear`, chrome.css:159-190)

Lo crea preload.js, entre "enviar a dispositivo" y el avatar.
- `flex: none`, **36×36**, `margin: 0 6px 0 2px`, sin padding/borde, `border-radius: 50%`, fondo
  transparente, color `--md-text`, centrado, `cursor: pointer`, no-drag, transición `background-color .15s linear`.
- Icono `::before` 24×24: Material Symbols `settings` (path largo en chrome.css:185; copiar literal).
- Hover **o** menú abierto (`aria-expanded="true"`): fondo `--md-state-on` (chrome.css:188-189).
- Foco: outline 2px `--md-text`, offset 2px.

### A.6 Panel de ajustes (`.md-settings`, chrome.css:192-274)

"Mismo estilo que el menú rápido. Sin backdrop-filter; al abrir solo se animan opacity y transform."
- Contenedor: `position: fixed; z-index: 3000; width: 320px; box-sizing: border-box; padding: 6px;
  background: --md-raised (#1e1e1e); border: 1px solid --md-raised-line; border-radius: 14px;
  box-shadow: --md-raised-shadow; color: --md-text; font: 14px/1.3 system-ui, -apple-system, "Segoe UI",
  sans-serif; user-select: none`, no-drag. La posición exacta (debajo del engranaje) la calcula preload.js.
- Animación de entrada `md-settings-in .12s ease-out`: de `opacity 0, translateY(-4px)` a `opacity 1,
  none` (chrome.css:208-210).
- **Fila** `.md-set-row` (chrome.css:211-231): flex, centrado vertical, `gap: 10px`, ancho 100 %,
  `min-height: 36px`, `padding: 8px 10px`, sin borde, `border-radius: 8px`, fondo transparente, texto a la
  izquierda, `cursor: pointer`. Hover (no desactivada): `--md-state-hover`. Foco: outline 2px `--md-text`,
  offset **−2px** (por dentro). Desactivada: `cursor: default` (sin cambio de color en este CSS).
- `.md-set-label { min-width: 0 }` (232).
- `.md-set-r` — zona derecha de la fila (233): `margin-left: auto`, flex, `gap: 6px`, color `--md-text-3`,
  13 px.
- `.md-kbd` — tecla de atajo (234-237): fondo `--md-state-hover`, borde 1px `--md-line`, radio 4px,
  `padding: 2px 7px`, color `--md-text-2`, `12px/1.3` system-ui, `nowrap`.
- `.md-chev` — chevron (238-243): 16×16, color `--md-text-3` (máscara Material `chevron_right`, path
  `M504-480 320-664l56-56 240 240-240 240-56-56 184-184Z`); transición `transform .15s ease-out`;
  `.open` → `rotate(90deg)`.
- `.md-set-sep` — separador (244): alto 1px, `--md-line`, `margin: 6px -6px` (sale hasta los bordes del panel).
- `.md-set-sub` — sub-bloque sangrado (246-248): `margin: 2px 0 4px 10px; padding-left: 10px;
  border-left: 1px solid --md-line`; sus filas `min-height: 30px; padding: 6px 10px; font-size: 13px`.
- `.md-set-row.recording` (grabando un atajo) (249): fondo `--md-state-on`.
- `.md-set-hint` (250): `margin-left: auto`, `--md-text-2`, 12 px, alineado a la derecha.
- `.md-set-err` (251): `margin: 0 10px 4px`, `--md-danger`, 12 px.
- `.md-set-cap` — título de sección (252): `margin: 8px 10px 2px`, `--md-text-3`, 12 px, peso 500.
- `.md-dim` (253): `opacity: .4`.
- **Pie** (255-265): `.md-set-foot` flex, centrado, `gap: 10px`, `padding: 2px 4px 2px 10px`; versión
  `.md-set-ver` `--md-text-3` 12 px a la izquierda; botón `.md-set-btn` a la derecha (`margin-left: auto;
  padding: 6px 8px; border-radius: 6px`, fondo transparente, `--md-text`, `500 13px/1.3` system-ui; hover
  `--md-state-hover`; foco outline 2px offset −2px; desactivado color `--md-text-off`, cursor default).
- **Radio** `.md-rad` (269-270): 14×14, borde 2px `--md-text-3`, círculo; `.on` → borde `--md-text` y punto
  central `radial-gradient(--md-text 0 3px, transparent 4px)` (punto blanco de radio 3 px, borde suave hasta 4).
- **Interruptor** `.md-sw` (271-274), 30×18 ("como el de Reproducción automática de la cola"):
  pista `border-radius: 9px`, fondo `--md-state-on`, `margin-left: auto`; bola `::after` 14×14 en
  `top: 2px; left: 2px`, fondo `--md-text-2`, transición `transform .15s ease-out`.
  `.on` → pista `--md-text` (blanca), bola `--md-on-fill` (negra) y `translateX(12px)`.

---

## B. Letras y Now Playing (lyrics.css + lyrics-view.js)

### B.0 Arquitectura y ciclo de vida

- lyrics.css se inyecta siempre, en cualquier documento; todo va bajo `ytmusic-player-page` (lyrics.css:1-2).
- lyrics-view.js corre en el mundo principal de la página; main.js le antepone `lib/lyrics-timing.js` dentro
  de un bloque `{ }` y llama a `init` (lyrics-view.js:1-2; main.js:119-130). Si ya existía una versión,
  la destruye antes (lyrics-view.js:9).
- Principios del autor (lyrics-view.js:4-8): el estado va en atributos de `ytmusic-player-page` y **el CSS
  hace todo el movimiento**; el JS solo mide tamaños cuando cambian. **Sin bucles**: todo por eventos del
  `<video>`, del reproductor y de la barra. Sin innerHTML (Trusted Types).
- API pública (lyrics-view.js:742-794): `init(opts)`, `receive(videoId, result)`, `setFullscreen(on)`,
  `destroy()`.
- `init(opts)` (744-761): `opts = { open: bool (settings.lyricsOpen), fullscreen: bool (win.isFullScreen()),
  strings: {...} }` (main.js:128). `wantOpen` se toma de `opts.open` **solo la primera vez**; los textos se
  actualizan cada vez (cambio de idioma). Llama `check('init')`, `markOpen()`, `setFullscreen(opts.fullscreen)`,
  reescribe el texto del crédito (si letra sincronizada) y de la nota de no sincronizada, y `applyState()`.

### B.1 IPC (forma exacta de datos)

Página → preload (lyrics-view.js:99-102): `html.dataset.mdLyricsMsg = JSON.stringify(msg)` + evento
`md-lyrics`. preload.js:386-396 lo lee, lo borra y reenvía solo:

| msg (página) | Canal IPC | Payload | Efecto en main |
|---|---|---|---|
| `{type:'request', track:{videoId, title, artist, album, duration}}` (lyrics-view.js:457) | `lyrics:request` | objeto `track` | Valida `videoId` con `/^[\w-]{6,20}$/`; recorta `title/artist/album` a 300 caracteres (no-cadena → `''`); `duration` número finito o `null` (main.js:711-720). Busca en LRCLIB (UA `ByTune/<versión> (https://github.com/Jscruzgg/ByTune)`, main.js:703) y, si no hay sincronizada, en la propia página. Responde con `__mdLyrics.receive(videoId, result)`; si la búsqueda lanza → `{kind:'none'}` (main.js:722-725). |
| `{type:'open', open: bool}` (lyrics-view.js:161) | `lyrics:open` | bool | Guarda `settings.lyricsOpen` si cambia (main.js:735-740). |
| `{type:'fullscreen', on: bool}` (lyrics-view.js:178, 730) | `np:fullscreen` | bool | Si `win.isFullScreen() !== on` → `win.setFullScreen(on)`; si no, solo `notifyFullscreen()` (recordar el estado real) (main.js:728-734). |

Main → página: `__mdLyrics.setFullscreen(win.isFullScreen())` al entrar/salir de pantalla completa,
también con **Esc** (`before-input-event`) (main.js:141-146; lyrics-view.js:629-630).

**Resultado de letra** (`receive`, lyrics-view.js:618-626; lyrics-source.js:38-80):
- `{ kind: 'synced', lines: [{ t: segundos, text, gap?: true }], source: 'LRCLIB' }`
- `{ kind: 'plain', lines: [string, ...] (''= salto de estrofa), source: 'LRCLIB' | <texto de fuente de YT> }`
- `{ kind: 'none' }`
- Válido si `kind` es synced/plain, `lines` es array no vacío. Si `videoId` no coincide con la canción actual
  se ignora (respuesta vieja). `source` no-cadena → `''`.

### B.2 Datos: LRC y huecos (lrc.js) y tiempos (lyrics-timing.js)

- `parseLrc` (lrc.js:7-21): quita BOM; por línea busca todas las marcas `\[(\d+):(\d{1,2})(?:[.:](\d{1,3}))?\]`
  (minutos:segundos[.fracción], separador `.` o `:`, fracción de 1-3 dígitos = `n/10^len`). `t` redondeado a
  3 decimales. Líneas sin marca (metadatos `[ar:...]`, texto suelto) se descartan. Texto = línea sin marcas,
  `trim`. Una línea con varias marcas genera una entrada por marca. Se ordena por `t`.
- `addGaps(lines, {minGap = 6})` (lrc.js:23-38), con `GAP_TEXT = '• • •'` y `LINE_DURATION = 4` s:
  1. Las líneas de texto vacío se convierten en hueco en su sitio (`{t, text:'• • •', gap:true}`).
  2. Si la primera no es hueco y empieza en `t ≥ 6` → se antepone hueco en `t = 0`.
  3. Entre dos líneas consecutivas no-hueco con `next.t − line.t ≥ 6 + 4 = 10 s` → hueco en `line.t + 4`.
- `activeIndex(times, t)` (lyrics-timing.js:5-19): búsqueda binaria; índice de la última línea con
  `inicio ≤ t`; `−1` antes de la primera.
- `msUntilNext(times, t, rate)` (21-29): `null` si `rate ≤ 0`/no válido, `t` no finito o no hay siguiente;
  si no `max(0, ceil(round6((times[next] − t)·1000 / rate)))` (redondeo a 6 decimales antes del ceil para
  evitar ruido de coma flotante).

### B.3 Estructura de la vista (DOM propio y orden)

Creado en `build()` (lyrics-view.js:149-222) e insertado en `ensureDom()` (224-253) dentro de
`ytmusic-player-page #main-panel`. Orden final de hijos de `#main-panel` (YT pone `#player` antes):

1. `#player` (carátula/vídeo de YT)
2. `#md-lyrics` (área de letra) — `insertBefore(area, info)`
3. `button.md-np-btn.md-lyrics-btn` (letras)
4. `button.md-np-btn.md-fs-btn` (pantalla completa)
5. `button.md-np-btn.md-av-btn` (Canción/Vídeo)
6. `#md-np-info` (fila de título) — siempre al final (`panel.append`)

(`#av-id`, el selector original de YT, existe pero se oculta, lyrics.css:134.)
`#md-np-info > .md-np-text > (.md-np-title, .md-np-artist)` (lyrics-view.js:150-155, 192).
`#md-lyrics > .md-lyrics-track > (líneas…)` (193-196).
YT puede rehacer el panel: `ensureDom` vuelve a insertar lo que falte (223-237).

Capas: `#main-panel` `position: relative; flex-direction: column` siempre (lyrics.css:167-173; YT lo pasaba a
fila en canciones sin vídeo). `#player` y `#md-np-info` `position: relative; z-index: 2` — la columna nunca
queda tapada por la letra (`z-index: 1`) (lyrics.css:174-179). Botones `z-index: 3` (lyrics.css:101-103).

### B.4 Carátula y fila de título

- Tamaño máximo de carátula (lyrics.css:9-11): `--md-np-art = (--md-vh, 100vh) − (--md-np-reserve, 430px)`
  (con `:where` para que el tema pueda afinarlo). `--md-np-reserve` = alto para barras, selector y título;
  el tema pone el suyo.
- Modo canción (`:not([data-md-video])`, lyrics.css:36-42): `#player` `max-width = max-height = --md-np-art`,
  `aspect-ratio: 1/1`, `height: auto`, `flex: none` → **siempre cuadrada**. Se decide por `data-md-video` y
  no por el `playback-mode` de YT porque YT pasa ~1 s por `NONE` al cambiar de canción con un recuadro negro
  16:9 y la carátula saltaba (lyrics.css:33-35).
- Modo vídeo (lyrics.css:43-48): solo `max-width: --md-np-art × 16/9` (el alto lo da la proporción de YT).
- `#player { margin-block: 0 }` — título pegado a la carátula (lyrics.css:418-420).
- Capa de YT sobre la carátula (`.song-media-controls`, oscurece y muestra botones al hover) **oculta** en
  Now Playing (lyrics.css:330-333).
- Pestaña "Letra" del panel derecho (2.ª de A continuación/Letra/Comentarios/Relacionado) **oculta**
  (lyrics.css:326-328).
- **Fila de título** `#md-np-info` (lyrics.css:50-92): flex, centrado vertical, `flex: none`,
  ancho `min(100%, --md-np-art)` (vídeo: `min(100%, --md-np-art·16/9)`, empieza en el borde izquierdo del
  vídeo), `margin-top: 20px`, `contain: inline-size` (su texto no ensancha el panel).
  - `.md-np-text` `flex: 1 1 auto; min-width: 0`.
  - Título y artista: una línea, `overflow: hidden`, puntos suspensivos.
  - **Título**: `"YouTube Sans", Roboto, Arial, sans-serif`, **26 px, 700**, `letter-spacing: -0.02em`,
    `line-height: 1.35` (hueco para descendentes g/p/y que con 1.2 se recortaban), `padding-bottom: 2px`,
    color `--md-text`.
  - **Artista**: `margin-top: 4px`, **17 px, 500**, `line-height: 1.3`, color `--md-text-2`. (Fuente
    heredada; no se declara familia.)
  - Texto de la barra de reproducción (lyrics-view.js:331-337): título = `.title`; byline `.byline` partida
    por `•`: artista = parte 0; álbum = parte 1 solo si hay 3 partes y la 3.ª es un año de 4 dígitos
    ("Artista • Álbum • Año"); en vídeos ("Artista • N vistas • N me gusta") no hay álbum.
  - Al cambiar el título (y sin reduced-motion), `.md-np-text` anima `opacity 0→1, translate 0 8px→0 0`,
    400 ms, `cubic-bezier(.22,.8,.3,1)` (lyrics-view.js:437-440).

### B.5 Botones de la esquina (lyrics.css:94-159; lyrics-view.js:132-191)

Fijos en la esquina inferior derecha de `#main-panel` (encima de la barra de reproducción, a la izquierda de la
cola); **no se mueven** al abrir/cerrar letra ni al cambiar de modo (nada salta).

| Botón | Posición (`right`, `bottom: 24px`) | Icono | Acción |
|---|---|---|---|
| Pantalla completa `.md-fs-btn` | `right: 24px` | FS_ON `M4 9V4h5`, `M15 4h5v5`, `M20 15v5h-5`, `M9 20H4v-5`; en pantalla completa FS_OFF `M9 4v5H4`, `M20 9h-5V4`, `M15 20v-5h5`, `M4 15h5v5` | ver B.9 |
| Letras `.md-lyrics-btn` | `right: 80px` (24+44+12) | bocadillo `M4 5h16v11H9l-5 4z`, `M8 9h8`, `M8 12.5h5` | `wantOpen = !wantOpen`, `send open`, `applyState()` (lyrics-view.js:159-163) |
| Canción/Vídeo `.md-av-btn` | `right: 136px` (80+44+12) | en canción: VIDEO `M3 6h18v12H3z`, `M10 9.5v5l4-2.5z`; en vídeo: auriculares `M4 18v-5a8 8 0 0 1 16 0v5`, `M4 15h3v5H4z`, `M17 15h3v5h-3z` (muestra el modo **al que** cambia) | pulsa `#av-id .song-button` (si en vídeo) o `.video-button` (si en canción) del selector oculto de YT (187-191) |

De derecha a izquierda: pantalla completa, letras, Canción/Vídeo; huecos de 12 px.
- Iconos SVG (lyrics-view.js:139-148): **22×22**, viewBox `0 0 24 24`, sin relleno, trazo `currentColor`
  **2 px**, extremos y uniones redondeados.
- Botón: `position: absolute; z-index: 3`, **44×44**, círculo, sin borde, fondo `--md-state-on`, color
  `--md-text`, `cursor: pointer`, `appearance: none` (lyrics.css:99-120).
- Transición: `opacity .3s ease, visibility 0s linear .3s, translate var(--md-dur) var(--md-ease)`.
  **Sin fundido de colores** a propósito: fondo e icono cambiaban a la vez y a medio camino quedaban los dos
  oscuros; el cambio de color es inmediato (lyrics.css:121-123).
- Hover (no desactivado): fondo `--md-state-press` (lyrics.css:142). Foco: outline 2px `--md-text` offset 2.
- **Activo** (letras con `aria-pressed="true"`, y pantalla completa cuando está activa): fondo `--md-text`
  (blanco), icono `--md-on-fill` (negro) (lyrics.css:144-148). Hover activo: sigue blanco con
  `box-shadow: inset 0 0 0 22px --md-state-ink` (oscurece 10 %); `:active`: `--md-state-ink-press` (16 %)
  (lyrics.css:149-159).
- Desactivado: `opacity: .35`, `cursor: default` (lyrics.css:128-131). Letras se **desactiva en modo vídeo**
  (sigue en su sitio, atenuado) (lyrics-view.js:587).
- Canción/Vídeo **oculto** si la página no tiene `has-av-switcher` o si el toggle de YT tiene
  `toggle-disabled` (canción sin vídeo) (lyrics.css:136-139).
- Textos (aria-label y title): letras `lyrics.show` "Mostrar letra" / `lyrics.hide` "Ocultar letra";
  Canción/Vídeo `np.toVideo` "Cambiar a vídeo" / `np.toSong` "Cambiar a canción"; pantalla completa
  `np.fullscreen` "Pantalla completa" / `np.exitFullscreen` "Salir de pantalla completa" (i18n.js:50-58;
  inglés i18n.js:124-132).
- En pantalla completa con la barra de reproducción visible, los botones suben
  `translate: 0 −(--ytmusic-player-bar-height, 72px)` (lyrics.css:479-482), animado con `--md-dur/--md-ease`.

### B.6 Estado (applyState / writeState, lyrics-view.js:553-599)

- Modo canción/vídeo desde el atributo `playback-mode` de `#player` (observado, lyrics-view.js:73, 239-243):
  - `NONE`: se mantiene el modo anterior; si sigue en NONE pasados **NONE_MS = 2000 ms** (temporizador que
    re-llama a applyState) → `videoMode = true` (YT enseña el vídeo) (lyrics-view.js:40, 558-565).
  - Otro valor: `videoMode = (mode !== 'ATV_PREFERRED')` (569). `ATV_PREFERRED` = canción.
  - Si cambia el modo respecto a `data-md-video` → la escritura va dentro de `morph` (transición de vista).
- `shown = wantOpen && !videoMode`. Al pasar a visible, se mide (582).
- Atributos de página: `data-md-video`, `data-md-has-lyrics`, `data-md-lyrics-mode` = `synced`|`plain`|`none`
  (none solo si status none; null mientras carga), `data-md-lyrics="open"` si shown (583-586).
- Al ocultar: `offset = 0`, cancela temporizador de rueda, quita `is-manual` (593-597). Luego `sync()`.
- `wantOpen` se persiste (`lyrics:open`) y se restaura en `init`.

### B.7 Disposición con la letra abierta (measure, lyrics-view.js:255-313; lyrics.css:161-214)

Constantes (lyrics-view.js:14-23): `GUTTER = 24`, `EDGE = 32`, `THUMB = 88`, `THUMB_GAP = 18`, `BTN = 44`,
`BTN_INSET = 24`, `MIN_COVER = 300`, `MIN_LYRICS = 400`.

Se mide (sin transformaciones, `offset*`) el panel (`W`, `H`), la carátula (`cw`, `top = cover.offsetTop`)
y la fila de título (`bottom = info.offsetTop + info.offsetHeight`). Si `W` o `cw` son 0, no hace nada.
- `fit = min(cw, W/2 − 2·24)`; `maxBottom = H − 24 − 44 − 16` (la letra acaba sobre los botones).
- **Estrecha** si `fit < 300` o `W/2 − 40 < 400` → `data-md-lyrics-narrow` (lyrics-view.js:283-284).

**Ancha** (lyrics-view.js:285-297; lyrics.css:194-202): la columna (carátula + título) se centra en la mitad
izquierda y se encoge alrededor de su centro si no cabe:
- `s = fit / cw` → `--md-lyr-s`; `--md-lyr-tx = −W/4`.
- `yc = (top + bottom)/2`; para cada elemento `dy(e) = (1 − s)·(yc − (e.offsetTop + e.offsetHeight/2))`
  → `--md-lyr-cover-y`, `--md-lyr-info-y` (el escalado se hace sobre el centro propio; dy lo lleva al centro
  común).
- Carátula: `translate: tx cover-y; scale: s`. Título: `translate: tx info-y; scale: s`.
- Área de letra: `top = yc − s·(bottom − top)/2` → `--md-lyr-top`;
  `h = min(yc + s·(bottom − top)/2, maxBottom) − lyrTop` → `--md-lyr-h` (mismo alto que la columna,
  recortado por arriba de los botones).
- Horizontal de la letra (lyrics.css:219-226): `left: 50%; right: 40px`.

**Estrecha** (lyrics-view.js:298-312; lyrics.css:203-214, 242-245): miniatura arriba a la izquierda, título a
su lado, letra debajo; todo con transformaciones (cambiar anchos movería el panel de YT):
- Carátula: `scale = 88/cw` (`--md-lyr-thumb-s`); centro llevado a `(32 + 44, 32 + 44)`:
  `thumb-x = 32 + 44 − (cover.offsetLeft + cw/2)`, `thumb-y = 32 + 44 − (cover.offsetTop + cover.offsetHeight/2)`.
- Título: `infoX = 32 + 88 + 18 = 138`; `ninfo-x = 138 − (W − infoW)/2` (la fila está centrada en el panel);
  `ninfo-y = 76 − (info.offsetTop + info.offsetHeight/2)` (centrado vertical con la miniatura); sin escala.
  Ancho máximo del texto `--md-lyr-ninfo-w = max(0, W − 138 − 32)` → título largo no se mete en la cola.
- Letra: `top = 32 + 88 + 24 = 144`; `h = min(H − 32, maxBottom) − 144`; `left: 32px; right: 32px`.
- Valores escritos redondeados: px a 0,1 (`setVar`), sin unidad a 0,001 (`setNum`) (lyrics-view.js:113-114).
  Solo se escriben si cambian. `--md-lyr-*` se escriben en `#main-panel`; el resto en `<html>` (118-123).
- Cuándo se mide: ResizeObserver sobre `#main-panel`, `#player`, `#md-np-info` y el evento `md-measure` de
  preload (al plegar la barra lateral, FLIP), ambos vía `snapMeasure` (lyrics-view.js:80-90); y al abrir la
  vista (582). **`snapMeasure`** pone `data-md-snap` (sin transición) y lo quita tras **dos fotogramas**
  (doble `requestAnimationFrame`): al cambiar de tamaño la columna va directa a su sitio (si no, se deslizaba
  desde la posición vieja cortada por la izquierda) (lyrics-view.js:77-85; lyrics.css:189-193).
- Si `#player` ya no está conectado (YT lo cambia al entrar/salir de ≤615 px), se rehace `ensureDom`
  (lyrics-view.js:260-262).
- **Diseño móvil (≤615)**: al entrar, `fixMwebTime` a los 300, 800, 1500 y 3000 ms: si los controles de YT
  muestran "NaN" se copia el tiempo de la barra (`"0:00 / 2:57"` partido por `/`) en su nodo de texto
  (lyrics-view.js:264-270, 315-326). (Peculiaridad de YT; en la réplica: no mostrar nunca NaN.)

### B.8 Área de letra: tipografía, colores, resaltado y desplazamiento

**Contenedor `#md-lyrics`** (lyrics.css:219-245): `position: absolute; z-index: 1; top: --md-lyr-top (32px por
defecto); height: --md-lyr-h (100% − 64px por defecto); left: 50%; right: 40px; overflow: hidden`.
Máscara vertical: `linear-gradient(transparent 0, #000 10%, #000 82%, transparent 100%)` → bordes de arriba
(0–10 %) y abajo (82–100 %) difuminados. Cerrado: `opacity 0; translate 60px 0; visibility hidden;
pointer-events none`.

**Pista `.md-lyrics-track`** (lyrics.css:246-252): `position: relative` (las líneas miden su `offsetTop` desde
aquí), `padding: 0 5% 0 6px`; `transition: transform .6s cubic-bezier(.22,.8,.3,1)`; con rueda
(`#md-lyrics.is-manual`) `transform .15s linear` ("sigue al gesto sin el retardo largo").

**Línea `.md-line`** (lyrics.css:254-269):
- `margin: 0; padding: 12px 0`; `"YouTube Sans", Roboto, Arial, sans-serif`; **30 px, 700, line-height 40 px**;
  color `--md-text` (blanco) siempre; `overflow-wrap: anywhere`; `transform-origin: left center`.
- Jerarquía **por opacidad** (para animarlo en GPU), no por color:
  - **Futura** (por defecto): `opacity .56` (= nivel `--md-text-3`).
  - **Pasada** `.is-past`: `opacity .38` (= `--md-text-off`) (lyrics.css:281).
  - **Activa** `.is-active`: `opacity 1; scale 1.04` (lyrics.css:282), escalada desde el borde izquierdo.
  - Comentario: con .3 y .18 se perdían sobre carátulas claras (2,2:1 y 1,6:1) (lyrics.css:263-265).
- Transición `opacity .4s ease, scale .4s ease`.
- **Hueco** `.is-gap` (texto "• • •"): `letter-spacing: 8px`; misma lógica de opacidad que el resto (es una
  línea más con su `t`).
- Con letra sincronizada (`data-md-lyrics-mode="synced"`) todas las líneas `cursor: pointer` (284).
- Estrecha: `padding: 8px 0; font-size: 24px; line-height: 32px` (lyrics.css:276-280).
- Entrada de líneas nuevas: keyframe `md-line-in` desde `opacity 0; translate 0 12px`, `.5s
  cubic-bezier(.22,.8,.3,1)`, aplicado a **todo hijo** de la pista, solo sin reduced-motion
  (lyrics.css:270-275). Se dispara cada vez que `render()` reemplaza los hijos (otra canción, o llegada de
  la letra tras las barras de espera).

**Render** (lyrics-view.js:460-498), una vez por canción:
- `status 'none'` → una línea `.md-line.md-lyrics-empty` con `lyrics.none` = "No hay letra para mostrar aquí :("
  (en: "No lyrics to show here :("); `opacity .6`, cursor default; pista con `padding-top: 48px`
  (lyrics.css:314-316). Es un "aviso tenue arriba del área" con estilo de línea (30 px bold).
- **Cargando** (sin letra aún) → 3 líneas `.is-placeholder` con un `<span>` de ancho **70 %, 52 %, 62 %**;
  línea `opacity 1`, cursor default; span `display:block; height: 24px; margin: 8px 0; border-radius: 8px;
  background: --md-state-on` (lyrics.css:285-293). (La línea mantiene su `padding: 12px 0`.)
- **Sincronizada** → una `.md-line` (o `.md-line.is-gap`) por entrada con `data-t = t`, y al final
  `.md-lyrics-credit` con `lyrics.credit` = "Letra de LRCLIB" (en: "Lyrics from LRCLIB").
- **Plana** → primero `.md-lyrics-note` con `lyrics.unsynced` = "Esta letra no está sincronizada con la
  canción"; luego una `.md-line` por texto, o `.md-line.is-break` si la cadena es vacía; al final crédito:
  si `source === 'LRCLIB'` → `lyrics.credit` traducido; si no, el texto de fuente que trae YT tal cual; si
  vacío, sin crédito.
- Tras render: `times`, `lineEls` (solo las primeras `times.length`), `active = −2`, `offset = 0`, reset de
  `lastY/lastBase`, `transform` vacío, `scrollTop = 0`.
- **Crédito** `.md-lyrics-credit` (lyrics.css:294-300): `margin: 32px 0 0; padding: 0 0 48px; 14px/20px;
  --md-text-3`.
- **Nota** `.md-lyrics-note` (lyrics.css:307-313): `margin: 0 0 16px; 14px/20px; --md-text-3`.
- **Modo plano** (lyrics.css:301-306, 317-318): `#md-lyrics` con `overflow-y: auto` y **barra invisible**
  (`scrollbar-width: none`) → desplazamiento nativo; pista `padding-top: 48px`; líneas `opacity 1` (todas
  blancas), `padding: 8px 0` (mismo tamaño 30/40, o 24/32 en estrecha — ojo: la regla de padding de plano
  tiene la misma especificidad aproximada que la de estrecha; el orden de origen hace ganar a la de plano,
  `8px 0` en ambas); salto `.is-break` alto **32 px**, sin padding. Sin clic, sin seguimiento.

**Sincronía** (lyrics-view.js:500-550):
- `running()` = vista mostrada **y** hay tiempos **y** Now Playing abierto (`player-page-open`) **y**
  documento visible (507-510). Si no, no corre nada.
- `sync()`: para el temporizador; tiempo `now()`; `setActive(activeIndex(times, t))`; `place()`; si pausado,
  fin; si no, **un único `setTimeout`** a `msUntilNext(times, t, playbackRate) + 15 ms` ("unos ms de margen
  para caer ya dentro de la línea siguiente") (539-550).
- `now()` (381-390): pregunta al reproductor de YT `getCurrentTime()` solo si `getVideoData().video_id` es
  el de la canción actual (si no, `null` y se espera al siguiente evento); si no hay reproductor,
  `video.currentTime`.
- `setActive(i)` (512-521): solo toca las líneas entre el índice viejo y el nuevo (todas la primera vez):
  `is-past` si `j < i`, `is-active` si `j === i`. Con `i = −1` ninguna activa ni pasada (todas futuras).
- **`place()` — posición** (522-538): línea de referencia `lineEls[max(0, active)]`.
  - `anchor = clientHeight(área) × 0.3` → **la línea activa se coloca al 30 % del alto del área** (no
    centrada).
  - `base = anchor − line.offsetTop`.
  - Con `is-manual` y `lastBase` conocido: `offset += base − lastBase` (mientras se mueve a mano el
    seguimiento se para: si cambia la línea, la letra no se mueve).
  - `y = clamp(base − offset, anchor − trackHeight, anchor)` — no se sale del principio ni del final;
    `offset = base − y`.
  - `translateY(round(y))` en la pista, solo si cambió. El movimiento lo anima la transición CSS
    (.6 s `cubic-bezier(.22,.8,.3,1)`, o .15 s linear en manual). No hay spring.
- Eventos del `<video>` (355-377), `passive`: `loadeddata`, `durationchange` → `check('video')`+`sync`;
  `playing` → `check('playing')`+`sync`; `play`, `pause`, `seeked`, `ratechange` → `sync`; `waiting` →
  `stopTimer`; `timeupdate` → red de seguridad: si `running()` y `activeIndex(t) !== active` → `sync`.
- `visibilitychange` → `sync` (lyrics-view.js:739). Cambio de `player-page-open` → `onPageChange` → `sync`.

**Clic en línea → salto** (lyrics-view.js:197-208): solo con letra sincronizada y línea con `data-t`:
`offset = 0`, cancela rueda, quita `is-manual`, `player.seekTo(t, true)` (por el reproductor de YT para que
su barra se entere) o `video.currentTime = t`.

**Rueda** (lyrics-view.js:209-221): solo con vista mostrada y letra sincronizada (`passive`):
`offset += deltaY` (×40 si `deltaMode === 1`, líneas); añade `is-manual`; `place()`; **4000 ms** después de la
última vuelta: `offset = 0`, quita `is-manual`, `place()` (vuelve sola a la línea actual).

### B.9 Abrir / cerrar la letra: secuencia animada (lyrics.css:180-188, 216-241)

Solo en modo canción (en vídeo la letra no se abre).
- **Abrir**: la columna (carátula y título) sale **enseguida**: `translate .6s cubic-bezier(.22,.8,.3,1),
  scale .6s (igual), opacity .3s ease`, sin retardo. La letra aparece **0,2 s después** desde la derecha:
  `opacity .45s ease .2s`, `translate .6s cubic-bezier(.22,.8,.3,1) .2s` (de `60px 0` a `0 0`), visibility
  inmediata.
- **Cerrar**: la letra se va rápido: `opacity .2s ease`, `translate .2s ease` (a 60 px a la derecha),
  `visibility 0s linear .2s`; la columna **espera 0,2 s** y vuelve: `translate .6s … .2s, scale .6s … .2s,
  opacity .3s ease .2s`.
- `data-md-snap` anula estas transiciones de la columna (B.7).

### B.10 Carga de la letra (check / newTrack, lyrics-view.js:400-458)

- Cada evento llama a `check(source)`: rehace enlaces (DOM, `<video>`, título de la barra); si faltan barra,
  vídeo o panel, observa todo el documento hasta que aparezcan (startObserver, 93-95, 428-435).
- La barra se observa (`childList, characterData, subtree` en `.title` y `.byline`) → `check('bar')`.
- Canción nueva (`video_id` distinto) → `newTrack`: `status = 'loading'`, `lyrics = null`, `render()` (barras
  de espera), `applyState()`, y temporizador: a **FORCE_AFTER = 6000 ms**, si aún no se pidió →
  `check('force')`; luego a **LOAD_TIMEOUT − FORCE_AFTER = 9000 ms** más (total 15 s) → `giveUp`: `status =
  'none'` ("no hay letra") (lyrics-view.js:24-25, 405-421).
- Se pide la letra (una vez por canción) cuando: la barra está lista (`barReady`: título no vacío y igual al
  del reproductor o distinto del de la canción anterior) salvo `force`; y hay duración fiable
  (`duration()`: `video.duration` finito > 0 y, si el reproductor da duración, diferencia ≤ 3 s), o la fuente
  es `playing`/`force`, o ya está sonando (`!paused && currentTime > 0.5`) (lyrics-view.js:339-353, 448-457).
  `duration` puede ir `null`.

### B.11 Pantalla completa (lyrics-view.js:628-715; lyrics.css:422-523)

La pone main.js en la **ventana** (no la Fullscreen API del documento: con ella YT cambia a su propio diseño).
Esc también sale (main.js). Al cerrar Now Playing se sale de pantalla completa.

**Entrar** (botón, lyrics-view.js:169-179): si no está en pantalla completa, guarda `--md-win-h = innerHeight`
y pone `data-md-fs-pending`; envía `np:fullscreen`. La ventana crece primero; mientras `fs-pending`, lo que
depende del alto usa el alto de antes (`--md-vh = --md-win-h`; `.content` alto `--md-win-h`; barra de
reproducción, su fondo y la cola con `bottom: 100vh − --md-win-h`), y la franja de la barra de tareas la
cubre el fondo de Now Playing (lyrics.css:444-451). Al llegar `setFullscreen(true)` todo viaja a la vez (una
transición de vista se cancela si la ventana cambia de tamaño a medias, por eso este orden).

**setFullscreen(on)** (662-677): dentro de `morph` si cambia: guarda `--md-screen-h = screen.height`, quita
`fs-pending`, pone/quita `data-md-fullscreen`, cambia los paths del icono FS_ON↔FS_OFF, textos, cancela
temporizador y `hideControls()`.

**Qué cambia en pantalla completa** (lyrics.css):
- Oculto `#nav-bar-divider`; `html, body { overflow: hidden }` (la página de detrás no se desplaza) (427-433).
- Now Playing alto `--md-screen-h` (alto de pantalla, no de ventana) y carátula mayor:
  `--md-np-art = min(screen-h − 448px, 100vw − 96px)` (reserva 448: selector ~110, título ~85, márgenes,
  barra de controles ~92, barra superior +48) (434-443).
- `.content` sin padding: los paneles ya no reservan sitio (453).
- `#main-panel` `padding-top: 64 + 16 = 80px` (la letra empieza bajo la barra superior) (461-463).
- Todo Now Playing se encoge hacia el hueco libre cuando entra el menú lateral o la cola:
  `#main-panel { translate: --md-fs-x 0; scale: --md-fs-s }`, transición `--md-dur --md-ease` (454-460).
  `fitContent()` (651-657): `l` = ancho de `#guide-wrapper` si `data-md-guide`, `r` = ancho de la cola si
  `data-md-side`; `--md-fs-s = (innerWidth − l − r)/innerWidth`, `--md-fs-x = (l − r)/2`.
- Barras/paneles (barra de reproducción + fondo, barra superior + fondo, menú lateral `#guide`,
  `#mini-guide` + fondo, cola) con `transition: translate var(--md-dur) var(--md-ease), opacity var(--md-dur)
  var(--md-ease)`, también **durante la transición de vista de salida** (466-468):
  - Barra de reproducción: oculta `translate 0 100%; opacity 0`; con `data-md-controls` → `0 0; 1` (469-478).
  - Barra superior: `no-drag` (además una zona de arrastre se queda los movimientos del ratón y la barra se
    escondería con el puntero encima); oculta `translate 0 −100%; opacity 0`; visible con `data-md-topbar`
    **o** con foco de teclado visible dentro (`:focus-within:has(:focus-visible)`; el foco de un clic no
    cuenta); su fondo en Now Playing es transparente (483-507).
  - Menú lateral: oculto `translate −100% 0; opacity 0`; visible con `data-md-guide` (492-499).
  - Cola: `padding-bottom: 0` (la barra de reproducción la tapa por abajo al aparecer), oculta
    `translate 100% 0; opacity 0`; visible con `data-md-side` o foco visible dentro (508-518).
  - Tiradores de redimensionar `.md-resize` ocultos (519-520).
  - **Cursor oculto** en todo si no está `data-md-pointer` (521-523).

**Puntero en pantalla completa** (onPointerMove, lyrics-view.js:691-715), constantes:
`CONTROLS_MS = 3000`, `EDGE_PX = 150`, `SIDE_PX = 16` (a 24 px están los botones), `CORNER_PAD = 96`.
- Si el objetivo está dentro de un menú emergente (`ytmusic-popup-container, tp-yt-iron-dropdown`), las
  barras se quedan como están. Si no:
  - `guide` = `x < 150` o puntero sobre el menú lateral.
  - `top` = `y < 150` o puntero sobre la barra superior.
  - Si **no** está en la esquina de los botones (`inCorner`): `controls` = `y > innerHeight − 150` o sobre la
    barra de reproducción; `side` = `x ≥ innerWidth − 16` o sobre la cola. En la esquina, ni la barra de
    abajo ni la cola cambian (saldrían —y la de abajo subiría los botones— justo al ir a pulsarlos).
  - `data-md-guide = guide`; `data-md-topbar = top || guide || side` (la barra superior sale también con el
    menú lateral y con la cola); `fitContent()`.
- Siempre `data-md-pointer` (cursor visible). Si el puntero está sobre barra, barra superior, menú lateral,
  cola o menú emergente → cancela el temporizador (no se esconde); si no → `hideControlsLater()`: a los
  **3000 ms sin mover** `hideControls()` quita pointer/controls/topbar/guide/side y `fitContent()`.
- `inCorner(e)` (678-690): `l`/`t` = mínimos `left`/`top` de los tres botones visibles (ancho > 0);
  esquina = `x ≥ l − 96` y `y ≥ t − 96` (hasta los bordes de la pantalla).

**Cerrar Now Playing** (onPageChange, lyrics-view.js:716-736): `data-md-np-open` en `<html>` refleja
`player-page-open`; al cerrar: si pantalla completa → `send fullscreen false`; `data-md-np-closing` durante
**CLOSING_MS = 400 ms**. Mientras open/closing, el pulgar de la barra de desplazamiento de la ventana es
transparente; abierto, la barra mide 0 de ancho (lyrics.css:21-32).

### B.12 Fondo

lyrics.css y lyrics-view.js **no definen ningún fondo** para Now Playing: ni color adaptativo, ni portada
difuminada, ni blur. Se menciona "el espacio de color de Now Playing" (lyrics.css:423; lyrics-view.js:631) y
"su fondo" (lyrics.css:446), que vienen de YT/theme.css (fuera de este documento). La barra de reproducción de
Now Playing usa `--md-glass-bar` según chrome.css:14 (aplicado en otro archivo). **No consta aquí.**

### B.13 Teclado y ratón (resumen)

- Clic: botones de la esquina, líneas sincronizadas (salto).
- Rueda sobre el área sincronizada: desplazamiento manual, 4 s de vuelta. En plano: scroll nativo.
- `pointermove` (global, passive): solo hace algo en pantalla completa.
- Teclado: no hay atajos en estos archivos. Foco visible con outline 2px blanco en todos los botones; en
  pantalla completa el foco de teclado dentro de barra superior o cola las mantiene visibles. Esc sale de
  pantalla completa (en main.js).

### B.14 Reordenar la cola (lyrics.css:544-560; queue-reorder.js)

- Elementos de la cola: `cursor: pointer` siempre.
- Con `html[data-md-q-dragging]`: los demás elementos `transition: transform .18s ease` (se apartan); el
  arrastrado `.md-q-drag`: `position: relative; z-index: 3; transition: none; background: --md-state-press;
  box-shadow: --md-raised-shadow; border-radius: 8px`; todo `cursor: pointer; user-select: none`.

---

## C. Animaciones (inventario)

| # | Qué | Propiedades | Duración / retardo | Easing | Disparo | Origen |
|---|---|---|---|---|---|---|
| 1 | Botones atrás/adelante | background-color, opacity | .15s | linear | hover/disabled | chrome.css:70 |
| 2 | Píldora volver a YT | background-color | .15s | linear | hover | chrome.css:122 |
| 3 | Engranaje | background-color | .15s | linear | hover/abierto | chrome.css:177 |
| 4 | Panel ajustes entrada `md-settings-in` | opacity 0→1, translateY(−4px)→0 | .12s | ease-out | al abrir | chrome.css:208-210 |
| 5 | Chevron ajustes | transform rotate 0→90° | .15s | ease-out | `.open` | chrome.css:239,243 |
| 6 | Interruptor (bola) | transform translateX 0→12px | .15s | ease-out | `.on` | chrome.css:272,274 |
| 7 | Botones esquina | opacity .3s; visibility 0s tras .3s; translate | .3s / --md-dur (.35s) | ease / --md-ease | desactivar; subir en FS | lyrics.css:123 |
| 8 | Columna al abrir letra | translate, scale (.6s); opacity (.3s) | sin retardo | cubic-bezier(.22,.8,.3,1); ease | abrir | lyrics.css:185-188 |
| 9 | Columna al cerrar letra | igual | retardo .2s | igual | cerrar | lyrics.css:181-184 |
| 10 | Área letra al abrir | opacity .45s; translate 60px→0 .6s | retardo .2s | ease; cubic-bezier(.22,.8,.3,1) | abrir | lyrics.css:240 |
| 11 | Área letra al cerrar | opacity, translate →60px | .2s; visibility a los .2s | ease | cerrar | lyrics.css:233 |
| 12 | Pista de letra (seguimiento) | transform translateY | .6s | cubic-bezier(.22,.8,.3,1) | cambio de línea | lyrics.css:249 |
| 13 | Pista con rueda | transform | .15s | linear | `is-manual` | lyrics.css:252 |
| 14 | Línea activa/pasada | opacity, scale (1→1.04) | .4s | ease | cambio de línea | lyrics.css:268 |
| 15 | Entrada de líneas `md-line-in` | opacity 0→(su valor), translate 0 12px→0 | .5s | cubic-bezier(.22,.8,.3,1) | nuevo render (solo sin reduced-motion) | lyrics.css:272-275 |
| 16 | Título/artista nueva canción (JS) | opacity 0→1, translate 0 8px→0 | 400 ms | cubic-bezier(.22,.8,.3,1) | cambia el título (sin reduced-motion) | lyrics-view.js:439 |
| 17 | Now Playing encogido en FS | translate x, scale | --md-dur .35s | --md-ease | menú/cola entran | lyrics.css:459 |
| 18 | Barras/paneles en FS | translate, opacity | .35s | --md-ease | atributos de puntero; también en la salida | lyrics.css:466-468 |
| 19 | Transición de vista (morph) | posición/tamaño de carátula, título, letra, 3 botones | .35s | --md-ease | entrar/salir FS, canción↔vídeo | lyrics.css:525-542; lyrics-view.js:601-616 |
| 20 | Cola reordenar | transform | .18s | ease | arrastre | lyrics.css:549 |
| 21 | Snap | ninguna (2 fotogramas) | — | — | resize del panel / md-measure | lyrics.css:191-193; lyrics-view.js:80-85 |

Detalle de **morph** (#19): `document.startViewTransition`; elementos con nombre: `md-np-cover` (#player),
`md-np-info`, `md-lyrics`, `md-fs-btn`, `md-lyrics-btn`, `md-av-btn`. Raíz: vieja oculta, nueva sin animación
(el resto de la página cambia **sin fundido**; las barras se deslizan con sus propias transiciones). Grupos:
`.35s --md-ease`. Título y letra viajan a su tamaño natural (`width: auto`, sin estirarse). La letra **sin
fundido**: la vieja oculta, la nueva sin animación (las líneas no coinciden entre disposiciones y se veían
dobles). Se omite (aplicación inmediata) si no hay API, documento oculto, reduced-motion o Now Playing
cerrado. Lo que llega mientras una espera se aplica con ella (cola `Set`). Réplica: interpolar rectángulos de
carátula/título/botones de su posición vieja a la nueva en .35 s con cubic-bezier(.2,0,0,1); la letra
aparece directamente en su nueva caja.

`prefers-reduced-motion: reduce`: desactiva #15, #16 y #19 (morph); el resto de transiciones CSS siguen.

---

## D. Responsive

| Umbral | Cambios | Origen |
|---|---|---|
| ≤ 615 px (YT `is-mweb-modernization-enabled`, diseño móvil) | Barra: logo solo icono (26 px, margin-left 40), sin cast ni Adelante. Now Playing como YT: **sin** fila de título, letra ni botones de la esquina; carátula sin transformaciones; `--md-np-art-fit = min(100vw − 64, 100vh − 436)`, `--md-np-art = fit` o **0** si `fit < 64` (truco ×1000); `#main-panel margin-top: −28px`; `#player-controls top: inner-height − 334px`; `.content height: 100%` | chrome.css:87-94; lyrics.css:335-354 |
| 616–939 px | Carátula y cola apiladas (cola debajo, se llega desplazando). `--md-st-h = max(343, vh − 244)` (con letra abierta `max(420, vh − 244)`); `--md-np-art = max(0, min(st-h − 183, 100vw − 200))`; `#main-panel` alto `st-h`, `padding-top: 16`, `justify-content: flex-start`; `#side-panel margin-top: 16`. Desglose del autor: visible = ventana − 64 − 92 − 16; dentro: 16, carátula, título (83), 16, botones (44 + 24); mínimo carátula 160 (con letra 420 de panel) | lyrics.css:356-382 |
| ≥ 940 px | Now Playing a toda la ventana (`top/bottom/left 0, width 100vw, height 100%`), paneles por encima (fijos fuera de FS, escondidos en FS); `.content padding: 64 (arriba) | cola (--md-np-side, 400) | barra reproducción (72) | menú (--ytmusic-guide-width, 0)`; cola `position: fixed; z-index: 3; top 0; padding-top 72; padding-bottom 72; right: 100% − 100vw; width --md-np-side (400)` | lyrics.css:384-417 |
| Disposición estrecha de la letra (JS) | `min(cw, W/2 − 48) < 300` o `W/2 − 40 < 400` → miniatura 88 px + letra debajo, líneas 24/32 | lyrics-view.js:283; lyrics.css:276-280 |

`--md-live-pad-l/-r`, `--md-live-side` y `--md-np-side` los escribe preload.js durante el arrastre de bordes de
panel (lyrics.css:400-402, 414); valor por defecto de la cola 400 px.

Nota: lyrics.css:353 tiene dos reglas en la misma línea (falta un salto); no afecta al resultado.

---

## E. Comentarios del autor con intenciones (selección literal parafraseada)

1. Una sola escala de color: blancos con opacidad para texto/estados, negros con opacidad para superficies;
   contraste text-2 ≥ 6,5:1, text-3 ≥ 4,7:1 (chrome.css:1-5).
2. La barra de YT hace de barra de título; con la búsqueda abierta deja de arrastrar para que los clics lleguen
   (chrome.css:30-45).
3. Flechas atrás/adelante como Spotify, círculo solo al hover (chrome.css:46-48).
4. Píldora "volver" centrada arriba porque Google no pone nada ahí y mover su página dejaba bandas
   (chrome.css:96-100).
5. Ajustes sin backdrop-filter; solo opacidad y transform al abrir (chrome.css:192-193).
6. Misma duración/curva para todo lo que se mueve a la vez (lyrics.css:12-13).
7. Carátula siempre cuadrada en canción pese al `NONE` intermedio de YT (lyrics.css:33-35; lyrics-view.js:558-560).
8. Botones de esquina fijos, nada salta al cambiar de modo; sin fundido de color para no pasar por "ambos
   oscuros"; activos blancos que se oscurecen al hover "como el botón de reproducir" (lyrics.css:94-98,
   121-122, 149-150).
9. Solo se animan transform/opacidad, sin `!important` en propiedades animadas (para que vayan a GPU)
   (lyrics.css:161-166).
10. Al cerrar, la columna espera a que la letra se haya ido (lyrics.css:180).
11. Jerarquía de líneas por opacidad .56/.38 porque .3/.18 se perdían sobre carátulas claras (lyrics.css:263-265).
12. La rueda sigue al gesto sin el retardo largo; vuelve sola a los 4 s (lyrics.css:251; lyrics-view.js:209).
13. Clic de línea por el reproductor de YT "así su barra se entera" (lyrics-view.js:197).
14. Sin bucles: un único setTimeout hasta la próxima línea + red de seguridad en timeupdate (lyrics-view.js:44,
    368-371).
15. Medir solo cuando algo cambia de tamaño; snap sin transición al redimensionar para evitar deslizamientos
    cortados (lyrics-view.js:77-79, 255-258).
16. Pantalla completa de ventana (no del documento); la ventana crece antes y el contenido conserva el alto
    viejo hasta el aviso para que la transición de vista no se cancele (lyrics-view.js:170-173, 629-630).
17. En la esquina de los botones no salen barra inferior ni cola para no mover los botones bajo el puntero
    (lyrics-view.js:700-701).
18. La cola en FS solo pegado al borde (16 px) porque a 24 px están los botones (lyrics-view.js:637).
19. Now Playing nunca se desplaza (`overflow: clip`): en FS el `scrollIntoView` de YT corría todo a la izquierda
    (lyrics.css:320-324).
20. Atributos en `<html>` en vez de `:has()` sobre toda la página por rendimiento (lyrics.css:23-25;
    lyrics-view.js:720-721).

---

## F. Checklist de réplica

1. Paleta: definir las 21 constantes `--md-*` de A.1 con los RGBA exactos.
2. `--md-dur = 350 ms`, `--md-ease = cubic-bezier(.2,0,0,1)` para todo movimiento conjunto.
3. Ventana sin marco; barra superior de 64 px arrastrable salvo controles; reservar a la derecha el ancho de los
   botones nativos + 8 px.
4. Barra superior no arrastra con la búsqueda enfocada ni en pantalla completa.
5. Atrás en x=64, Adelante en x=104, 36×36, centrados en vertical, chevron 22 px; círculo `--md-state-on` solo
   al hover; desactivado opacidad .35; transición .15 s lineal.
6. Logo desplazado 80 px (40 px en ≤615); en ≤615 ocultar Adelante y cast, logo 26 px.
7. Engranaje 36×36, icono 24, margen `0 6 0 2`, fondo `--md-state-on` al hover o abierto.
8. Panel de ajustes 320 px, padding 6, radio 14, `#1e1e1e`, borde 1px blanco 10 %, sombra 0/16/40 negro 50 %,
   14 px/1.3; entrada 120 ms ease-out desde y −4 px y opacidad 0.
9. Filas: min 36, padding 8/10, radio 8, gap 10, hover blanco 8 %; sub-filas min 30, 6/10, 13 px; sub-bloque
   sangrado 10 + línea izquierda.
10. Kbd, chevron (rota 90° en 150 ms), separador 1 px a sangre, caption 12/500 `--md-text-3`, error `#ff8b8b`
    12 px, hint 12 `--md-text-2`, dim .4.
11. Interruptor 30×18, bola 14 a 2 px, on: pista blanca + bola negra desplazada 12 px (150 ms ease-out); radio
    14 px borde 2, on: punto blanco r=3.
12. Pie: versión 12 px `--md-text-3` izq.; botón 500 13 px der., radio 6, desactivado `--md-text-off`.
13. Píldora "volver" (solo fuera de YT Music): top 14, centrada, alto 36, radio 18, negro 55 % (75 % hover),
    500 14 px, chevron 22, foco negro.
14. Carátula modo canción cuadrada, lado ≤ `alto ventana − reserva` (430 por defecto); vídeo ancho ≤ art·16/9.
15. Mantener modo canción durante `NONE` hasta 2000 ms; luego vídeo. `ATV_PREFERRED` = canción.
16. Fila de título 20 px bajo la carátula, ancho = carátula (o vídeo); título 26/700/−0.02em/lh 1.35 blanco;
    artista 17/500/lh 1.3 blanco 70 %, mt 4; una línea con elipsis.
17. Artista = 1.ª parte del byline por `•`; álbum = 2.ª si hay 3 partes y la 3.ª es año de 4 dígitos.
18. Al cambiar de canción el bloque título/artista entra con fade + subida de 8 px en 400 ms cubic-bezier(.22,.8,.3,1).
19. Tres botones redondos 44 px a bottom 24, right 24 / 80 / 136 (FS, letras, Canción/Vídeo); iconos 22 px trazo
    2 con los paths exactos de B.5.
20. Botón apagado: blanco 14 %, hover blanco 20 %; activo: blanco con icono negro, hover/press oscurecido 10/16 %;
    cambio de color sin transición.
21. Letras desactivado (opacidad .35) en modo vídeo; Canción/Vídeo oculto si la canción no tiene vídeo; icono
    del modo destino.
22. Tooltips/labels i18n exactos de B.5.
23. Persistir `wantOpen` (lyrics:open) y restaurarlo al arrancar.
24. Letra abierta ancha: columna centrada en la mitad izquierda (tx = −W/4), escala `min(cw, W/2−48)/cw`
    alrededor del centro común; área desde x=50 % hasta 40 px del borde, mismo alto que la columna, sin bajar
    de `H − 84`.
25. Estrecha si `fit < 300` o `W/2 − 40 < 400`: miniatura 88 px centrada en (76,76), título en x=138 centrado con
    la miniatura y ancho máx. `W − 170`, área desde y=144 hasta `min(H−32, H−84)`, márgenes laterales 32.
26. Al redimensionar, saltar a la nueva disposición sin transición (2 fotogramas).
27. Abrir: columna 600 ms cubic-bezier(.22,.8,.3,1) inmediata; letra tras 200 ms, fade 450 ms + desliza desde
    +60 px en 600 ms. Cerrar: letra 200 ms ease; columna vuelve tras 200 ms.
28. Máscara vertical del área: transparente 0 → opaco 10 % → opaco 82 % → transparente 100 %.
29. Pista con padding `0 5% 0 6px`.
30. Líneas: YouTube Sans/Roboto 30 px bold, lh 40, padding 12/0 (estrecha 24/32, padding 8), blancas; opacidad
    futura .56, pasada .38, activa 1 con escala 1.04 desde la izquierda; 400 ms ease.
31. Huecos "• • •" con tracking 8 px; generados según lrc.js (vacías → hueco; inicial si primera ≥ 6 s; entre
    líneas separadas ≥ 10 s, a +4 s).
32. Línea activa = última con inicio ≤ t (−1 antes de la primera: todas futuras, referencia línea 0).
33. Línea activa colocada al 30 % del alto del área, con clamp a principio/fin; desplazamiento animado 600 ms
    cubic-bezier(.22,.8,.3,1).
34. Un solo temporizador hasta la siguiente línea (`ceil(Δt·1000/rate) + 15 ms`); recalcular en play, pause,
    seeked, ratechange, loadeddata, durationchange, playing, visibilidad; parar en waiting; corrección en
    timeupdate.
35. Sin trabajo si letra oculta, Now Playing cerrado o ventana oculta.
36. Clic en línea sincronizada → seek a su `t`, cancelar modo manual; cursor mano en líneas sincronizadas.
37. Rueda: desplazamiento manual (×40 si en líneas), animación 150 ms lineal, sin seguir la canción; vuelve tras
    4000 ms de inactividad.
38. Entrada de líneas nuevas: fade desde 0 y +12 px en 500 ms (no con reduced-motion).
39. Cargando: 3 barras de 24 px de alto, radio 8, blanco 14 %, anchos 70/52/62 %.
40. Sin letra: "No hay letra para mostrar aquí :(" (estilo línea, opacidad .6), 48 px arriba.
41. Pedir a los 6 s aunque falten datos; rendirse a los 15 s → sin letra.
42. Sincronizada: crédito final "Letra de LRCLIB" 14/20 `--md-text-3`, margen superior 32, inferior 48.
43. Plana: nota "Esta letra no está sincronizada con la canción" (14/20, `--md-text-3`, mb 16), padding-top 48,
    líneas blancas opacas padding 8, saltos de 32 px, scroll nativo sin barra, crédito = "Letra de LRCLIB" si
    LRCLIB o el texto de fuente de YT.
44. Petición: `{videoId, title, artist, album, duration|null}`; validar videoId `^[\w-]{6,20}$`, recortar a 300.
    Ignorar respuestas de otra canción.
45. Pantalla completa de ventana; Esc sale; al cerrar Now Playing, sale.
46. En FS: carátula `min(alto pantalla − 448, ancho − 96)`; padding superior 80; barras y paneles ocultos
    deslizándose (abajo, arriba, izquierda, derecha) 350 ms --md-ease con opacidad.
47. FS: barra inferior con y > alto − 150 o sobre ella; superior con y < 150, o con menú o cola; menú con
    x < 150; cola con x ≥ ancho − 16 o sobre ella; en la esquina de botones (96 px de margen) no cambian barra
    inferior ni cola.
48. FS: ocultar todo y el cursor a los 3000 ms sin mover, salvo con el puntero sobre barra/panel/menú.
49. FS: botones de la esquina suben 72 px con la barra inferior visible.
50. FS: con menú/cola visibles, escalar Now Playing a `(W − l − r)/W` y desplazar `(l − r)/2`.
51. FS: foco de teclado dentro de barra superior o cola las mantiene visibles; sin tiradores de redimensionar.
52. Transición morph (350 ms) de carátula, título, letra (sin fundido) y botones al entrar/salir de FS y al
    cambiar canción↔vídeo; desactivada con reduced-motion.
53. Responsive: ≤615 sin fila de título/letra/botones; 616–939 apilado con fórmulas de D; ≥940 Now Playing a
    toda la ventana con paneles encima (cola 400 px por defecto, barra reproducción 72, barra superior 64).
54. Now Playing nunca se desplaza (clip); pestaña "Letra" de YT y capa de controles sobre la carátula ocultas.
55. Cola: arrastrar con transición 180 ms ease, elemento arrastrado con fondo blanco 20 %, sombra elevada, radio 8.
56. Fondo de Now Playing: no definido en estos archivos (ver otros documentos).
