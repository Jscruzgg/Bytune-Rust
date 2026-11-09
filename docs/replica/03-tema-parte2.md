# 03 · Tema (theme.css), parte 2 — líneas 948–1888

Fuente: `C:\Users\JuanC\Documents\ByTune\src\theme.css`, desde el comentario que abre en la línea 948
(la línea 950 está a mitad de ese comentario; la primera regla completa es la de la línea 953) hasta
el final del archivo (línea 1888).

Cubre: píldora Me gusta / No me gusta de la barra del reproductor (y la capa de estado del botón
play/pausa), la página Now Playing (`ytmusic-player-page`): fondo con la carátula difuminada,
carátula, pestañas, selector Canción/Vídeo (`ytmusic-av-toggle`), letras, cola en Material 3
(cabecera, filtros, fila de reproducción automática con interruptor, subcabecera, canciones), la
página de álbumes/playlists al estilo Spotify, la disposición de Now Playing con la cola a la
derecha (`@media (min-width: 940px)`) y las reglas de rendimiento.

Convenciones: casi todas las declaraciones llevan `!important` (hay que ganar a los estilos de
YouTube Music). Se indica cuando una regla posterior sobrescribe a una anterior: **el valor que vale
es el último/más específico**, y es el que hay que replicar.

---

## 0. Valores de referencia que usa este tramo (definidos fuera de él)

No se documentan aquí, pero hacen falta para leer los valores concretos.

De `src/chrome.css:6-28` (paleta):

| Variable | Valor |
|---|---|
| `--md-bg` | `#000` |
| `--md-text` | `#fff` |
| `--md-text-2` | `rgba(255,255,255,.7)` |
| `--md-text-3` | `rgba(255,255,255,.56)` |
| `--md-text-off` | `rgba(255,255,255,.38)` |
| `--md-on-fill` | `#000` |
| `--md-glass` | `rgba(12,12,12,.6)` |
| `--md-glass-bar` | `rgba(12,12,12,.72)` |
| `--md-bar` | `#0e0e0e` |
| `--md-raised` | `#1e1e1e` |
| `--md-field` | `rgba(255,255,255,.08)` |
| `--md-state-hover` | `rgba(255,255,255,.08)` |
| `--md-state-on` | `rgba(255,255,255,.14)` |
| `--md-state-press` | `rgba(255,255,255,.2)` |
| `--md-state-ink` | `rgba(0,0,0,.1)` |
| `--md-state-ink-press` | `rgba(0,0,0,.16)` |
| `--md-line` | `rgba(255,255,255,.08)` |
| `--md-line-strong` | `rgba(255,255,255,.24)` |

De la parte 1 de `theme.css` (no es mi tramo, solo el valor):

| Variable | Valor | Origen |
|---|---|---|
| `--md-radius` | `10px` | theme.css:10 |
| `--md-gap` | `8px` | theme.css:11 |
| `--md-font-display` | `"YouTube Sans", Roboto, Arial, sans-serif` | theme.css:15 |
| `--m3-font` | `Roboto, "Segoe UI Variable Text", system-ui, sans-serif` | theme.css:761 |
| `--m3-ease` | `cubic-bezier(.2, 0, 0, 1)` ("emphasized" de M3) | theme.css:765 |

Variables que pone JavaScript (`preload.js`) o YouTube, usadas aquí:

- `--md-art`: URL de la carátula actual (`url(...)`), la pone preload.js (comentario theme.css:1036-1038).
- `--md-right-w`: ancho de la cola elegido por el usuario arrastrando su borde, 280–480 px (preload.js; theme.css:1816-1818).
- `--md-live-side`: ancho de la cola mientras se arrastra su borde (preload.js, no se hereda; theme.css:1832).
- `--md-vh`: alto de ventana efectivo (lyrics.css lo redefine en pantalla completa pendiente, `lyrics.css:447`); por defecto `100vh`.
- `--ytmusic-guide-width`: ancho de la barra lateral (guía) de YT; por defecto `0px` en el cálculo.
- `--ytmusic-nav-bar-height`: alto de la barra superior de YT; por defecto `64px`.
- `--ytmusic-player-bar-height`: alto de la barra del reproductor; por defecto `92px`.
- Atributo `data-md-long` en el título de álbum: lo pone preload.js en títulos de más de 24 caracteres (theme.css:1580).
- Atributo `html[data-md-fullscreen]`: modo pantalla completa (lo pone la app).

---

## 1. Variables definidas en este tramo

| Variable | Valor | Dónde se define | Para qué |
|---|---|---|---|
| `--md-like-w` | `105px` (comentario: `52 + 1 + 52`) | `ytmusic-player-bar` (theme.css:966-968) | Ancho total de la píldora Me gusta/No me gusta: dos botones de 52 px + divisor de 1 px. Se usa para reservar hueco en `#right-controls` y como ancho de la píldora. |
| `anchor-name: --md-right-controls` | (nombre de ancla CSS) | `ytmusic-player-bar #right-controls .right-controls-buttons` (theme.css:963-965) | Ancla CSS (anchor positioning) para colocar la píldora de Me gusta junto al grupo de botones de la derecha (volumen…). |
| `--md-np-reserve` | `360px` | `ytmusic-player-page` (theme.css:1073) | Alto que se reserva en Now Playing para todo lo que no es la carátula, cuando el tema está activo. lyrics.css calcula `--md-np-art: calc(var(--md-vh, 100vh) - var(--md-np-reserve, 430px))` (lyrics.css:10), así que con el tema la carátula mide `100vh − 360px`. |
| `--md-al-pad` | `32px` | `ytmusic-two-column-browse-results-renderer` (theme.css:1437) | Relleno lateral de la página de álbum/playlist. |
| `--md-np-side` | `var(--md-right-w, clamp(340px, 28vw, 440px))` | `ytmusic-player-page` dentro de `@media (min-width: 940px)` (theme.css:1818) | Ancho de la cola a la derecha de Now Playing: el del usuario, o proporcional (28 % del ancho, entre 340 y 440 px). |
| `--md-np-art` (redefinida) | `min(calc(var(--md-vh, 100vh) - 430px), calc(100vw - var(--ytmusic-guide-width, 0px) - var(--md-np-side) - 96px))` | idem (theme.css:1821) | Lado de la carátula en ventanas anchas. Comentario theme.css:1819-1820: alto útil = ventana − barra superior (64) − barra inferior (92) − márgenes (64) − selector Canción/Vídeo con su hueco (~110) − título y artista (~90) = 430. La segunda rama la limita por el ancho disponible (ventana − guía − cola − 96 px). |
| `--paper-tabs-selection-bar-color` | `var(--md-text)` = `#fff` | `ytmusic-player-page tp-yt-paper-tabs` (theme.css:1102) | Color del indicador de pestaña de Polymer. |
| `--ytmusic-play-button-size` | `56px` | botón play del álbum (theme.css:1668) | Variables internas de YT para su botón play. |
| `--ytmusic-play-button-icon-size` | `28px` | theme.css:1669 | |
| `--ytmusic-responsive-header-primary-button-width` | `56px` | theme.css:1670 | |
| `--ytmusic-play-button-background-color` | `var(--md-text)` = `#fff` | theme.css:1671 | |
| `--ytmusic-play-button-icon-color` | `var(--md-on-fill)` = `#000` | theme.css:1672 | |

---

## 2. Zonas de la interfaz (en el orden del archivo)

### 2.1 Píldora Me gusta / No me gusta en la barra del reproductor (theme.css:948-1024)

**Qué es:** `ytmusic-player-bar` es la barra del reproductor inferior. Dentro, `ytmusic-like-button-renderer`
contiene dos botones (`yt-button-shape.like` y `yt-button-shape.dislike`, cada uno con un `<button>`).
YT los pone en `.middle-controls` (la zona central con carátula/título). `#right-controls` es el bloque
derecho de la barra (volumen, repetir, aleatorio, expandir); dentro, `.right-controls-buttons` es el
grupo de botones en sí.

**Intención (comentario theme.css:948-952):** como en la app móvil: una sola píldora tonal con una línea
divisoria fina en medio. Al marcar Me gusta no cambia el fondo: YT cambia el icono a su versión rellena.
La píldora **sigue en su sitio del DOM** (si se mueve, YT confunde sus clics con clics en la barra, que
abren Now Playing) y se **dibuja** junto al volumen con anclaje CSS. El bloque derecho reserva el hueco
de la píldora con padding, para que el flujo evite solapes.

Resultado visual a replicar: **a la izquierda del grupo de botones de la derecha (volumen…), separada
8 px, una píldora de 105 × 40 px, radio 20 px, fondo `rgba(255,255,255,.14)`, con dos mitades de
52 × 40 px (pulgar arriba | pulgar abajo) y una línea vertical de 1 px entre ellas.**

Detalle:

- `ytmusic-player-bar #right-controls` (theme.css:953-958):
  - `padding-left: calc(var(--md-like-w) + 8px)` = **113 px** (hueco reservado para la píldora + 8 px).
  - `pointer-events: none`: su zona vacía (incluido el hueco reservado) no recoge clics, para que lleguen
    a la píldora, que está debajo en el orden de hit-testing (comentario theme.css:955-956).
- `ytmusic-player-bar #right-controls > *` → `pointer-events: auto` (theme.css:959-961): sus botones sí
  reciben clics.
- `.right-controls-buttons` es el ancla `--md-right-controls` (theme.css:962-965). Comentario: el ancla es
  el grupo de botones, no todo el bloque, que ocupa la columna entera.
- `ytmusic-player-bar .middle-controls, .middle-controls-buttons` → `position: static` (theme.css:969-973):
  para que la referencia de posición absoluta de la píldora sea la propia barra (que contiene el ancla),
  no sus contenedores.
- `ytmusic-like-button-renderer` (theme.css:974-995):
  - `position: absolute`, `position-anchor: --md-right-controls`.
  - `left: auto`, `right: calc(anchor(left) + 8px)` → su borde derecho queda a **8 px** del borde
    izquierdo del grupo de botones derecho.
  - `top: anchor(center)` + `translate: 0 -50%` → centrada verticalmente con ese grupo.
  - `z-index: 2` (por encima del bloque derecho, que va después en el DOM y la tapaba; si no, el clic
    caía en la barra y abría Now Playing; theme.css:982-983).
  - `width: 105px` (`--md-like-w`), `height: 40px`, `display: inline-flex`, `align-items: center`,
    `flex: none`, `gap: 0`, `margin: 0`.
  - `border-radius: 20px`, `background: var(--md-state-on)` = `rgba(255,255,255,.14)`, `overflow: hidden`.
- `.like-button-wrapper` y `yt-button-shape` → `margin: 0` (theme.css:996-999).
- `yt-button-shape.dislike` → `position: relative` (theme.css:1000-1002) y su `::before` es el **divisor**
  (theme.css:1003-1012): `content: ""`, `position: absolute`, `left: 0`, `top: 10px`, `bottom: 10px`
  (es decir, 20 px de alto centrado en los 40 px), `width: 1px`, `background: var(--md-line-strong)` =
  `rgba(255,255,255,.24)`, `pointer-events: none`.
- Cada `<button>` (theme.css:1013-1020): `width: 52px`, `height: 40px`, `border-radius: 0` (el redondeo lo
  da el recorte del contenedor), `background-color: transparent`, `color: var(--md-text)` = `#fff`.
  - Me gusta: `padding: 8px 12px 8px 16px` (theme.css:1021) → icono desplazado hacia la derecha (más
    relleno en el lado exterior izquierdo).
  - No me gusta: `padding: 8px 16px 8px 12px` (theme.css:1022) → simétrico (más relleno en el exterior
    derecho).
  - Icono: con `52 − 16 − 12 = 24 px` de ancho de contenido y `40 − 16 = 24 px` de alto → icono de
    24 × 24 (deducido; el tamaño del icono no se fija aquí).

### 2.2 Botón play/pausa de la barra: capas de estado e icono (theme.css:1025-1033)

**Qué es:** `#play-pause-button.ytmusic-player-bar #button` es el círculo blanco de 44 px definido justo
antes de mi tramo (theme.css:931-946: fondo `#fff`, icono `#000`, radio 50 %, padding 10 px,
`transition: box-shadow .15s linear`).

- Comentario theme.css:1025: "capa de estado sobre el relleno: on-primary al 8 % / 12 %" (los valores reales
  de la variable son 10 % y 16 % de negro).
- Hover (theme.css:1026): `box-shadow: inset 0 0 0 44px var(--md-state-ink)` = sombra interior que cubre
  todo el círculo con `rgba(0,0,0,.1)` → el blanco se oscurece un 10 %.
- Active (theme.css:1027): `box-shadow: inset 0 0 0 44px var(--md-state-ink-press)` = `rgba(0,0,0,.16)`.
- `yt-icon` dentro (theme.css:1028-1033): `24 × 24 px`, `color` y `fill`: `var(--md-on-fill)` = `#000`.

### 2.3 Now Playing: contenedor y fondo con la carátula difuminada (theme.css:1035-1068)

**Qué es:** `ytmusic-player-page` es la página a pantalla completa del reproductor ("Now Playing") que
YT abre al pulsar la barra. Hijos relevantes: `> .content` (contenedor de los dos paneles),
`#main-panel` (carátula/vídeo `#player`, `#song-image`, `#thumbnail`), `#side-panel` (pestañas
"A continuación / Letra / Relacionado" y su contenido: la cola, la letra…).

Título de sección en el archivo: "Now Playing (Apple Music)".

- `ytmusic-player-page` (theme.css:1039-1046):
  - `background: var(--md-bg)` = `#000`.
  - `overflow: hidden`.
  - `will-change: auto` — comentario: YT la deja siempre en su propia capa (`will-change: transform`);
    con el panel de la cola en una posición fraccionaria, el texto se pintaba sin antialias de subpíxel
    y se veía borroso. Chromium sigue creando la capa por su cuenta mientras dura la animación de
    abrir/cerrar. (En la réplica: el texto de Now Playing debe pintarse nítido, alineado a píxel.)
- **Capa 1 — `::before`, degradado con el color de la carátula** (theme.css:1047-1060). Intención: "bien
  presente (Apple Music)". Truco de rendimiento (comentario theme.css:1036-1038): el blur se aplica a una
  capa de 64 × 64 px y luego se escala × 40 en la GPU; coste mínimo y solo se repinta al cambiar de canción.
  - `content: ""`, `position: absolute`, `left: 0`, `top: 0`, `width: 64px`, `height: 64px`.
  - `background: var(--md-art, none) center / cover no-repeat` (la carátula, recortada a cuadrado y
    centrada; sin carátula no hay imagen → se ve el negro).
  - `filter: blur(4px) saturate(1.6) brightness(.55)`.
  - `transform-origin: 0 0`, `transform: scale(40)` → la capa cubre 2560 × 2560 px desde la esquina
    superior izquierda (el desenfoque efectivo equivale a 4 × 40 = 160 px a escala real; deducido).
  - `pointer-events: none`.
  - Efecto: un campo de color muy difuso, saturado ×1,6 y oscurecido al 55 % de brillo, que llena la página.
- **Capa 2 — `::after`, oscurecido hacia abajo** (theme.css:1061-1068), para que el texto y la barra se lean
  con carátulas claras:
  - `content: ""`, `position: absolute`, `inset: 0`.
  - `background: linear-gradient(180deg, rgba(0,0,0,.15), rgba(0,0,0,.55))` (de arriba 15 % negro a
    abajo 55 % negro).
  - `pointer-events: none`.
- Orden de pila: fondo `#000` → `::before` (carátula) → `::after` (degradado) → `> .content` encima con
  `position: relative; z-index: 1` (theme.css:1076-1079).

### 2.4 Now Playing: reserva de alto, selector desactivado, carátula (theme.css:1070-1088)

- Comentario theme.css:1070-1072: el tamaño de la carátula (`--md-np-art`) y la fila de título y artista bajo
  ella están en `lyrics.css`, que se inyecta siempre (con o sin tema). Aquí solo el alto reservado con el
  tema y `--md-np-art` afinado en ventanas anchas (ver 5.1).
- `ytmusic-player-page { --md-np-reserve: 360px; }` (theme.css:1073).
- `ytmusic-player-page ytmusic-av-toggle[toggle-disabled]` → `visibility: hidden` (theme.css:1074-1075):
  si la canción no tiene vídeo, YT deja el selector Canción/Vídeo desactivado; se **oculta pero conserva
  su hueco**, para que la carátula no se mueva.
- Radio **12 px** en: `#song-image img`, `#thumbnail img`, `#song-image`, `#player.ytmusic-player-page`
  (theme.css:1080-1085). `#song-image` es la carátula de la canción; `#thumbnail` la miniatura;
  `#player` el reproductor de vídeo.
- `#song-image` → `box-shadow: 0 24px 60px rgba(0,0,0,.55)` (theme.css:1086-1088): sombra grande y
  difusa debajo de la carátula.

### 2.5 Now Playing: panel lateral (primera definición) y overflow (theme.css:1089-1098)

- `ytmusic-player-page #side-panel` (theme.css:1089-1094), comentario: "panel de cola/letra como tarjeta
  translúcida (color sólido con alfa, sin backdrop-filter)":
  - `border-radius: var(--md-radius)` = 10 px → **sobrescrito** a 16 px en theme.css:1202, y a 0 en
    ventanas ≥ 940 px (theme.css:1839).
  - `padding: 0 8px`, `box-sizing: border-box`.
- `ytmusic-player-page:not([is-mweb-modernization-enabled]) > .content` → `overflow-x: hidden`
  (theme.css:1098). Comentario theme.css:1095-1097: en ventanas pequeñas YT apila carátula y cola; que
  nada genere scroll horizontal. Pero no en el diseño móvil (≤ 615 px, que YT marca con el atributo
  `is-mweb-modernization-enabled`): ahí `.content` mide solo la cabecera y el recorte escondía los
  controles, que YT coloca debajo con posición absoluta.

### 2.6 Now Playing: pestañas (theme.css:1099-1130)

**Qué es:** `tp-yt-paper-tabs` (Polymer) es la fila de pestañas del panel lateral ("A CONTINUACIÓN",
"LETRA", "RELACIONADO"); cada una es `tp-yt-paper-tab`; `#selectionBar` es la barra indicadora bajo la
pestaña seleccionada. Comentario theme.css:1099-1100: "primary tabs" de Material 3; etiqueta
title-small, indicador de 3 px con esquinas superiores redondeadas, divisor de 1 px y capa de estado al
pasar el ratón.

- `tp-yt-paper-tabs` (theme.css:1101-1104): `--paper-tabs-selection-bar-color: #fff`;
  `border-bottom: 1px solid var(--md-line)` = `rgba(255,255,255,.08)` (divisor bajo toda la fila).
- `tp-yt-paper-tab` (theme.css:1105-1110): `height: 48px`, `color: var(--md-text-2)` =
  `rgba(255,255,255,.7)`, `text-transform: none` (**no** en mayúsculas, a diferencia de YT),
  `transition: background-color .15s linear`.
- Tipografía de la pestaña y todo su contenido (theme.css:1111-1118): `font-family: var(--m3-font)`
  (Roboto…), `font-size: 14px`, `font-weight: 500`, `line-height: 20px`, `letter-spacing: .1px`.
- Estados: ver 3.
- `#selectionBar` (theme.css:1125-1130): `height: 3px`, `border: none`, `border-radius: 3px 3px 0 0`,
  `background: #fff`. (El ancho y su desplazamiento los gestiona Polymer: ocupa el ancho de la pestaña
  seleccionada; no se fija aquí.)

### 2.7 Elementos de la cola, primera definición genérica (theme.css:1131-1141)

**Qué es:** `ytmusic-player-queue-item` es cada fila de canción de la cola. Estas reglas no están
limitadas a `ytmusic-player-page` y son más débiles que las de 2.12; quedan solapadas en parte.

- `border-radius: 8px` (theme.css:1132) → **dentro de Now Playing queda 12 px** (theme.css:1384, más
  específica).
- `border-bottom: none` (sin separadores entre filas).
- `:hover` → `background: var(--md-state-hover)` = `rgba(255,255,255,.08)` (theme.css:1135).
- `[selected]`, `[play-button-state="playing"]`, `[play-button-state="paused"]` → `background:
  var(--md-state-on)` = `rgba(255,255,255,.14)` (theme.css:1136-1140). `play-button-state` es el atributo
  que YT pone a la fila que suena (reproduciendo o en pausa).
- `ytmusic-player-queue-item img` → `border-radius: 4px` (theme.css:1141) → **sobrescrito a 8 px** para
  la miniatura en `.left-items` (theme.css:1393).

### 2.8 Selector Canción/Vídeo — av-toggle (theme.css:1142-1183)

**Qué es:** `ytmusic-av-toggle` es el conmutador "Canción | Vídeo" que YT pone encima de la carátula
en Now Playing. Dentro, `.av-toggle` es el contenedor con dos `<button>` (Canción y Vídeo);
el activo tiene `aria-pressed="true"`.

Intención (comentario theme.css:1143-1145): como en la app móvil, **solo iconos** (auriculares / vídeo)
dentro de una píldora oscura; el elegido va en una píldora interior algo más clara. El texto se oculta
con `font-size: 0`, así los lectores de pantalla lo siguen leyendo.

Nota: el CSS **no define los iconos** (auriculares / cámara de vídeo); el comentario dice que el
selector muestra solo iconos, pero de dónde salen (YT u otro script) no está en este tramo. No está
claro sin mirar otro archivo.

Resultado visual: píldora de **44 px de alto**, radio 22 px, fondo `rgba(255,255,255,.08)`, relleno
4 px, con dos botones de **56 × 36 px**, radio 18 px, separados 4 px → ancho total = 4 + 56 + 4 + 56 + 4 =
**124 px** (deducido). El activo tiene fondo `rgba(255,255,255,.2)` e icono blanco; el inactivo, fondo
transparente e icono `rgba(255,255,255,.7)`.

- `ytmusic-av-toggle` → `background: transparent` (theme.css:1146).
- `.av-toggle` (theme.css:1147-1156): `display: inline-flex`, `gap: 4px`, `height: 44px`, `padding: 4px`,
  `border: none`, `border-radius: 22px`, `background: var(--md-field)` = `rgba(255,255,255,.08)`,
  `box-sizing: border-box`.
- `.av-toggle::before` y `::after` → `display: none` (theme.css:1157-1158): se eliminan los adornos de YT
  (probablemente su indicador deslizante; no se ve en este tramo qué dibujaban).
- `.av-toggle button` (theme.css:1159-1178):
  - `-webkit-appearance: none; appearance: none` — comentario: sin esto Chromium pinta el botón nativo
    claro de Windows (solo cuenta los fondos de la propia página, no los de esta hoja de usuario).
  - `display: inline-flex`, `align-items: center`, `justify-content: center` (icono centrado).
  - `min-width: 0`, `width: 56px`, `height: 36px`, `margin: 0`, `padding: 0`, `border: none`.
  - `border-radius: 18px`, `background: transparent`, `color: var(--md-text-2)` = `rgba(255,255,255,.7)`.
  - `font-size: 0` (texto "Canción"/"Vídeo" invisible pero presente para accesibilidad).
  - `transition: background-color .15s linear, color .15s linear`.
- Estados: ver 3.

### 2.9 Letras grandes (theme.css:1184-1190)

**Qué es:** `ytmusic-description-shelf-renderer .description` es el texto de la letra (pestaña "Letra")
en su forma estática de YT. Comentario: "Letras grandes estilo Apple Music". (La letra sincronizada
está en lyrics.css, no aquí.)

- `font-size: 22px`, `line-height: 1.45` (= 31,9 px), `font-weight: 700`, `color: #fff`.

### 2.10 Cola de Now Playing en Material 3: panel y cabecera (theme.css:1192-1258)

Comentario theme.css:1192-1195: panel = surface container con esquinas de 16 px; cabecera con botón
contorneado; fila de reproducción automática con interruptor M3; canciones como elementos de lista de dos
líneas (64 px) y subcabecera con divisor. Todo en Roboto con la escala tipográfica de M3.

- `ytmusic-player-page #side-panel` y todos sus descendientes → `font-family: var(--m3-font)`
  (`Roboto, "Segoe UI Variable Text", system-ui, sans-serif`) (theme.css:1196-1199).
- `ytmusic-player-page #side-panel` (theme.css:1200-1204): `background: var(--md-glass)` =
  `rgba(12,12,12,.6)`, `border-radius: 16px`, `padding: 0 8px`. **En ventanas ≥ 940 px esto se
  sustituye** (fondo transparente, radio 0, tarjeta dibujada con `::before`; ver 5.1).

**Cabecera de la cola** — `ytmusic-queue-header-renderer` es la cabecera "Reproduciendo de / <nombre de
la cola>" con el botón "Guardar" (`#buttons`, un `ytmusic-chip-cloud-chip-renderer`). `.container-name`
contiene dos líneas: la primera es "Reproduciendo de" y la última el nombre.

Hay dos bloques; el segundo ("Cabecera de la cola compacta: menos relleno, 'Guardar' y filtros a 32 px",
theme.css:1240) sobrescribe al primero. **Valores finales:**

| Elemento | Primer bloque (sobrescrito) | **Valor final** |
|---|---|---|
| `ytmusic-queue-header-renderer` padding | `12px 8px 4px` (theme.css:1207) | **`6px 4px 0`** (theme.css:1242) |
| idem `min-height` | — | **`0`** (theme.css:1243) |
| idem `align-items` | **`center`** (theme.css:1208, se mantiene) | `center` |
| Línea 1 "Reproduciendo de" (`.container-name > :first-child`) | 12px, peso 500, `letter-spacing .5px`, color `--md-text-2` (theme.css:1210-1215) | **11px / line-height 14px**, peso **500**, `letter-spacing .5px`, color **`rgba(255,255,255,.7)`** (theme.css:1245-1248 + lo no sobrescrito de 1210-1215) |
| Línea 2 nombre (`.container-name > :last-child`) | 16px, peso 500, lh 24px, `letter-spacing .15px`, color `#fff` (theme.css:1216-1222) | **14px / line-height 20px**, peso **500**, `letter-spacing .15px`, color **`#fff`** (theme.css:1249-1252) |
| "Guardar" (`#buttons ytmusic-chip-cloud-chip-renderer a`) alto | 40px (theme.css:1229) | **32px** (theme.css:1254) |
| idem padding | `0 16px 0 12px` | **`0 12px 0 10px`** |
| idem gap icono-texto | 8px | **6px** |
| idem font-size | — | **13px** (theme.css:1257) |
| idem color | **`#fff`** (theme.css:1232, se mantiene) | `#fff` |
| idem radio (`.gradient-box` y `a`) | **20px** (theme.css:1224-1227, se mantiene) | 20px (con 32 px de alto da una píldora completa) |
| Icono de "Guardar" (`#buttons yt-icon`) | **18 × 18 px, margin 0** (theme.css:1234-1238) | 18 × 18 px |

Comentario del primer bloque (theme.css:1223): "'Guardar': píldora tonal de 40 px con icono de 18 px, como
las de la app móvil". El fondo de la píldora "Guardar" es el de 2.11 (`rgba(255,255,255,.14)`) si no está
seleccionada.

Nota de especificidad: la regla de la cabecera (theme.css:1253) tiene más especificidad que la de los
filtros (theme.css:1270, `padding: 0 12px`), así que "Guardar" queda con `0 12px 0 10px`.

### 2.11 Filtros de la cola (chips) (theme.css:1259-1294)

**Qué es:** `ytmusic-chip-cloud-renderer` es la fila de "chips" de filtro de la cola (Todo, Música
conocida, Descubrir…); `#chips` su contenedor; cada chip es `ytmusic-chip-cloud-chip-renderer` con un `<a>`.
`ytmusic-player-queue > #steering-chips` es el contenedor de esos filtros dentro de la cola.

- `#side-panel ytmusic-chip-cloud-renderer` (theme.css:1259-1263): `padding: 4px 0 6px`, `margin: 0`,
  `min-height: 0`.
- `#chips` → `gap: 6px` (theme.css:1264-1266).
- Chip → `margin: 0` (theme.css:1267-1269).
- Chip `a` (theme.css:1270-1273): `height: 32px`, `padding: 0 12px`.
- Chip `a` y descendientes → `font-size: 13px` (theme.css:1274-1277). Familia: la de `#side-panel`
  (Roboto…). Peso y radio: no se fijan aquí para los filtros (quedan los de YT / reglas de la parte 1).
- `ytmusic-player-queue > #steering-chips` (theme.css:1281-1286): `position: sticky`, `top: 0`,
  `z-index: 2`, `background: transparent`. Comentario theme.css:1278-1280: los filtros quedan fijos arriba
  al desplazar la cola, así van con la canción actual (preload.js la deja justo debajo). Sin fondo propio:
  preload.js recorta la lista en el borde inferior de la barra, así nada se ve detrás y la barra es la
  misma tarjeta. (En la réplica: la lista se recorta bajo la fila de filtros, que no tiene fondo.)
- Color de fondo (comentario theme.css:1287-1288: "el fondo aclarado, como los botones de la barra lateral;
  la seleccionada sigue blanca"):
  - No seleccionada (`:not([is-selected]) a`) → `background: var(--md-state-on)` = `rgba(255,255,255,.14)`
    (theme.css:1289-1291). Aplica también a "Guardar".
  - Seleccionada: no se toca aquí; "sigue blanca" (fondo blanco de YT; el color del texto no se define
    en este tramo, presumiblemente oscuro, no confirmado).

### 2.12 Fila de reproducción automática con interruptor M3 (theme.css:1296-1363)

**Qué es:** dentro del panel hay dos elementos con clase `.autoplay` (comentario theme.css:1297-1298): la
fila del interruptor ("Reproducción automática" + descripción + interruptor), que cuelga de un `div`, y la
subcabecera de la cola, que cuelga de `ytmusic-player-queue`. Se distinguen por el padre (sin `:has()`, más
caro). `#automix` es el interruptor (`tp-yt-paper-toggle-button`): `.toggle-container` su caja,
`#toggleBar` la pista, `#toggleButton` el asa, `.toggle-label` su etiqueta.

- Fila `#side-panel div > .autoplay` (theme.css:1299-1303): `padding: 8px`, `margin: 4px 0`,
  `align-items: center`.
- Título (`> div:first-child > :first-child`) (theme.css:1304-1310): `16px`, peso `400`, `line-height 24px`,
  `letter-spacing .5px`, color `#fff` (body-large M3).
- Descripción (`> div:first-child > :last-child:not(:first-child)`) (theme.css:1311-1316): `12px`,
  `line-height 16px`, `letter-spacing .4px`, color `rgba(255,255,255,.7)` (body-small).
- **Interruptor** (comentario theme.css:1317-1318: pista de 52 × 32; apagado = contorno con asa de 16 px;
  encendido = relleno primary con asa de 24 px. Solo se animan transform y colores):
  - `#automix` y `.toggle-container` (theme.css:1319-1329): `52 × 32 px`, `padding: 0` (y `margin: 0` en
    `#automix`).
  - `#toggleBar` (pista) apagado (theme.css:1330-1341): `top: 0`, `left: 0`, `52 × 32 px`,
    `box-sizing: border-box`, `border-radius: 16px`, `opacity: 1`, `background: var(--md-state-hover)` =
    `rgba(255,255,255,.08)`, `border: 2px solid var(--md-text-3)` = `rgba(255,255,255,.56)`,
    `transition: background-color .2s linear, border-color .2s linear`.
  - `#toggleButton` (asa) apagado (theme.css:1342-1352): `top: 0`, `left: 0`, `16 × 16 px`,
    `border-radius: 50%`, `box-shadow: none`, `background: rgba(255,255,255,.56)`,
    `transform: translate(8px, 8px)` (centrada verticalmente: (32 − 16)/2 = 8; a 8 px del borde izquierdo),
    `transition: transform .2s var(--m3-ease), width .2s var(--m3-ease), height .2s var(--m3-ease),
    background-color .2s linear` con `--m3-ease` = `cubic-bezier(.2, 0, 0, 1)`.
  - Encendido `#automix[checked] #toggleBar` (theme.css:1353-1356): `background: #fff`,
    `border-color: #fff`.
  - Encendido `#automix[checked] #toggleButton` (theme.css:1357-1362): `24 × 24 px`, `background: #000`
    (`--md-on-fill`), `transform: translate(24px, 4px)` (centrada: (32 − 24)/2 = 4; derecha: 52 − 24 − 4 = 24).
  - `.toggle-label` → `display: none` (theme.css:1363).

### 2.13 Subcabecera "Reproducción automática activada" (theme.css:1365-1378)

- `ytmusic-player-queue > .autoplay` (theme.css:1366-1370): `margin: 8px 0 4px`,
  `padding: 16px 8px 8px`, `border-top: 1px solid rgba(255,255,255,.08)` (divisor encima).
- `.title` (theme.css:1371-1378): `opacity: 1` (comentario: YT lo deja al 50 %, 3,2:1 de contraste sobre
  carátulas de color), `14px`, peso `500`, `line-height 20px`, `letter-spacing .1px`, color
  `var(--md-text-3)` = `rgba(255,255,255,.56)` (title-small).

### 2.14 Canciones de la cola (theme.css:1380-1428)

Comentario theme.css:1380: elemento de lista de dos líneas (64 px), miniatura de 40 px con esquinas de 8 px.

- `ytmusic-player-page ytmusic-player-queue-item` (theme.css:1381-1386): `min-height: 64px`,
  `padding: 0 12px 0 8px`, `border-radius: 12px`, `transition: background-color .15s linear`.
- Miniatura: `.left-items`, su `yt-img-shadow`, `img` y `ytmusic-item-thumbnail-overlay-renderer` (la capa de
  play/ecualizador sobre la miniatura) → `40 × 40 px`, `border-radius: 8px` (theme.css:1387-1394).
- `.left-items` → `margin-right: 16px` (theme.css:1395).
- `.song-title` (theme.css:1396-1402): `16px`, peso `400`, `line-height 24px`, `letter-spacing .5px`,
  color `#fff`.
- `.byline-wrapper` y descendientes (artista • álbum) (theme.css:1403-1409): `14px`, `line-height 20px`,
  `letter-spacing .25px`, color `rgba(255,255,255,.7)`.
- `.duration` (theme.css:1410-1416): `12px`, peso `500`, `letter-spacing .5px`, color
  `rgba(255,255,255,.7)`, `font-variant-numeric: tabular-nums` (cifras de ancho fijo).
- Estados: ver 3.
- `ytmusic-player-queue-item { contain-intrinsic-size: auto 64px }` (theme.css:1428): tamaño estimado para
  `content-visibility` (ver 2.19).

### 2.15 Álbumes y playlists al estilo Spotify: contenedor (theme.css:1430-1464)

**Qué es:** `ytmusic-two-column-browse-results-renderer` es la página de álbum/playlist de YT, con dos
columnas: `#primary` (cabecera estrecha a la izquierda) y `#secondary` (lista a la derecha, con
`ytmusic-section-list-renderer` y scroll propio).

Comentario theme.css:1430-1434: aquí se apilan: cabecera ancha (carátula a la izquierda; tipo, título enorme y
datos a la derecha), fila de acciones con el play grande primero y la lista a todo el ancho con una fila de
encabezado "# Título ⏱". El fondo sigue siendo el degradado de la carátula de YT.

- Contenedor (theme.css:1435-1444), comentario: ancho máximo de 1400 px centrado (en ventanas grandes no se
  estira de borde a borde): `--md-al-pad: 32px`, `display: block`, `box-sizing: border-box`,
  `width: auto`, `max-width: 1400px`, `margin: 0 auto`, `padding: 0 32px 32px`.
- `> #primary`, `> #secondary` (theme.css:1445-1456): `position: static`, `width: auto`, `height: auto`,
  `max-width: none`, `max-height: none`, `margin: 0`, `padding: 0`. Comentario: YT deja la columna de
  cabecera con `height: 0` (flota "sticky" junto a la lista); apilada necesita su alto real o la lista se
  le dibuja encima.
- `> #secondary > ytmusic-section-list-renderer` (theme.css:1457-1464): `height: auto`, `max-height: none`,
  `overflow: visible`, `margin: 0`, `padding: 0` (la lista deja de tener scroll propio; desplaza la página).

### 2.16 Cabecera de álbum/playlist (theme.css:1466-1647)

**Qué es:** `ytmusic-responsive-header-renderer` es la cabecera (carátula, tipo/año, título, artista,
estadísticas, descripción, botones). En playlists propias va envuelta en
`ytmusic-editable-playlist-detail-header-renderer`.

- **Playlists propias** (theme.css:1466-1477): YT envuelve la cabecera en un flex estrecho y centrado (421 px)
  que aplastaba el título; se estira a todo el ancho igual que en álbumes:
  `ytmusic-editable-playlist-detail-header-renderer` → `display: block`, `width: auto`, `max-width: none`,
  `margin: 0`, `padding: 0`; su `> ytmusic-responsive-header-renderer` → `width: 100%`.
- **Dueño de la playlist** `.facepile-container` (avatar + nombre) (theme.css:1478-1484): va donde en álbumes
  va el artista: `grid-area: by`, `justify-content: flex-start`, `align-items: center`, `margin: 0`.
- `.facepile-container`, `.strapline`, `.second-subtitle-container` → `align-self: center`
  (theme.css:1485-1491). Comentario: tienen alturas distintas (32 px con avatar y 17 px) y alineados por
  abajo sus textos quedaban desfasados 7 px.
- **Lápiz "editar portada"** `.thumbnail-edit-button-wrapper` (playlists propias) (theme.css:1492-1501): YT lo
  ancla a la esquina inferior derecha de la carátula con un contenedor de 0 × 0. Va en la celda de la
  carátula, en su esquina: `grid-area: cover`, `align-self: end`, `justify-self: end`,
  `margin: 0 24px 0 0` (la columna incluye los 24 px de separación), `z-index: 1`.

**Rejilla de la cabecera** (theme.css:1503-1524): "[carátula] [tipo · año / título / artista • canciones,
duración]".

```
grid-template-columns: calc(clamp(160px, 18vw, 232px) + 24px) auto minmax(0, 1fr);
grid-template-areas:
  "cover type  type"
  "cover title title"
  "cover by    stats"
  "cover desc  desc"
  "act   act   act";
grid-template-rows: 1fr auto auto auto auto;
column-gap: 0;
align-items: end;
justify-items: start;
width: auto; max-width: none; margin: 0;
padding: 40px 0 0;
text-align: left;
```

- Primera columna = lado de la carátula + 24 px de separación (comentario theme.css:1506).
- `column-gap: 0`: sin hueco entre columnas, porque separaría "artista" de "• canciones"; la carátula
  lleva su margen (comentario theme.css:1515).
- La fila 1 es `1fr` y todo se alinea abajo (`end`): el texto queda pegado a la parte inferior de la
  carátula, como Spotify.

**Carátula** `> .thumbnail` (theme.css:1525-1533): `grid-area: cover`, `width` y `height:
clamp(160px, 18vw, 232px)` (cuadrada, 18 % del ancho de ventana entre 160 y 232 px), `margin: 0 24px 0 0`,
`box-shadow: 0 8px 24px rgba(0,0,0,.5)`, `border-radius: 8px`, `overflow: hidden`. Su `yt-img-shadow` e `img`
(theme.css:1534-1540): `100% × 100%`, `object-fit: cover`, `border-radius: 0`.

**Tipo y año** ("EP • 2026") `> .subtitle-wrapper` (theme.css:1541-1552): `grid-area: type`,
`align-self: end`, `justify-content: flex-start`, `margin: 0 0 4px`; todo su texto `14px`, peso `500`,
color `#fff`.

**Título enorme** `> h1` (theme.css:1553-1558): `grid-area: title`, `margin: 0 0 16px`, `max-width: 100%`.
`h1 .title` (theme.css:1559-1579):
- `font-family: var(--md-font-display)` = `"YouTube Sans", Roboto, Arial, sans-serif`.
- `font-size: clamp(32px, 5.5vw, 88px)`, `font-weight: 800`.
- `line-height: 1.1` — comentario: líneas juntas; para no cortar los trazos descendentes (g, p, y) basta
  relleno abajo (el recorte del line-clamp solo afecta a la última línea); subir el interlineado
  separaba demasiado las dos líneas de títulos largos.
- `letter-spacing: -0.03em`, `text-align: left`, color `#fff`.
- Máximo **2 líneas** con "…": `display: -webkit-box`, `-webkit-line-clamp: 2`,
  `-webkit-box-orient: vertical`, `overflow: hidden`.
- `padding: .1em .12em .3em 0` y `margin: -.1em -.12em -.2em 0` — comentario: relleno también a los lados,
  porque con el interletrado negativo la última letra ("?", "y") sobresale de su caja y el overflow la
  cortaba. Efecto neto: la caja de recorte es mayor que el texto sin desplazar el layout (salvo +0,1em
  abajo: `.3em − .2em`).
- Títulos de más de 24 caracteres (`.title[data-md-long]`, marcados por preload.js) (theme.css:1580-1583):
  `font-size: clamp(28px, 3.4vw, 56px)` ("más pequeños, como Spotify").

**Artista** `> .strapline` (theme.css:1584-1589): `grid-area: by`, `justify-content: flex-start`, `margin: 0`.
- Avatar `.strapline-thumbnail`, su `yt-img-shadow` e `img` (theme.css:1590-1602): círculo de **24 px**
  (`width/height/min-height: 24px`, `margin: 0`, `border-radius: 50%`, `object-fit: cover`,
  `overflow: hidden`). Comentario: YT dibujaba la imagen a 24 × 16 (estirada); se recorta en vez de
  deformarla.
- `.strapline-thumbnail` → `margin-right: 8px`, `flex: none` (theme.css:1603-1606).
- `.strapline-text` → `margin: 0`; su texto `14px`, peso `700`, color `#fff` (theme.css:1607-1615).

**Estadísticas** `> .second-subtitle-container` ("N canciones, duración") (theme.css:1616-1630):
`grid-area: stats`, `justify-content: flex-start`, `margin: 0`. `::before` con `content: "•"`,
`margin: 0 6px`, `font-size: 14px`, color `rgba(255,255,255,.7)`. Texto `14px`, color
`rgba(255,255,255,.7)`. Resultado: `Artista • 12 canciones, 45 min` en una línea.

**Descripción** `> #header-description` (theme.css:1631-1647), comentario: alineada a la izquierda (YT la
centra), 2 líneas y "…" (el recorte lo hace YT): `grid-area: desc`, `margin: 8px 0 0`, `max-width: 720px`.
`#description-button` → `justify-content: flex-start`, `text-align: left`. `.description` y descendientes →
`text-align: left`, `14px`, `line-height 20px`, color `rgba(255,255,255,.7)`.

### 2.17 Fila de acciones del álbum (theme.css:1648-1719)

**Qué es:** `#action-buttons` contiene `ytmusic-play-button-renderer` (play grande), "Guardar"
(`ytmusic-toggle-button-renderer`), y otros (aleatorio, menú "⋮"…).

Comentario theme.css:1648-1649: play grande primero y el resto como círculos iguales, a 12 px; se anulan los
márgenes propios de YT (sumaban 16 px a cada botón).

- `> #action-buttons` (theme.css:1650-1657): `grid-area: act`, `justify-content: flex-start`,
  `align-items: center`, `gap: 12px`, `margin: 24px 0 8px`, `width: auto`.
- `#action-buttons > *` → `margin: 0`, `flex: none` (theme.css:1658-1661).
- **Play** (theme.css:1662-1676): `order: -1` (va primero), `width: 56px`, `min-width: 0` (YT le fuerza 64 px y
  quedaba ovalado), `height: 56px`, `margin-right: 4px` (separación total con el siguiente: 12 + 4 = 16 px),
  variables `--ytmusic-play-button-*` (ver §1), `background: #fff`, `border-radius: 50%`,
  `transition: transform .12s ease`. Icono 28 px negro (vía variables).
  - `.content-wrapper` (theme.css:1677-1683): `56 × 56 px`, `margin: 0`, centrado (`justify-content` y
    `align-items: center`).
- **Resto de botones** `> :not(ytmusic-play-button-renderer)` (theme.css:1685-1692): `40 × 40 px`,
  `border-radius: 50%`, `display: flex`, centrados.
- **"Guardar"** (theme.css:1693-1705): comentario: trae un botón interior de 24 px descentrado; se iguala al
  círculo. Su `a`, `yt-icon-button` y `#button`: `40 × 40 px`, `margin: 0`, `padding: 8px`, `display: flex`,
  centrado, `box-sizing: border-box`.
- Iconos `yt-icon` y `.ytSpecButtonShapeNextIcon` → `24 × 24 px` (theme.css:1706-1710).
- `button` y `yt-icon-button` → `background: transparent`, `color: rgba(255,255,255,.7)` (theme.css:1711-1715).

### 2.18 Lista de canciones del álbum (theme.css:1721-1800)

**Qué es:** `ytmusic-shelf-renderer` es la lista; `> div > #contents` contiene las filas
`ytmusic-responsive-list-item-renderer`. Comentario theme.css:1722-1723: YT fija la lista a ~921 px centrada;
aquí ocupa todo el ancho. Su `#contents` va dentro de un div intermedio.

- `ytmusic-shelf-renderer` (theme.css:1724-1729): `position: relative`, `width: auto`, `max-width: none`,
  `margin: 8px 0 32px`.
- `> div > #contents` (theme.css:1730-1733): `position: relative`, `padding-top: 44px` (hueco para la fila de
  encabezado de 36 px + 8 px).
- **Encabezado "#"** `#contents::before` (theme.css:1734-1743): `content: "#"`, `position: absolute`,
  `top: 10px`, `left: 16px`, `width: 32px`, `text-align: center`, `14px`, color `rgba(255,255,255,.7)`.
  (Se alinea con la columna del número: fila con `padding-left: 16px` y `.left-items` de 32 px.)
- **Encabezado "Título" + reloj + divisor** `#contents::after` (theme.css:1744-1760): `content: "Título"`,
  `position: absolute`, `top: 0`, `left: 0`, `right: 0`, `height: 36px`, `padding-left: 64px`
  (16 + 32 + 16: alinea con el título de las filas), `line-height: 38px`, `box-sizing: border-box`, `14px`,
  color `rgba(255,255,255,.7)`, `border-bottom: 1px solid rgba(255,255,255,.08)`, `pointer-events: none`.
  - Fondo: icono de reloj (`schedule` de Material Symbols) en SVG inline, `no-repeat right 24px center /
    18px 18px` (18 × 18 px, a 24 px del borde derecho, centrado verticalmente), color de relleno
    **`#b3b3b3`**. SVG exacto (viewBox `0 -960 960 960`):
    `m612-292 56-56-148-148v-184h-80v216l172 172ZM480-80q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Zm0-400Zm0 320q133 0 226.5-93.5T800-480q0-133-93.5-226.5T480-800q-133 0-226.5 93.5T160-480q0 133 93.5 226.5T480-160Z`
    (theme.css:1758).
- **Filas** (theme.css:1761-1770), comentario: 56 px, número gris, resaltado al pasar el ratón:
  `min-height: 0`, `height: 56px`, `margin: 0` (YT añade 8 px abajo a cada fila), `padding: 0 16px`,
  `border-radius: 6px`, `border: none`, `transition: background-color .12s linear`.
- `.left-items` (columna número/miniatura) → `32 × 32 px`, `margin-right: 16px` (theme.css:1774-1778).
- Tipografía (comentario theme.css:1779): la de YT Music tal cual (familia, tamaño y peso); solo colores:
  - `.index` (número) → `rgba(255,255,255,.7)`, `tabular-nums` (theme.css:1780-1783).
  - `.title` y descendientes → `#fff` (theme.css:1784-1787).
  - `.secondary-flex-columns` y descendientes (artista, álbum) → `rgba(255,255,255,.7)` (theme.css:1788-1791).
  - `.fixed-column` (duración) → `rgba(255,255,255,.7)`, `tabular-nums` (theme.css:1792-1795).
  - Tamaños/familia exactos de YT: **no están en este archivo** (no claro; habría que medirlos en YT).
- La que suena (theme.css:1796-1800): `[play-button-state="playing"]` o `"paused"` → `.title *` con
  `font-weight: 700` (blanco y negrita; comentario: Spotify la marca en su color de acento).

### 2.19 Margen inferior y rendimiento (theme.css:1865-1888)

- `ytmusic-app-layout > #content` → `padding-bottom: calc(var(--md-gap) * 2)` = **16 px** (theme.css:1865-1866):
  espacio para la barra flotante del reproductor al final del contenido. (`ytmusic-app-layout` es el layout
  raíz de YT; `#content` la zona de páginas.)
- Rendimiento ("no calcular ni pintar estanterías fuera de pantalla", theme.css:1868-1884):
  - `ytmusic-carousel-shelf-renderer`, `ytmusic-shelf-renderer`, `ytmusic-grid-renderer`,
    `ytmusic-immersive-carousel-shelf-renderer` → `content-visibility: auto`, `contain-intrinsic-size: auto 360px`.
  - `ytmusic-responsive-list-item-renderer` → `content-visibility: auto`, `contain-intrinsic-size: auto 56px`.
  - `ytmusic-player-queue-item` → `content-visibility: auto`, `contain-intrinsic-size: auto 64px`.
  - Equivalente nativo: virtualizar listas/estanterías fuera de pantalla con alturas estimadas 360 / 56 / 64 px.
- `ytmusic-app-layout:not([player-page-open]) ytmusic-player-page` → `content-visibility: hidden`
  (theme.css:1885-1888): "el reproductor a pantalla completa no debe costar nada cuando está cerrado".
  `player-page-open` es el atributo que YT pone al layout con Now Playing abierto. En nativo: no dibujar
  Now Playing cuando está cerrado.

---

## 3. Estados

| Elemento | Reposo | Hover | Active (pulsado) | Seleccionado / encendido | Origen |
|---|---|---|---|---|---|
| Botones Me gusta / No me gusta | fondo transparente sobre la píldora `rgba(255,255,255,.14)`; icono `#fff` | `background-color: rgba(255,255,255,.08)` | `rgba(255,255,255,.14)` | marcado: el fondo NO cambia; YT cambia el icono a relleno | theme.css:1013-1024, 948-949 |
| Play/pausa de la barra | círculo `#fff`, icono `#000` | `box-shadow: inset 0 0 0 44px rgba(0,0,0,.1)` | `inset 0 0 0 44px rgba(0,0,0,.16)` | — | theme.css:1026-1027 |
| Pestañas de Now Playing | texto `rgba(255,255,255,.7)` | `background: rgba(255,255,255,.08)` | `rgba(255,255,255,.14)` | `.iron-selected` / `[aria-selected="true"]`: texto `#fff` + barra de 3 px blanca | theme.css:1105-1130 |
| av-toggle (Canción/Vídeo) | botón transparente, icono `rgba(255,255,255,.7)` | `background: rgba(255,255,255,.08)` | (no definido) | `[aria-pressed="true"]`: `background: rgba(255,255,255,.2)`, icono `#fff` | theme.css:1179-1183 |
| av-toggle deshabilitado | `[toggle-disabled]` → `visibility: hidden` (conserva el hueco) | | | | theme.css:1075 |
| Chips de la cola (y "Guardar") | no seleccionado: `rgba(255,255,255,.14)` | `rgba(255,255,255,.2)` | (no definido) | seleccionado: blanco (de YT, sin tocar) | theme.css:1289-1294 |
| Interruptor reproducción automática | pista `rgba(255,255,255,.08)` con borde 2 px `rgba(255,255,255,.56)`; asa 16 px `rgba(255,255,255,.56)` en (8, 8) | (no definido) | (no definido) | `[checked]`: pista y borde `#fff`; asa 24 px `#000` en (24, 4) | theme.css:1330-1362 |
| Canción de la cola (Now Playing) | transparente | `rgba(255,255,255,.08)` | `rgba(255,255,255,.14)` | `[selected]`: fondo `rgba(255,255,255,.14)`, título `#fff` peso 500. Además (regla genérica) `[play-button-state="playing"/"paused"]` → fondo `rgba(255,255,255,.14)` | theme.css:1417-1427, 1136-1140 |
| Botón play del álbum | `#fff`, escala 1 | `transform: scale(1.05)` | — | — | theme.css:1684 |
| Otros botones de acción del álbum | icono `rgba(255,255,255,.7)`, fondo transparente | icono `#fff` | — | — | theme.css:1711-1719 |
| Filas de la lista del álbum | transparente | `rgba(255,255,255,.08)` | — | la que suena (`playing`/`paused`): título peso 700 | theme.css:1771-1773, 1797-1800 |

Focus: no hay reglas de `:focus` / `:focus-visible` en este tramo.

---

## 4. Animaciones y transiciones

No hay `@keyframes` en este tramo.

| Elemento | Propiedad | Duración | Easing | Retardo | Origen |
|---|---|---|---|---|---|
| Botones Me gusta / No me gusta | `background-color` | .15s | `linear` | 0 | theme.css:1019 |
| Pestañas de Now Playing | `background-color` | .15s | `linear` | 0 | theme.css:1109 |
| Botones av-toggle | `background-color`, `color` | .15s cada una | `linear` | 0 | theme.css:1177 |
| Pista del interruptor | `background-color`, `border-color` | .2s | `linear` | 0 | theme.css:1340 |
| Asa del interruptor | `transform`, `width`, `height` | .2s | `cubic-bezier(.2, 0, 0, 1)` (`--m3-ease`) | 0 | theme.css:1351 |
| Asa del interruptor | `background-color` | .2s | `linear` | 0 | theme.css:1351 |
| Canciones de la cola | `background-color` | .15s | `linear` | 0 | theme.css:1385 |
| Play grande del álbum | `transform` (escala 1 → 1.05) | .12s | `ease` | 0 | theme.css:1675 |
| Filas de la lista del álbum | `background-color` | .12s | `linear` | 0 | theme.css:1769 |
| (Play/pausa de la barra, fuera del tramo) | `box-shadow` | .15s | `linear` | 0 | theme.css:945 |

El fondo difuminado de Now Playing no se anima (solo se repinta al cambiar de canción, comentario
theme.css:1036-1038). La animación de abrir/cerrar Now Playing la hace YT (mencionada en theme.css:1044).

---

## 5. Responsive

### 5.1 `@media (min-width: 940px)` — Now Playing con la cola a la derecha (theme.css:1802-1863)

Comentario theme.css:1802-1807: la cola es una tarjeta flotante como la barra lateral: el fondo de la carátula
pasa por detrás y la tarjeta (pseudoelemento, `z-index: -1` dentro de `#side-panel`, que es `fixed`) deja la
misma franja con la barra superior, la del reproductor y el borde derecho. La carátula queda centrada en el
espacio restante. Solo cuando YT pone los dos paneles lado a lado; en ventanas estrechas los apila y se respeta.

Con ventana ≥ 940 px:

- `ytmusic-player-page > .content` (theme.css:1809-1814): `padding: 0`, `gap: 0`, `height: 100%`,
  `box-sizing: border-box`.
- `ytmusic-player-page` (theme.css:1815-1822): define `--md-np-side` y redefine `--md-np-art` (ver §1).
  Ejemplo: ventana 1280 × 800 sin guía y sin ancho elegido → `--md-np-side = clamp(340, 358.4, 440) = 358.4px`;
  `--md-np-art = min(800 − 430, 1280 − 0 − 358.4 − 96) = min(370, 825.6) = 370px` (cálculo ilustrativo).
- `#main-panel` (carátula) (theme.css:1823-1830): `flex: 1 1 auto`, `min-width: 0`, `margin: 0`,
  `padding: 32px`, `justify-content: center`, `box-sizing: border-box`.
- `#side-panel` (cola) (theme.css:1831-1842): `flex: 0 0 var(--md-live-side, var(--md-np-side))`,
  `width: var(--md-live-side, var(--md-np-side))`, `max-width: none`, `height: 100%`, `margin: 0`,
  `padding: 8px 12px 0` (sobrescrito justo después), `border-radius: 0`, `background: transparent`,
  `box-sizing: border-box`.
- `html body ytmusic-player-page #side-panel` (theme.css:1843-1846) — "html body" para pesar más que el relleno
  de lyrics.css (`html ytmusic-player-page #side-panel`):
  `padding: calc(nav 64px + gap 8px + 8px) calc(gap 8px + 8px) calc(player-bar 92px + gap 8px) 8px` =
  **80px 16px 100px 8px** con los valores por defecto.
- **Tarjeta** `#side-panel::before` (theme.css:1847-1858): `content: ""`, `position: absolute`, `z-index: -1`,
  `top: calc(64px + 8px)` = **72px**, `bottom: calc(92px + 8px)` = **100px**, `left: 0`, `right: 8px`
  (`--md-gap`), `border-radius: 10px` (`--md-radius`), `background: rgba(12,12,12,.6)` (`--md-glass`),
  `border: 1px solid rgba(255,255,255,.08)` (`--md-line`).
  - Geometría resultante: la tarjeta empieza 8 px bajo la barra superior, acaba 8 px sobre la barra del
    reproductor y deja 8 px al borde derecho de la ventana. El contenido (pestañas, cola) queda 8 px dentro
    de la tarjeta por arriba (80 − 72), 8 px por la izquierda y 8 px por la derecha (16 − 8); por abajo el
    contenido acaba exactamente en el borde inferior de la tarjeta (100 px).
- **Pantalla completa** (`html[data-md-fullscreen]`, sin barra de reproducción) (theme.css:1859-1862): el
  `padding-bottom` del panel pasa a `var(--md-gap)` = 8 px y el `bottom` de la tarjeta a 8 px: tarjeta y lista
  acaban a 8 px del borde (antes la lista seguía hasta abajo, fuera de la tarjeta).

Por debajo de 940 px: se usa la disposición apilada de YT con la tarjeta de 2.10 (fondo `rgba(12,12,12,.6)`,
radio 16 px, padding `0 8px`) y `overflow-x: hidden` en `.content` salvo en el diseño móvil.

### 5.2 Otros umbrales que aparecen

- `≤ 615 px` (diseño móvil de YT, atributo `is-mweb-modernization-enabled`): no se aplica el `overflow-x: hidden`
  de `.content` (theme.css:1095-1098). No es un `@media` propio; depende del atributo que pone YT.
- Tamaños fluidos con `clamp()` (sin media query):
  - Carátula del álbum: `clamp(160px, 18vw, 232px)` (theme.css:1507, 1527-1528).
  - Título del álbum: `clamp(32px, 5.5vw, 88px)`; largo: `clamp(28px, 3.4vw, 56px)` (theme.css:1561, 1582).
  - Ancho de la cola: `clamp(340px, 28vw, 440px)` (theme.css:1818).
- No hay `@container` en este tramo.

---

## 6. Comentarios del autor (intenciones), resumidos por zona

- theme.css:948-952 — Me gusta/No me gusta como en la app móvil: píldora tonal con divisor; al marcar no cambia
  el fondo; sigue en su sitio del DOM (si se mueve, YT confunde sus clics con clics en la barra) y se dibuja
  junto al volumen con anclaje CSS; el bloque derecho reserva su hueco con padding.
- theme.css:955-956 — la zona vacía del bloque derecho no recoge clics, para que lleguen a la píldora.
- theme.css:962 — el ancla es el grupo de botones, no todo el bloque.
- theme.css:969 — la referencia de posición debe ser la barra.
- theme.css:977 — borde derecho de la píldora a 8 px del grupo de botones.
- theme.css:982-983 — z-index 2: si no, el clic caía en la barra y abría Now Playing.
- theme.css:1025 — capa de estado sobre el relleno del play.
- theme.css:1035-1038 — Now Playing "Apple Music"; blur en capa de 64 × 64 escalada ×40 en GPU; `--md-art` lo pone preload.js.
- theme.css:1042-1044 — `will-change: auto` para evitar texto borroso por capa en posición fraccionaria.
- theme.css:1047 — degradado con el color de la carátula, "bien presente".
- theme.css:1062 — oscurecer hacia abajo para leer texto con carátulas claras.
- theme.css:1070-1072 — tamaño de carátula y fila título/artista en lyrics.css; aquí el alto reservado.
- theme.css:1074 — selector Canción/Vídeo oculto sin mover la carátula si no hay vídeo.
- theme.css:1089 — panel como tarjeta translúcida, sin backdrop-filter.
- theme.css:1095-1097 — sin scroll horizontal al apilar, excepto en diseño móvil (≤ 615 px).
- theme.css:1099-1100 — pestañas "primary tabs" de M3.
- theme.css:1143-1145 — selector solo iconos, píldora oscura, elegido en píldora más clara; texto con font-size 0 por accesibilidad.
- theme.css:1160-1161 — `appearance: none` para evitar el botón nativo claro de Windows.
- theme.css:1184 — letras grandes estilo Apple Music.
- theme.css:1192-1195 — cola en Material 3 (surface container 16 px, botón contorneado, interruptor M3, listas de dos líneas de 64 px, subcabecera con divisor, Roboto).
- theme.css:1205, 1223, 1240 — cabecera label-medium / title-medium; "Guardar" píldora tonal; versión compacta a 32 px.
- theme.css:1278-1280 — filtros fijos arriba al desplazar; sin fondo; preload.js recorta la lista bajo ellos.
- theme.css:1287-1288 — chips con fondo aclarado; el seleccionado sigue blanco.
- theme.css:1296-1298 — dos `.autoplay` distinguidos por el padre, sin `:has()`.
- theme.css:1317-1318 — interruptor M3: medidas; solo se animan transform y colores.
- theme.css:1372 — YT deja el título de la subcabecera al 50 % (3,2:1); se pone al 100 %.
- theme.css:1380 — canciones: lista de dos líneas 64 px, miniatura 40 px con radio 8 px.
- theme.css:1419-1420 — la que suena con el fondo aclarado (un gris fijo se perdía sobre carátulas de color).
- theme.css:1430-1435 — álbumes/playlists apilados al estilo Spotify; máximo 1400 px centrado.
- theme.css:1445-1446, 1457 — columnas con alto real; lista sin scroll propio.
- theme.css:1466-1467, 1478, 1485-1486, 1492-1494 — playlists propias a todo el ancho; dueño en la celda del artista; alineación vertical centrada (desfase de 7 px); lápiz de editar portada en la esquina de la carátula.
- theme.css:1506, 1515 — primera columna incluye los 24 px; sin column-gap.
- theme.css:1563-1565, 1570, 1575-1576 — interlineado 1.1, line-clamp 2, relleno para no cortar descendentes ni la última letra.
- theme.css:1580 — títulos > 24 caracteres más pequeños (preload.js los marca).
- theme.css:1590-1591 — avatar recortado con object-fit (YT lo estiraba a 24 × 16).
- theme.css:1631 — descripción a la izquierda, 2 líneas (recorte de YT).
- theme.css:1648-1649, 1665, 1693 — fila de acciones a 12 px; play sin min-width de 64 px; "Guardar" igualado al círculo.
- theme.css:1721-1723, 1757 — encabezado "# Título ⏱" con pseudoelementos; lista a todo el ancho; reloj de Material Symbols.
- theme.css:1761, 1765, 1779, 1796 — filas de 56 px sin el margen de 8 px de YT; tipografía de YT tal cual; la que suena en negrita.
- theme.css:1802-1807, 1816-1817, 1819-1820, 1832, 1843, 1859-1860 — cola a la derecha como tarjeta flotante; ancho elegible; cálculo del alto útil de la carátula; ancho en vivo al arrastrar; "html body" para ganar a lyrics.css; pantalla completa.
- theme.css:1865, 1869, 1885 — margen para la barra flotante; no pintar lo que está fuera de pantalla ni Now Playing cerrado.

---

## 7. Checklist de réplica

1. `#right-controls` de la barra reserva 113 px a su izquierda (`105 + 8`) para la píldora Me gusta.
2. La píldora Me gusta/No me gusta mide 105 × 40 px, radio 20 px, fondo `rgba(255,255,255,.14)`, recorta su contenido.
3. Su borde derecho queda 8 px a la izquierda del grupo de botones derecho (volumen…), centrada verticalmente con él, por encima (z-index 2).
4. Cada mitad mide 52 × 40 px; Me gusta con padding `8 12 8 16`, No me gusta con `8 16 8 12`; icono `#fff`.
5. Divisor vertical de 1 × 20 px (top/bottom 10 px) `rgba(255,255,255,.24)` en el borde izquierdo de No me gusta.
6. Hover en cada mitad: `rgba(255,255,255,.08)`; pulsado: `rgba(255,255,255,.14)`; transición .15s linear.
7. Marcar Me gusta no cambia el fondo, solo el icono a relleno.
8. Play/pausa de la barra: hover oscurece el círculo blanco con `rgba(0,0,0,.1)`, pulsado `rgba(0,0,0,.16)`; icono 24 px `#000`.
9. Now Playing: fondo `#000` + carátula a 64 × 64 con `blur(4px) saturate(1.6) brightness(.55)` escalada ×40 desde (0,0).
10. Encima, degradado vertical `rgba(0,0,0,.15)` → `rgba(0,0,0,.55)` en toda la página.
11. El contenido va por encima de ambas capas (z-index 1); el fondo solo se recalcula al cambiar de canción.
12. Texto de Now Playing nítido (sin capas en posición fraccionaria).
13. Con tema, `--md-np-reserve = 360px` → carátula = alto de ventana − 360 px (fórmula de lyrics.css), en ventanas < 940 px.
14. Selector Canción/Vídeo invisible (con su hueco) cuando no hay vídeo.
15. Carátula, miniatura y vídeo con radio 12 px; carátula con sombra `0 24px 60px rgba(0,0,0,.55)`.
16. Sin scroll horizontal en Now Playing apilado (salvo diseño móvil ≤ 615 px).
17. Pestañas: alto 48 px, Roboto 14 px peso 500, lh 20 px, tracking .1px, sin mayúsculas, color `rgba(255,255,255,.7)`; seleccionada `#fff`.
18. Pestañas: hover `rgba(255,255,255,.08)`, pulsada `rgba(255,255,255,.14)`, transición .15s linear.
19. Divisor de 1 px `rgba(255,255,255,.08)` bajo la fila de pestañas; indicador de 3 px blanco con radio `3 3 0 0`.
20. Selector Canción/Vídeo: píldora 44 px de alto, padding 4, gap 4, radio 22, fondo `rgba(255,255,255,.08)`, sin borde ni pseudoelementos.
21. Botones del selector: 56 × 36 px, radio 18, solo icono centrado (texto con tamaño 0, accesible), color `rgba(255,255,255,.7)`.
22. Selector: hover `rgba(255,255,255,.08)`; activo `rgba(255,255,255,.2)` con icono `#fff`; transición .15s linear de fondo y color.
23. Letra estática: 22 px, peso 700, interlineado 1.45, `#fff`.
24. Panel lateral (< 940 px): fondo `rgba(12,12,12,.6)`, radio 16 px, padding horizontal 8 px, todo en Roboto.
25. Cabecera de la cola: padding `6 4 0`, centrada; "Reproduciendo de" 11/14 px peso 500 tracking .5 `rgba(255,255,255,.7)`; nombre 14/20 px peso 500 tracking .15 `#fff`.
26. "Guardar" de la cola: 32 px alto, padding `0 12 0 10`, gap 6, 13 px, radio 20, icono 18 px, texto `#fff`, fondo `rgba(255,255,255,.14)`, hover `.2`.
27. Fila de filtros: padding `4 0 6`, gap 6 px; chips 32 px, padding `0 12`, 13 px; no seleccionados `rgba(255,255,255,.14)` (hover `.2`); seleccionado blanco.
28. Fila de filtros pegada arriba (sticky, z-index 2) al desplazar la cola, sin fondo; la lista se recorta bajo ella.
29. Fila de reproducción automática: padding 8, margen vertical 4, centrada; título 16/24 peso 400 tracking .5 `#fff`; descripción 12/16 tracking .4 `rgba(255,255,255,.7)`.
30. Interruptor apagado: pista 52 × 32, radio 16, fondo `rgba(255,255,255,.08)`, borde 2 px `rgba(255,255,255,.56)`; asa 16 px `rgba(255,255,255,.56)` en (8, 8); sin etiqueta.
31. Interruptor encendido: pista y borde `#fff`; asa 24 px `#000` en (24, 4).
32. Transiciones del interruptor: colores .2s linear; posición y tamaño del asa .2s `cubic-bezier(.2, 0, 0, 1)`.
33. Subcabecera "Reproducción automática activada": margen `8 0 4`, padding `16 8 8`, divisor superior 1 px `rgba(255,255,255,.08)`; texto 14/20 peso 500 tracking .1 `rgba(255,255,255,.56)` opacidad 1.
34. Canción de la cola: alto mínimo 64 px, padding `0 12 0 8`, radio 12, sin separadores.
35. Miniatura de la cola 40 × 40 px, radio 8, margen derecho 16 px (incluida la capa de play encima).
36. Título de canción 16/24 peso 400 tracking .5 `#fff`; artista 14/20 tracking .25 `rgba(255,255,255,.7)`; duración 12 px peso 500 tracking .5 `rgba(255,255,255,.7)` con cifras tabulares.
37. Canción de la cola: hover `.08`, pulsada `.14`, transición .15s linear; la que suena fondo `.14` y título peso 500.
38. Álbum/playlist: contenedor de máx. 1400 px centrado, padding `0 32 32`, columnas de YT apiladas, lista sin scroll propio.
39. Cabecera en rejilla: columna 1 = carátula + 24 px; áreas cover/type/title/by/stats/desc/act; fila 1 `1fr`; todo alineado abajo y a la izquierda; padding superior 40 px; sin hueco entre columnas.
40. Carátula del álbum `clamp(160, 18vw, 232)` px cuadrada, radio 8, sombra `0 8px 24px rgba(0,0,0,.5)`, imagen en cover.
41. Tipo/año encima del título: 14 px peso 500 `#fff`, margen inferior 4 px.
42. Título: "YouTube Sans"/Roboto, `clamp(32, 5.5vw, 88)` px (largo > 24 car.: `clamp(28, 3.4vw, 56)`), peso 800, interlineado 1.1, tracking −0.03em, máx. 2 líneas con "…", margen inferior 16 px, sin cortar descendentes ni la última letra.
43. Artista: avatar circular 24 px (recortado, no estirado) + 8 px + nombre 14 px peso 700 `#fff`; en playlists propias el dueño ocupa ese hueco.
44. Estadísticas: "• " (margen 6 px a cada lado) + texto 14 px `rgba(255,255,255,.7)`, en la misma línea que el artista; ambos centrados verticalmente entre sí.
45. Descripción: margen superior 8, máx. 720 px, izquierda, 14/20 `rgba(255,255,255,.7)`, 2 líneas con "…".
46. Lápiz de editar portada (playlist propia) en la esquina inferior derecha de la carátula.
47. Fila de acciones: margen `24 0 8`, gap 12, play primero.
48. Play del álbum: círculo 56 px blanco, icono 28 px negro, margen derecho extra 4 px, hover escala 1.05 en .12s ease.
49. Otros botones de acción: círculos de 40 px, icono 24 px `rgba(255,255,255,.7)` (hover `#fff`), fondo transparente; "Guardar" con padding 8 centrado.
50. Lista del álbum a todo el ancho, margen `8 0 32`, con encabezado de 36 px: "#" centrado en 32 px a 16 px del borde (top 10), "Título" a 64 px (lh 38), reloj 18 px `#b3b3b3` a 24 px de la derecha, todo 14 px `rgba(255,255,255,.7)`, divisor inferior 1 px `rgba(255,255,255,.08)`; filas empiezan 44 px bajo el inicio.
51. Filas del álbum: 56 px, padding `0 16`, radio 6, sin margen ni borde, hover `.08` en .12s linear; columna número 32 × 32 + 16 px.
52. Colores de filas: número y duración `rgba(255,255,255,.7)` tabulares; título `#fff`; artista/álbum `rgba(255,255,255,.7)`; la que suena con título peso 700.
53. Ventana ≥ 940 px: Now Playing con carátula a la izquierda (padding 32, centrada) y cola a la derecha de ancho `--md-right-w` o `clamp(340, 28vw, 440)` (en vivo `--md-live-side` al arrastrar).
54. Ventana ≥ 940 px: lado de carátula = `min(alto − 430, ancho − guía − cola − 96)`.
55. Ventana ≥ 940 px: tarjeta de la cola de top 72 / bottom 100 / left 0 / right 8 px, radio 10, fondo `rgba(12,12,12,.6)`, borde 1 px `rgba(255,255,255,.08)`; contenido con padding `80 16 100 8`.
56. Pantalla completa (≥ 940 px): tarjeta y contenido acaban a 8 px del borde inferior.
57. Margen inferior de 16 px al final del contenido de cualquier página.
58. Virtualizar estanterías (est. 360 px), filas de lista (56 px) y filas de cola (64 px) fuera de pantalla; no dibujar Now Playing cerrado.

### Puntos no aclarados en este tramo

- De dónde salen los iconos (auriculares/vídeo) del selector Canción/Vídeo: el CSS solo oculta el texto.
- Color del texto del chip seleccionado ("sigue blanca" se refiere al fondo de YT).
- Tamaño/familia de la tipografía de las filas del álbum (se deja la de YT).
- Qué dibujaban `.av-toggle::before/::after` en YT (se eliminan).
