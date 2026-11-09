# 05 — Ventanas, funciones del proceso principal y menú rápido

Documento de réplica de ByTune (Electron, `C:\Users\JuanC\Documents\ByTune`, versión `1.0.23` según `package.json:4`) para reconstruirlo en Rust + egui (`ByTune-rs`). Cubre el proceso principal (`src/main.js`), el menú rápido (`src/quickmenu/*`), las librerías puras de `src/lib/` (atajos, i18n, biblioteca, caché, letras), el empaquetado y las actualizaciones.

Convenciones:

- `main.js:153` = línea 153 de `src/main.js`. `qm.js` = `src/quickmenu/quickmenu.js`, `qm.css` = `quickmenu.css`, `qm.html` = `quickmenu.html`, `qmpre.js` = `src/quickmenu/preload.js`, `preload.js` = `src/preload.js` (preload de la página principal; su interfaz la documenta otra parte, aquí solo se cita en lo que toca a IPC, grabación del atajo y actualizaciones).
- Cuando el código y los documentos de diseño (`docs/superpowers/`) no coinciden, **manda el código**; las diferencias están en el apartado M.
- "No está claro" o "comportamiento de Electron" marca cosas que el código no fija y dependen de la plataforma.

---

## Índice

- A. Ventanas
- B. Ajustes (`settings.json`) y carpeta de datos
- C. Bandeja del sistema
- D. Barra de tareas (thumbar) e iconos
- E. Atajos de teclado
- F. Notificaciones, inicio con Windows, cerrar a la bandeja, instancia única, user agent, switches, navegación, zoom, F5, Alt+flechas
- G. Actualizaciones (electron-updater), empaquetado y publicación
- H. IPC (todos los canales) y llamadas a la página
- I. Menú rápido completo
- J. i18n (todas las cadenas)
- K. Letras y biblioteca
- L. Checklist de réplica
- M. Diferencias entre la spec/plan y el código

---

## A. Ventanas

Hay **tres** `BrowserWindow`: principal, menú rápido y aviso ("splash") de actualización. Además hay un `Tray` (apartado C) y notificaciones nativas (F).

### A.1 Ventana principal (`createWindow`, `main.js:152-262`)

| Propiedad | Valor | Origen |
|---|---|---|
| Tamaño inicial | `settings.bounds` (por defecto `{ width: 1280, height: 820 }`, sin `x`/`y` → Electron la centra en la pantalla principal) | `main.js:47`, `main.js:154` |
| Posición | La guardada en `settings.bounds.x/y` si existe (tras el primer movimiento/redimensión se guarda `{x,y,width,height}`) | `main.js:242` |
| Mínimo | `minWidth: 480`, `minHeight: 400` | `main.js:155-156` |
| Título | `'YouTube Music'`; luego `"<título> — <artista> · YouTube Music"` o `'YouTube Music'` si no hay canción (ver C.3) | `main.js:157`, `main.js:382` |
| Icono | `build/icon.ico` en Windows (respaldo `build/icon.png`); en otras plataformas `icon.png`; si no existe ninguno, imagen vacía | `main.js:65-69` |
| Color de fondo | `#000000` | `main.js:159` |
| Barra de menús | `autoHideMenuBar: true` + `setMenuBarVisibility(false)` (no hay menú) | `main.js:160`, `main.js:195` |
| Se muestra | `show: false`; se muestra en `ready-to-show` | `main.js:161`, `main.js:209` |
| Marco | Con marco de sistema pero **sin barra de título**: `titleBarStyle: 'hidden'` | `main.js:163` |
| Botones de ventana | `titleBarOverlay: { color: '#00000000', symbolColor: '#ffffff', height: 64 }`: minimizar/maximizar/cerrar nativos de Windows dibujados sobre la página, fondo transparente, símbolos blancos, 64 px de alto (igual que la barra superior de YT Music) | `main.js:167` |
| Transparencia | No (solo el overlay de los botones es transparente) | — |
| always-on-top / skipTaskbar | No / no (pero ver D: `setSkipTaskbar(false)` en reintentos) | — |
| Maximizada al abrir | `if (settings.maximized) win.maximize()` antes de mostrar | `main.js:208` |
| Carga | `win.loadURL('https://music.youtube.com/')` | `main.js:11`, `main.js:212` |

`webPreferences` (`main.js:168-177`):

- `partition: 'persist:ytmusic'` — sesión persistente (cookies de Google, sesión iniciada entre reinicios). Vive dentro de la carpeta de datos (`userData/Partitions/ytmusic`, convención de Electron).
- `preload: src/preload.js`, `contextIsolation: true`, `sandbox: true`, `nodeIntegration: false`, `spellcheck: false`.
- `backgroundThrottling: true` (Chromium frena la página oculta; el audio que suena está exento).

Identidad en Windows (`main.js:199-207`): `win.setAppDetails({ appId: 'local.youtube-music-client', appIconPath: <exe instalado> o iconPath en desarrollo, appIconIndex: 0, relaunchCommand: '"<exe>"' (instalada) o '"<electron.exe>" "<appPath>"' (desarrollo), relaunchDisplayName: 'ByTune' })`. Sirve para que "Anclar a la barra de tareas" ancle ByTune y no `electron.exe`. Al arrancar, si está empaquetada: `app.setAppUserModelId('local.youtube-music-client')` (`main.js:932`).

Tema nativo: `nativeTheme.themeSource = 'dark'` (menús, barras de scroll y diálogos oscuros) (`main.js:933`).

Eventos de la ventana principal:

| Evento | Qué hace | Origen |
|---|---|---|
| `webContents dom-ready` (cada documento nuevo, cargas y recargas; la navegación interna SPA de YT no lo dispara) | `pageApiReady = null`; `setRecording(false)`; inyecta `chrome.css`, `theme.css` (si `settings.theme`), `lyrics.css` y después ejecuta lyrics-timing + lyrics-view + queue-reorder; inyecta Motion en el mundo aislado 999; manda `strings`; manda `panels`; manda `covers` guardadas si hay alguna; a los 4000 ms lanza `refreshCovers`, `refreshLibrary` y `refreshSuggested` | `main.js:180-194` |
| `ready-to-show` (una vez) | `win.show()` | `main.js:209` |
| `show` | `thumbarAfterShow()` (D) | `main.js:210` |
| `focus` | `updateThumbar(true)` | `main.js:211` |
| `resize`, `move` | `rememberBounds()`: si está en pantalla completa no guarda nada; si no está maximizada ni minimizada, `settings.bounds = win.getBounds()`; siempre `settings.maximized = win.isMaximized()`. **Solo en memoria**: se escribe a disco en la siguiente `saveSettings()` (al cerrar, al salir o por cualquier otro cambio) | `main.js:239-246` |
| `enter-full-screen`, `leave-full-screen` | `notifyFullscreen()` → `window.__mdLyrics.setFullscreen(<bool>)` en la página | `main.js:143-146`, `main.js:247-248` |
| `blur` | `setRecording(false)` (si se estaba grabando un atajo, los atajos globales vuelven) | `main.js:250` |
| `close` | `saveSettings()`; si no se está saliendo (`!quitting`) y `settings.closeToTray` → `preventDefault()` + `win.hide()` | `main.js:252-255` |
| `webContents will-prevent-unload` | `preventDefault()` siempre: se ignora el "¿Salir de la página?" que pone YT Music mientras suena algo (si no, Electron bloquea en silencio el cierre y "Salir" de la bandeja no hacía nada) | `main.js:259` |
| `closed` | `win = null`; si existe la ventana del menú rápido, `qmWin.destroy()` (si no, la app no saldría) | `main.js:261` |
| `before-input-event` | atajos de ventana (E.2) | `main.js:226-237` |
| `setWindowOpenHandler` / `will-navigate` | navegación interna/externa (F.7) | `main.js:215-221` |

`showWindow()` (`main.js:264-269`): si no hay ventana la crea (`createWindow()`); si está minimizada `restore()`; luego `show()` y `focus()`.

Pantalla completa: la pide la página por IPC `np:fullscreen` (`main.js:728-734`); `Esc` en pantalla completa sale (E.2). En pantalla completa no se guardan bounds (al salir vuelve el tamaño anterior).

Arranque oculto: si `process.argv` incluye `--hidden` (lo añade "Iniciar con Windows"), se registra `win.once('ready-to-show', () => win.hide())` (`main.js:938`). Ese listener se registra **después** del `once('ready-to-show', show)` de `createWindow`, así que en el mismo evento primero se muestra y luego se oculta (puede verse un parpadeo; el código no lo evita). Réplica: arrancar sin mostrar la ventana.

Inyecciones en la página (detalle en otros documentos; aquí el mecanismo, `main.js:95-140`):

- CSS como hojas de **origen usuario** (`insertCSS(css, { cssOrigin: 'user' })`), tres tipos: `theme` (`theme.css`, solo si `settings.theme`), `chrome` (`chrome.css`, siempre) y `lyrics` (`lyrics.css`, siempre). Se leen del disco en cada carga. Antes de insertar se quitan todas las claves anteriores de ese tipo. Las inyecciones van en una cola en serie.
- JS: `{ lyrics-timing.js + lyrics-view.js } { queue-reorder.js } true` con `executeJavaScript(src, true)`, y luego `window.__mdLyrics.init({ open: settings.lyricsOpen, fullscreen: win.isFullScreen(), strings })` (también al cambiar de idioma, `main.js:85`).
- `vendor/motion.js` en el mundo aislado `999` (`executeJavaScriptInIsolatedWorld`), cacheado en memoria tras la primera lectura.
- Todo esto solo si la URL actual tiene host exactamente `music.youtube.com` (`onYouTubeMusic`, `main.js:402-404`).

### A.2 Ventana del menú rápido (`createQuickMenu`, `main.js:487-531`)

Se crea **perezosamente** la primera vez que se abre el menú y se reutiliza (se oculta, no se destruye).

| Propiedad | Valor | Origen |
|---|---|---|
| Tamaño | `QM_SIZE = { width: 588, height: 700 }` (fijo) | `main.js:490` |
| Marco | `frame: false` | `main.js:495` |
| Transparencia | `transparent: true`, `backgroundColor: '#00000000'` | `main.js:496-497` |
| Sombra de ventana | `hasShadow: false` (la sombra la dibuja el panel en CSS) | `main.js:498` |
| Siempre encima | `alwaysOnTop: true` | `main.js:499` |
| Barra de tareas | `skipTaskbar: true` | `main.js:500` |
| Redimensionable / minimizable / maximizable / pantalla completa | `false` / `false` / `false` / `false` | `main.js:501-504` |
| Se muestra | `show: false`; se muestra en `showQuickMenu` | `main.js:505` |
| Título / icono | `'ByTune'` / el icono de la app | `main.js:506-507` |
| Menú | `setMenuBarVisibility(false)` | `main.js:516` |
| Contenido | `loadFile(src/quickmenu/quickmenu.html)`; `qmLoaded` = promesa de `did-finish-load` | `main.js:517-518` |
| webPreferences | `preload: src/quickmenu/preload.js`, `contextIsolation: true`, `sandbox: true`, `nodeIntegration: false`, `spellcheck: false`. Sin `partition` (sesión por defecto, no la de YT) | `main.js:508-514` |

Geometría efectiva: la ventana es de 588×700 transparente; el panel visible mide 540 px de ancho y está a 60 px del borde superior y 24 px de los laterales (padding del `body`, I.3). Todo lo que queda fuera del panel es transparente pero **pertenece a la ventana**: un `mousedown` ahí cierra el menú (I.4).

Eventos:

- `blur` → `hideQuickMenu(false)` (se oculta sin devolver el foco: el foco ya está en otra ventana) (`main.js:519`).
- `before-input-event`: si es `keyDown` y coincide con el atajo del menú → `preventDefault()`; y si no es autorepetición y el atajo **no** está registrado como global → `hideQuickMenu()` (`main.js:521-526`). En modo global lo atiende `globalShortcut`.
- `will-navigate` → siempre `preventDefault()`; `setWindowOpenHandler` → siempre `deny` (`main.js:528-529`).
- `closed` → `qmWin = null; qmLoaded = null` (`main.js:530`).

Posición (`placeQuickMenu(overWin)`, `main.js:535-543`):

- Pantalla: si `overWin` (se abre sobre la app) → `screen.getDisplayMatching(win.getBounds())`; si no → la pantalla más cercana al cursor (`getDisplayNearestPoint(getCursorScreenPoint())`).
- Con `a = display.workArea` (sin barra de tareas): `x = a.x + round((a.width - 588) / 2)`, `y = a.y + max(0, round((a.height - 700) / 2))`. Centrada horizontal **y verticalmente** en el área de trabajo (si la pantalla es más baja que 700, pegada arriba).
- `setBounds` se llama **dos veces** seguidas con los mismos valores (en Windows, al saltar a una pantalla con otro escalado, el primero deja mal el tamaño).

Mostrar (`showQuickMenu(fromOutside)`, `main.js:545-576`):

1. Si ya se está abriendo (`qmOpening`) no hace nada (evita reentradas).
2. `overWin = win visible && win enfocada`.
3. Si `fromOutside` y `settings.quickMenu.fromOtherApps === 'front'` → `showWindow()` y `overWin = true`.
4. Crea la ventana si no existe; espera `qmLoaded`; si mientras tanto se destruyó, sale.
5. Envía `qm:show` con `{ strings, recent: recentItems(settings.recent, settings.library), suggested: settings.suggested, track, shiftEnter }` **antes** de mostrar (el menú se reinicia con cada `qm:show`).
6. `qmOverWin = overWin && !!win`; `placeQuickMenu(qmOverWin)`; `qmWin.show()`; `qmWin.focus()`.
7. Si el último intento de refrescar la biblioteca fue hace más de 60 s → `refreshLibrary()` en segundo plano.

Ocultar (`hideQuickMenu(refocus = true)`, `main.js:580-585`): cancela el temporizador pendiente de búsqueda de canciones (`songsTimer`); si no está visible no hace nada; `qmWin.hide()` (sin animación de cierre); si `refocus` y se abrió sobre la app (`qmOverWin`) y la ventana principal es visible → `win.focus()` (Windows no devuelve el foco solo y, en pantalla completa, la barra de tareas quedaba encima).

`toggleQuickMenu(fromOutside)` (`main.js:587-590`): visible → `hideQuickMenu()`; si no → `showQuickMenu(fromOutside)`.

Quién abre/cierra el menú:

| Origen | Llamada |
|---|---|
| Atajo en la ventana principal (modo "con la app enfocada") | `toggleQuickMenu(false)` (`main.js:229`) |
| Atajo global (modo "en cualquier lugar") | `toggleQuickMenu(!win \|\| !win.isFocused())` (`main.js:775`) |
| Atajo con el foco en el propio menú (modo enfocado) | `hideQuickMenu()` (`main.js:524`) |
| `Esc` o `mousedown` fuera del panel | IPC `qm:close` → `hideQuickMenu()` (`main.js:650`) |
| Perder el foco | `hideQuickMenu(false)` |
| Activar playlist/álbum | `hideQuickMenu()` + `showWindow()` (`main.js:642-643`) |

### A.3 Aviso de actualización (`showUpdateSplash(version)`, `main.js:863-869`)

- `new BrowserWindow({ width: 340, height: 110, frame: false, resizable: false, icon, backgroundColor: '#000000', show: false })`. No es always-on-top, aparece en la barra de tareas (valores por defecto), posición por defecto (centrada).
- Contenido: `data:text/html;charset=utf-8,` + HTML codificado:
  ```html
  <body style="margin:0;height:100vh;display:grid;place-items:center;background:#000;color:#fff;font:14px 'Segoe UI',sans-serif;-webkit-app-region:drag">{texto}</body>
  ```
  con `{texto} = t(lang, 'upd.splash', { version })` → "Actualizando ByTune a la versión 1.2.3…" / "Updating ByTune to version 1.2.3…". Texto centrado, blanco sobre negro, Segoe UI 14 px; toda la ventana sirve para arrastrarla.
- Se muestra en `ready-to-show`. Se destruye solo si la actualización falla; si va bien, la app sale y se cierra con ella.

---

## B. Ajustes (`settings.json`) y carpeta de datos

### B.1 Carpeta de datos y migración

- Por defecto Electron usa `%APPDATA%\ByTune` (nombre del producto; `package.json:3`).
- **Migración "Music Desktop"** (`main.js:31-35`): si existe `%APPDATA%\Music Desktop` (`app.getPath('appData') + 'Music Desktop'`), se usa **esa** carpeta como `userData` (la app se llamaba así; ahí están la sesión iniciada, los ajustes y la caché). No se copia ni se renombra nada: se sigue usando la carpeta vieja indefinidamente. Las instalaciones nuevas usan "ByTune". Se hace **antes** del bloqueo de instancia única (que vive en la carpeta de datos).
- Solo desarrollo: `YTM_USER_DATA=<carpeta>` fuerza otra carpeta (tiene prioridad sobre la anterior) (`main.js:38`).
- Archivos propios en `userData`: `settings.json` y `updater.log` (G). Más la partición `persist:ytmusic` de Chromium.

### B.2 Lectura y escritura

- Ruta: `userData/settings.json` (`main.js:45`).
- Lectura (`main.js:53-59`): `settings = { ...defaults, ...JSON.parse(archivo) }` (fusión superficial; si el archivo no existe o no es JSON válido, se usan los defaults en silencio). Las claves desconocidas del archivo se conservan y se vuelven a escribir.
- `quickMenu` se fusiona en profundidad: `{ ...defaults.quickMenu, ...settings.quickMenu }` (`main.js:56`).
- Si `library`, `recent` o `suggested` no son arrays → `[]` (`main.js:57-59`). (`covers`, `panels`, `bounds` no se validan.)
- Escritura (`saveSettings`, `main.js:60-62`): `fs.writeFileSync(archivo, JSON.stringify(settings, null, 2))`, síncrona, con sangría de 2 espacios, sin escritura atómica, errores ignorados.
- Se guarda: al cerrar la ventana, en `before-quit`, y tras cada cambio listado abajo.

### B.3 Todas las claves

| Clave | Tipo | Por defecto | Quién la cambia | Efecto |
|---|---|---|---|---|
| `bounds` | `{ x?, y?, width, height }` (números) | `{ width: 1280, height: 820 }` | `rememberBounds` en `resize`/`move` (solo si no está maximizada, minimizada ni en pantalla completa) | Tamaño y posición de la ventana principal al crearla (`main.js:154`) |
| `maximized` | bool | `false` | `rememberBounds` (no en pantalla completa) | Si `true`, `win.maximize()` al crear (`main.js:208`) |
| `notifications` | bool | `true` | Bandeja / ajustes (`SWITCHES.notifications`) | Notificación al cambiar de canción (F.1) |
| `closeToTray` | bool | `true` | Bandeja / ajustes | Cerrar oculta en vez de salir (F.3) |
| `theme` | bool | `true` | Bandeja / ajustes (`SWITCHES.theme`: guarda y `applyTheme()` al momento) | Inyectar `theme.css` |
| `covers` | objeto `{ browseId: urlPortada }` | `{}` | `refreshCovers` (resultado de `covers.js`, solo si trae alguna) | Se manda a la página (`covers`) en cada `dom-ready` y tras refrescar |
| `lang` | `'es' \| 'en' \| null` | `null` | IPC `lang` (idioma de la página) | Idioma inicial en el siguiente arranque (J) |
| `library` | `LibraryItem[]` | `[]` | `refreshLibrary` (solo si llega una lista no vacía) | Biblioteca del menú rápido, filtrado y recientes (K.2) |
| `recent` | `string[]` (browseIds, máx. 6) | `[]` | `recordRecent` (IPC `opened` y abrir desde el menú) | Sección "Recientes" del menú |
| `suggested` | `Song[]` | `[]` | `refreshSuggested` (solo si llega lista no vacía) | Sección "Sugerencias" del menú |
| `panels` | `{ left?: px, right?: px }` | `{}` | IPC `panels`: cada lado solo si es número finito, recortado a `left ∈ [240, 420]`, `right ∈ [280, 480]` y redondeado; un lado ausente se omite (vuelve al ancho por defecto) | Anchos de barra lateral y cola; se manda a la página en `dom-ready` (`main.js:188`, `main.js:688-697`) |
| `lyricsOpen` | bool | `false` | IPC `lyrics:open` (solo si cambia) | Panel de letras abierto en Now Playing (`init({ open })`) |
| `quickMenu.accelerator` | string (acelerador Electron) | `'CommandOrControl+Shift+Space'` | `settings:set` (tras validar y, en global, registrar) | Atajo del menú rápido |
| `quickMenu.reach` | `'focused' \| 'global'` | `'focused'` | `settings:set` | `focused`: solo con la ventana principal enfocada (o con el menú abierto); `global`: `globalShortcut` del sistema |
| `quickMenu.fromOtherApps` | `'float' \| 'front'` | `'float'` | `settings:set` | Al abrir con el atajo global desde otra app: `float` = flota sin tocar la ventana; `front` = trae la ventana al frente y el menú encima |
| `quickMenu.shiftEnter` | `'queue' \| 'next'` | `'queue'` | `settings:set` | Mayús+Intro en una canción: `queue` = al final de la cola; `next` = a continuación |

No están en `settings.json`:

- **Iniciar con Windows** (`startWithWindows`): vive en el registro (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, valor `local.youtube-music-client`), leído con `app.getLoginItemSettings().openAtLogin` (F.2).
- `version` (de `app.getVersion()`), que solo se manda a la página en `settingsView()`.

Comentario de `defaults` en `main.js:49-50`: `panels` = "ancho elegido de la barra lateral y de la cola", `lyricsOpen` = "panel de letras abierto en Now Playing".

### B.4 Interruptores compartidos (`SWITCHES`, `main.js:332-346`)

| Clave | get | set |
|---|---|---|
| `theme` | `settings.theme` | guarda, `saveSettings()`, `applyTheme()` |
| `notifications` | `settings.notifications` | guarda, `saveSettings()` |
| `closeToTray` | `settings.closeToTray` | guarda, `saveSettings()` |
| `startWithWindows` | `app.getLoginItemSettings().openAtLogin` | `app.setLoginItemSettings({ openAtLogin: v, args: ['--hidden'] })` |

`changeSwitch(key, value)`: si el valor actual es distinto llama a `set`; **siempre** llama a `refreshUi()` (rehace la bandeja).

Etiquetas (`SWITCH_LABELS`, en este orden): `theme`→`set.theme`, `notifications`→`set.notify`, `closeToTray`→`set.closeToTray`, `startWithWindows`→`set.startWithWindows`.

`settingsView()` (`main.js:349-352`), lo que recibe el menú del engranaje:

```js
{ theme, notifications, closeToTray, startWithWindows, version: app.getVersion(),
  quickMenu: { accelerator, reach, fromOtherApps, shiftEnter },
  display: { shortcut: displayAccelerator(accelerator) } }   // p. ej. 'Ctrl + Shift + Space'
```

---

## C. Bandeja del sistema

### C.1 Creación (`createTray`, `main.js:387-393`)

- `new Tray(icon)` con el icono de la app **sin redimensionar** (con el `.ico`, Windows elige el tamaño según el escalado).
- Se crea en `whenReady` tras `createWindow()` (`main.js:939`), luego `refreshUi()`.

### C.2 Clics

- `click`: si la ventana existe, es visible **y** está enfocada → `win.hide()`; si no → `showWindow()` (`main.js:390`).
- `double-click`: `showWindow()` (`main.js:391`).
- Clic derecho: menú contextual (`setContextMenu`).

### C.3 Tooltip y título (`refreshUi`, `main.js:376-385`)

- `trackLabel()` = `"<título> — <artista>"` (raya larga U+2014 con espacios) si hay artista, `"<título>"` si no, y `'YouTube Music'` si no hay título (`main.js:327-329`).
- Tooltip: `trackLabel()`; si supera 127 caracteres → primeros 124 + `'…'` (U+2026).
- Título de la ventana principal: `"<trackLabel> · YouTube Music"` si hay título; si no `'YouTube Music'`.
- `refreshUi` también: `updateThumbar(false)` y, si existe la ventana del menú, `qm:track` con `track`.

### C.4 Menú contextual (orden exacto, `buildTrayMenu`, `main.js:358-374`)

| # | Elemento | es | en | Acción |
|---|---|---|---|---|
| 1 | Etiqueta deshabilitada | `trackLabel()` (p. ej. "Canción — Artista" o "YouTube Music") | igual | — (`enabled: false`) |
| 2 | Separador | | | |
| 3 | Botón | `Pausar` si suena, `Reproducir` si no | `Pause` / `Play` | `player('playPause')` (sin cambio optimista aquí) |
| 4 | Botón | `Siguiente` | `Next` | `player('next')` |
| 5 | Botón | `Anterior` | `Previous` | `player('previous')` |
| 6 | Separador | | | |
| 7 | Botón | `Mostrar ventana` | `Show window` | `showWindow()` |
| 8 | Casilla | `Tema ByTune` | `ByTune theme` | `changeSwitch('theme', checked)` + `notifySettings()` |
| 9 | Casilla | `Notificar cambio de canción` | `Notify on song change` | ídem `notifications` |
| 10 | Casilla | `Cerrar a la bandeja` | `Close to tray` | ídem `closeToTray` |
| 11 | Casilla | `Iniciar con Windows` | `Start with Windows` | ídem `startWithWindows` |
| 12 | Separador | | | |
| 13 | Botón | `Salir` | `Quit` | `quitting = true; app.quit()` |

Las casillas muestran `SWITCHES[key].get()`. `notifySettings()` manda `settings:changed` a la página para que el menú del engranaje, si está abierto, se redibuje (`main.js:354-356`).

### C.5 Cuándo se reconstruye

El menú se reconstruye entero en cada `refreshUi()`, que se llama en: creación de la bandeja; IPC `track` (cada cambio de canción/estado); clic en el botón de reproducir/pausa del thumbar y del menú rápido; `changeSwitch` (bandeja y ajustes); `settings:set`; IPC `lang` cuando el idioma cambia.

---

## D. Barra de tareas (thumbar) e iconos

### D.1 Botones de la vista previa (`main.js:283-325`)

Solo en Windows (`THUMB = null` en otras plataformas). Imágenes `build/thumb-prev.png`, `thumb-play.png`, `thumb-pause.png`, `thumb-next.png` (16 px; existen también `@2x` de 32 px, que Electron elige solo en pantallas con escalado por la convención de nombres `@2x`). Si `thumb-play` está vacía (no existe), no se ponen botones.

Tres botones, en este orden:

| # | Tooltip | Icono | Clic |
|---|---|---|---|
| 1 | `t('tray.previous')` (Anterior / Previous) | prev | `player('previous')` |
| 2 | `tray.pause` si suena / `tray.play` si no (Pausar/Reproducir, Pause/Play) | pause si suena / play si no | `player('playPause')`, luego **cambio optimista** `track.playing = !track.playing` y `refreshUi()` (la página confirma el estado real enseguida) |
| 3 | `t('tray.next')` (Siguiente / Next) | next | `player('next')` |

`updateThumbar(force)`: no hace nada si no hay ventana o está destruida; sin `force`, no hace nada si la última llamada tuvo éxito (`thumbOk`) y no cambiaron `track.playing` ni `track.title`. Guarda `thumbPlaying`, `thumbTitle`, y `thumbOk = win.setThumbarButtons([...])` (booleano devuelto por Electron).

`thumbarAfterShow()` (evento `show`): cancela reintentos previos y lo intenta a los 0, 500, 1500, 4000, 10000 y 30000 ms. Cada intento: solo si la ventana es visible → `win.setSkipTaskbar(false)` (fuerza a Electron a volver a dar de alta los botones; Windows los olvida al ocultar la ventana y el alta falla si llega antes de que Windows cree el botón de la barra de tareas, p. ej. al abrir por primera vez o tras una actualización) y `updateThumbar(true)`. Con `YTM_DEBUG_PORT` se registra en consola `[thumbar] <ms> ms tras mostrar: puestos|no aceptados`.

También `focus` de la ventana → `updateThumbar(true)`.

### D.2 Iconos generados (`scripts/make-icon.mjs`, `npm run icon`; también lo ejecuta `npm run dist`)

Genera, sin dependencias, en `build/`:

- `icon.png` (256×256) e `icon.ico` con los tamaños 16, 20, 24, 32, 40, 48, 64, 128, 256 (cada uno rasterizado aparte; BMP 32 bits con máscara AND vacía en los pequeños y PNG en 256). Diseño: círculo rojo `rgb(223, 28, 67)` (`#DF1C43`) de diámetro completo, con tres barras blancas verticales de puntas redondeadas que bajan de alto formando un "play". Coordenadas sobre un lienzo de 536: barras `[x, y, alto]` = `[182, 143, 250]`, `[262, 183, 170]`, `[342, 223, 90]`, ancho 56, radios de esquina `tl 28, tr 28, br 28, bl 20`. Antialias por 4×4 muestras por píxel.
- `thumb-{prev,play,pause,next}.png` (16 px) y `@2x` (32 px): glifos blancos sobre transparente. En coordenadas 0..1: `play` = triángulo hacia la derecha base x 0.28, punta x 0.8, centro y 0.5, semialto 0.32; `pause` = rectángulos x 0.24–0.42 y 0.58–0.76, y 0.18–0.82; `next` = triángulo x 0.18–0.66 (centro 0.5, semialto 0.3) + barra x 0.68–0.8, y 0.2–0.8; `prev` = `next` reflejado horizontalmente.

Empaquetado: `package.json:21-26` incluye `src/**/*`, `build/icon.png`, `build/icon.ico`, `build/thumb-*.png`.

---

## E. Atajos de teclado

### E.1 Atajos globales (`globalShortcut`, `main.js:755-815`)

Fijos (`PLAYER_SHORTCUTS`):

| Acelerador | Acción |
|---|---|
| `CommandOrControl+Alt+Right` | `player('next')` |
| `CommandOrControl+Alt+Left` | `player('previous')` |
| `CommandOrControl+Alt+M` | si la ventana existe y es visible → `win.hide()`; si no → `showWindow()` |

- Se registran en `whenReady` (`registerPlayerShortcuts`, `main.js:943`), cada uno solo si `!globalShortcut.isRegistered(acc)`. Si otro programa ya lo tiene, `register` devuelve `false` y no pasa nada (sin aviso).
- **No hay atajo global de play/pausa** (chocaba con el del menú rápido): play/pausa va por la tecla multimedia del teclado (la gestiona Windows/Chromium con la Media Session de la página; la réplica nativa tendrá que atenderla, p. ej. con SMTC) o por `Alt+↓` en el menú rápido.
- Nunca se usa `unregisterAll` en marcha: cada atajo se quita por su nombre. Solo en `will-quit` se hace `globalShortcut.unregisterAll()` (`main.js:913`).

Atajo del menú rápido en modo global:

- `qmGlobalAcc`: acelerador registrado ahora como global, o `null`.
- `qmShortcutIsGlobal()` = `qmGlobalAcc !== null && qmGlobalAcc === settings.quickMenu.accelerator`.
- Callback: `toggleQuickMenu(!win || !win.isFocused())` (es "desde fuera" si la ventana principal no está enfocada).
- `registerQuickMenuGlobal(acc)`: `globalShortcut.register(acc, cb)` dentro de `try` (excepción → `false`).

`applyQuickMenuShortcut(next)` (`main.js:783-799`), devuelve `'ok' | 'invalid' | 'reserved' | 'taken'`:

1. `validateAccelerator(next.accelerator)`; si falla devuelve `'invalid'` o `'reserved'` sin tocar nada.
2. Quita siempre el global anterior (`prev = qmGlobalAcc`), `qmGlobalAcc = null`.
3. Si `next.reach === 'global'`: intenta registrar el nuevo; si falla, vuelve a registrar `prev` (si lo había; si se consigue, `qmGlobalAcc = prev`) y devuelve `'taken'` **sin guardar**.
4. Si va bien (o en modo `focused`): `settings.quickMenu = { ...next }`, `saveSettings()`, `'ok'`.

Al arrancar se llama `applyQuickMenuShortcut(settings.quickMenu)` (`main.js:944`). Si el guardado es inválido no se registra nada; si está ocupado, `qmGlobalAcc` queda `null` y, como `qmShortcutIsGlobal()` es falso, el atajo sigue funcionando **dentro de la ventana** (before-input-event) aunque `reach` diga `global`.

### E.2 Atajos de la ventana principal (`before-input-event`, `main.js:226-237`)

Solo `keyDown`, y nada si se está grabando un atajo (`recording`). Cadena `if / else if` exclusiva, en este orden:

| # | Condición | Acción |
|---|---|---|
| 1 | `matches(input, settings.quickMenu.accelerator)` | `preventDefault()`; si no es autorepetición y el atajo no es global → `toggleQuickMenu(false)` |
| 2 | `input.key === 'F5'` (con cualquier modificador) | `wc.reload()` + `preventDefault()` |
| 3 | `input.key === 'Escape'` y la ventana está en pantalla completa | `win.setFullScreen(false)` + `preventDefault()` (si no está en pantalla completa, Esc llega a la página) |
| 4 | `input.alt` y `ArrowLeft` y `canGoBack()` | `goBack()` + `preventDefault()` |
| 5 | `input.alt` y `ArrowRight` y `canGoForward()` | `goForward()` + `preventDefault()` |
| 6 | `input.control` y (`key === '='` o `key === '+'`) | zoom `+0.5` niveles + `preventDefault()` |
| 7 | `input.control` y `key === '-'` | zoom `-0.5` + `preventDefault()` |
| 8 | `input.control` y `key === '0'` | zoom `0` + `preventDefault()` |

Notas: los modificadores extra no se excluyen (Ctrl+Shift+0 también restablece; Alt+Ctrl+← no llega aquí si el global lo captura antes). Si Alt+← no puede ir atrás, la tecla llega a la página. El zoom es el de Chromium (nivel 0 = 100 %, cada nivel ×1.2); el código no lo guarda.

### E.3 Atajos del menú rápido

Ver I.5 (teclado completo). En la ventana del menú, `before-input-event` solo atiende el propio atajo (cerrar, A.2).

### E.4 Conversión de teclas (`src/lib/shortcut.js`)

`fromKeyEvent(e)` (`shortcut.js:62-73`): acepta un `KeyboardEvent` del DOM (`ctrlKey/altKey/shiftKey/metaKey`) o un `input` de Electron (`control/alt/shift/meta`).

1. Devuelve `null` si `e.key` no es string, es `''`, o es una tecla modificadora: `Control`, `Alt`, `AltGraph`, `Shift`, `Meta`, `OS`, `Super`, `CapsLock`.
2. Tecla principal, por prioridad:
   1. `keyFromCode(e.code)` (independiente de la distribución y de Mayús):
      - `KeyA`..`KeyZ` → `A`..`Z`
      - `Digit0`..`Digit9` → `0`..`9`
      - `Numpad0`..`Numpad9` → `num0`..`num9`
      - `F1`..`F24` → igual
      - Tabla `CODE_NAMES`: `Space→Space`, `Enter→Enter`, `NumpadEnter→Enter`, `Tab→Tab`, `Backspace→Backspace`, `Delete→Delete`, `Insert→Insert`, `Escape→Escape`, `Home→Home`, `End→End`, `PageUp→PageUp`, `PageDown→PageDown`, `ArrowUp→Up`, `ArrowDown→Down`, `ArrowLeft→Left`, `ArrowRight→Right`, `PrintScreen→PrintScreen`, `Minus→-`, `Equal→=`, `BracketLeft→[`, `BracketRight→]`, `Backslash→\`, `Semicolon→;`, `Quote→'`, `Comma→,`, `Period→.`, `Slash→/`, `Backquote→` `` ` ``, `NumpadAdd→numadd`, `NumpadSubtract→numsub`, `NumpadMultiply→nummult`, `NumpadDivide→numdiv`, `NumpadDecimal→numdec`.
      - Cualquier otro `code` → `null`.
   2. Si no: `KEY_NAMES[e.key]`: `' '→Space`, `'+'→Plus`, `ArrowUp→Up`, `ArrowDown→Down`, `ArrowLeft→Left`, `ArrowRight→Right`.
   3. Si no: si `e.key` es una sola letra a-z/A-Z → mayúscula.
   4. Si no: `e.key` tal cual (p. ej. `ñ` o teclas muertas; luego la validación lo rechaza).
3. Modificadores en orden fijo: `CommandOrControl` (ctrl), `Alt`, `Shift`, `Super` (meta/Win); se unen con `+`.

Ejemplos: con teclado español, Mayús+3 llega con `key '·'` y `code 'Digit3'` → `Shift+3`; la ñ es `code 'Semicolon'` → `;`. AltGr en Windows llega como Ctrl+Alt.

`validateAccelerator(acc)` (`shortcut.js:83-91`):

- No string o vacío → `{ ok: false, reason: 'invalid' }`.
- Se parte por `+` (la última parte es la tecla). Inválido si: alguna parte está vacía (`'CommandOrControl+'`, `'CommandOrControl++'`), la tecla no es válida, no hay ningún modificador, o algún modificador no es uno de `CommandOrControl`, `Alt`, `Shift`, `Super`.
- Tecla válida (`isValidKey`): `^[A-Z0-9]$`; `F1`–`F24`; `num0`–`num9`; nombres `Space, Plus, Enter, Return, Tab, Backspace, Delete, Insert, Escape, Esc, Home, End, PageUp, PageDown, Up, Down, Left, Right, PrintScreen, VolumeUp, VolumeDown, VolumeMute, MediaNextTrack, MediaPreviousTrack, MediaStop, MediaPlayPause, numadd, numsub, nummult, numdiv, numdec`; o un carácter de puntuación ASCII de la regex `/^[!-*,-/:-@[-`{-~]$/` (todo ASCII imprimible no alfanumérico salvo `+`, que se escribe `Plus`).
- Si es exactamente uno de los reservados (`CommandOrControl+Alt+Right`, `CommandOrControl+Alt+Left`, `CommandOrControl+Alt+M`) → `{ ok: false, reason: 'reserved' }`.
- Si no → `{ ok: true }`. (Basta un modificador cualquiera: `Shift+A` es válido.)

`matches(input, acc)` = `fromKeyEvent(input) === acc` (comparación exacta de cadenas; el orden de modificadores es siempre el canónico).

`displayAccelerator(acc)` (`shortcut.js:98-103`): partes unidas por `' + '`, renombrando `CommandOrControl→Ctrl`, `Super→Win`, `Plus→+`. Ej.: `'CommandOrControl+Shift+Space'` → `'Ctrl + Shift + Space'`. Vacío o no string → `''`.

### E.5 Grabación de un atajo nuevo (engranaje de la página + main)

Flujo completo (interfaz en `preload.js`, lógica en `main.js`):

1. En el menú de ajustes, clic en la fila "Atajo" → `startRecording()`: `recordingKey = true`, borra el error, `ipcRenderer.send('settings:recording', true)` (`preload.js:274-277`). La fila muestra `set.shortcutHint` ("Pulsa la nueva combinación de teclas").
2. Main `setRecording(true)` (`main.js:804-815`): `recording = true`; quita los tres atajos globales fijos y el global del menú rápido. Mientras `recording`, `before-input-event` de la ventana principal ignora **todas** las teclas (ni F5 ni zoom ni el atajo del menú), para que la página reciba la combinación tal cual.
3. En la página, un `keydown` en captura sobre `window` (`preload.js:333-358`): `preventDefault` + `stopImmediatePropagation` (no llega a YouTube). `Esc` **sin modificadores** cancela (`stopRecording()` y redibujar). Las autorrepeticiones se ignoran. Si no, manda `{ key, code, ctrlKey, altKey, shiftKey, metaKey }` por `invoke('settings:key')`.
4. Main `settings:key` (`main.js:822-828`): normaliza los campos (`String(...)`, `!!...`) y devuelve `fromKeyEvent(...)` (acelerador o `null` si solo hay modificadores, y entonces se sigue grabando). Con modificadores, Esc se graba como cualquier otra tecla (p. ej. `CommandOrControl+Shift+Escape`).
5. Con un acelerador: `stopRecording()` (manda `settings:recording false` → main vuelve a registrar los globales fijos y el del menú; si este ya no se puede registrar, `qmGlobalAcc = null`) y `invoke('settings:set', { quickMenu: { accelerator } })`.
6. Main `settings:set` valida y aplica (E.1/H). Si responde `{ error }`, la página muestra bajo la fila: `taken` → `set.shortcutTaken`, `invalid` → `set.shortcutInvalid`, `reserved` → `set.shortcutReserved` (cualquier otro → `set.shortcutInvalid`) y conserva el atajo anterior. Si va bien, la fila muestra `display.shortcut`.
7. La grabación también termina en main si la ventana principal pierde el foco (`blur`) o se recarga (`dom-ready`); y en la página al cerrar el panel de ajustes.

---

## F. Otras funciones del proceso principal

### F.1 Notificaciones (IPC `track`, `main.js:677-684`)

Al recibir `track { title, artist, playing }`:

- `changed` = hay título **y** (título o artista distintos del anterior).
- Se guarda `track = { title: info.title || '', artist: info.artist || '', playing: !!info.playing }` y `refreshUi()`.
- Notificación solo si: `changed` y `info.playing` y `settings.notifications` y `Notification.isSupported()` y la ventana principal **no** está enfocada (o no existe). `new Notification({ title: track.title, body: track.artist, icon, silent: true }).show()` — silenciosa, con el icono de la app; el clic no hace nada (no hay listener).

### F.2 Inicio con Windows

- Activar/desactivar: `app.setLoginItemSettings({ openAtLogin: v, args: ['--hidden'] })`; estado: `app.getLoginItemSettings().openAtLogin` (`main.js:336-339`). Arranca con `--hidden` (ventana oculta, solo bandeja).
- Reparación al abrir (`refreshLoginItem`, `main.js:920-927`, solo Windows y app empaquetada): ejecuta `reg query HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v local.youtube-music-client` (sin ventana). Si hay error (no existe la entrada), salida vacía, o la salida ya contiene la ruta del `.exe` actual (sin distinguir mayúsculas) → nada. Si existe y apunta a otra ruta (reinstalación en otra carpeta) → `setLoginItemSettings({ openAtLogin: true, args: ['--hidden'] })` con el `.exe` actual.

### F.3 Cerrar a la bandeja

- Cerrar la ventana (X o Alt+F4): `saveSettings()`; si `closeToTray` y no se está saliendo → se oculta (`main.js:252-255`).
- `window-all-closed` (`main.js:947-949`): en Windows/Linux, si `!closeToTray` → `app.quit()`.
- `before-quit` → `quitting = true; saveSettings()` (`main.js:912`). "Salir" de la bandeja y la actualización ponen `quitting = true` antes de `app.quit()`.

### F.4 Instancia única (`main.js:40-43`, `main.js:936`)

- `app.requestSingleInstanceLock()`; si falla → `app.quit()` y no se ejecuta nada más.
- `second-instance` → `showWindow()`. Se registra **después** de la comprobación de actualizaciones: abrir la app otra vez durante la descarga no abre la versión vieja (la segunda instancia sale y la primera no reacciona).

### F.5 User agent (`main.js:16-18`)

`app.userAgentFallback` sin el token `Electron/<versión>` ni `<app.getName()>/<versión>` (`ByTune/1.0.23`), quitando también el espacio previo. Queda el UA estándar de Chrome de esa versión de Chromium (Google rechaza el inicio de sesión si el UA dice "Electron"). Para LRCLIB se usa otro UA: `ByTune/<versión> (https://github.com/Jscruzgg/ByTune)` (`main.js:703`).

### F.6 Switches de Chromium

- Siempre: `enable-gpu-rasterization`, `enable-zero-copy` (`main.js:21-22`).
- Solo con la variable `YTM_DEBUG_PORT`: `remote-debugging-port=<puerto>` y `disable-features=CalculateNativeWinOcclusion` (sigue pintando tapada, para medir FPS) (`main.js:25-29`).
- Otras variables de entorno: `YTM_USER_DATA` (B.1), `PORTABLE_EXECUTABLE_DIR` (la pone el ejecutable portable de electron-builder; desactiva actualizaciones, G).

### F.7 Navegación interna/externa (`main.js:13`, `main.js:148-150`, `main.js:214-221`)

Hosts internos (regex sobre `new URL(url).hostname`; URL inválida → externa):

| Regex | Cubre |
|---|---|
| `/(^\|\.)music\.youtube\.com$/` | `music.youtube.com` y subdominios |
| `/(^\|\.)accounts\.google\.[a-z.]+$/` | `accounts.google.com`, `accounts.google.es`, `accounts.google.co.uk`… |
| `/(^\|\.)accounts\.youtube\.com$/` | inicio de sesión de YouTube |
| `/(^\|\.)consent\.youtube\.com$/` | página de consentimiento de cookies |
| `/(^\|\.)google\.com$/` | `google.com` y **cualquier** `*.google.com` |

`www.youtube.com`, `youtu.be` y demás → externos.

- `window.open` / `target=_blank` (`setWindowOpenHandler`): interno → `win.loadURL(url)` en la misma ventana; externo → `shell.openExternal(url)` (navegador del sistema). Siempre `deny` (nunca se abre otra ventana).
- `will-navigate` (navegaciones de documento iniciadas por la página): externo → `preventDefault()` + `openExternal`. (Las redirecciones de servidor y la navegación SPA no pasan por aquí.)

### F.8 Zoom, F5, Alt+flechas

Ver E.2: `F5` recarga; `Alt+←/→` historial (si se puede); `Ctrl + =`/`Ctrl + +` acercar 0,5 niveles, `Ctrl + -` alejar 0,5, `Ctrl + 0` restablecer; `Esc` sale de pantalla completa.

### F.9 Controles del reproductor (`PLAYER_JS`, `main.js:272-281`)

Se ejecutan en la página con `executeJavaScript` (errores ignorados). Literal:

- `playPause`: `(() => { const b = document.querySelector('ytmusic-player-bar[slot="player-bar"] #play-pause-button'); if (b) b.click(); else { const v = document.querySelector('video'); if (v) v.paused ? v.play() : v.pause(); } })()`
- `next`: `document.querySelector('ytmusic-player-bar[slot="player-bar"] .next-button')?.click()`
- `previous`: `document.querySelector('ytmusic-player-bar[slot="player-bar"] .previous-button')?.click()`
- `like`: `document.querySelector('ytmusic-player-bar[slot="player-bar"] yt-button-shape.like button')?.click()` (definido pero no lo usa ningún botón/atajo de main).

### F.10 Tareas periódicas

- `refreshLibrary`, `refreshSuggested`, `refreshCovers`: cada 15 min (`setInterval` desde el arranque del proceso, `main.js:457`, `466`, `675`) y a los 4 s de cada `dom-ready`.
- `refreshLibrary` además al abrir el menú si el último intento tiene más de 60 s (`libraryAt` se actualiza al **empezar** cada intento). Si ya hay una petición en curso, se reutiliza.

---

## G. Actualizaciones (electron-updater), empaquetado y publicación

### G.1 Flujo (`main.js:852-910`)

"Como Discord": en **cada arranque, antes de abrir la ventana**, se busca una versión más nueva en GitHub Releases (`provider: github, owner: Jscruzgg, repo: ByTune`, `package.json:42-48`, repo público, sin token). Si la hay, se descarga mostrando solo el aviso pequeño, se instala en silencio encima de la actual (misma carpeta, accesos directos y anclajes, sin asistente) y se abre la nueva. Sin preguntar y sin "más tarde". Sin conexión o ante cualquier fallo se abre la versión actual y se reintenta en el siguiente arranque.

`checkAndInstall()`: si ya hay una comprobación en curso devuelve la misma promesa (dos clics no descargan dos veces).

`runUpdate()` devuelve `'installing' | 'latest' | 'unsupported' | 'error'`:

1. `'unsupported'` si: no está empaquetada (`!app.isPackaged`), es la portable (`process.env.PORTABLE_EXECUTABLE_DIR` definido) o no se puede cargar `electron-updater`.
2. `getUpdater()` (una sola vez): `autoUpdater` con `logger` que escribe en `userData/updater.log` (`<ISO-8601> <nivel> <mensaje>\n`, niveles `info/warn/error`; `debug` descartado), `autoDownload = false`, `autoInstallOnAppQuit = false`, `installDirectory = carpeta del .exe actual` (el instalador recibe `/D=`; sin esto podría dejar una segunda copia), y `on('error')` → log.
3. `checkForUpdates()` en carrera con un tope de **10 000 ms** (`UPDATE_CHECK_TIMEOUT`; al vencer, error "update check timed out").
4. Sin resultado o `!result.isUpdateAvailable` → `'latest'`.
5. `showUpdateSplash(result.updateInfo.version)` (A.3).
6. `await downloadUpdate()` (sin tope de tiempo).
7. `updater.install(true, true)`: silencioso (`/S`) y `--force-run` (abre la nueva al terminar). Si devuelve falso → error "installer did not start".
8. `quitting = true; app.quit()`; devuelve `'installing'`.
9. Cualquier excepción → log `warn keeping current version: <mensaje>`, destruye el aviso si existía, `'error'`.

En `whenReady` (`main.js:929-945`), el orden es: `setAppUserModelId` (empaquetada) → `nativeTheme.themeSource = 'dark'` → `await checkAndInstall()`; si es `'installing'`, **no se crea nada más** → `second-instance` → `createWindow()` → `--hidden` → `createTray()` → `refreshLoginItem()` → `registerPlayerShortcuts()` → `applyQuickMenuShortcut(settings.quickMenu)`.

Botón "Buscar actualizaciones" del pie del menú de ajustes: IPC `update:check` → mismo `checkAndInstall()` (con la app abierta: si hay versión nueva aparece el aviso y la app se cierra y se reabre actualizada). Estados del botón en la página (`preload.js:138`, `preload.js:243`, `preload.js:265-271`):

| Estado | Texto es | Texto en | Botón |
|---|---|---|---|
| `''` (inicial) | Buscar actualizaciones | Check for updates | activo |
| `checking` (mientras espera) | Buscando… | Checking… | deshabilitado |
| `installing` | Buscando… | Checking… | deshabilitado |
| `latest` | Ya tienes la última versión | You're up to date | activo |
| `unsupported` | Solo en la versión instalada | Installed version only | activo |
| `error` (o respuesta desconocida/fallo IPC) | No se pudo comprobar | Couldn't check | activo |

Junto al botón: `ByTune <versión>`. Al abrir el panel de ajustes el estado se reinicia a `''` salvo que esté `checking`.

### G.2 Empaquetado (`package.json`)

- `appId: local.youtube-music-client`, `productName: ByTune`, salida `dist/`.
- Windows: icono `build/icon.ico`; destinos `nsis` y `portable`.
- NSIS: `oneClick: false`, `allowToChangeInstallationDirectory: true`, artefacto `ByTune-Setup-${version}.${ext}`. Portable: `ByTune-${version}-portable.${ext}`.
- Dependencia de ejecución: `electron-updater ^6.8.9`; desarrollo: `electron ^44.9.2`, `electron-builder ^26.15.3`.
- Scripts: `icon` (make-icon), `start` (`electron .`), `test` (`node --test "test/**/*.test.js"`), `dist` (`make-icon` + `electron-builder --win --publish never`), `release` (`scripts/release.mjs`).

### G.3 Publicación

`scripts/release.mjs` (`npm run release`): versión y tag `v<version>` de `package.json`; comprueba que no hay cambios sin commit, que `HEAD` está en `origin/main` (tras `git fetch`) y que el release `v<version>` no existe (`gh release view`); ejecuta `npm test` y `npm run dist`; exige `dist/ByTune-Setup-<v>.exe`, su `.blockmap`, `dist/latest.yml` (lo que lee electron-updater, debe contener `version: <v>`) y `dist/ByTune-<v>-portable.exe`; crea el release con `gh release create v<v> <archivos> --target <HEAD> --title "ByTune <v>" --notes "ByTune <v>. Installed copies update themselves on launch; for a first install, download ByTune-Setup-<v>.exe."`.

`.github/workflows/release.yml`: al **cerrarse con merge** un PR a `main`, en `windows-latest` (bash), con `contents: write` y concurrencia `release` en serie (sin cancelar): checkout de `main`, Node 22 con caché npm, `npm ci`; si el release `v<version>` ya existe, `npm version patch --no-git-tag-version`, commit `Release <version>` como `github-actions[bot]` y push a `main`; luego `npm run release`.

---

## H. IPC: todos los canales

### H.1 Página principal (`src/preload.js`) → main

| Canal | Tipo | Datos | Comprobación de remitente | Qué hace main |
|---|---|---|---|---|
| `track` | send | `{ title, artist, playing }` (la página lo manda solo cuando cambia, y al momento en `play`/`pause` del `<video>`) | ninguna | Actualiza `track`, `refreshUi()`, notificación (F.1) |
| `opened` | send | `location.href` (cuando cambia) | `e.sender === win.webContents` | `idFromUrl`; si es un álbum abierto como `VL` + `playlistId` de un álbum de la biblioteca, se traduce a su `id` (`MPREb_…`); `recordRecent(id)` (`main.js:479-485`) |
| `panels` | send | `{ left?, right? }` | ventana principal y objeto | Recorta, redondea, guarda (B.3) |
| `lang` | send | código de idioma de `<html lang>` (solo cuando cambia y no vacío) | ninguna | `pickLang`; guarda `settings.lang` si cambia; si cambió el idioma `refreshUi()`; **siempre** responde `strings` al remitente (y `initLyrics()` si es la ventana principal) (`main.js:744-753`) |
| `lyrics:request` | send | `{ videoId, title, artist, album, duration }` | ventana principal, en YT Music, objeto; `videoId` debe cumplir `/^[\w-]{6,20}$/` | Busca la letra (K.1) y la entrega con `window.__mdLyrics.receive(videoId, result)`; textos recortados a 300 caracteres; `duration` solo si es número finito, si no `null` |
| `lyrics:open` | send | boolean | ventana principal, en YT Music, boolean | Guarda `settings.lyricsOpen` si cambia |
| `np:fullscreen` | send | boolean | ventana principal, en YT Music, boolean | Si difiere del estado real → `setFullScreen(on)`; si no → `notifyFullscreen()` (recordarle el estado real) |
| `settings:recording` | send | boolean | ventana principal | `setRecording(!!on)` (E.5) |
| `settings:get` | invoke | — | ventana principal (si no, `null`) | `settingsView()` |
| `settings:key` | invoke | `{ key, code, ctrlKey, altKey, shiftKey, metaKey }` | ventana principal y objeto (si no, `null`) | `fromKeyEvent(...)` → acelerador o `null` |
| `settings:set` | invoke | patch (ver abajo) | ventana principal y objeto (si no, `null`) | Aplica y responde `settingsView()` o `{ error: 'invalid'\|'reserved'\|'taken' }` |
| `update:check` | invoke | — | ventana principal (si no, `null`) | `checkAndInstall()` → `'installing'\|'latest'\|'unsupported'\|'error'` |

`settings:set` (`main.js:831-850`). Patch: `{ theme?, notifications?, closeToTray?, startWithWindows?, quickMenu?: { accelerator?, reach?, fromOtherApps?, shiftEnter? } }`.

1. Si `patch.quickMenu` es objeto: `next = copia de settings.quickMenu`; `accelerator` si es string; `reach ∈ {focused, global}`, `fromOtherApps ∈ {float, front}`, `shiftEnter ∈ {queue, next}` (valores fuera de lista se ignoran). Si cambian `accelerator` o `reach` → `applyQuickMenuShortcut(next)`; si no es `'ok'` → **responde `{ error }` en ese momento** (los interruptores del mismo patch no se aplican). Si no cambian → guarda `next` directamente.
2. Para cada clave de `SWITCHES` con valor boolean → `changeSwitch`.
3. `refreshUi()` y responde `settingsView()`. (No manda `settings:changed`: el remitente ya tiene la respuesta.)

### H.2 Main → página principal

| Canal | Datos | Cuándo |
|---|---|---|
| `strings` | `{ lang, strings: STRINGS[lang] }` | En cada `dom-ready` y como respuesta a cada `lang` |
| `panels` | `settings.panels` | En cada `dom-ready` |
| `covers` | `{ browseId: url }` | En `dom-ready` (si hay guardadas) y tras cada `refreshCovers` con resultado |
| `settings:changed` | `settingsView()` | Tras cambiar un interruptor desde la bandeja |

### H.3 Menú rápido (`qmpre.js`, `window.qm`) ↔ main

`window.qm` (contextBridge, `qmpre.js:11-24`). `suscribir(canal, cb)` hace `removeAllListeners(canal)` antes de `on` (una recarga no duplica oyentes) y pasa solo los datos.

| API | Canal | Tipo | Datos | Respuesta / efecto |
|---|---|---|---|---|
| `qm.query(seq, text)` | `qm:query` | invoke | `{ seq, text }` | `{ seq, library: LibraryItem[], songs: Song[] \| null }` (null si remitente no es el menú) |
| `qm.onSongs(cb)` | `qm:songs` | main→menú | `{ seq, songs: Song[] }` | canciones tras el debounce |
| `qm.activate(item, shift)` | `qm:activate` | invoke | `{ item: { kind: 'song'\|'playlist'\|'album', id }, shift }` | `{ keepOpen: boolean, note?: 'added'\|'playingNext'\|'failed' }` |
| `qm.close()` | `qm:close` | send | — | `hideQuickMenu()` |
| `qm.player(action)` | `qm:player` | send | `'previous'\|'playPause'\|'next'` (otros ignorados) | `player(action)`; en `playPause`, cambio optimista de `track.playing` + `refreshUi()` |
| `qm.onTrack(cb)` | `qm:track` | main→menú | `{ title, artist, playing }` | en cada `refreshUi()` si existe la ventana |
| `qm.onShow(cb)` | `qm:show` | main→menú | `{ strings, recent, suggested, track, shiftEnter }` | cada vez que se abre |

Todas las de entrada comprueban `fromQuickMenu(e)` (`e.sender === qmWin.webContents`).

`qm:query` y `qm:activate`: detalle en I.6 e I.7.

### H.4 Llamadas a la página que no son IPC

- `pageApi(call, ...args)` (`main.js:395-439`): ejecuta `src/page-api.js` una vez por documento (instala `window.__md`) y luego `window.__md[call](...args)`. Llamadas permitidas: `library`, `searchSongs`, `suggestions`, `play`, `queue`, `open`, `lyrics`. Nunca rechaza: devuelve `{ error }` con `'sin ventana'`, `'llamada desconocida: <call>'`, `'fuera de YouTube Music'`, `'tiempo agotado'` (20 000 ms), `'sin respuesta'` (resultado no objeto) o el mensaje de la excepción. Si `window.__md` ha desaparecido (documento cambió), reinstala y reintenta una vez. Firmas usadas: `library()` → `{ items }`; `suggestions()` → `{ songs }`; `searchSongs(text)` → `{ songs }`; `play(videoId)`; `queue(videoId, 'next'|'end')` → `{ ok }`; `open(browseId)` → `{ ok }`; `lyrics(videoId)` → `{ lines, source }` o `{ error }`. (Su implementación está en `page-api.js`, fuera de esta parte.)
- `covers.js` (se ejecuta tras asegurar `page-api.js`) → `{ covers: { browseId: url } }`.
- `window.__mdLyrics.init(opts)`, `.setFullscreen(bool)`, `.receive(videoId, result)`.
- `PLAYER_JS` (F.9). Motion en mundo aislado 999. Hojas CSS de usuario.

---

## I. Menú rápido completo

### I.1 Objetivo (de la spec)

Lanzador flotante tipo Spotlight/Raycast, de teclado primero: atajo → escribir → flechas → Intro, sin ratón. Minimalista: sin etiquetas de acción por fila. Abrir una playlist o álbum **nunca** interrumpe lo que suena. Coste nulo mientras está cerrado.

### I.2 HTML (`qm.html`, literal)

```html
<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'self'; style-src 'self'; img-src https:">
  <title>ByTune</title>
  <link rel="stylesheet" href="quickmenu.css">
</head>
<body>
  <div class="qm-toast" id="toast" role="status" aria-live="polite">
    <svg class="qm-toast-ok" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true"><path d="m5 12 5 5 9-10"/></svg>
    <svg class="qm-toast-err" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true"><path d="M12 7v6M12 17h.01"/></svg>
    <span id="toastText"></span><span class="qm-toast-song" id="toastSong"></span>
  </div>
  <div class="qm" id="qm">
    <div class="qm-input">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#fff" stroke-width="2" aria-hidden="true"><circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/></svg>
      <input id="q" type="text" autocomplete="off" spellcheck="false" role="combobox" aria-expanded="true" aria-controls="list">
      <span class="qm-spin" aria-hidden="true"></span>
    </div>
    <div class="qm-div"></div>
    <div class="qm-list" id="list" role="listbox"></div>
    <div class="qm-np off" id="np">
      <div class="qm-np-text"><span class="qm-np-title" id="npTitle"></span><span class="qm-np-artist" id="npArtist"></span></div>
      <!-- tabindex -1: el teclado se queda en el campo de búsqueda (Intro activa la fila, no el botón) -->
      <button type="button" tabindex="-1" data-act="previous"><svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M6 5h2v14H6zM20 5v14L9 12z"/></svg></button>
      <button type="button" tabindex="-1" data-act="playPause" class="qm-np-play">
        <svg class="qm-np-icon-play" width="20" height="20" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M8 5v14l11-7z"/></svg>
        <svg class="qm-np-icon-pause" width="20" height="20" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M7 5h4v14H7zM13 5h4v14h-4z"/></svg>
      </button>
      <button type="button" tabindex="-1" data-act="next"><svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M16 5h2v14h-2zM4 5v14l11-7z"/></svg></button>
    </div>
    <div class="qm-foot">
      <div class="qm-keys" id="foot"></div>
      <button type="button" tabindex="-1" class="qm-more" id="more" aria-expanded="false"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="m6 9 6 6 6-6"/></svg></button>
    </div>
  </div>
  <script src="quickmenu.js"></script>
</body>
</html>
```

Estructura de arriba abajo:

1. **Aviso (toast)** `#toast`, fijo arriba del todo (fuera del panel, en el margen de 60 px).
2. **Panel** `.qm#qm`:
   1. Campo `.qm-input`: lupa (círculo r=7 en (11,11) + línea (20,20)→(16.5,16.5), trazo blanco 2, 18 px, opacidad .55), `<input id="q">`, rueda de carga `.qm-spin`.
   2. Separador `.qm-div`.
   3. Lista `.qm-list#list` (`role=listbox`): etiquetas de sección, filas, separadores entre secciones.
   4. **Lo que suena** `.qm-np#np` (oculto con `.off` si no hay canción): título + artista, botones anterior (18 px), reproducir/pausa (20 px, círculo blanco), siguiente (18 px). Iconos (viewBox 24): anterior = barra `M6 5h2v14H6z` + triángulo `M20 5v14L9 12z`; play = `M8 5v14l11-7z`; pausa = `M7 5h4v14H7zM13 5h4v14h-4z`; siguiente = barra `M16 5h2v14h-2z` + triángulo `M4 5v14l11-7z`.
   5. **Pie** `.qm-foot`: atajos `#foot` + botón chevron `#more` (16 px, `m6 9 6 6 6-6`, trazo 2).

CSP: solo scripts/estilos propios; imágenes solo `https:`.

### I.3 CSS (`qm.css`, literal y comentado)

```css
:root {
  --md-text: #fff;
  --md-text-2: rgba(255,255,255,.7);
  --md-text-3: rgba(255,255,255,.56);
  --md-on-fill: #000;
  --md-raised: #1e1e1e;
  --md-raised-line: rgba(255,255,255,.1);
  --md-raised-shadow: 0 16px 40px rgba(0,0,0,.5);
  --md-field: rgba(255,255,255,.08);
  --md-state-hover: rgba(255,255,255,.08);
  --md-state-on: rgba(255,255,255,.14);
  --md-state-ink: rgba(0,0,0,.1);
  --md-line: rgba(255,255,255,.08);
  --md-line-strong: rgba(255,255,255,.24);
  --md-danger: #ff8b8b;
}
html, body { margin: 0; background: transparent; overflow: hidden; }
body {
  padding: 60px 24px 48px;
  font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
  color: var(--md-text);
  user-select: none;
}

.qm {
  width: 540px;
  background: var(--md-raised);
  border: 1px solid var(--md-raised-line);
  border-radius: 16px;
  box-shadow: var(--md-raised-shadow);
  padding: 8px;
  box-sizing: border-box;
  animation: qm-in .12s ease-out;
}
@keyframes qm-in { from { opacity: 0; transform: translateY(-4px); } to { opacity: 1; transform: none; } }

.qm-input {
  display: flex; align-items: center; gap: 10px;
  background: var(--md-field); border: 1px solid var(--md-line); border-radius: 10px;
  padding: 11px 14px; font-size: 16px;
}
.qm-input svg { flex: none; opacity: .55; }
.qm-input input {
  flex: 1; min-width: 0; background: none; border: 0; outline: 0; padding: 0;
  color: var(--md-text); caret-color: var(--md-text); font: inherit; user-select: text;
}
.qm-input input::placeholder { color: var(--md-text-3); }

.qm-div { height: 1px; background: var(--md-line); margin: 8px -8px; }
.qm-list { max-height: 440px; overflow-y: auto; overflow-x: hidden; }
.qm-list::-webkit-scrollbar { width: 0; }

.qm-label { margin: 10px 10px 6px; font-size: 13px; color: var(--md-text-3); font-weight: 500; }
.qm-list .qm-div { margin: 8px -8px; }

.qm-row {
  display: flex; align-items: center; gap: 12px;
  padding: 6px 10px; border-radius: 8px; font-size: 15px; cursor: default;
}
.qm-row.sel { background: var(--md-state-on); }
.qm-art { width: 32px; height: 32px; border-radius: 4px; flex: none; background: var(--md-state-on); object-fit: cover; }
.qm-title { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.qm-meta { margin-left: auto; padding-left: 12px; font-size: 13px; color: var(--md-text-3); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; flex: none; max-width: 55%; }

.qm-spin {
  flex: none; width: 16px; height: 16px; box-sizing: border-box; border-radius: 50%;
  border: 2px solid var(--md-line-strong); border-top-color: var(--md-text); opacity: 0;
}
.qm.loading .qm-spin { opacity: 1; animation: qm-spin .7s linear infinite; }
@keyframes qm-spin { to { transform: rotate(360deg); } }

.qm-toast {
  position: fixed; top: 12px; left: 50%; max-width: 500px; box-sizing: border-box;
  display: flex; align-items: center; gap: 8px;
  padding: 8px 14px; border-radius: 999px; font-size: 14px; white-space: nowrap;
  background: var(--md-text); color: var(--md-on-fill); box-shadow: var(--md-raised-shadow);
  opacity: 0; transform: translate(-50%, 6px); transition: opacity .15s, transform .15s;
  pointer-events: none;
}
.qm-toast.show { opacity: 1; transform: translate(-50%, 0); }
.qm-toast svg { flex: none; }
.qm-toast.err { background: var(--md-raised); color: var(--md-danger); border: 1px solid var(--md-raised-line); }
.qm-toast .qm-toast-err, .qm-toast.err .qm-toast-ok { display: none; }
.qm-toast.err .qm-toast-err { display: block; }
.qm-toast-song { min-width: 0; overflow: hidden; text-overflow: ellipsis; color: rgba(0,0,0,.6); }
.qm-toast-song:not(:empty)::before { content: '· '; }

.qm-np {
  display: flex; align-items: center; gap: 4px; margin: 8px -8px 0; padding: 8px 12px 0 18px;
  border-top: 1px solid var(--md-line);
}
.qm-np.off { display: none; }
.qm-np-text { flex: 1; min-width: 0; display: flex; flex-direction: column; font-size: 13px; }
.qm-np-title, .qm-np-artist { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.qm-np-artist { color: var(--md-text-2); font-size: 12px; }
.qm-np button {
  flex: none; display: grid; place-items: center; width: 34px; height: 34px; padding: 0;
  border: 0; border-radius: 50%; background: none; color: var(--md-text-2); cursor: pointer;
}
.qm-np button:hover { background: var(--md-state-hover); color: var(--md-text); }
.qm-np .qm-np-play { background: var(--md-text); color: var(--md-on-fill); }
.qm-np .qm-np-play:hover { background: var(--md-text); color: var(--md-on-fill); box-shadow: inset 0 0 0 20px var(--md-state-ink); }
.qm-np-icon-pause, .qm-np.playing .qm-np-icon-play { display: none; }
.qm-np.playing .qm-np-icon-pause { display: block; }

.qm-foot {
  display: flex; align-items: flex-start; gap: 8px; margin: 8px -8px -8px; padding: 9px 10px 9px 18px;
  border-top: 1px solid var(--md-line); font-size: 12px; color: var(--md-text-3);
}
.qm-keys { flex: 1; display: flex; flex-wrap: wrap; gap: 6px 16px; }
.qm-more {
  flex: none; display: grid; place-items: center; width: 22px; height: 20px; padding: 0;
  border: 0; border-radius: 4px; background: none; color: var(--md-text-2); cursor: pointer;
}
.qm-more:hover { background: var(--md-state-hover); color: var(--md-text); }
.qm-more[aria-expanded="true"] svg { transform: rotate(180deg); }
.qm-foot span { white-space: nowrap; }
.qm-foot kbd {
  background: var(--md-state-hover); border: 1px solid var(--md-line); border-radius: 4px;
  padding: 1px 6px; color: var(--md-text-2); font-family: inherit; font-size: 11px;
}

.qm.empty > .qm-div { display: none; }
```

Lectura para la réplica (valores derivados):

- **Tipografía**: `system-ui` → en Windows 11, Segoe UI Variable. Tamaños: campo 16 px; filas 15 px; etiquetas de sección 13 px peso 500; detalles 13 px; título "lo que suena" 13 px y artista 12 px; pie 12 px; teclas `kbd` 11 px; aviso 14 px.
- **Colores**: panel `#1e1e1e` con borde 1 px blanco 10 %; campo blanco 8 % con borde blanco 8 %; fila seleccionada blanco 14 %; portada vacía blanco 14 %; texto principal `#fff`, secundario blanco 70 %, terciario blanco 56 %; líneas blanco 8 %.
- **Sombra** del panel y del aviso: `0 16px 40px rgba(0,0,0,.5)` (desplazamiento y 16, desenfoque 40, sin expansión).
- **Radios**: panel 16, campo 10, fila 8, portada 4, `kbd` 4, botón chevron 4, botones de reproducción círculo, aviso píldora (999).
- **Espaciado**: panel padding 8 (borde incluido en 540 por `box-sizing`); campo padding 11/14 y separación 10 entre lupa, input y rueda; separador 1 px con margen vertical 8 y horizontal −8 (ocupa todo el ancho interior del panel, de borde a borde); lista altura máxima 440 px con scroll vertical sin barra visible (ancho 0) y sin scroll horizontal; etiqueta margen 10 arriba, 10 lados, 6 abajo; fila padding 6/10, separación 12; portada 32×32 `object-fit: cover`; detalles a la derecha (`margin-left: auto`), padding izq. 12, como mucho 55 % del ancho de la fila, con elipsis; el título también con elipsis.
- **Lo que suena**: margen superior 8 y laterales −8 (a borde), padding 8 arriba, 12 derecha, 0 abajo, 18 izquierda, borde superior 1 px; botones 34×34, separación 4; el de reproducir es un círculo blanco con icono negro; hover de los otros: fondo blanco 8 % y texto blanco; hover del de reproducir: oscurece con una sombra interior negra 10 %.
- **Pie**: margen 8 arriba, −8 lados y abajo (pegado al borde inferior del panel), padding 9/10/9/18, borde superior 1 px; atajos en flex con salto de línea, separación 6 vertical y 16 horizontal; cada pista `<span>` sin salto interno; `kbd` padding 1/6. Botón chevron 22×20, alineado arriba; abierto → flecha girada 180° (sin transición).
- **Sin resultados** (`.qm.empty`): se oculta el separador bajo el campo (solo campo, "lo que suena" y pie).
- **Rueda**: 16×16, borde 2 px blanco 24 % con el tramo superior blanco; invisible (opacidad 0) salvo con `.qm.loading` → opacidad 1 y rotación 360° cada 0,7 s, lineal, infinita.
- **Animación de apertura**: `qm-in` 0,12 s `ease-out`, de opacidad 0 y `translateY(-4px)` a opacidad 1 y sin desplazamiento. Ojo: es una animación CSS del elemento `.qm`, que no se recrea al reabrir (la ventana se oculta y se vuelve a mostrar con el mismo documento), así que en el código actual solo se ejecuta al cargar el documento por primera vez, con la ventana aún oculta. **No está claro** que llegue a verse; la intención es 120 ms de fundido+desplazamiento al abrir. **Cierre: sin animación** (`qmWin.hide()` inmediato).
- **Aviso**: fijo a 12 px del borde superior de la ventana, centrado horizontalmente, ancho máximo 500, píldora blanca con texto negro (éxito, con check) o, en error, fondo `#1e1e1e`, texto `#ff8b8b`, borde blanco 10 % e icono "!" (`M12 7v6M12 17h.01`). Entra con transición de 0,15 s (curva por defecto `ease`) en opacidad 0→1 y desplazamiento y 6 px→0; sale al revés. No recibe clics. El nombre de la canción va tras `· ` en negro 60 % con elipsis (en el aviso de error ese color se mantiene, sobre fondo oscuro).

### I.4 Comportamiento (`qm.js`)

Estado (`qm.js:36-50`): `strings`, `shiftEnter` (`'queue'`), `recent`, `suggested`, `toastTimer`, `pieAbierto` (pie desplegado), `library`, `songs`, `songsText` (texto de la consulta a la que pertenecen las canciones), `latestSeq`, `flat` (filas en orden), `selIndex`, `selKey`, `rows` (Map clave→fila DOM, para no recrear filas ni portadas al teclear), `activando`.

Constantes: `TOAST_MS = 2200`; `NOTE_KEY = { added: 'qm.added', playingNext: 'qm.playingNext', failed: 'qm.queueFailed' }`.

Seguridad: los datos vienen de la página (no de fiar): todo texto con `textContent`; portadas solo como `img.src` y solo si empiezan por `https://`.

**Normalización de elementos** (`normalizar`, `qm.js:58-68`) → `{ kind, id, title, cover, details }`:

- Con `videoId` → canción: `kind 'song'`, `id = videoId`, `details = artista · duración` (une con `' · '` solo los que existan).
- Sin `id` → descartado.
- `type === 'album'` → `kind 'album'`, `details = "Álbum · artista"` (`qm.album`).
- Cualquier otro → `kind 'playlist'`, `details = "Playlist"` (`qm.playlist`).
- Clave de fila: `kind + ':' + id`.

**Fila** (`crearFila`): `div.qm-row[role=option]` con portada (`img.qm-art`, `alt=''`, no arrastrable; si falla la carga se sustituye por un `div.qm-art` gris; sin URL https → `div.qm-art` gris), `span.qm-title` y `span.qm-meta`. Las filas se reutilizan por clave mientras el menú está abierto; el Map se vacía en cada apertura.

**Secciones** (`secciones`, `qm.js:115-126`):

- Campo vacío (o solo espacios): **Recientes** (`qm.recent`) = `recent` (elementos de la biblioteca) y **Sugerencias** (`qm.suggested`) = `suggested` (canciones).
- Escribiendo: **Tu biblioteca** (`qm.library`) = `library` (filtrado, máx. 5) y **Canciones** (`qm.songs`) = `songs`.

**Pintado** (`render`, `qm.js:143-173`): para cada sección **no vacía**: separador `div.qm-div` antes de cada sección salvo la primera, etiqueta `div.qm-label` y filas (cada fila recibe `id = 'qm-r' + índice`). Sustituye todo el contenido de la lista. `.qm.empty` si no hay filas. Selección: se conserva la fila con la clave `selKey` si sigue en la lista; si no, la primera. `pintarSeleccion`: clase `sel` y `aria-selected` en la fila elegida, `aria-activedescendant` del input, y `scrollIntoView({ block: 'nearest' })`.

**Búsqueda** (`alEscribir`, `qm.js:257-284`; evento `input`):

1. `seq = ++latestSeq`; `selKey = null` (vuelve a la primera fila).
2. Texto vacío tras `trim` → `library = []`, `songs = []`, sin rueda, `render()` (vuelven Recientes/Sugerencias), fin.
3. Rueda encendida (`.loading`, `aria-busy="true"` en la lista).
4. Si hay canciones y el texto nuevo **no** es ampliación ni recorte del anterior (`text.startsWith(songsText) || songsText.startsWith(text)`), se vacían (si es ampliación/recorte se quedan las viejas hasta que lleguen las nuevas: sin parpadeos). `songsText = text`.
5. `consultar(seq, text)` (`qm.query` envuelto en `latestOnly`: solo se entrega la respuesta de la última llamada; las anteriores resuelven `undefined`). Al responder, si `res.seq !== latestSeq` se ignora; `library = res.library`; si `res.songs` es array (estaban en caché) → `songs = res.songs` y se apaga la rueda; `render()`.

`alLlegarCanciones(msg)` (`qm:songs`): se ignora si `msg.seq !== latestSeq` o el campo está vacío; `songs = msg.songs` (no array → `[]`, sección oculta); `songsText = input.value`; rueda apagada; `render()`.

**Debounce** (en main, no en el menú): 200 ms tras la última tecla (I.6).

**Apertura** (`alMostrar`, `qm:show`, `qm.js:294-316`): toma `strings`, `shiftEnter` (`'next'` o, cualquier otro valor, `'queue'`), `recent`, `suggested`; vacía `library`, `songs`, `songsText`; `activando = false`; `latestSeq++` (descarta respuestas pendientes de la vez anterior); rueda apagada; oculta el aviso; pinta "lo que suena"; pie recogido; nuevo Map de filas; `selKey = null`; campo vacío; placeholder `qm.placeholder`; pinta pie y lista; foco en el campo. Cada apertura empieza de cero (no recuerda el texto anterior).

**Lo que suena** (`pintarTrack`, `qm:show` y `qm:track`): sin título → `.off` (oculto); `.playing` si suena (muestra el icono de pausa); título y artista; `aria-label` de los botones: `tray.previous`, `tray.pause`/`tray.play`, `tray.next`. Clic en un botón → `qm.player(data-act)`.

**Activar** (`activar(shift, indice = selIndex)`, `qm.js:213-227`):

- Sin fila → nada. Mayús con algo que no sea canción → nada. Si hay una activación en curso (`activando`) → nada (hasta que responda).
- `qm.activate({ kind, id }, shift)`; al responder, si `keepOpen` y hay `note` con texto → `mostrarAviso(texto, título de la fila, note === 'failed')`. Errores ignorados.

**Aviso** (`mostrarAviso`): reinicia el temporizador, pone texto y canción, `.err` si es error, `.show`, y lo quita a los **2200 ms**. Textos: `added` → "Añadida a la cola" / "Added to the queue"; `playingNext` → "Sonará a continuación" / "Plays next"; `failed` → "No se pudo añadir a la cola" / "Couldn't add to the queue" (rojo).

**Pie** (`pintarPie`, `qm.js:229-247`): pistas `<span><kbd>TECLA</kbd> texto</span>` (las teclas, en todos los idiomas, con el nombre impreso en el teclado):

- Siempre: `Shift + Enter` + (`qm.nextHint` "a continuación"/"play next" si `shiftEnter === 'next'`, si no `qm.queueHint` "a la cola"/"add to queue"); `Alt + ← ↓ →` + `qm.playback` ("anterior / pausa / siguiente" / "previous / pause / next").
- Desplegado (`pieAbierto`), además: `↑↓` + `qm.move` ("mover"/"move"); `Enter` + `qm.playOpen` ("reproducir / abrir"/"play / open"); `Esc` + `qm.close` ("cerrar"/"close").
- Botón `#more`: `aria-expanded`, `aria-label` y `title` = `qm.allShortcuts` ("Todos los atajos (F1)" / "All shortcuts (F1)"). Clic o `F1` → alterna. Se recoge en cada apertura.

### I.5 Teclado completo (`qm.js:318-335`)

`keydown` en `document`; ignorado si `e.isComposing` (IME).

| Tecla | Efecto |
|---|---|
| `Alt + ←` | `qm.player('previous')` (sin repetición; `preventDefault`) — también mientras se escribe |
| `Alt + ↓` | `qm.player('playPause')` (sin repetición) |
| `Alt + →` | `qm.player('next')` (sin repetición) |
| `↓` | mover selección +1, **circular** (de la última a la primera); `preventDefault` (el cursor del campo no se mueve) |
| `↑` (también `Alt + ↑`, que no está en el mapa de Alt y cae aquí) | mover −1, circular; `preventDefault` |
| `Enter` | `activar(false)`; con la tecla mantenida no se repite |
| `Shift + Enter` | `activar(true)` (solo canciones; sin repetición: si no, encolaría muchas veces). Cualquier otro modificador con Enter (Ctrl+Enter) cuenta como Enter |
| `Esc` | `qm.close()` → main oculta y devuelve el foco a la app si se abrió sobre ella |
| `F1` | alterna el pie con todos los atajos |
| El atajo del menú (p. ej. Ctrl+Shift+Space) | cierra (main, A.2) |
| `Tab`, `Inicio`, `Fin`, `RePág`, `AvPág` | **no gestionadas**: comportamiento por defecto del navegador. Los botones tienen `tabindex=-1`, así que el único elemento enfocable es el campo; el código no define qué pasa con Tab (no está claro; en la réplica, lo razonable es que no haga nada) |
| Resto | escriben en el campo |

Ratón (`qm.js:339-366`):

- `mousedown` en cualquier punto **fuera del panel** (el margen transparente de la ventana) → `qm.close()`.
- `mousedown` en la lista, en "lo que suena" y en el chevron → `preventDefault` (no quitan el foco al campo).
- `mousemove` sobre una fila → la selecciona (no hay estilo `:hover` en filas: el ratón mueve la misma selección que el teclado).
- Clic en una fila → `activar(false, índice)` (el clic nunca es "Mayús").

### I.6 Consulta en main (`qm:query`, `main.js:594-620`)

- `text` recortado a 200 caracteres. Clave de caché: `text.trim().toLowerCase()` con espacios colapsados (`\s+` → `' '`).
- Caché de canciones `createCache(100, 10 * 60 * 1000)`: 100 entradas (sale la más vieja), frescas 10 min, después sirven como "viejas".
- `songsSeq = seq`; cancela el temporizador pendiente.
- Si la clave no está vacía y no hay entrada **fresca** → temporizador de **200 ms**: `pageApi('searchSongs', text)` (con el texto original); si devuelve array lo guarda en caché; si falla y había entrada vieja → no manda nada (se quedan las que ya se ven); si ya hay una consulta más nueva (`seq !== songsSeq`) o el menú se destruyó → no manda nada; si no → `qm:songs { seq, songs }` (`[]` si falló).
- Respuesta inmediata: `{ seq, library: filterLibrary(settings.library, text), songs: hit ? hit.value : null }`. Con entrada vieja, devuelve las viejas al instante **y** pide las nuevas.
- `hideQuickMenu` cancela el temporizador.

### I.7 Activación en main (`qm:activate`, `main.js:622-648`)

El menú nunca se cierra solo: lo cierra main.

| Elemento | Sin Mayús | Con Mayús |
|---|---|---|
| Canción | `pageApi('play', id)` (sin esperar; sustituye la cola, como en la búsqueda de YT); responde `{ keepOpen: true }` → **el menú sigue abierto** | `pageApi('queue', id, shiftEnter === 'next' ? 'next' : 'end')` (esperando); responde `{ keepOpen: true, note }` con `note = 'failed'` si no `res.ok`, `'playingNext'` si `next`, `'added'` si no |
| Playlist / álbum | `hideQuickMenu()` (con devolución de foco), `showWindow()` (la página abierta tiene que verse aunque la ventana esté oculta, minimizada o detrás), `pageApi('open', id)` y, si `ok`, `recordRecent(id)`; responde `{ keepOpen: false }`. La reproducción no se toca | nada, `{ keepOpen: true }` |
| Otro / id no válido / remitente ajeno | `{ keepOpen: true }` | ídem |

### I.8 "reach" y "fromOtherApps"

- `reach: 'focused'` (por defecto, texto "Con la app enfocada"): el atajo solo lo atiende `before-input-event` de la ventana principal (abre/cierra) y el de la ventana del menú (cierra). Nada desde otras apps.
- `reach: 'global'` ("En cualquier lugar"): `globalShortcut` lo registra; funciona desde cualquier programa. `fromOutside = !win || !win.isFocused()`.
  - `fromOtherApps: 'float'` ("Flotando sobre la app", por defecto): el menú aparece encima de lo que haya, en la pantalla del cursor, sin tocar la ventana (aunque esté oculta o minimizada). Reproducir suena en segundo plano. Abrir playlist/álbum muestra la ventana.
  - `fromOtherApps: 'front'` ("Trayendo la ventana al frente"): `showWindow()` primero y el menú encima, en la pantalla de la ventana.
- En el menú de ajustes, el grupo "Desde otras apps" está deshabilitado (40 % de opacidad según el plan) salvo con "En cualquier lugar" (es interfaz de la página, documentada en otra parte).
- Con la app enfocada el menú siempre se abre sobre su pantalla.

### I.9 Recientes y sugerencias

- Recientes = `recentItems(settings.recent, settings.library)` (K.2): hasta 6 elementos de la biblioteca abiertos hace poco (desde el menú o navegando por la página); si no queda ninguno, los 6 primeros de la biblioteca.
- Sugerencias = `settings.suggested` (canciones del inicio de YT Music, `pageApi('suggestions')`), sin límite en main (el número lo da `page-api.js`).

---

## J. i18n — todas las cadenas (`src/lib/i18n.js`)

Reglas:

- `pickLang(code)`: `'es'` si `code` es string y, tras `trim()`, cumple `/^es(?![a-z])/i` (es, es-419, ES-es, `es_ES`); cualquier otra cosa (incluido `null`, `''`, `esp`) → `'en'` (`i18n.js:156-158`).
- `t(lang, key, vars)`: el texto del idioma, si no el inglés, si no la propia clave; sustituye `{nombre}` por `vars[nombre]` (si no existe en `vars`, deja el marcador) (`i18n.js:162-165`).
- Idioma activo (`main.js:79`): `pickLang(settings.lang || app.getLocale())`; cuando la página informa (`lang`) ese manda y se guarda. Afecta a bandeja, thumbar, menú rápido, aviso de actualización, y se manda a la página (`strings`).
- Los nombres de canciones, playlists y álbumes de YouTube nunca se traducen. `'YouTube Music'` del título/bandeja no se traduce.

| Clave | es | en |
|---|---|---|
| `tray.play` | Reproducir | Play |
| `tray.pause` | Pausar | Pause |
| `tray.next` | Siguiente | Next |
| `tray.previous` | Anterior | Previous |
| `tray.show` | Mostrar ventana | Show window |
| `tray.quit` | Salir | Quit |
| `set.theme` | Tema ByTune | ByTune theme |
| `set.notify` | Notificar cambio de canción | Notify on song change |
| `set.closeToTray` | Cerrar a la bandeja | Close to tray |
| `set.startWithWindows` | Iniciar con Windows | Start with Windows |
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
| `set.title` | Ajustes | Settings |
| `upd.splash` | Actualizando ByTune a la versión {version}… | Updating ByTune to version {version}… |
| `upd.check` | Buscar actualizaciones | Check for updates |
| `upd.checking` | Buscando… | Checking… |
| `upd.latest` | Ya tienes la última versión | You're up to date |
| `upd.unsupported` | Solo en la versión instalada | Installed version only |
| `upd.error` | No se pudo comprobar | Couldn't check |
| `nav.back` | Atrás | Back |
| `nav.forward` | Adelante | Forward |
| `nav.home` | Volver a YouTube Music | Back to YouTube Music |
| `lyrics.show` | Mostrar letra | Show lyrics |
| `lyrics.hide` | Ocultar letra | Hide lyrics |
| `lyrics.credit` | Letra de LRCLIB | Lyrics from LRCLIB |
| `lyrics.none` | No hay letra para mostrar aquí :( | No lyrics to show here :( |
| `lyrics.unsynced` | Esta letra no está sincronizada con la canción | These lyrics aren't synced to the song |
| `np.fullscreen` | Pantalla completa | Full screen |
| `np.exitFullscreen` | Salir de pantalla completa | Exit full screen |
| `np.toVideo` | Cambiar a vídeo | Switch to video |
| `np.toSong` | Cambiar a canción | Switch to song |
| `qm.placeholder` | Buscar canciones o ir a una playlist… | Search songs or jump to a playlist… |
| `qm.recent` | Recientes | Recent |
| `qm.suggested` | Sugerencias | Suggested |
| `qm.library` | Tu biblioteca | Your library |
| `qm.songs` | Canciones | Songs |
| `qm.playlist` | Playlist | Playlist |
| `qm.album` | Álbum | Album |
| `qm.added` | Añadida a la cola | Added to the queue |
| `qm.playingNext` | Sonará a continuación | Plays next |
| `qm.queueFailed` | No se pudo añadir a la cola | Couldn't add to the queue |
| `qm.move` | mover | move |
| `qm.playOpen` | reproducir / abrir | play / open |
| `qm.queueHint` | a la cola | add to queue |
| `qm.nextHint` | a continuación | play next |
| `qm.close` | cerrar | close |
| `qm.playback` | anterior / pausa / siguiente | previous / pause / next |
| `qm.allShortcuts` | Todos los atajos (F1) | All shortcuts (F1) |

Caracteres especiales: `…` es U+2026 (un carácter) en `upd.splash`, `upd.checking`, `qm.placeholder`. Los apóstrofos ingleses son `'` ASCII. 61 claves en cada idioma (mismas claves en ambos).

---

## K. Letras y biblioteca

### K.1 Letras (`lyrics-source.js`, `lyrics-query.js`, `lrc.js`, `cache.js`)

Petición (IPC `lyrics:request`, H.1) → `lyricsSource.get(req)`; si lanza → `{ kind: 'none' }`; si mientras tanto la ventana se cerró o salió de YT Music no se entrega. Configuración (`main.js:703-708`): `fetchJson(url, { signal })` con `net.fetch` de Electron y `User-Agent: ByTune/<versión> (https://github.com/Jscruzgg/ByTune)`; devuelve `{ status, body }` con `body` = JSON solo si `status === 200`, si no `null`. `pageLyrics(videoId)` = `pageApi('lyrics', videoId)`.

Resultados posibles:

- `{ kind: 'synced', lines: [{ t, text, gap? }], source: 'LRCLIB' }`
- `{ kind: 'plain', lines: string[], source }` (`source` = el de la página o `'LRCLIB'`)
- `{ kind: 'none' }`

`createLyricsSource({ fetchJson, pageLyrics, timeoutMs = 5000, max = 100 })`:

- Caché por `videoId`, 100 entradas, sin caducidad (toda la sesión; `FOREVER = Number.MAX_SAFE_INTEGER`). Peticiones en curso deduplicadas por `videoId` (`pending`).
- `get`: acierto de caché → ese valor; en curso → la misma promesa; si no, `resolveLyrics` y guarda en caché solo si `cacheable`.

URLs (`lookupUrls`, `lyrics-query.js:31-47`), base `https://lrclib.net/api/`, parámetros con `URLSearchParams` omitiendo vacíos/nulos, en este orden y sin duplicados:

1. Si hay duración (> 0 y finita): `get?track_name=<título>&artist_name=<artista>&album_name=<álbum>&duration=<round(duración)>`.
2. Si hay duración y (título limpio ≠ título o artista principal ≠ artista): `get` con título limpio y artista principal (mismo álbum y duración).
3. Siempre: `search?track_name=<título limpio>&artist_name=<artista principal>`.

- Título limpio (`cleanTitle`): quita repetidamente sufijos finales entre `()` o `[]` que sean `official music video`, `music video`, `official lyric video`, `lyric video`, `official video`, `video`, `official audio`, `audio`, `official visualizer`, `visualizer`, `official mv`, `mv`, `feat…`/`ft…` (con o sin punto, hasta el cierre), o `with …`; sin distinguir mayúsculas. Regex literal: `/\s*[(\[]\s*(?:(?:official\s+)?(?:music\s+video|lyric\s+video|video|audio|visualizer|mv)|(?:feat|ft)\b\.?[^)\]]*|with\s[^)\]]*)\s*[)\]]\s*$/i`. Si el resultado queda vacío se usa el título original.
- Artista principal (`primaryArtist`): lo anterior al primer separador `/\s+y\s+|\s+&\s+|\s+and\s+|,\s*|\s+x\s+|\s+feat\.\s+/i`, sin espacios.

Consulta (`queryLrclib`), recorre las URLs en orden; `complete = true` al empezar:

- Excepción de red → `complete = false`; si se abortó, se para; si no, siguiente URL.
- 404 → siguiente URL (no afecta a `complete`). Otro estado ≠ 200 → `complete = false`, siguiente.
- `get` (`url` contiene `/api/get?`): el registro es el cuerpo. `search`: `pickSearchResult(lista, duración)` = primer resultado con `syncedLyrics` cuya duración difiera ≤ 3 s (o cualquiera con `syncedLyrics` si no se sabe la duración); si no, el primero con `plainLyrics`; si no, `null` (→ siguiente URL).
- `trusted` = es `get`, o no se sabe la duración, o `|registro.duration − duración| ≤ 3` (`TOLERANCE = 3`). Si `trusted` y tiene `syncedLyrics`: `addGaps(parseLrc(...))`; si salen líneas → sincronizada y **para**.
- Si aún no hay letra plana y el registro tiene `plainLyrics` → se guarda (partida por `\r?\n`) y sigue.
- Toda la fase LRCLIB tiene tope de **5000 ms**: al vencer se aborta (AbortController) y se sigue como `{ synced: null, plain: null, complete: false }`.

Resolución (`resolveLyrics`):

1. Sincronizada de LRCLIB → `{ value: synced, cacheable: complete }`.
2. Si no, `pageLyrics(videoId)` (excepción → `{ error }`). `cacheable = complete && !page.error`.
3. Página con líneas → `{ kind: 'plain', lines: page.lines, source: page.source }`.
4. Si no, plana de LRCLIB → `{ kind: 'plain', lines, source: 'LRCLIB' }`.
5. Si no → `{ kind: 'none' }`.

LRC (`lrc.js`): marcas `/\[(\d+):(\d{1,2})(?:[.:](\d{1,3}))?\]/g` (fracción = dígitos / 10^n); quita BOM inicial; líneas sin marca (metadatos `[ar:…]`, texto suelto) se descartan; una línea con varias marcas se repite en cada tiempo; `t` redondeado a 3 decimales; ordenado por `t`. `addGaps` (`minGap = 6`, `LINE_DURATION = 4`, texto del hueco `• • •`): las líneas vacías se convierten en hueco `{ t, text: '• • •', gap: true }` en su sitio; si la primera línea (no hueco) empieza en ≥ 6 s, se añade un hueco en `t = 0`; entre dos líneas no-hueco consecutivas separadas ≥ 10 s (6 + 4) se inserta un hueco en `t = línea.t + 4`.

### K.2 Biblioteca (`library.js`)

`LibraryItem = { id: browseId ('VL…' playlist, 'MPRE…' álbum), type: 'playlist'|'album', title, artist?, cover, playlistId? }`. `Song = { videoId, title, artist, duration, cover }` (según la spec/plan; lo producen `page-api.js`).

- `normalize(s)`: `String(s ?? '')`, NFD, quita marcas combinantes U+0300–U+036F, minúsculas ("Más" → "mas").
- `filterLibrary(items, query, limit = 5)`: consulta normalizada y recortada; vacía → `[]`. Primero los títulos que **empiezan** por la consulta y luego los que la **contienen**, manteniendo el orden original dentro de cada grupo; como mucho 5. Solo se busca en el título (no en el artista).
- `pushRecent(recent, id, libraryIds, max = 6)`: si el id no está en la biblioteca, devuelve `recent` sin cambios; si no, lo pone delante sin duplicados y corta a 6. `recordRecent` (`main.js:470-475`) solo guarda si la lista cambió.
- `recentItems(recent, items, limit = 6)`: ids → elementos, saltando los que ya no existen, máx. 6; si no queda ninguno, `items.slice(0, 6)`.
- `idFromUrl(url)`: solo host `music.youtube.com`; `/playlist?list=X` → `'VLX'` (sin `list` → `null`); `/browse/(MPRE|VL)[\w-]+` exacto → ese id; otra cosa → `null`. Los álbumes abiertos como `/playlist?list=OLAK5uy_…` se traducen en `main.js:483` al `id` (`MPREb_…`) del álbum de la biblioteca cuyo `'VL' + playlistId` coincida.

### K.3 Caché genérica (`cache.js`)

`createCache(max, freshMs, now = Date.now)`: Map por orden de inserción. `get(key)` → `undefined` o `{ value, fresh: now − at ≤ freshMs }`. `set(key, value)`: borra y vuelve a insertar (pasa al final) con `at = now`; si se supera `max`, borra la más antigua insertada. Estilo stale-while-revalidate (no hay LRU por lectura: leer no renueva).

---

## L. Checklist de réplica

1. Carpeta de datos: usar `%APPDATA%\Music Desktop` si existe; si no, `%APPDATA%\ByTune`.
2. Instancia única; una segunda instancia muestra la ventana de la primera (solo tras la comprobación de actualizaciones).
3. `settings.json` con sangría de 2 espacios, fusión con defaults y `quickMenu` fusionado en profundidad; arrays inválidos → `[]`; claves desconocidas se conservan.
4. Defaults exactos: bounds 1280×820, maximized false, notifications true, closeToTray true, theme true, covers {}, lang null, library/recent/suggested [], panels {}, lyricsOpen false, quickMenu {CommandOrControl+Shift+Space, focused, float, queue}.
5. Ventana principal: mínimo 480×400, fondo negro, sin barra de título, botones nativos superpuestos de 64 px con símbolos blancos y fondo transparente.
6. La ventana se muestra solo cuando el contenido está listo; maximizada si se guardó así.
7. Bounds se recuerdan solo si no está maximizada/minimizada/pantalla completa; `maximized` se recuerda salvo en pantalla completa.
8. Sesión web persistente (partición `persist:ytmusic`).
9. UA de Chrome sin "Electron" ni "ByTune".
10. Switches GPU raster y zero-copy.
11. Hosts internos: music.youtube.com, accounts.google.*, accounts.youtube.com, consent.youtube.com, *.google.com; el resto, al navegador del sistema; nunca ventanas nuevas.
12. Ignorar siempre el diálogo beforeunload.
13. F5 (con cualquier modificador) recarga.
14. Esc sale de pantalla completa solo si lo está.
15. Alt+←/→ atrás/adelante solo si se puede.
16. Ctrl+=/+ y Ctrl+- cambian el zoom 0,5 niveles; Ctrl+0 lo restablece.
17. Atajos globales Ctrl+Alt+→ (siguiente), Ctrl+Alt+← (anterior), Ctrl+Alt+M (mostrar/ocultar); sin global de play/pausa.
18. Cerrar con `closeToTray` oculta; sin él, sale.
19. "Salir" de la bandeja sale de verdad (aunque haya música).
20. Arranque con `--hidden` no muestra la ventana.
21. Iniciar con Windows: entrada `local.youtube-music-client` en `HKCU\...\Run` con `--hidden`; reparar si apunta a otra ruta.
22. Bandeja: icono de la app; clic alterna (oculta solo si visible y enfocada); doble clic muestra.
23. Tooltip = canción — artista (máx. 127, si no 124 + "…"); "YouTube Music" sin canción.
24. Título de la ventana "<canción — artista> · YouTube Music" o "YouTube Music".
25. Menú de bandeja en el orden de C.4 con 3 separadores, etiqueta de canción deshabilitada y 4 casillas.
26. Bandeja, thumbar y menú se reconstruyen al cambiar canción/estado/idioma/ajuste.
27. Thumbar: anterior / reproducir-pausa / siguiente con tooltips traducidos e iconos blancos de 16 px (32 px @2x).
28. Thumbar: cambio optimista del icono al pulsar reproducir/pausa.
29. Thumbar: reintentos a 0, 0,5, 1,5, 4, 10 y 30 s tras mostrar la ventana, y al enfocarla.
30. Notificación silenciosa con icono al cambiar de canción si suena, está activada y la ventana no tiene el foco.
31. Idioma: settings.lang o idioma del sistema; luego el de la página; regla "empieza por es" → español, si no inglés; se guarda.
32. 61 cadenas es/en exactas (tabla J), con "…" U+2026.
33. Validación de atajos: ≥1 modificador de {Ctrl, Alt, Shift, Win} + tecla válida; reservados → error "reserved"; ocupado → "taken" y se conserva el anterior.
34. Conversión de teclas por código físico (tabla E.4), orden fijo Ctrl, Alt, Shift, Win.
35. Mostrar atajos como "Ctrl + Shift + Space" (Win para Super, + para Plus).
36. Grabación: mientras dura, ningún atajo propio actúa; Esc sin modificadores cancela; solo-modificadores sigue grabando; termina al perder el foco o recargar.
37. Atajo del menú en modo enfocado: abre/cierra con la ventana principal enfocada y cierra con el menú enfocado; sin autorepetición.
38. Atajo en modo global: abre/cierra desde cualquier app; si no se puede registrar al arrancar, funciona como enfocado.
39. `fromOtherApps: front` trae la ventana antes de abrir el menú.
40. Ventana del menú: 588×700, sin marco, transparente, sin sombra de sistema, siempre encima, fuera de la barra de tareas, no redimensionable, creada una vez y reutilizada.
41. Posición: centrada horizontal y verticalmente en el área de trabajo de la pantalla de la app (si se abre sobre ella) o de la del cursor.
42. El menú se oculta al perder el foco (sin devolver foco) y con Esc/atajo/clic fuera (devolviendo el foco a la app si se abrió sobre ella).
43. Al abrir: campo vacío y enfocado, placeholder traducido, pie recogido, aviso oculto, selección en la primera fila.
44. Panel 540 px, a 60 px del borde superior de la ventana y 24 de los lados; fondo #1e1e1e, borde blanco 10 %, radio 16, sombra 0 16 40 negro 50 %, padding 8.
45. Campo: fondo blanco 8 %, borde blanco 8 %, radio 10, padding 11/14, texto 16 px, lupa 18 px al 55 %, rueda de 16 px a la derecha.
46. Rueda visible y girando (0,7 s lineal) mientras se buscan canciones.
47. Separador de 1 px de borde a borde bajo el campo; oculto si no hay filas.
48. Lista de máx. 440 px con scroll sin barra visible.
49. Etiquetas de sección 13 px peso 500, blanco 56 %, márgenes 10/10/6.
50. Filas: padding 6/10, radio 8, 15 px, portada 32 px radio 4 (gris blanco 14 % si falta o falla), título con elipsis, detalles a la derecha 13 px blanco 56 % máx. 55 %.
51. Fila seleccionada blanco 14 %; el ratón selecciona al moverse; sin hover propio.
52. Sin texto: Recientes (≤6 de la biblioteca, o los 6 primeros) + Sugerencias; con texto: Tu biblioteca (≤5) + Canciones.
53. Secciones vacías no se muestran; separador entre secciones.
54. Detalles: canción "Artista · 3:45", álbum "Álbum · Artista", playlist "Playlist".
55. Filtrado de biblioteca sin mayúsculas ni acentos, primero "empieza por" y luego "contiene", en orden original.
56. Canciones: debounce de 200 ms en el proceso principal; caché de 100 consultas, fresca 10 min, devolviendo las viejas al instante y refrescando.
57. Solo cuenta la respuesta de la última consulta (seq).
58. Las canciones previas se mantienen si el texto nuevo amplía o recorta el anterior; si no, desaparecen.
59. Si la búsqueda falla sin caché, la sección Canciones desaparece.
60. ↑/↓ circulares; la selección se conserva al refrescar si la fila sigue.
61. Intro en canción: suena (sustituye la cola) y el menú **sigue abierto**.
62. Intro en playlist/álbum: cierra el menú, muestra la ventana, abre su página sin tocar la reproducción y lo añade a Recientes.
63. Mayús+Intro en canción: a la cola o a continuación según ajuste, con aviso; en playlist/álbum no hace nada.
64. Intro/Mayús+Intro sin autorepetición y sin activaciones solapadas.
65. Aviso: píldora blanca con check y texto negro, o variante oscura roja con "!", 14 px, a 12 px del borde, "· canción" tras el texto, 2,2 s, transición 0,15 s de opacidad y 6 px.
66. Alt+← / Alt+↓ / Alt+→ en el menú: anterior / play-pausa / siguiente, también escribiendo, sin repetición.
67. F1 y el chevron alternan el pie completo; chevron girado 180° cuando está abierto.
68. Pie: "Shift + Enter a la cola|a continuación", "Alt + ← ↓ → anterior / pausa / siguiente"; desplegado añade "↑↓ mover", "Enter reproducir / abrir", "Esc cerrar"; teclas en `kbd` 11 px.
69. Bloque "lo que suena" solo con canción: título 13 px, artista 12 px blanco 70 %, botones 34 px, el central círculo blanco con icono negro play/pausa; cambio optimista.
70. Clic fuera del panel (dentro de la ventana) cierra; los botones no quitan el foco al campo.
71. Animación de apertura prevista 120 ms ease-out (opacidad + 4 px); cierre instantáneo.
72. Biblioteca, sugerencias y portadas se refrescan cada 15 min y 4 s tras cada carga; biblioteca también al abrir el menú si pasó > 1 min.
73. Recientes: máx. 6, solo ids de la biblioteca, guardados solo si cambian; se registran al navegar en la página y al abrir desde el menú; álbumes `VLOLAK…` traducidos a `MPREb_…`.
74. Paneles laterales: left 240–420, right 280–480, redondeados.
75. Letras: LRCLIB (get completo, get limpio, search) con tope de 5 s, tolerancia de 3 s, respaldo de la página y luego plana de LRCLIB; caché de sesión de 100 canciones solo con respuestas completas; UA propio.
76. LRC: huecos "• • •" en líneas vacías, al inicio si empieza ≥ 6 s y entre líneas separadas ≥ 10 s (en t+4).
77. Actualización al arrancar antes de la ventana: tope de 10 s para comprobar; aviso 340×110 negro "Actualizando ByTune a la versión X…"; instalación silenciosa en la misma carpeta y reapertura; fallos → versión actual y log en `updater.log`.
78. Sin actualizaciones en la portable ni sin empaquetar ("Solo en la versión instalada").
79. Botón de actualizaciones con estados Buscar / Buscando… / Ya tienes la última versión / Solo en la versión instalada / No se pudo comprobar.
80. Tema oscuro nativo para menús y diálogos.
81. AppUserModelID `local.youtube-music-client` y datos de anclaje (relaunch "ByTune").
82. Icono: círculo `#DF1C43` con tres barras blancas redondeadas (medidas D.2), `.ico` multi-tamaño.
83. Cambios de interruptores desde la bandeja se reflejan en el menú de ajustes abierto y viceversa.
84. `settings:set` con atajo inválido/ocupado no aplica nada más del mismo cambio.

---

## M. Diferencias entre la spec/plan (`docs/superpowers/`) y el código

Manda el código; se listan para que la réplica no copie la spec por error.

| Tema | Spec / plan | Código actual |
|---|---|---|
| Intro en una canción | El menú se cierra | El menú **sigue abierto** (`keepOpen: true`, `main.js:636-637`) |
| Nota tras Mayús+Intro | Nota apagada en la fila, 1,5 s | Aviso tipo píldora encima del panel, 2,2 s (`TOAST_MS`), con variante de error |
| Posición del menú | Sobre la app: 80 px bajo su borde superior; si no, al 20 % de la altura de la pantalla del cursor | Centrado horizontal y vertical en el área de trabajo de la pantalla (`main.js:535-543`) |
| Reservados | Incluye `CommandOrControl+Alt+Space` (play/pausa global) | Solo 3 reservados; el global de play/pausa ya no existe (`shortcut.js:5-9`, `main.js:757`) |
| Secciones sin texto | Solo Recientes | Recientes + Sugerencias |
| Pie | 4 pistas fijas (↑↓, Enter, Shift+Enter, Esc) | 2 pistas + 3 al desplegar (F1/chevron), incluye Alt+flechas |
| Fila seleccionada | `#2c2c2c` | blanco 14 % sobre `#1e1e1e` |
| Bloque "lo que suena" y controles | No existe | Existe (I.4) |
| Nombre | "Music Desktop" | "ByTune" (migración B.1) |
| `qm.query` | `query(text)` | `query(seq, text)` |

Otras cosas no claras en el código:

- Si la animación `qm-in` llega a verse (I.3).
- Qué hace `Tab` en el menú (no gestionado).
- El orden mostrar→ocultar con `--hidden` puede producir un parpadeo (A.1).
- El número de canciones de búsqueda y de sugerencias lo decide `page-api.js` (el plan dice hasta 5 en búsqueda); main no lo limita.
