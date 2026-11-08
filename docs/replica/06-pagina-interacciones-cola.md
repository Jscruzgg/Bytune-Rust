# 06 — Página: interacciones, ajustes, paneles, cola y API de YouTube Music

Fuentes documentadas (leídas completas):

- `src/preload.js` (922 líneas) — preload de Electron, mundo aislado, con `ipcRenderer`.
- `src/page-api.js` (274 líneas) — se ejecuta en el mundo principal de la página (`webContents.executeJavaScript` desde `main.js`) e instala `window.__md`.
- `src/covers.js` (58 líneas) — mundo principal; marca la barra lateral y pide portadas.
- `src/queue-reorder.js` (144 líneas) — mundo principal; arrastrar y soltar en la cola.
- Tests de `test\`: ninguno prueba directamente estos cuatro archivos. Sirven de apoyo indirecto `test\shortcut.test.js` (traducción de la pulsación que hace `settings:key`) y `test\library.test.js` (`idFromUrl`, que usa la URL que manda `opened`). Los textos de la interfaz salen de `src\lib\i18n.js` (consultado solo para dar los textos literales de las claves).

Importante: casi todo el **aspecto** (colores, tamaños, sombras, transiciones CSS) de las clases `md-*` y atributos `data-md-*` que se describen aquí está en `theme.css` / `chrome.css` / `lyrics.css`, que **no** forman parte de este documento. Aquí se documenta lo que hace el JS: qué nodos crea, qué clases y atributos pone, qué variables CSS escribe, qué eventos escucha y qué animaciones lanza por JS. Cuando el comentario del autor dice qué hace el CSS, se indica como "según el comentario".

Notación: `preload.js:310` = línea 310 de ese archivo.

---

## A. `src/preload.js`

### A.0 Ciclo de vida y bucle principal

- Constante `MAIN_BAR = 'ytmusic-player-bar[slot="player-bar"]'` (preload.js:5). Motivo (preload.js:4): YT a veces crea una segunda `ytmusic-player-bar` oculta; la visible es la del slot `player-bar`. Todas las consultas de la barra usan este selector.
- Al `DOMContentLoaded` (preload.js:919-922): llama a `ensureHomeButton()` una vez y arranca `setInterval(read, 1000)` → **la función `read()` se ejecuta cada 1000 ms** durante toda la vida de la página.
- Además `read()` se ejecuta inmediatamente en los eventos `play` y `pause` del `<video>` (preload.js:887-890), para que la bandeja y los botones de la barra de tareas cambien de icono sin el retraso de hasta un segundo.
- Orden exacto de cada pasada de `read()` (preload.js:864-917):
  1. `reportLang()`
  2. `reportOpened()`
  3. `ensureBackButton()`
  4. `ensureHomeButton()`
  5. `ensureGear()`
  6. `markLongTitle()`
  7. `applyCovers()`
  8. `updateResizers()`
  9. `watchNavRight()`
  10. `watchQueue()`
  11. Si no hay `MAIN_BAR`, termina aquí. Si la hay: `arrangeBar(bar)`, lectura de canción, enganche al vídeo, `syncTime()`, envío `track`, mejora de miniaturas y `--md-art`.
- Patrón general: casi todo es **idempotente** y **solo escribe al DOM si el valor cambia** (comparaciones antes de `setAttribute`, `toggleAttribute`, `style.setProperty`). Hay que replicar ese "solo si cambia" porque el autor lo usa para no forzar recálculos de estilo.

### A.1 Canción actual, play/pausa (IPC `track`)

preload.js:875-902.

- `title` = `textContent` de `MAIN_BAR .title` (o `''`).
- `byline` = `textContent` de `MAIN_BAR .byline`. Formato "Artista • Álbum • Año": se toma **solo el primer tramo** antes de `•` (`byline.split('•')[0]`) (preload.js:879-881).
- `video` = primer `document.querySelector('video')`.
- `playing = !!(video && !video.paused && !video.ended)` (preload.js:896).
- Objeto `info = { title: title.trim(), artist: artist.trim(), playing }`. Se serializa con `JSON.stringify` y **solo se envía `ipcRenderer.send('track', info)` si cambió** respecto al último (`last`) (preload.js:898-902).
- Enganche al vídeo (preload.js:883-891): si el `<video>` encontrado es distinto del ya enganchado (`boundVideo`), se le añaden (todos `passive: true`):
  - `timeupdate` → `syncTime`
  - `durationchange` → `syncTime`
  - `play` → `read`
  - `pause` → `read`
  No se quitan del vídeo anterior (no hace falta: el anterior deja de existir o se ignora).

### A.2 Tiempo transcurrido / total (`data-md-elapsed`, `data-md-total`)

`syncTime()` preload.js:12-24, llamado en `timeupdate`, `durationchange` y en cada `read()` (preload.js:892).

- Lee `MAIN_BAR .time-info` (texto tipo `"1:23 / 3:45"`) y `MAIN_BAR #progress-bar`. Si falta alguno, no hace nada.
- Solo actúa si el texto (trim) cambió respecto a `lastTime` (≈ una vez por segundo).
- `split('/')` y `trim` de cada parte → `elapsed` (por defecto `''`) y `total` (por defecto `''`).
- Escribe en `#progress-bar` los atributos `data-md-elapsed` y `data-md-total`. Según el comentario (preload.js:12), `theme.css` los pinta con `attr()` **a los lados de la barra de progreso** (izquierda transcurrido, derecha total).

### A.3 Reordenación de la barra del reproductor (`arrangeBar`)

preload.js:26-50. Se ejecuta en cada `read()` con barra.

Principio (preload.js:26-29): se reordena **moviendo nodos**, no con posiciones calculadas, para que el flujo propio de YT reparta el espacio y nada se pise a ningún tamaño de ventana. Los nodos movidos siguen siendo los de YT (YT los sigue controlando). Idempotente: si YT repinta, la siguiente pasada lo recoloca.

1. **Menú "⋮"**: `bar .middle-controls ytmusic-menu-renderer` se mueve con `appendChild` al final de `bar #left-controls` (la columna central de botones; "al final de la fila de botones, sobre el tiempo total") (preload.js:37-39). Se re-ejecuta cada segundo (appendChild de un nodo que ya es último hijo no cambia nada visible).
2. **Volumen** (preload.js:40-49): dentro de `bar #right-controls .right-controls-buttons` se busca `.volume` (el botón) y en toda la barra `#volume-slider`. Si existen ambos y el padre del botón **no** tiene ya la clase `md-volume`, se crea `<span class="md-volume">`, se inserta antes del botón (`volume.before(wrap)`) y se meten dentro botón y deslizador (`wrap.append(volume, slider)`). Intención: el deslizador pasa a un **panel flotante sobre su botón** (lo dibuja el CSS), así no reserva 100 px vacíos en la barra.
3. **Píldora "me gusta" NO se mueve** (preload.js:30-32): si sale de su contenedor original, YT trata sus clics como clics en la barra (abre Now Playing y el "me gusta" deja de responder). Se queda donde está y `theme.css` la dibuja junto al volumen con anclaje CSS (anchor positioning).

Para la réplica nativa: el orden visual resultante de la barra es: columna central con los botones de transporte y al final el menú "⋮"; a la derecha un botón de volumen cuyo deslizador aparece flotante encima; el "me gusta" junto al volumen.

### A.4 Textos (IPC `strings`)

preload.js:52-71.

- `strings` = objeto `{ clave: texto }` que manda `main.js` por IPC `'strings'` (`msg.strings`). El preload en sandbox no puede cargar `src/lib/i18n.js` (preload.js:52).
- `txt(key)` devuelve `strings[key]` o, si no existe, **la propia clave** (preload.js:71).
- Al recibir `strings` (preload.js:60-70): re-etiqueta los botones atrás/adelante si existen, el botón `.md-home` si existe, el engranaje si existe (`labelGear`) y, si el panel de ajustes está abierto, lo redibuja (`renderSettings`).
- `labelNav(btn, key)` (preload.js:54-59): si hay texto, pone `title` y `aria-label` (solo si difieren).

Textos usados (de `src/lib/i18n.js`, ES / EN):

| Clave | ES | EN |
|---|---|---|
| `nav.back` | Atrás | Back |
| `nav.forward` | Adelante | Forward |
| `nav.home` | Volver a YouTube Music | Back to YouTube Music |
| `set.title` | Ajustes | Settings |
| `set.quickMenu` | Menú rápido | Quick menu |
| `set.shortcut` | Atajo | Shortcut |
| `set.shortcutHint` | Pulsa la nueva combinación de teclas | Press the new key combination |
| `set.shortcutTaken` | Otro programa ya usa ese atajo | Another program already uses that shortcut |
| `set.shortcutInvalid` | Usa al menos Ctrl, Alt, Shift o Win más otra tecla | Use at least Ctrl, Alt, Shift or Win plus another key |
| `set.shortcutReserved` | ByTune ya usa ese atajo para otra cosa | ByTune already uses that shortcut for something else |
| `set.works` | Funciona | Works |
| `set.focused` | Con la app enfocada | When the app is focused |
| `set.anywhere` | En cualquier lugar | Anywhere |
| `set.fromOtherApps` | Desde otras apps | From other apps |
| `set.float` | Flotando sobre la app | Floating over the app |
| `set.front` | Trayendo la ventana al frente | Bringing the window to the front |
| `set.shiftEnter` | Shift + Enter en una canción | Shift + Enter on a song |
| `set.queue` | Añadir al final de la cola | Add to the end of the queue |
| `set.next` | Reproducir a continuación | Play next |
| `set.theme` | Tema ByTune | ByTune theme |
| `set.notify` | Notificar cambio de canción | Notify on song change |
| `set.closeToTray` | Cerrar a la bandeja | Close to tray |
| `set.startWithWindows` | Iniciar con Windows | Start with Windows |
| `upd.check` | Buscar actualizaciones | Check for updates |
| `upd.checking` | Buscando… | Checking… |
| `upd.latest` | Ya tienes la última versión | You're up to date |
| `upd.unsupported` | Solo en la versión instalada | Installed version only |
| `upd.error` | No se pudo comprobar | Couldn't check |

Nota: las etiquetas `aria-label` de los tiradores de redimensionado **no** pasan por i18n: están fijas en español en el código (ver A.9).

### A.5 Botones "atrás" y "adelante" en la barra superior

preload.js:72-100. Llamado en cada `read()`.

- YT Music no los trae; se añaden "como en Spotify". YT navega con el historial del navegador, así que basta `history.back()` / `history.forward()`.
- `NAV_BUTTONS = [['md-back', 'nav.back'], ['md-forward', 'nav.forward']]` (preload.js:75).
- `ensureBackButton()`: busca `ytmusic-nav-bar`; para cada par, si no existe un hijo directo `:scope > .md-back` / `.md-forward`, crea `<button type="button" class="md-back">` (o `md-forward`), con `click` → `history.back()` o `history.forward()`, y lo **añade al final** de `ytmusic-nav-bar` (`appendChild`) (preload.js:87-96). La posición visual final la decide el CSS (no está en este archivo).
- Solo al crear `md-back`, si existe `window.navigation` (Navigation API), se suscribe a `currententrychange` → `updateNav(nav)` (preload.js:95).
- Etiqueta: `title` y `aria-label` = `nav.back` / `nav.forward`. Los botones no tienen texto ni icono en JS (el icono lo pone el CSS).
- `updateNav(nav)` (preload.js:76-83): habilitado/deshabilitado:
  - atrás: `navigation.canGoBack` si existe la API; si no, `history.length > 1`.
  - adelante: `navigation.canGoForward` si existe la API; si no, **siempre `false`** (deshabilitado).
  - Solo escribe `disabled` si cambia.
  - Se llama en cada `ensureBackButton()` (cada segundo) y en cada `currententrychange`.

### A.6 Botón "volver a YouTube Music" (páginas de Google)

preload.js:102-123. Llamado en `DOMContentLoaded` y en cada `read()`.

- Solo si `location.hostname !== 'music.youtube.com'` y existe `document.body` (p. ej. inicio de sesión de Google, consentimiento). Allí no existe la barra de YT, así que no hay botón atrás ni salida (preload.js:102-103).
- Crea una vez `<button type="button" class="md-home">YouTube Music</button>` como hijo directo de `body` (`body > .md-home`); texto visible literal **"YouTube Music"**; `title`/`aria-label` = `nav.home`.
- Clic → `location.assign('https://music.youtube.com/')`.
- Según el comentario (preload.js:104): flota **centrado arriba** (estilo en `chrome.css`), lejos del formulario, del logo y de los botones de la ventana.

### A.7 Engranaje y menú de ajustes dentro de la página

preload.js:125-357.

#### A.7.1 Botón engranaje

- `ensureGear()` (preload.js:145-160), en cada `read()`:
  - Contenedor: `ytmusic-nav-bar #right-content`. Si no existe, nada.
  - Crea una vez `<button type="button" class="md-gear" aria-haspopup="true" aria-expanded="false">`. Clic → si el panel está abierto lo cierra, si no lo abre.
  - Posición: **justo antes del avatar** (`:scope > ytmusic-settings-button`), es decir, entre "enviar a dispositivo" (cast) y el avatar (preload.js:126). Sin sesión iniciada no hay avatar y va **al final** de `#right-content`. Solo se mueve si no está ya en su sitio (`gear.nextElementSibling !== avatar` o `gear.parentElement !== right`).
  - `title` y `aria-label` = `txt('set.title')` ("Ajustes"); solo se escriben si cambian.
  - El icono no está en JS (CSS).

#### A.7.2 Estado

Variables (preload.js:130-138):

- `panel`: el `div.md-settings` mientras está abierto, si no `null`.
- `state`: última respuesta de main (`settings:get` / `settings:set` / `settings:changed`). Forma deducida del uso:
  - `state.quickMenu = { reach: 'focused'|'global', fromOtherApps: 'float'|'front', shiftEnter: 'queue'|'next', accelerator? }`
  - `state.display.shortcut` — texto del atajo para mostrar (p. ej. `"Ctrl + Shift + Space"`, ver `displayAccelerator` en `test\shortcut.test.js:58-61`).
  - `state.theme`, `state.notifications`, `state.closeToTray`, `state.startWithWindows` — booleanos.
  - `state.version` — cadena de versión.
- `expanded` (false): sub-bloque del menú rápido desplegado.
- `recordingKey` (false): grabando un atajo.
- `shortcutError` (`''`): `'taken' | 'invalid' | 'reserved'`. Mapa `SHORTCUT_ERRORS` → claves `set.shortcutTaken` / `set.shortcutInvalid` / `set.shortcutReserved`; cualquier otro valor usa `set.shortcutInvalid` (preload.js:215).
- `updateStatus` (`''`): `'' | 'checking' | 'installing' | 'latest' | 'unsupported' | 'error'`. `UPDATE_LABELS`: `''`→`upd.check`, `checking`→`upd.checking`, `installing`→`upd.checking` (mismo texto "Buscando…"), `latest`→`upd.latest`, `unsupported`→`upd.unsupported`, `error`→`upd.error` (preload.js:138).

Coste cero cerrado (preload.js:129): los oyentes de clic fuera, de tamaño y de blur solo existen con el panel abierto.

#### A.7.3 Apertura, colocación y cierre

- `openSettings()` (preload.js:297-314), asíncrono:
  1. Si ya hay panel, nada.
  2. `await ipcRenderer.invoke('settings:get')` (error → `null`). Si no hay respuesta, o mientras tanto se abrió un panel, o el engranaje ya no existe/no está conectado, aborta.
  3. `state = res`; `expanded = false` (siempre se abre **plegado**); `shortcutError = ''`; si `updateStatus !== 'checking'` se resetea a `''` (una comprobación en curso conserva su estado "Buscando…").
  4. Crea `<div class="md-settings" role="dialog">`, lo rellena (`renderSettings`), lo añade **al final de `body`** y lo coloca (`placeSettings`).
  5. `gear.aria-expanded = "true"`.
  6. Añade: `document` `pointerdown` en **captura** → `onOutside`; `window` `resize` → `closeSettings`; `window` `blur` → `closeSettings`.
- `placeSettings()` (preload.js:285-290): ancho supuesto **320 px** (fijo en JS; el ancho real lo pone el CSS). `top = round(rect.bottom del engranaje + 8)` px; `left = round(max(8, min(rect.right − 320, innerWidth − 320 − 8)))` px. Es decir: bajo el engranaje con 8 px de separación, **alineado por la derecha** con el engranaje y sin salirse de la ventana (margen mínimo 8 px a cada lado). Se escribe en `panel.style.top/left` (posición presumiblemente `fixed` en CSS).
- `onOutside(e)` (preload.js:292-295): con `e.composedPath()`; si el camino no incluye ni el panel ni el engranaje → cierra. (El clic en el engranaje lo maneja su propio `click`, que alterna.)
- `closeSettings()` (preload.js:315-324): si hay panel: `stopRecording()`, quita los tres oyentes, `panel.remove()`, `panel = null`, `gear.aria-expanded = "false"`.
- Cierres: clic/pulsación fuera, redimensionar la ventana, perder el foco la ventana (`blur`), Esc, volver a pulsar el engranaje.
- No hay animación de apertura/cierre en JS (si la hay es CSS; no está en este archivo).

#### A.7.4 Estructura del panel (`renderSettings`)

preload.js:195-253. Se redibuja **entero** cada vez (`panel.replaceChildren(...)`); antes guarda el `data-key` del elemento enfocado dentro del panel y después le devuelve el foco (preload.js:197-198, 252). `panel.aria-label = txt('set.title')`.

Constructores:

- `el(tag, cls, text)` (preload.js:162-167).
- `row(key, label, onClick, extra)` (preload.js:169-177): `<button type="button" class="md-set-row" data-key="{key}">` con `<span class="md-set-label">{label}</span>` y opcionalmente `extra` a la derecha.
- `radio(key, label, on, disabled, onClick)` (preload.js:178-186): una `row` con clase añadida `md-set-radio`, `role="radio"`, `aria-checked="true|false"`, con **antepuesto** `<span class="md-rad">` (o `md-rad on` si marcado) — el círculo va a la izquierda de la etiqueta — y `disabled`.
- `group(label, dim, ...rows)` (preload.js:187-193): `<div class="md-set-group">` (más `md-dim` si `dim`), `role="radiogroup"`, `aria-label={label}`, con `<div class="md-set-cap">{label}</div>` (título del grupo) seguido de las filas.
- `kbd(text)` → `<kbd class="md-kbd">{text}</kbd>`.

Orden exacto de nodos dentro de `div.md-settings`:

1. **Cabecera "Menú rápido"** — `row('qm', 'Menú rápido', …, right)`, `aria-expanded = expanded`.
   - `right` = `<span class="md-set-r">` con: si **plegado**, `<kbd class="md-kbd">{state.display.shortcut}</kbd>`; y siempre `<span class="md-chev">` (con clase `open` si desplegado).
   - Clic: `expanded = !expanded`, `stopRecording()`, `shortcutError = ''`, redibuja.
2. **Si desplegado**: `<div class="md-set-sub">` con, en orden:
   1. Fila **"Atajo"** — `row('shortcut', 'Atajo', …, value)`, clase `recording` si se está grabando.
      - `value`: grabando → `<span class="md-set-hint">Pulsa la nueva combinación de teclas</span>`; si no → `<span class="md-set-r"><kbd class="md-kbd">{state.display.shortcut}</kbd></span>`.
      - Clic: alterna grabar / dejar de grabar; redibuja.
   2. Si hay `shortcutError`: `<div class="md-set-err" role="alert">{texto del error}</div>`.
   3. Grupo **"Funciona"** (no atenuado): radios `reach-focused` "Con la app enfocada" (marcado si `reach !== 'global'`) y `reach-global` "En cualquier lugar" (marcado si `reach === 'global'`). Clic → `settings:set { quickMenu: { reach: 'focused'|'global' } }`.
   4. Grupo **"Desde otras apps"**, **atenuado (`md-dim`) y con radios deshabilitados si `reach !== 'global'`**: `from-float` "Flotando sobre la app" (marcado si `fromOtherApps === 'float'`), `from-front` "Trayendo la ventana al frente" (marcado si `=== 'front'`). Clic → `{ quickMenu: { fromOtherApps } }`.
   5. Grupo **"Shift + Enter en una canción"**: `shift-queue` "Añadir al final de la cola" (`shiftEnter === 'queue'`), `shift-next` "Reproducir a continuación" (`=== 'next'`). Clic → `{ quickMenu: { shiftEnter } }`.
3. `<div class="md-set-sep">` (separador).
4. Cuatro **interruptores**, en este orden, cada uno `row(key, label, …, <span class="md-sw">` o `md-sw on`) con `role="switch"` y `aria-checked`:
   - `theme` — "Tema ByTune"
   - `notifications` — "Notificar cambio de canción"
   - `closeToTray` — "Cerrar a la bandeja"
   - `startWithWindows` — "Iniciar con Windows"
   Clic → `settings:set { [key]: !state[key] }`.
5. `<div class="md-set-sep">`.
6. **Pie** `<div class="md-set-foot">`: `<span class="md-set-ver">ByTune {state.version}</span>` y `<button type="button" class="md-set-btn" data-key="update">{texto según updateStatus}</button>`, deshabilitado si `updateStatus` es `checking` o `installing`. Clic → `checkUpdates`.

#### A.7.5 Cambios de ajustes e IPC

- `changeSettings(patch)` (preload.js:255-261): `invoke('settings:set', patch)`. Respuesta con `error` → `shortcutError = res.error` (el estado no cambia). Respuesta válida → `state = res`, `shortcutError = ''`. Siempre redibuja. Errores de IPC se ignoran.
- `settings:changed` (preload.js:326-330): cuando cambian desde la bandeja, main manda el estado nuevo; se guarda y, si el panel está abierto, se redibuja.
- `checkUpdates()` (preload.js:263-271): `updateStatus = 'checking'`, redibuja, `invoke('update:check')`; error → `'error'`; respuesta fuera de `UPDATE_LABELS` → `'error'`. Con `'installing'` la app se cierra (comentario preload.js:263).

#### A.7.6 Grabación del atajo y teclado del panel

- `startRecording()` (preload.js:273-277): `recordingKey = true`, `shortcutError = ''`, `send('settings:recording', true)` (main deja de atender sus atajos mientras tanto).
- `stopRecording()` (preload.js:278-282): solo si estaba grabando: `recordingKey = false`, `send('settings:recording', false)`.
- Oyente de teclado (preload.js:332-357): `window.addEventListener('keydown', …, true)` — en **captura sobre window** y registrado al cargar el preload (antes que los scripts de YT), para ir primero. Solo actúa si el panel está abierto:
  - **Grabando**: `preventDefault()` + `stopImmediatePropagation()` en **todas** las teclas (ninguna llega a YouTube).
    - `Escape` **sin** Ctrl/Alt/Shift/Meta → cancela la grabación y redibuja. Con modificadores (p. ej. Ctrl+Mayús+Esc) se graba como cualquier otra (preload.js:339-340).
    - `e.repeat` → ignora.
    - Envía `{ key, code, ctrlKey, altKey, shiftKey, metaKey }` con `invoke('settings:key', ev)`; main traduce (`src/lib/shortcut.js`). Respuesta `null` = solo modificadores → sigue grabando. Respuesta acelerador (y aún grabando) → `stopRecording()` y `changeSettings({ quickMenu: { accelerator: acc } })`. Si main lo rechaza, llega `{ error: 'taken'|'invalid'|'reserved' }` y se muestra el error.
    - Traducción (confirmada por `test\shortcut.test.js`): formato Electron con orden fijo `CommandOrControl+Alt+Shift+Super+Tecla`; se prefiere `code` sobre `key` (teclado español: Mayús+3 → `3`; `ñ` en `Semicolon` → `;`; numpad → `num1`); `+` → `Plus`; flechas → `Up/Down/Left/Right`. Texto mostrado: `Ctrl + Shift + Space`, `Win + Alt + K`. Validación: debe haber modificador + tecla; hay 3 atajos reservados (p. ej. `CommandOrControl+Alt+M`).
  - **No grabando**: `Escape` → `preventDefault`, `stopImmediatePropagation`, cierra el panel y **devuelve el foco al engranaje**.
  - Otras teclas sin grabar: no se interceptan.

### A.8 Portadas de playlists en la barra lateral

preload.js:359-380 (junto con `covers.js`, ver B.3).

- IPC `'covers'`: `main.js` manda el mapa `{ browseId: url }`; se guarda y se aplica (preload.js:363). También se aplica en cada `read()`.
- `applyCovers()`: para cada `ytmusic-guide-entry-renderer[data-md-id]` (el atributo lo pone `covers.js`), toma `url = covers[data-md-id]` y su `tp-yt-paper-item`. Si falta alguno, salta. Si no existe ya un hijo directo `img.md-cover`, crea `<img class="md-cover" alt="" loading="lazy" decoding="async">` y lo **antepone** (`prepend`) dentro del `tp-yt-paper-item` (queda a la izquierda del texto, "como Spotify"). Solo cambia `src` si el atributo `src` actual es distinto.
- Solo toca el DOM si falta la miniatura o cambió su portada.

### A.9 Redimensionado de la barra lateral (izquierda) y de la cola (derecha)

preload.js:398-594.

#### A.9.1 Límites y variables

`PANELS` (preload.js:409-412):

| Lado | min (px) | max (px) | Variable CSS | `aria-label` (fijo, sin i18n) |
|---|---|---|---|---|
| `left` (barra lateral) | 240 | 420 | `--md-left-w` | "Ajustar ancho de la barra lateral" |
| `right` (cola / panel lateral de Now Playing) | 280 | 480 | `--md-right-w` | "Ajustar ancho de la cola" |

- `clampPanel(side, w) = round(min(max, max(min, w)))` (preload.js:426).
- En reposo el ancho es una variable CSS (preload.js:400-402):
  - `--md-left-w` se escribe en `<html>` (`style`). De ella sale `--ytmusic-guide-width` (en `theme.css`), que YT usa en toda la página.
  - `--md-right-w` se escribe en `ytmusic-player-page` y en el tirador `.md-resize-right` (solo se usa ahí).
  - Sin valor guardado se quita la propiedad (`removeProperty`) → el CSS usa su ancho por defecto.
- Ancho efectivo izquierdo: además del max 420 hay un tope dependiente de la ventana, `--md-left-eff` en `theme.css`, replicado en JS: `min(w, max(240, innerWidth/2 − 164))` (preload.js:483-484 y 608). Es decir, la barra lateral nunca pasa de `50vw − 164 px` (pero nunca baja de 240 por esa regla).

#### A.9.2 Persistencia

- `main.js` guarda los anchos y los manda al cargar por IPC `'panels'` (`{ left?: number, right?: number }`); si no es objeto se usa `{}`. Se aplican con `applyPanelWidths()` (preload.js:414, 425).
- `savePanels()` → `ipcRenderer.send('panels', panelWidths)` (objeto completo) (preload.js:427).
- Se guarda: al soltar un arrastre (si hubo movimiento), en doble clic (restablecer) y en cada pulsación de flecha.

#### A.9.3 Tiradores (`makeResizer`)

preload.js:512-576. Creados una vez cada uno desde `updateResizers()`.

- `<div class="md-resize md-resize-left">` / `md-resize-right`, `tabIndex = 0` (enfocable), `role="separator"`, `aria-orientation="vertical"`, `aria-label` según tabla; añadido al final de `body`. Tras crearlo se escribe su variable (`writePanelVar`).
- Posición/aspecto: CSS. En vivo, el izquierdo recibe `left: calc({px} − 4px)` (preload.js:489) → el tirador está centrado en el borde (presumiblemente 8 px de ancho; no confirmado en JS).

**Arrastre con ratón/puntero:**

1. `pointerdown` en el tirador (preload.js:549-565): solo botón principal (`button === 0`); `preventDefault()`. Si quedaba un arrastre sin cerrar (`cover`), lo cierra antes (`onUp()`). `setPointerCapture(pointerId)` (errores ignorados).
2. Crea una **capa** `<div class="md-resize-cover">` al final de `body`: cubre toda la ventana para que el cursor sea `col-resize` en todas partes y no se seleccione nada, **sin** tocar el estilo de la página (comentario preload.js:555-556: la alternativa `html.md-resizing *` recalculaba la página entera al empezar y al acabar).
3. Contexto `ctx = { pp: npOpen(), contentWidth }`, donde `contentWidth = contentWidthRule()` solo si es el lado izquierdo y Now Playing no está abierto. `liveProps = new Map()`.
4. `window` `pointermove` → `onMove`; `window` `pointerup`, `pointercancel`, `lostpointercapture` en **captura** → `onUp`.
5. `onMove` (preload.js:525-532): `pending = clientX` (izquierda) o `innerWidth − clientX` (derecha). Como máximo **un cálculo por fotograma** (`requestAnimationFrame`): aplica `liveLeft` o `liveRight` con `clampPanel(side, pending)`.
6. `onUp` (preload.js:537-548): quita oyentes, suelta la captura, cancela el rAF pendiente. Si hubo movimiento (`pending !== null`): `setPanel(side, pending)` (clamp + escribe variable si cambió) y `savePanels()`. Luego `liveClear()` (quita todos los estilos provisionales) y quita la capa. Variable definitiva y limpieza **en la misma tarea**: un solo recálculo grande.
   - Caso límite resuelto (preload.js:533-535): fin del arrastre por **cualquier** camino (soltar, cancelar o perder la captura), escuchado en `window`. Con solo `pointerup` en el tirador, una captura perdida dejaba la capa del cursor encima de toda la ventana, y la capa tapaba el tirador: no había forma de salir.
   - Un clic sin mover (incluido cada clic del doble clic) no cambia nada: `pending` sigue `null`.

**Doble clic** (preload.js:566): `delete panelWidths[side]`, `writePanelVar(side)` (quita la variable → ancho por defecto del CSS) y `savePanels()`. Sin animación.

**Teclado** (preload.js:567-573), con foco en el tirador: `ArrowLeft` / `ArrowRight` (`preventDefault`). Crece si (`ArrowRight` y lado izquierdo) o (`ArrowLeft` y lado derecho) — es decir, la flecha mueve el borde en esa dirección. Paso **16 px**: `setPanel(side, currentPanelWidth(side) ± 16)` y `savePanels()`.
- `currentPanelWidth(side)` (preload.js:434-438): el valor guardado si existe; si no, el ancho medido de `#guide-wrapper` (izquierda) o `ytmusic-player-page #side-panel` (derecha); si no existe, el `min`.

#### A.9.4 Redimensionado en vivo ("el contenido se reacomoda mientras se arrastra")

Contexto de rendimiento (preload.js:398-406): cambiar una variable en `<html>` recalcula los ~6000 elementos visibles (~50 ms): en cada fotograma daba 11–24 fps. Por eso, **durante el arrastre la variable no se toca**: el ancho se escribe solo en los elementos que lo usan (~70 elementos por fotograma en Now Playing). Al soltar se escribe la variable y se quita todo lo provisional.

No hay animación con duración: el contenido **sigue al puntero en vivo** (un ancho por fotograma). El comentario no habla de una animación "content slides"; lo que existe es este reflujo en vivo.

- `liveSet(el, prop, value, important)` (preload.js:448-453): `el.style.setProperty(prop, value, important ? 'important' : '')` y apunta el par para borrarlo. `liveClear()` (preload.js:454-458): borra todas.
- Propiedades registradas no heredables (preload.js:440-445), `syntax: '*'`, `inherits: false`: `--md-live-pad-l`, `--md-live-pad-r`, `--md-live-side`. Motivo: las reglas de las hojas con `!important` (margen de Now Playing, ancho de la cola) ganan a un estilo en línea; leen estas variables, que no se heredan (cambiarlas solo recalcula ese elemento). Sin valor, cada regla usa el de siempre.
- `npOpen()` (preload.js:478-481): devuelve `ytmusic-player-page` si existe `ytmusic-app-layout[player-page-open]`, la página **no** tiene `is-mweb-modernization-enabled` y `matchMedia('(min-width: 940px)')`; si no, `null`.
- `NP_ART` (preload.js:477): `min(calc(var(--md-vh, 100vh) - 430px), calc(100vw - var(--ytmusic-guide-width, 0px) - var(--md-np-side) - 96px))` — tamaño de la carátula de Now Playing, redeclarado en `#main-panel`.
- `liveLeft(w, ctx)` (preload.js:482-500), con `px = min(w, max(240, innerWidth/2 − 164)) + 'px'`:
  - `tp-yt-app-drawer#guide` → `width: px !important`
  - `#guide-wrapper` → `width: px !important`
  - `ytmusic-nav-bar` → `--ytmusic-guide-width: px` (~50 elementos: buscador y logo)
  - `.md-resize-left` → `left: calc(px − 4px)`
  - Si Now Playing abierto: `ytmusic-player-page > .content` → `--md-live-pad-l: px`; `#main-panel` → `--ytmusic-guide-width: px` y `--md-np-art: NP_ART`.
  - Si no: `ytmusic-app-layout > #content` → `--ytmusic-guide-width: px` y, si se encontró, `--ytmusic-content-width: {fórmula}`.
- `contentWidthRule()` (preload.js:459-474): `--ytmusic-content-width` (ancho de las filas de carátulas) la declara YT en `ytmusic-app` con `--ytmusic-guide-width` dentro, con una fórmula por tamaño de ventana. Se recorren todas las hojas (`document.styleSheets`, ignorando las que lanzan por CORS), entrando en reglas `@media`, y se toma la última regla con `selectorText === 'ytmusic-app'` que declare esa variable y cuya media (si la hay) coincida. Se copia para declararla en la página con el ancho nuevo.
- `liveRight(w, ctx)` (preload.js:501-510): solo si Now Playing abierto; `px = w + 'px'`:
  - `#side-panel` → `--md-live-side: px`
  - `ytmusic-player-page > .content` → `--md-live-pad-r: px`
  - `#main-panel` → `--md-np-side: px` y `--md-np-art: NP_ART`
  - `.md-resize-right` → `--md-right-w: px`

#### A.9.5 Visibilidad de los tiradores (`updateResizers`)

preload.js:577-594, cada segundo.

- Crea los tiradores si faltan.
- `guideOpen = ytmusic-app existe && no tiene guide-collapsed && no tiene is-hidden-guide-page && innerWidth >= 936`.
- `queueOpen = ytmusic-app-layout existe && tiene player-page-open && innerWidth >= 940`.
- Escribe en `<html>` los atributos booleanos `data-md-guide-open` y `data-md-queue-open` (solo si cambian). El CSS muestra solo los tiradores de los paneles visibles.
- Now Playing puede crearse después de recibir los anchos: si `ytmusic-player-page` existe y su `--md-right-w` en línea no coincide con el guardado, se reescribe.

Observación: la barra lateral **plegada** (72 px) no muestra tirador (porque `guideOpen` es falso).

### A.10 Plegar/desplegar la barra lateral (animación FLIP)

preload.js:596-725. Es la animación JS más elaborada.

#### A.10.1 Por qué (preload.js:596-603)

Al plegar, YT cambia `--ytmusic-guide-width` (heredada por ~13 000 elementos): un recálculo de ~120 ms en Now Playing. Animar esa variable lo repetía en cada fotograma; una View Transition congelaba la pantalla ~200 ms. Solución FLIP: en el mismo fotograma del cambio (antes de pintar), lo que se movió vuelve con `transform` a donde estaba y desde ahí llega a su sitio con un muelle; el panel se recorta con la forma de la tarjeta. Nada de capturas: todo sigue vivo. Tras el cambio no se lee ninguna medida que obligue a recalcular (lo del panel se lee antes y su ancho final se calcula).

#### A.10.2 Constantes y temporización

- `MOTION = { duration: 350, easing: 'cubic-bezier(.2, 0, 0, 1)' }` (preload.js:604) — fallback.
- `GUIDE_MIN = 72` px — ancho plegada (preload.js:605).
- `GUIDE_INSET = 8` px — de x = 0 a la tarjeta (`--md-gap`); las filas empiezan ahí (preload.js:643).
- `guideTiming()` (preload.js:612-616): si existe `globalThis.Motion` (librería Motion que `main.js` inyecta en este mundo), usa `Motion.spring(0.28, 0.08)` → su `String()` es `"{ms}ms {easing linear(...)}"`, se divide por `'ms '` → `{ duration: ms, easing: 'linear(...)' }`. Es un muelle de duración visual 0,28 s y rebote 0,08 ("corto y con poco rebote: el recorte no debe pasarse de los 72 px y comerse los iconos"). Motion lo convierte en una curva `linear()` que el navegador anima solo en GPU. Sin Motion: 350 ms `cubic-bezier(.2,0,0,1)`. (La duración exacta en ms del muelle la calcula Motion; no está en el código.)
- `guideOpenWidth()` (preload.js:608): `min(parseFloat(html.style --md-left-w) || 240, max(240, innerWidth/2 − 164))` — ancho desplegado sin medir.
- Propiedad registrada `--md-guide-w`: `syntax '<length-percentage>'`, `inherits: false`, `initialValue '100%'` (preload.js:640-642). Ancho de las filas mientras se pliega; fuera de la animación vale 100 %. Solo cambia en `ytmusic-guide-renderer`.

#### A.10.3 Secuencia (oyente `click` en `document`, captura, preload.js:653-725)

1. Detecta clic en `ytmusic-nav-bar #guide-button` (botón hamburguesa) salvo si `guidePassthrough` (clic reenviado por nosotros).
2. **Sin animación** (deja pasar el clic tal cual a YT) si: no es ese botón, falta `ytmusic-app` o `#guide-wrapper`, `document.hidden` (con la ventana oculta las animaciones no avanzan ni acaban), `innerWidth < 936` (en ventanas estrechas YT usa `#mini-guide` y el menú flotante), o `prefers-reduced-motion: reduce`.
3. `e.stopImmediatePropagation()` — YT no recibe este clic todavía.
4. `guideDone()` — si había una animación anterior a medias, acaba en su sitio.
5. Lee: `was = app.hasAttribute('guide-collapsed')`; `timing`; `from = ancho medido de #guide-wrapper`; `to = was ? guideOpenWidth() : 72`.
6. Lee el estilo calculado de `#guide-wrapper::before` (la tarjeta): `top`, `height`, `left`, `borderTopRightRadius`. `bottom = top + height`. Forma del recorte: `rect({top} {w}px {bottom}px {left} round {radio})` — la tarjeta no cambia al plegar; solo se mueve su borde derecho, con las esquinas redondeadas todo el rato.
7. `targets = guideTargets()` y sus rects iniciales (`first`). Solo con Now Playing abierto: `ytmusic-player-page #main-panel > #player` (carátula; "box": se traslada y escala), `#main-panel > #md-np-info` (título) y `#md-lyrics` (letra), solo si tienen `offsetWidth`. Todos con `transform`.
8. Si se va a **plegar** (`!was`): `<html data-md-guide-closing>` — según el comentario, el panel sigue ancho por dentro hasta que el recorte llega a 72 px (theme.css).
9. Instala un `MutationObserver` sobre `ytmusic-app`, `attributeFilter: ['guide-collapsed']`: cuando el atributo cambia respecto a `was`, se desconecta y llama a `play()`. Las mutaciones se notifican en la misma tarea → antes de pintar.
10. `queueMicrotask`: `guidePassthrough = true; btn.click(); guidePassthrough = false;` — reenvía el clic a YT al acabar este clic (un `click()` dentro de otro `click()` programático sobre el mismo botón no hace nada, de ahí la microtarea).
11. Plazo de seguridad **500 ms** (`setTimeout`): si YT no llegó a cambiar el atributo, se desconecta el observador y se quita `data-md-guide-closing`. `guideWait` guarda cómo cancelar observador y plazo; otro clic antes lo cancela (`guideDone`). Bug evitado (preload.js:716-718): sin cancelarlo, el plazo quitaba `data-md-guide-closing` a mitad del pliegue siguiente y la tarjeta se quedaba sin ancho (sin fondo) hasta el final.

`play()` (preload.js:676-701), todas con `timing`:

- **Recorte** del panel: `#guide-wrapper.animate({ clipPath: [rect(from), rect(to)] }, { fill: 'forwards' })`. Al terminar (`finished`) → `guideDone()`.
- **Tarjeta** (`::before`): `wide = max(from, to)`; anima `right` de `{wide − from}px` a `{wide − to}px` con `pseudoElement: '::before'`, `fill: 'forwards'`. Si no, la tarjeta se quedaba ancha y el recorte se comía su borde derecho. El panel mide lo del estado ancho.
- `#guide-wrapper[data-md-guide-moving]`: mientras dura, sin barra de desplazamiento (plegada no la hay; estrechaba 10 px las playlists).
- **Filas**: `ytmusic-guide-renderer.animate({ '--md-guide-w': [`${from − 8}px`, `${to − 8}px`] }, { fill: 'forwards' })` — el resaltado, la línea y la píldora "Nueva lista" se estrechan al mismo paso (la píldora acaba en el círculo) y al final no hay nada que recolocar.
- Si hay `targets` (Now Playing): `document.dispatchEvent(new Event('md-measure'))` para que `lyrics-view.js` coloque ya carátula y letra en su sitio nuevo (su ResizeObserver llegaría tarde); se leen los rects nuevos y se anima cada uno desde `guideFrom(t, first, last)` hasta `none` (sin `fill`):
  - Escala propia `s = parseFloat(getComputedStyle(el).scale) || 1` (la de `lyrics.css`).
  - No "box" (título, letra): `transform: translate((a.left − b.left)/s px, (a.top − b.top)/s px)` → `none`.
  - "box" (carátula, escala desde su centro): `dx = (centroX_a − centroX_b)/s`, `dy = (centroY_a − centroY_b)/s`, `transform: translate(dx px, dy px) scale(a.width / b.width)` → `none`.
  - Rama `translate` (para elementos sin `transform: true`: `translate: "{dx}px 0"` → `0 0`) existe en `guideFrom` pero **ningún target la usa** en el código actual. El comentario de `guideTargets` (preload.js:617-619) dice que también se mueve "el contenido de la página (translate…)", pero `guideTargets()` solo devuelve elementos de Now Playing. Discrepancia comentario/código: fuera de Now Playing solo se animan recorte, tarjeta y filas.
- `guideDone()` (preload.js:647-652): cancela el plazo/observador pendiente, quita `data-md-guide-closing` de `<html>` y `data-md-guide-moving` de `#guide-wrapper`, y **cancela** todas las animaciones guardadas (con `fill: forwards`, cancelarlas devuelve el estilo real, que ya es el final).

### A.11 Espacio a la derecha de la barra superior (`--md-nav-right`)

preload.js:727-754.

- `measureNavRight()`: rects de todos los hijos de `ytmusic-nav-bar #right-content` con ancho > 2 px; `--md-nav-right = ceil(innerWidth − min(left) + 16) px` en `<html>` (solo si cambia). Incluye los iconos (transmitir, ajustes/engranaje, avatar) y el espacio de los botones de ventana; `theme.css` coloca el buscador para no pisarlos.
- Se mide: en `window` `resize` (pasivo, throttled a un rAF), con un `ResizeObserver` sobre `#right-content` (se reinstala si el nodo cambia) y en cada `read()`.

### A.12 Títulos largos (`data-md-long`)

preload.js:756-763. En cada `read()`: `ytmusic-responsive-header-renderer > h1 .title`; si el texto (trim) tiene **más de 24 caracteres**, atributo `data-md-long` (solo si cambia). El CSS lo muestra más pequeño (como Spotify) en vez de dos líneas gigantes.

### A.13 Idioma (IPC `lang`)

preload.js:765-773. En cada `read()`: `document.documentElement.lang`; si no está vacío y cambió → `send('lang', code)`.

### A.14 URL abierta (IPC `opened`)

preload.js:775-781. En cada `read()`: si `location.href` cambió → `send('opened', location.href)`. Sirve para los Recientes del menú rápido; `main` convierte la URL con `idFromUrl` (`test\library.test.js:55-62`: `/playlist?list=PL1` → `VLPL1`; `/browse/MPREb_x` → `MPREb_x`; otras → `null`). Detección por sondeo de 1 s (no hay evento de navegación).

### A.15 Cola de Now Playing: alineación a la canción actual, "lock", recorte de filtros

preload.js:783-862.

- Constantes: `QUEUE_LOCK = 160` px, `QUEUE_GAP = 8` px (aire sobre la canción actual).
- Contenedor con scroll: `ytmusic-player-page #side-panel #tab-renderer` (`queueScroller`).
- `queueAnchor(sc)` (preload.js:792-800): la canción actual visible = primer `ytmusic-player-queue-item[selected]` con `offsetHeight` (YT marca también la versión oculta canción/vídeo de alto 0). `top` de los filtros = `offsetHeight` de `ytmusic-player-queue > #steering-chips` (fijos arriba). Ancla = `max(0, round(sel.top − sc.top + sc.scrollTop − chips − 8))`.
- `alignQueue()` (preload.js:801-808): solo si hay scroller, `<html data-md-np-open>` (lo pone `lyrics-view.js`) y el scroller tiene alto. Si hay ancla: `queuePending = false` y, si la diferencia > 1 px, `scrollTo({ top: ancla, behavior: 'smooth' })`. Comportamiento: **al pasar de canción, la actual se desliza hasta arriba** y las anteriores quedan ocultas encima.
- `watchQueue()` (preload.js:828-862), cada `read()`:
  - Una vez: `MutationObserver` en `<html>` sobre `data-md-np-open`; si hay alineación pendiente, `setTimeout(alignQueue, 150)` (si cambió la canción con Now Playing cerrado). Se instala aquí y no al cargar el preload porque entonces `<html>` aún no existe.
  - Al encontrar un scroller nuevo:
    - `scroll` (pasivo): apunta `lastScroll = performance.now()` y `clipQueue(sc)`.
    - `scrollend` (pasivo): reinicia un temporizador de **250 ms**; al vencer, si hubo scroll hace < 240 ms, no hace nada. Si no: con `a = ancla`, si `a !== null && scrollTop > 0 && scrollTop < a && (a − scrollTop) < min(160, a/2)` → `scrollTo({ top: a, behavior: 'smooth' })`.
      - Es un "lock" **solo hacia arriba**: si al soltar la lista queda poco por encima de la actual (< 160 px) vuelve sola; más arriba se queda (se ven las anteriores) y al volver bajando se imanta en el mismo sitio (la condición es la misma en ambos casos: estar por encima del ancla y a menos del umbral al terminar de desplazar; no depende de la dirección del último gesto). Hacia abajo (scrollTop ≥ ancla) no hace nada.
      - Casos límite (comentarios preload.js:785-787, 838-839, 848-849): `scroll-snap` nativo no sirve (imanta en los dos sentidos y no dejaba bajar un poco para ver las siguientes). `scrollend` llega también entre golpes de rueda seguidos: se esperan 250 ms sin scroll; si no, devolvía la lista tras el primer golpe y no dejaba subir. Con la actual cerca del principio, subir del todo la devolvía siempre: en el tope (`scrollTop = 0`) no actúa y el umbral nunca pasa de la mitad de lo que hay encima (`a/2`).
    - `MutationObserver` sobre el scroller (`attributes`, `attributeFilter ['selected']`, `subtree`): si alguna mutación es de un elemento que **ahora** tiene `selected` → `queuePending = true` y `setTimeout(alignQueue, 150)` (se alinea cuando YT termina de colocar la suya).
    - `queuePending = true` y `alignQueue()` inmediato.
- `clipQueue(sc)` (preload.js:810-827): los filtros (`#steering-chips`) van pegados arriba (sticky, CSS) y la lista pasa por debajo. Se recorta cada lista `ytmusic-player-queue > #contents` y `#automix-contents` con `clip-path: inset({cut}px 0 0 0)`, donde `cut = max(0, round(bottomFiltros − topLista))`, **solo si los filtros están pegados** al borde superior (`chips.top − sc.top < 1`); si no, `cut = 0` y se quita el `clip-path`. Así no se transparenta nada y la barra comparte el fondo de la tarjeta. Caso límite: YT a veces pone los filtros después de las canciones (mezcla con reproducción automática) y, sin esta comprobación, el recorte se comía la lista entera. Solo escribe si cambia (`WeakMap`).

### A.16 Miniaturas nítidas y fondo difuminado (`--md-art`)

preload.js:903-916, cada `read()`:

- YT pone miniaturas a 60 px; en pantallas con escalado se ven borrosas. Las URL de Google aceptan otro tamaño en el sufijo `=wNN-hNN`:
  - Miniatura de la barra (`MAIN_BAR .middle-controls img`): `=w60-h60` → `=w240-h240`.
  - Miniaturas de la cola (`ytmusic-player-queue-item img[src*="=w60-h60"]`, que el tema muestra a 40 px): → `=w120-h120`.
  - Las ya cambiadas dejan de coincidir con el selector (idempotente).
- `--md-art` en `<html>`: `url("{src}")` de la miniatura de la barra si existe y no es `data:`; si no `none`. Solo si cambia. `theme.css` lo usa para el **fondo difuminado**.

### A.17 Puente de letras (`md-lyrics`)

preload.js:382-396. `lyrics-view.js` corre en la página sin `ipcRenderer`; deja un JSON en `<html data-md-lyrics-msg>` y lanza el evento `md-lyrics` en `document`. El preload lo lee, **lo borra** y solo reenvía lo esperado:

- `{type:'request', track:{…}}` → `send('lyrics:request', track)` (pedir la letra).
- `{type:'open', open:boolean}` → `send('lyrics:open', open)` (recordar el panel abierto).
- `{type:'fullscreen', on:boolean}` → `send('np:fullscreen', on)` (pantalla completa).
- JSON inválido o tipos incorrectos → se ignora.

### A.18 Resumen de IPC del preload

| Dirección | Canal | Contenido |
|---|---|---|
| main → página | `strings` | `{ strings: {clave: texto} }` |
| main → página | `settings:changed` | estado de ajustes |
| main → página | `covers` | `{ browseId: url }` |
| main → página | `panels` | `{ left?, right? }` |
| página → main (invoke) | `settings:get` | → estado |
| página → main (invoke) | `settings:set` | parche → estado o `{error}` |
| página → main (invoke) | `settings:key` | evento de tecla → acelerador o `null` |
| página → main (invoke) | `update:check` | → `'latest'|'unsupported'|'error'|'installing'` |
| página → main (send) | `settings:recording` | `true/false` |
| página → main (send) | `panels` | `{ left?, right? }` |
| página → main (send) | `lang` | código de idioma |
| página → main (send) | `opened` | `location.href` |
| página → main (send) | `track` | `{ title, artist, playing }` |
| página → main (send) | `lyrics:request` / `lyrics:open` / `np:fullscreen` | ver A.17 |

### A.19 Resumen de clases/atributos/variables que añade el preload

| Elemento | Qué | Para qué |
|---|---|---|
| `#progress-bar` | `data-md-elapsed`, `data-md-total` | tiempos a los lados de la barra |
| `span.md-volume` | envoltorio de `.volume` + `#volume-slider` | panel flotante de volumen |
| `ytmusic-nav-bar > button.md-back / .md-forward` | botones nuevos, `disabled` | atrás / adelante |
| `body > button.md-home` | botón nuevo | salir de páginas de Google |
| `#right-content > button.md-gear` | botón nuevo, `aria-expanded` | abrir ajustes |
| `body > div.md-settings` y descendientes `md-set-*`, `md-rad`, `md-sw`, `md-kbd`, `md-chev`, `md-dim`, `recording`, `on`, `open` | panel | ajustes |
| `tp-yt-paper-item > img.md-cover` | imagen nueva | portada de playlist |
| `body > div.md-resize.md-resize-left/right` | tiradores | redimensionado |
| `body > div.md-resize-cover` | capa temporal | cursor durante arrastre |
| `<html>` `data-md-guide-open`, `data-md-queue-open` | atributos | mostrar tiradores |
| `<html>` `data-md-guide-closing` | atributo temporal | pliegue de la barra |
| `#guide-wrapper` `data-md-guide-moving` | atributo temporal | sin scrollbar durante pliegue |
| `h1 .title` `data-md-long` | atributo | título largo más pequeño |
| `<html>` style `--md-left-w`, `--md-nav-right`, `--md-art` | variables | ancho barra, buscador, fondo |
| `ytmusic-player-page`, `.md-resize-right` style `--md-right-w` | variable | ancho cola |
| `#contents`, `#automix-contents` style `clip-path` | recorte | lista bajo filtros pegados |
| Propiedades registradas | `--md-live-pad-l/-r`, `--md-live-side` (`*`), `--md-guide-w` (`<length-percentage>`, 100 %) | arrastre y pliegue |

---

## B. `src/page-api.js` y `src/covers.js`

### B.1 `page-api.js`: infraestructura

Se ejecuta en el mundo principal (tiene `ytcfg`, cookies de sesión, elementos de YT). Se puede ejecutar varias veces: solo (re)instala `window.__md` (page-api.js:1-3). Todos los métodos devuelven promesa y, **salvo `browse`**, nunca lanzan: resuelven `{ error: mensaje }` (envoltorio `safe`, page-api.js:52-55).

**Petición autenticada `api(path, body, { auth = 'required' })`** (page-api.js:24-49):

- Requiere `window.ytcfg.get`; si no → error `'sin ytcfg'`.
- Cookie `SAPISID` o, si no, `__Secure-3PAPISID`. Sin ella y `auth !== 'optional'` → error `'sin sesión'`.
- Cabeceras: `Content-Type: application/json`, `X-Origin: location.origin` (`https://music.youtube.com`).
- Con cookie: `ts = floor(Date.now()/1000)`; `hash = hex(SHA-1("{ts} {SAPISID} {origin}"))`; `Authorization: SAPISIDHASH {ts}_{hash}`; `X-Goog-AuthUser: ytcfg.get('SESSION_INDEX') || 0`.
- `POST /youtubei/v1/{path}?prettyPrint=false&key={ytcfg.get('INNERTUBE_API_KEY')}`, `credentials: 'include'`, cuerpo `JSON.stringify({ context: ytcfg.get('INNERTUBE_CONTEXT'), ...body })`.
- HTTP no ok → error `'HTTP {status}'`. Devuelve el JSON.
- Para la réplica nativa: hacen falta las cookies de la sesión, `INNERTUBE_API_KEY`, `INNERTUBE_CONTEXT` (cliente WEB_REMIX con idioma/región) y `SESSION_INDEX`, que en el original salen de `ytcfg` de la página.

**Utilidades de parseo:**

- `walk(o, visit)`: recorrido en profundidad de todo el JSON (preorden).
- `text(t)`: concatena `t.runs[].text`.
- `pageTypeOf(run)`: `run.navigationEndpoint.browseEndpoint.browseEndpointContextSupportedConfigs.browseEndpointContextMusicConfig.pageType`.
- `segments(t)`: divide los `runs` de un subtítulo por los runs cuyo texto (trim) es `•`; devuelve tramos no vacíos `{ runs, text(trim) }` (page-api.js:69-78). Ej.: "Álbum • Artista • 2020", "Artista y Otro • Álbum • 3:45".
- `artistOf(segs, fallback)`: el tramo con algún run cuyo `pageType === 'MUSIC_PAGE_TYPE_ARTIST'` (incluye " y ", ", "); si no, `fallback`.
- `coverOf(thumbs)`: de las miniaturas con `url`, la **primera con `width >= 120`** (la más pequeña que siga nítida en la fila del menú, asumiendo orden ascendente); si no hay, la última (la mayor); si no, `''`.
- `nextToken(json)`: último `nextContinuationData.continuation` o `continuationCommand.token` encontrado.
- Constantes: `SONGS_PARAMS = 'EgWKAQIIAWoSEAQQBRADEAkQChAVEBAQDhAR'` (filtro "Canciones" de la búsqueda; capturado del chip "Canciones" de YT), `PAGE_TYPE = { playlist: 'MUSIC_PAGE_TYPE_PLAYLIST', album: 'MUSIC_PAGE_TYPE_ALBUM' }`, `MAX_PAGES = 10`.

### B.2 `page-api.js`: métodos de `window.__md`

| Método | Endpoint / acción | Devuelve |
|---|---|---|
| `browse(body)` | `POST browse` (auth requerida); **lanza** si falla | JSON crudo (lo usa `covers.js`) |
| `library()` | `browse` `FEmusic_liked_playlists` y `FEmusic_liked_albums` + continuaciones | `{ items: LibraryItem[] }` o `{ error }` |
| `searchSongs(q)` | `search` con `{ query, params: SONGS_PARAMS }` | `{ songs: Song[] }` (≤ 5) |
| `suggestions()` | `browse` `FEmusic_home` | `{ songs: Song[] }` (≤ 5) |
| `play(videoId)` | `ytmusic-app.handleNavigationEndpoint({watchEndpoint})` | `{ ok: true }` |
| `queue(videoId, 'end'|'next')` | `queueAddEndpoint` vía `ytmusic-menu-service-item-renderer.handleCommand` | `{ ok: true }` |
| `open(browseId)` | `handleNavigationEndpoint({browseEndpoint})` | `{ ok: true }` |
| `lyrics(videoId)` | `next` + `browse` `MPLYt…` (auth opcional) | `{ lines, source }` o `{ none: true }` |

**`library()`** (page-api.js:99-149) — biblioteca para el menú rápido:

- Por cada sección (`FEmusic_liked_playlists`, luego `FEmusic_liked_albums`): `browse({ browseId })`, luego hasta **10** páginas `browse({ continuation: token })`. Cada sección en su propio `try`: si falla una (o una página intermedia), se devuelve lo conseguido.
- En cada respuesta, cada `musicTwoRowItemRenderer` → `libraryItem`; se deduplica por `id` (Set compartido) y se mantiene el orden de llegada (playlists primero, luego álbumes).
- `libraryItem(item)`: `id = item.navigationEndpoint.browseEndpoint.browseId`; tipo por prefijo: `VL…` → `playlist`, `MPRE…` → `album`, otro → descartado (p. ej. la tarjeta "Nueva lista"). `title = text(item.title).trim()` (vacío → descartado). `cover = coverOf(item.thumbnailRenderer.musicThumbnailRenderer.thumbnail.thumbnails)`.
  - Solo álbumes: `artist = artistOf(segments(subtitle), segs[1].text si hay ≥ 2 tramos)` ("Álbum • Artista • 2020": sin enlace de artista se usa el tramo del medio); se omite si vacío. `playlistId` (el `OLAK5uy_…` al que YT redirige con `/playlist?list=…`, para reconocerlo en Recientes): de `item.thumbnailOverlay.musicItemThumbnailOverlayRenderer.content.musicPlayButtonRenderer.playNavigationEndpoint.(watchEndpoint|watchPlaylistEndpoint).playlistId`; si no, el primer `watchPlaylistEndpoint.playlistId` que empiece por `OLAK` dentro de `item.menu`. Solo se guarda si es cadena y casa con `/^[\w-]+$/`.
- Sin ningún ítem → `{ error: primerError || 'biblioteca vacía' }`.
- `LibraryItem = { id, type, title, artist?, cover, playlistId? }`.

**`searchSongs(q)`** (page-api.js:177-181) — búsqueda del menú rápido: `query = String(q ?? '').trim()`; vacía → `{ songs: [] }` sin petición. Si no, `api('search', { query, params: SONGS_PARAMS })` → `firstSongs`.

**`suggestions()`** (page-api.js:183-184): `browse({ browseId: 'FEmusic_home' })` → `firstSongs` (la estantería "Selección rápida" es la primera con filas de canción).

**`firstSongs(json)`** (page-api.js:165-175): recorre en orden y toma las **5 primeras** `musicResponsiveListItemRenderer` distintas (por `videoId`) que sean canción.

**`songOf(r)`** (page-api.js:151-163):

- `cols[i] = r.flexColumns[i].musicResponsiveListItemFlexColumnRenderer.text`.
- `videoId = r.playlistItemData.videoId` o `cols[0].runs[0].navigationEndpoint.watchEndpoint.videoId`; sin él → no es canción.
- `title = text(cols[0]).trim()`.
- `segs = segments(cols[1])`; `artist = artistOf(segs, segs[0].text || '')`.
- `duration`: de los textos de los tramos de `cols[1]` más los `fixedColumns` (`musicResponsiveListItemFixedColumnRenderer.text`), recorridos **de atrás adelante**, el primero que case con `/^\d+(:\d\d)+$/` (p. ej. `3:45`, `1:02:03`); si no, `''`.
- `cover = coverOf(r.thumbnail.musicThumbnailRenderer.thumbnail.thumbnails)`.
- `Song = { videoId, title, artist, duration, cover }`.

**Acciones** (pasan por el manejo de comandos de YT, sin tocar `location`: no se recarga la página y la reproducción sigue, page-api.js:186-187):

- `app()`: `ytmusic-app` con `handleNavigationEndpoint` función; si no, error `'sin ytmusic-app'`.
- `checkVideoId`: cadena de exactamente **11** caracteres `[\w-]`; si no, `'videoId inválido'`.
- `play(videoId)`: `handleNavigationEndpoint({ watchEndpoint: { videoId } })` — como pulsar la canción en la búsqueda: sustituye la cola por esa canción más la radio automática.
- `queue(videoId, position)`: `position` `'end'` → `INSERT_AT_END`, `'next'` → `INSERT_AFTER_CURRENT_VIDEO`; otro → `'posición inválida'`. Crea un `<ytmusic-menu-service-item-renderer hidden>` dentro de `ytmusic-app` (sin `handleCommand` → `'sin menú de YT'`), `await item.handleCommand({ queueAddEndpoint: { queueTarget: { videoId, onEmptyQueue: { watchEndpoint: { videoId } } }, queueInsertPosition } })` y lo quita siempre (`finally`). Con la cola vacía YT reproduce la canción (`onEmptyQueue`).
- `open(browseId)`: `/^VL[\w-]+$/` → playlist, `/^MPRE[\w-]+$/` → álbum; otro → `'browseId inválido'`. `handleNavigationEndpoint({ browseEndpoint: { browseId, browseEndpointContextSupportedConfigs: { browseEndpointContextMusicConfig: { pageType } } } })`.

**`lyrics(videoId)`** (page-api.js:244-261) — no necesita sesión (`auth: 'optional'`):

1. `api('next', { videoId })`; busca el primer `browseEndpoint.browseId` que empiece por `MPLYt` (pestaña "Letra"). Sin él → `{ none: true }`.
2. `api('browse', { browseId })`; primer `musicDescriptionShelfRenderer`; `body = text(shelf.description)`. Vacío → `{ none: true }`.
3. `{ lines: body.split('\n'), source: text(shelf.footer).trim() }` — se conservan las líneas vacías (separan estrofas); `source` es el crédito ("Fuente: …").

### B.3 `src/covers.js`: portadas de la barra lateral

Mundo principal; `main.js` lo ejecuta (después de `page-api.js`) y usa el resultado para el IPC `covers` (A.8).

1. **Marcado de la barra lateral** (covers.js:6-25), instalado una sola vez (`window.__mdGuideTagger`):
   - `tag()`: para cada `ytmusic-guide-entry-renderer`, lee la propiedad JS `e.data.navigationEndpoint.browseEndpoint.browseId`; si empieza por `VL` y difiere, escribe `data-md-id`. Motivo: el id solo está en propiedades JS de YT, que el preload (mundo aislado) no ve; los atributos sí.
   - `MutationObserver` sobre `ytmusic-app` (o `body`), `childList + subtree`, agrupado a **un `tag()` por fotograma** (`requestAnimationFrame` con bandera `queued`). Sigue marcando cuando YT repinta. Más un `tag()` inmediato.
2. **Petición** (covers.js:27-57): requiere `window.__md.browse`; si no → `{ error: 'sin __md' }`.
   - `browse({ browseId: 'FEmusic_liked_playlists' })` (la misma petición que la página "Biblioteca") y hasta **10** continuaciones.
   - `scan`: para cada `musicTwoRowItemRenderer` con `browseId`, miniatura = **primera con `width >= 60`**, o la última; `covers[browseId] = url` (incluye cualquier browseId, no solo `VL`, pero solo las `VL` se marcan en la barra). Token siguiente igual que `nextToken`.
   - Error: si no se consiguió ninguna portada → `{ error: e.message }`; si hay alguna, se devuelve lo conseguido.
   - Resultado `{ covers: { browseId: url } }`.

Cuándo y cada cuánto lo ejecuta `main.js`: no está en estos archivos (lo cubre otro documento).

---

## C. `src/queue-reorder.js`: arrastrar y soltar en la cola

Mundo principal; `main.js` lo inyecta junto a `lyrics-view.js`. Si ya existía una instancia, la destruye antes (`window.__mdQueueReorder.destroy()`, queue-reorder.js:11).

### C.1 Modelo

- La cola de YT vive en su store Redux: `ytmusic-player-queue.queue.store` (`getState().queue` con `items` y `automixItems`).
- Canciones "A continuación" (sin atributo `is-automix`): se mueven con `dispatch({ type: 'MOVE_ITEM', payload: { fromIndex, toIndex } })`, como el propio YT.
- Canciones de **reproducción automática** (`is-automix`): se reordena una copia de `automixItems` (`splice` quitar en `from`, insertar en `to`) y `dispatch({ type: 'REPLACE_AUTOMIX_ITEMS', payload: { automixItems: list, shouldAssignIds: false } })`.
- Cada canción se mueve **solo dentro de su lista** (no se puede pasar de automix a la cola ni al revés).
- La siguiente canción sale ya en el nuevo orden.
- Límite reconocido (queue-reorder.js:9-10): YT también avisa a sus servidores de la cola y no hay forma de pedirlo desde aquí; si YT recarga la cola del servidor, puede recuperar el orden anterior.

### C.2 Inicio (`pointerdown`)

queue-reorder.js:21-38.

- Solo botón principal y destino `Element`.
- Debe estar dentro de un `ytmusic-player-queue-item` y **no** dentro de `CONTROLS = 'ytmusic-menu-renderer, yt-icon-button, button, a, .left-items'` (menú "⋮", botones, enlaces y la miniatura `.left-items`, cuyo clic reproduce, quedan como en YT). Se arrastra pulsando **el texto** de la fila.
- Debe existir el store con estado de cola.
- `items` = todas las filas de la misma lista (`[is-automix]` o `:not([is-automix])`) dentro de esa `ytmusic-player-queue`, en orden DOM; `from` = índice.
- Solo si lo que se ve es toda la lista de YT en su orden: `list.length === items.length` (si no, el índice sería otro).
- Fila normal con `el.data.canReorder === false` → no arrastrable.
- Crea `drag = { store, automix, items, from, to: from, startY: clientY, el, started: false }`. No hay `preventDefault` aquí.

### C.3 Umbral y arranque

- `THRESHOLD = 6` px de movimiento **vertical** (`|clientY − startY|`) para que el clic pase a ser arrastre (queue-reorder.js:14, 71-74).
- `start()` (queue-reorder.js:59-65): `started = true`; guarda los rects de todas las filas (`rects`) y `h` = alto de la fila arrastrada; `<html data-md-q-dragging>`; clase `md-q-drag` en la fila arrastrada.
- Aspecto del arrastre (sombra, opacidad, cursor, transiciones de las demás filas): **no está en este JS**; lo definen las hojas de estilo a partir de `data-md-q-dragging` y `.md-q-drag` (no documentadas aquí; ver documento del tema). El JS solo escribe `transform`.

### C.4 Movimiento (`pointermove`)

queue-reorder.js:67-91.

- `dy = clientY − startY`, acotado para que la fila **no salga de la lista**: `y = clamp(dy, rects[0].top − rects[from].top, rects[last].top − rects[from].top)`.
- Fila arrastrada: `style.transform = translateY(y px)` (sigue al puntero 1:1, sin rAF).
- Destino: el índice `i` cuyo rect original contiene el **centro** de la fila arrastrada (`center = rects[from].top + h_from/2 + y`, `top <= center < bottom`); si ninguno, se mantiene el anterior.
- Si cambió `to`, las filas de en medio **se apartan un hueco** (`h` = alto de la arrastrada):
  - Bajando (`from < to`): filas `from < i <= to` → `translateY(−h)`.
  - Subiendo (`from > to`): filas `to <= i < from` → `translateY(+h)`.
  - Resto: `transform = ''`.
- **No hay línea indicadora**: el indicador es el propio hueco que dejan las filas desplazadas.
- **No hay auto-scroll** de la lista durante el arrastre (no existe código para ello). Tampoco se actualizan los rects si la lista se desplaza durante el arrastre.
- Durante el arrastre solo se mueven transformaciones (queue-reorder.js:7-8).

### C.5 Fin

queue-reorder.js:93-110.

- `pointerup` → `end(true)` (aplicar). `pointercancel` → `end(false)`. `Escape` con arrastre iniciado → `end(false)` (cancelar).
- `end(apply)`: `drag = null`; si no había empezado (clic simple), nada más. Si había empezado: `swallowClick = apply` y `setTimeout(() => swallowClick = false, 0)`; quita `transform` de todas las filas, quita `md-q-drag` y `data-md-q-dragging`; si `apply` y `to !== from` → `move(d)` (errores ignorados). El cambio del DOM lo hace YT al procesar la acción del store.
- Animación al soltar: en JS no hay ninguna; las filas vuelven a `transform: ''` y YT repinta en el nuevo orden (si hay transición, sería CSS).
- Caso límite (queue-reorder.js:98-100): el clic que genera el navegador al soltar llega justo después, en la misma tarea, y debe tragarse (si no, reproduciría la canción); si no llega (se soltó sobre otra canción), el `setTimeout 0` evita que se pierda el siguiente clic del usuario.

### C.6 Clic en la fila = reproducir

queue-reorder.js:111-126.

- Si `swallowClick`: se traga (`stopPropagation` + `preventDefault`).
- Si no, clic en una fila fuera de `CONTROLS`: si `play-button-state` de la fila es `playing` o `paused` (la canción actual), no hace nada (ese botón la pausaría). Si no, `.click()` en su `ytmusic-play-button-renderer` (en la miniatura). Así un clic **en cualquier parte** de la fila la reproduce (YT solo lo hace desde la miniatura).

### C.7 Aislar a YT de los eventos

queue-reorder.js:49-57, 129-136.

- Todos los oyentes en `window`, **captura**, `passive: false` (llegan antes que cualquier oyente de YT, también los de `document`): `pointerdown`, `pointermove`, `pointerup`, `pointercancel`, `click`, `keydown`, y `own` para `mousedown`, `mousemove`, `mouseup`, `touchstart`, `touchmove`.
- `own(e)`: mientras hay una fila pulsada (`drag` no nulo, aunque aún no haya arrancado), `stopPropagation()`; si ya arrancó, además `preventDefault()`. `onPointerMove` también hace `stopPropagation()` mientras hay `drag`.
- Motivo (queue-reorder.js:49-52): con una canción pulsada, YT no recibe sus eventos de ratón: ni su propio arrastre de la cola (que con el tema no funciona y, sin él, movía las canciones a la vez) ni su gesto de **minimizar Now Playing arrastrando hacia abajo**. La miniatura queda fuera (onPointerDown no la coge), así que su clic sigue reproduciendo.
- `destroy()`: `end(false)` y quita todos los oyentes.

---

## D. Animaciones e interacciones JS (inventario completo)

| # | Qué | Dónde | Propiedad(es) | Duración / easing | Notas |
|---|---|---|---|---|---|
| 1 | Recorte del panel lateral al plegar/desplegar | preload.js:678 | `clip-path: rect(top {from→to}px bottom left round r)` | muelle Motion `spring(0.28, 0.08)` → `linear()`; fallback 350 ms `cubic-bezier(.2,0,0,1)` | `fill: forwards`; al terminar `guideDone` |
| 2 | Tarjeta `::before` de la barra lateral | preload.js:686 | `right: {wide−from}px → {wide−to}px` | igual que 1 | `pseudoElement: '::before'`, `fill: forwards` |
| 3 | Ancho de filas de la barra lateral | preload.js:690 | `--md-guide-w: {from−8}px → {to−8}px` | igual que 1 | en `ytmusic-guide-renderer`, `fill: forwards` |
| 4 | Carátula de Now Playing (FLIP) | preload.js:695-700 | `transform: translate(dx,dy) scale(wA/wB) → none` | igual que 1 | dx/dy entre centros, divididos por `scale` de lyrics.css |
| 5 | Título (`#md-np-info`) y letra (`#md-lyrics`) (FLIP) | preload.js:695-700 | `transform: translate(dx,dy) → none` | igual que 1 | divididos por `scale` |
| 6 | Alineación de la cola a la canción actual | preload.js:807 | `scrollTop` | `behavior: 'smooth'` (del navegador) | tras 150 ms del cambio de `selected` o de abrir NP |
| 7 | "Lock" de la cola | preload.js:851 | `scrollTop` | `smooth` | 250 ms tras `scrollend`, sin scroll en 240 ms |
| 8 | Redimensionado en vivo | preload.js:482-532 | anchos/variables | sin duración: 1 actualización por fotograma (rAF) | se limpia al soltar |
| 9 | Arrastre en la cola | queue-reorder.js:78, 86-90 | `transform: translateY` | sin duración en JS | transiciones, si existen, en CSS |

Sin animación (deliberadamente): pliegue de la barra lateral si la ventana está oculta, si `innerWidth < 936` o con `prefers-reduced-motion: reduce`. Panel de ajustes: sin animación JS. Doble clic de restablecer ancho: salto instantáneo.

Temporizadores y observadores (inventario):

| Tipo | Intervalo / objetivo | Dónde |
|---|---|---|
| `setInterval(read)` | 1000 ms | preload.js:921 |
| `setTimeout` plazo del pliegue | 500 ms | preload.js:719-723 |
| `setTimeout(alignQueue)` | 150 ms | preload.js:832, 858 |
| `setTimeout` lock de la cola | 250 ms (y exige 240 ms sin scroll) | preload.js:846-852 |
| `setTimeout` liberar `swallowClick` | 0 ms | queue-reorder.js:101 |
| `requestAnimationFrame` | arrastre de tiradores; `--md-nav-right`; tagger de covers.js | preload.js:527, 744; covers.js:21 |
| `MutationObserver` | `ytmusic-app` `guide-collapsed` (temporal por clic) | preload.js:703-708 |
| `MutationObserver` | `<html>` `data-md-np-open` | preload.js:832-833 |
| `MutationObserver` | scroller de la cola, `selected`, subtree | preload.js:855-859 |
| `MutationObserver` | `ytmusic-app` (o body) `childList` subtree | covers.js:18-23 |
| `ResizeObserver` | `ytmusic-nav-bar #right-content` | preload.js:751 |

Eventos de ratón/teclado (inventario):

| Evento | Destino | Fase | Acción |
|---|---|---|---|
| `click` | `.md-back` / `.md-forward` | burbuja | `history.back/forward` |
| `click` | `.md-home` | burbuja | ir a music.youtube.com |
| `click` | `.md-gear` | burbuja | abrir/cerrar ajustes |
| `click` | filas `.md-set-row` / `.md-set-btn` | burbuja | ver A.7.4 |
| `pointerdown` | `document` | captura | (solo con panel) cerrar si fuera |
| `resize`, `blur` | `window` | — | (solo con panel) cerrar |
| `keydown` | `window` | captura | panel: grabar atajo / Esc |
| `pointerdown`, `dblclick`, `keydown` | tiradores | burbuja | redimensionar / restablecer / ±16 px |
| `pointermove` | `window` | burbuja | (arrastrando) ancho en vivo |
| `pointerup`, `pointercancel`, `lostpointercapture` | `window` | captura | (arrastrando) fin |
| `click` | `document` | captura | pliegue animado de la barra lateral |
| `resize` | `window` | pasivo | medir `--md-nav-right` |
| `scroll`, `scrollend` | scroller de la cola | pasivo | recorte y lock |
| `md-lyrics` | `document` | — | puente de letras |
| `timeupdate`, `durationchange`, `play`, `pause` | `<video>` | pasivo | tiempos y `read()` |
| `currententrychange` | `window.navigation` | — | habilitar atrás/adelante |
| `pointer*`, `click`, `keydown`, `mouse*`, `touchstart/move` | `window` | captura | reordenar la cola (C) |

Atajos de teclado dentro de la página definidos en estos archivos: solo Esc (cerrar ajustes / cancelar grabación / cancelar arrastre de la cola) y ← / → sobre un tirador enfocado. No hay más atajos aquí.

---

## E. Comentarios del autor: intenciones y casos límite resueltos

1. Hay dos `ytmusic-player-bar`; la buena es `[slot="player-bar"]` (preload.js:4).
2. La barra se reordena moviendo nodos, no con posiciones, para que nada se pise a ningún tamaño (preload.js:26-29).
3. El "me gusta" no se puede mover: fuera de su contenedor YT abre Now Playing y el "me gusta" no responde (preload.js:30-32).
4. El volumen pasa a panel flotante para no reservar 100 px vacíos (preload.js:40-41).
5. El preload en sandbox no puede cargar i18n: los textos llegan por IPC (preload.js:52).
6. En las páginas de Google no hay salida sin el botón `.md-home` (preload.js:102-104).
7. El panel de ajustes cerrado no cuesta nada: oyentes solo con el panel abierto (preload.js:129).
8. El keydown se registra antes que YT y en captura para que, grabando, ninguna tecla llegue a YouTube (preload.js:332-333).
9. Esc a secas cancela la grabación; con modificadores se graba (preload.js:339).
10. El id de playlist solo está en propiedades JS que el mundo aislado no ve; por eso `data-md-id` (covers.js:6-8).
11. Cambiar una variable en `<html>` recalcula ~6000 elementos (~50 ms) → 11–24 fps; durante el arrastre se escriben solo ~70 elementos (preload.js:402-406).
12. Las reglas `!important` ganan al estilo en línea: por eso variables registradas no heredables (preload.js:440-442).
13. `html.md-resizing *` recalculaba toda la página: se usa una capa (preload.js:555-556).
14. Una captura de puntero perdida dejaba la capa encima sin salida: fin por pointerup/pointercancel/lostpointercapture en window (preload.js:533-535).
15. Animar `--ytmusic-guide-width` (~13 000 elementos, ~120 ms) o usar View Transitions (congela ~200 ms) era inaceptable: FLIP + recorte (preload.js:596-603).
16. El muelle debe tener poco rebote: el recorte no debe pasar de 72 px y comerse los iconos (preload.js:609-611).
17. `click()` programático dentro de otro `click()` del mismo botón no hace nada: se reenvía en microtarea (preload.js:709-710).
18. La tarjeta `::before` se anima también: si no, el recorte se comía su borde derecho (preload.js:683-685).
19. Sin scrollbar durante el pliegue: estrechaba 10 px las playlists (preload.js:687).
20. `md-measure`: el ResizeObserver de lyrics-view llegaría tarde para medir el FLIP (preload.js:692-693).
21. El plazo de 500 ms se cancela con otro clic; si no, dejaba la tarjeta sin fondo (preload.js:716-718).
22. Con la ventana oculta las animaciones no avanzan ni acaban: sin animación (preload.js:658).
23. Miniaturas de 60 px borrosas con escalado: se piden de 240/120 px (preload.js:903-905).
24. `scroll-snap` imanta en los dos sentidos: lock propio solo hacia arriba (preload.js:783-787).
25. `scrollend` llega entre golpes de rueda: esperar 250 ms sin scroll (preload.js:838-839).
26. Con la actual cerca del principio, el lock no actúa en el tope y nunca pasa de la mitad de lo de encima (preload.js:848-849).
27. YT marca `selected` también en la versión oculta (alto 0) (preload.js:793).
28. Los filtros a veces están después de las canciones (automix): el recorte solo si están pegados (preload.js:810-814).
29. El observador de `data-md-np-open` se instala en `read()` porque al cargar el preload `<html>` aún no existe (preload.js:829-830).
30. Reordenar solo si la lista visible coincide con la del store (si no, el índice sería otro) (queue-reorder.js:33).
31. El arrastre de cola propio de YT no funciona con el tema y movía a la vez; y su gesto de minimizar Now Playing hacia abajo: se aíslan sus eventos (queue-reorder.js:49-52).
32. El clic posterior a soltar se traga en la misma tarea; `setTimeout 0` para no perder el siguiente (queue-reorder.js:98-100).
33. No reproducir al pulsar la actual: su botón la pausaría (queue-reorder.js:118-119).
34. YT puede recuperar el orden de la cola desde el servidor (queue-reorder.js:9-10).
35. `library()`: cada sección por separado; si falla una se devuelve lo conseguido (page-api.js:143).
36. `play/queue/open` sin tocar `location`: sin recarga y la reproducción sigue (page-api.js:186-187).
37. La tarjeta "Nueva lista" de la biblioteca no es playlist ni álbum: se descarta (page-api.js:99-100).

Puntos poco claros / discrepancias detectadas:

- El comentario de `guideTargets` menciona animar "el contenido de la página" con `translate`, pero el código solo anima elementos de Now Playing (preload.js:617-629).
- `aria-label` de los tiradores fijo en español (no i18n).
- El ancho 320 px del panel de ajustes está fijo en `placeSettings`; el ancho real del CSS no está aquí.
- La duración real del muelle de Motion no está en el código (depende de `Motion.spring`).
- Sombra/opacidad/transiciones del arrastre de la cola, iconos de los botones, aspecto del panel: CSS, fuera de estos archivos.

---

## F. Checklist de réplica

1. `read()` corre cada 1000 ms desde `DOMContentLoaded`, y también en `play` y `pause` del vídeo.
2. Título = `.title` de la barra principal; artista = primer tramo de `.byline` antes de `•`; ambos con trim.
3. `playing` = hay vídeo, no pausado y no terminado.
4. Se notifica la canción solo si `{title, artist, playing}` cambió.
5. Tiempo "transcurrido / total" se separa por `/` y se pinta a izquierda y derecha de la barra de progreso; se actualiza solo cuando cambia el texto.
6. El menú "⋮" de la canción va al final de la fila de botones centrales.
7. El deslizador de volumen es un panel flotante sobre su botón; no ocupa sitio en la barra.
8. El "me gusta" se dibuja junto al volumen sin cambiar su comportamiento (clic no abre Now Playing).
9. Botón "Atrás" y "Adelante" en la barra superior; tooltip "Atrás"/"Adelante".
10. Atrás habilitado si se puede retroceder (o historial > 1); adelante solo si `canGoForward` (sin Navigation API, siempre deshabilitado).
11. Estado de atrás/adelante se reevalúa en cada cambio de entrada de navegación y cada segundo.
12. Fuera de music.youtube.com aparece un botón "YouTube Music" centrado arriba (tooltip "Volver a YouTube Music") que navega a `https://music.youtube.com/`.
13. Engranaje justo antes del avatar (o al final si no hay sesión); tooltip "Ajustes".
14. Al abrir, se piden los ajustes; el panel aparece 8 px bajo el engranaje, alineado a su borde derecho, con margen mínimo de 8 px a la ventana, ancho supuesto 320 px.
15. El panel se abre siempre con "Menú rápido" plegado y sin error.
16. Cabecera "Menú rápido": muestra el atajo en `kbd` solo plegada, y un chevron que gira/cambia al desplegar.
17. Desplegado: fila "Atajo" (con el atajo o el texto "Pulsa la nueva combinación de teclas" mientras graba, y estilo `recording`).
18. Mensaje de error del atajo bajo la fila: "Otro programa ya usa ese atajo" / "Usa al menos Ctrl, Alt, Shift o Win más otra tecla" / "ByTune ya usa ese atajo para otra cosa".
19. Grupo "Funciona": "Con la app enfocada" / "En cualquier lugar".
20. Grupo "Desde otras apps": "Flotando sobre la app" / "Trayendo la ventana al frente"; atenuado y deshabilitado salvo con "En cualquier lugar".
21. Grupo "Shift + Enter en una canción": "Añadir al final de la cola" / "Reproducir a continuación".
22. Radios con círculo a la izquierda de la etiqueta; título de grupo encima.
23. Separador; cuatro interruptores en orden: "Tema ByTune", "Notificar cambio de canción", "Cerrar a la bandeja", "Iniciar con Windows" (interruptor a la derecha).
24. Separador; pie con "ByTune {versión}" a la izquierda y botón "Buscar actualizaciones".
25. El botón de actualizaciones cambia a "Buscando…" (deshabilitado) y luego "Ya tienes la última versión" / "Solo en la versión instalada" / "No se pudo comprobar"; "installing" mantiene "Buscando…" y la app se cierra.
26. Al reabrir el panel, el estado de actualización vuelve a "Buscar actualizaciones" salvo si sigue buscando.
27. Cada cambio de ajuste se envía y el panel se redibuja con la respuesta; si la respuesta es error, se muestra el error y el estado no cambia.
28. Cambios hechos desde la bandeja se reflejan en el panel abierto.
29. El foco se conserva en la misma fila tras redibujar.
30. El panel se cierra con: clic fuera (ni panel ni engranaje), redimensionar la ventana, perder el foco, Esc (foco vuelve al engranaje), o pulsar el engranaje.
31. Cerrar el panel o plegar "Menú rápido" detiene la grabación del atajo.
32. Grabando: ninguna tecla llega a la página; Esc sin modificadores cancela; repeticiones ignoradas; solo modificadores sigue grabando; mientras se graba se desactivan los atajos globales de la app.
33. Formato del atajo: orden Ctrl, Alt, Shift, Win; se usa la tecla física (`code`); se muestra como "Ctrl + Shift + Space".
34. Portadas de playlists a la izquierda de cada entrada de la barra lateral (carga perezosa), solo para entradas `VL…` con portada conocida.
35. Portada elegida: primera miniatura de ancho ≥ 60 px (o la última); hasta 10 páginas de continuación.
36. Tirador en el borde de la barra lateral: mín. 240, máx. 420 px, y nunca más que `max(240, ventana/2 − 164)`.
37. Tirador en el borde de la cola (solo con Now Playing abierto y ventana ≥ 940 px): mín. 280, máx. 480 px; se arrastra desde el borde izquierdo de la cola (ancho = ventana − x).
38. Tirador de la barra lateral solo visible con la barra desplegada y ventana ≥ 936 px.
39. Arrastre solo con botón principal; cursor `col-resize` en toda la ventana y sin selección de texto durante el arrastre.
40. El contenido se reacomoda en vivo mientras se arrastra (una actualización por fotograma), incluidos buscador/logo de la barra superior, ancho de filas de carátulas, márgenes y tamaño de la carátula de Now Playing.
41. Al soltar se redondea, se acota y se guarda el ancho; un clic sin mover no cambia nada.
42. El arrastre termina también si se cancela o se pierde la captura (nunca queda la capa bloqueando).
43. Doble clic en el tirador: vuelve al ancho por defecto y se guarda.
44. Tirador enfocable con Tab; ← / → mueven el borde 16 px en esa dirección y se guarda.
45. Los anchos guardados se restauran al cargar.
46. Plegar/desplegar la barra lateral (≥ 936 px, ventana visible, sin reduce-motion) se anima: el panel se recorta desde la derecha con esquinas redondeadas entre el ancho actual y 72 px (o el ancho desplegado).
47. Duración/curva del pliegue: muelle (0,28 s, rebote 0,08); alternativa 350 ms `cubic-bezier(.2,0,0,1)`.
48. Durante el pliegue: la tarjeta de fondo se estrecha a la vez, las filas (resaltado, línea, píldora "Nueva lista" → círculo) se estrechan al mismo paso, sin barra de desplazamiento; al plegar el contenido sigue ancho hasta el final.
49. En Now Playing, al plegar/desplegar, carátula (traslación + escala desde el centro), título y letra se deslizan desde su posición anterior a la nueva con la misma curva.
50. Un segundo clic a mitad del pliegue termina el anterior en su sitio y empieza el nuevo; si YT no cambia en 500 ms, se deshace el estado intermedio.
51. El buscador de la barra superior deja libre a la derecha el ancho de los iconos + 16 px (se recalcula al cambiar la ventana o los iconos).
52. Títulos de álbum/playlist de más de 24 caracteres se muestran más pequeños.
53. El idioma de la página se notifica a la app cuando cambia.
54. La URL abierta se notifica cuando cambia (para Recientes: playlist `list=X` → `VLX`, `/browse/MPRE…`).
55. En la cola, al cambiar de canción (150 ms después) la actual se desplaza suavemente arriba, 8 px por debajo de los filtros fijos.
56. Si la canción cambió con Now Playing cerrado, se alinea al abrirlo (150 ms).
57. Lock de la cola: 250 ms después de dejar de desplazar (y sin scroll en 240 ms), si la lista queda por encima de la actual a menos de `min(160, ancla/2)` px y no está en el tope, vuelve suavemente; hacia abajo nunca actúa.
58. La lista de la cola se recorta bajo los filtros solo cuando estos están pegados arriba.
59. Miniatura de la barra pedida a 240×240; miniaturas de la cola a 120×120.
60. Fondo difuminado con la carátula actual (ninguno si no hay o es `data:`).
61. Puente de letras: pedir letra, recordar panel abierto, pantalla completa.
62. API: peticiones POST a `/youtubei/v1/{browse|search|next}` con `SAPISIDHASH`, `X-Goog-AuthUser`, `X-Origin`, `key` y `context` de Innertube.
63. Biblioteca = `FEmusic_liked_playlists` + `FEmusic_liked_albums` (≤ 10 continuaciones cada una), sin duplicados, playlists `VL…` y álbumes `MPRE…` con título, portada (primera ≥ 120 px), artista y `playlistId OLAK…` en álbumes.
64. Búsqueda del menú rápido: `search` con `params` de "Canciones", máx. 5 canciones distintas; consulta vacía → sin petición.
65. Sugerencias: primeras 5 canciones de `FEmusic_home`.
66. Canción = `{videoId, title, artist (tramo con enlace de artista o el primero), duration (último h:mm:ss válido), cover}`.
67. Reproducir sustituye la cola por la canción + radio; encolar al final o a continuación (con cola vacía reproduce); abrir álbum/playlist sin recargar.
68. Letra: `next` → browseId `MPLYt…` → `browse` → texto por líneas (vacías = estrofas) + crédito; sin sesión.
69. Cola: arrastrar una fila pulsando su texto (no miniatura, menú, botones ni enlaces); empieza tras 6 px verticales.
70. La fila arrastrada sigue al puntero sin salir de la lista; las demás se apartan una altura para dejar el hueco; destino = hueco que contiene el centro.
71. No hay línea indicadora ni auto-scroll durante el arrastre.
72. Esc o cancelación devuelven todo sin cambios; soltar aplica el movimiento dentro de la misma lista (cola o reproducción automática).
73. Filas de cola con `canReorder === false` no se arrastran; si la lista visible no coincide con la de YT, no se arrastra.
74. El clic que sigue a un arrastre no reproduce; el siguiente clic del usuario sí funciona.
75. Clic en cualquier parte de una fila de la cola (no la actual) la reproduce.
76. Con una fila pulsada, YT no recibe los eventos de ratón (ni su arrastre propio ni minimizar Now Playing arrastrando).
