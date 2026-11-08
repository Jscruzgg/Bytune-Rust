# 02 — Tema (theme.css), parte 1: líneas 1–950

Fuente: `C:\Users\JuanC\Documents\ByTune\src\theme.css`, líneas 1 a 950 (el último bloque es un
comentario de las líneas 948–952; la regla que describe empieza en la 953 y queda para la parte 2).

Este documento describe lo que el tema inyectado hace sobre la interfaz de YouTube Music (YTM) para
poder reproducirlo en una app nativa (Rust + egui). Casi todo lleva `!important` en el original
(para ganar a los estilos de YTM); aquí se omite salvo que importe.

## 0. Contexto y convenciones

- **Filosofía del autor (theme.css:1-4):** el tema mezcla Spotify (paneles redondeados, tarjetas
  con hover), Apple Music (tipografía, carátulas redondeadas, barra flotante) y la estructura de
  YTM. Regla de rendimiento declarada: nada de `backdrop-filter`, nada de blur animado,
  transiciones solo en `transform`/`opacity` y `content-visibility` para no pintar lo que no está
  en pantalla. (Nota: en este tramo hay varias transiciones de `background-color`, `height` y
  `box-shadow`, que contradicen en parte esa regla; se documentan tal cual. `content-visibility`
  no aparece en las líneas 1–950.)
- **Atributos `data-md-*` y clases `.md-*`:** los pone/crea `preload.js` (script de la app), no
  YTM. Se indica en cada caso.
- **Paleta `--md-*` (definida en chrome.css:6-28, documentada en otra parte):** se usa aquí con
  estos valores, para poder leer el documento sin saltar:

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
| `--md-raised-line` | `rgba(255,255,255,.1)` |
| `--md-raised-shadow` | `0 16px 40px rgba(0,0,0,.5)` |
| `--md-field` | `rgba(255,255,255,.08)` |
| `--md-state-hover` | `rgba(255,255,255,.08)` |
| `--md-state-on` | `rgba(255,255,255,.14)` |
| `--md-state-press` | `rgba(255,255,255,.2)` |
| `--md-state-ink` | `rgba(0,0,0,.1)` |
| `--md-state-ink-press` | `rgba(0,0,0,.16)` |
| `--md-line` | `rgba(255,255,255,.08)` |
| `--md-line-strong` | `rgba(255,255,255,.24)` |
| `--md-danger` | `#ff8b8b` |

- **Medidas globales de la ventana que fija este tramo:** barra superior de YTM = 64 px de alto
  (valor de YTM, que el tema da por hecho); barra del reproductor = **92 px** (YTM trae 72;
  theme.css:603-605). Barra lateral abierta = 240–420 px (ajustable), plegada = 72 px.

---

## 1. Variables definidas

### 1.1 En `:root` (theme.css:6-25)

| Variable | Valor | Para qué |
|---|---|---|
| `--md-radius` | `10px` | Radio estándar de tarjetas/paneles (tarjeta de la barra lateral, tarjetas de estantería). |
| `--md-gap` | `8px` | Separación estándar entre paneles y bordes de ventana. |
| `--md-font` | `Roboto, Arial, sans-serif` | Fuente de texto (la original de YTM). No se aplica en este tramo; queda definida. |
| `--md-font-display` | `"YouTube Sans", Roboto, Arial, sans-serif` | Fuente de títulos (la original de YTM). Definida, no usada en este tramo. |
| `--ytmusic-player-bar-background` | `var(--md-bar)` = `#0e0e0e` | Sobrescribe la variable de YTM: fondo de la barra del reproductor. |
| `--ytmusic-search-background` | `var(--md-field)` = `rgba(255,255,255,.08)` | Fondo del buscador. |
| `--ytmusic-text-primary` | `var(--md-text)` = `#fff` | Texto principal de toda YTM. |
| `--ytmusic-text-secondary` | `var(--md-text-2)` = `rgba(255,255,255,.7)` | Texto secundario de toda YTM. |

Comentarios del autor (theme.css:7-9, 12-13, 17-20):
- La paleta está en chrome.css porque esa hoja se inyecta siempre (también con el tema apagado,
  para el menú de ajustes y los botones de la barra superior).
- Se mantienen Roboto y YouTube Sans porque las cajas de texto de YTM tienen alto fijo medido para
  esas fuentes; con fuentes más altas las letras se cortan. **Para la réplica: usar Roboto (texto)
  y YouTube Sans (títulos), o métricas equivalentes.**
- Los fondos de página (colores, degradados de carátula) se dejan como en YTM.
- **Sin color de acento propio:** el blanco hace de acento; el color lo aportan la carátula y su
  fondo. Los rojos de YTM se quedan como YTM los tenga.

### 1.2 En `html` (theme.css:88-90)

| Variable | Valor | Para qué |
|---|---|---|
| `--md-left-eff` | `min(var(--md-left-w, 240px), max(240px, 50vw - 164px))` | Ancho efectivo de la barra lateral. `--md-left-w` lo pone preload.js al arrastrar (240–420 px). En ventanas estrechas se limita a `50vw − 164px` (pero nunca menos de 240 px), para que el botón atrás (en x = barra + 52 px) quede siempre a la izquierda del buscador centrado, incluso con el buscador a su mínimo de 200 px. |

### 1.3 Variables que se leen pero vienen de fuera

| Variable | Origen | Valor por defecto aquí |
|---|---|---|
| `--md-left-w` | preload.js (arrastre de la barra lateral) | 240px |
| `--md-guide-w` | preload.js (registrada con `@property`, según el comentario); 100 % salvo durante la animación de plegar/desplegar | 100% |
| `--md-right-w` | preload.js (ancho del panel derecho / cola) | `clamp(340px, 28vw, 440px)` |
| `--md-nav-right` | preload.js (mide lo que ocupan transmitir, ajustes, avatar y botones de ventana a la derecha de la barra superior) | 360px |
| `--ytmusic-guide-width` | YTM (240 px abierta) | — (el tema la sobrescribe) |
| `--ytmusic-player-bar-height` | YTM (72 px) | el tema la fija a 92px |

### 1.4 En `ytmusic-nav-bar`, solo con `@media (min-width: 820px)` (theme.css:385-389)

| Variable | Valor | Para qué |
|---|---|---|
| `--md-nav-l` | `calc(max(var(--ytmusic-guide-width, 240px), 240px) + 70px)` | Borde izquierdo del hueco libre para el buscador: barra lateral (mín. 240) + 70 px del botón atrás. |
| `--md-nav-r` | `var(--md-nav-right, 360px)` | Lo que ocupa la zona derecha de la barra superior. |
| `--md-search-w` | `clamp(160px, calc(100vw - var(--md-nav-l) - var(--md-nav-r)), 480px)` | Ancho del buscador: máx. 480 px (como Spotify), mín. 160 px, si no lo que quepa. |

### 1.5 En `ytmusic-player-bar` (theme.css:614, 649, 672, 967)

| Variable | Valor | Para qué |
|---|---|---|
| `--md-center-w` | `clamp(260px, 38vw, 600px)`; con `max-width: 900px` → `clamp(260px, 36vw, 600px)` | Ancho de la columna central (controles + progreso). |
| `--md-like-w` | `105px` (= 52 + 1 + 52; theme.css:967, fuera de mi tramo pero necesario); con `max-width: 760px` → `40px` (theme.css:672) | Ancho reservado para la píldora me gusta / no me gusta. |

### 1.6 Segundo `:root` "Material 3" (theme.css:760-766)

| Variable | Valor | Para qué |
|---|---|---|
| `--m3-font` | `Roboto, "Segoe UI Variable Text", system-ui, sans-serif` | Fuente de toda la barra del reproductor y de los chips. |
| `--m3-headphones` | SVG en data-URI (Material Symbols "headphones"), viewBox `0 -960 960 960`, path: `M360-120H200q-33 0-56.5-23.5T120-200v-280q0-75 28.5-140.5t77-114q48.5-48.5 114-77T480-840q75 0 140.5 28.5t114 77q48.5 48.5 77 114T840-480v280q0 33-23.5 56.5T760-120H600v-320h160v-40q0-117-81.5-198.5T480-760q-117 0-198.5 81.5T200-480v40h160v320Z` | Icono de la opción "Canción" del selector Canción/Vídeo. |
| `--m3-video` | SVG en data-URI (Material Symbols "smart_display"), viewBox `0 -960 960 960`, path: `m380-300 280-180-280-180v360ZM160-160q-33 0-56.5-23.5T80-240v-480q0-33 23.5-56.5T160-800h640q33 0 56.5 23.5T880-720v480q0 33-23.5 56.5T800-160H160Zm0-80h640v-480H160v480Z` | Icono de la opción "Vídeo". |
| `--m3-ease` | `cubic-bezier(.2, 0, 0, 1)` | Curva "emphasized" de Material 3. |

### 1.7 Otras locales

- `--md-av-icon` (theme.css:492-493): en `.song-button` = `var(--m3-headphones)`, en
  `.video-button` = `var(--m3-video)`.
- Variables de YTM sobrescritas en el botón de reproducir de las tarjetas (theme.css:543-548) y
  en la barra de progreso (theme.css:837-841): ver sus secciones.

---

## 2. Zonas de la interfaz (en el orden del archivo)

### 2.1 Barras de scroll (theme.css:27-50)

Afecta a todas las barras de desplazamiento de la ventana.

- `html { scrollbar-color: auto }` (l.30): YTM pone `scrollbar-color` en `html` (se hereda) y eso
  anula los estilos `::-webkit-scrollbar`; se devuelve a `auto`. Las barras que YTM oculta a
  propósito llevan su propio `scrollbar-color: transparent` y siguen ocultas.
- Barra (l.31): **10 px de ancho** (vertical), **8 px de alto** (horizontal), fondo transparente.
- Sin flechas (l.32): `::-webkit-scrollbar-button` oculto, 0×0.
- Carril y esquina transparentes (l.33).
- Pulgar (l.34-39): fondo `--md-state-press` = `rgba(255,255,255,.2)`, borde de 2 px transparente
  con `background-clip: padding-box` (el pulgar visible queda 4 px más estrecho que la barra:
  6 px visibles en la vertical, 4 px en la horizontal), radio 8 px.
- Pulgar hover (l.40): `--md-text-off` = `rgba(255,255,255,.38)`.
- **Carril de la barra de la página (l.41-50):** la barra de scroll de la ventana empieza 64 px
  por debajo de arriba (bajo la barra superior) y termina 92 px antes del borde inferior (encima de
  la barra del reproductor) — antes subía hasta detrás de los botones minimizar/maximizar/cerrar.
  Técnica: `::-webkit-scrollbar-track-piece:vertical:start { margin-top: 64px }` y
  `:vertical:end { margin-bottom: 92px }`, en general; y se anula (`margin: 0`) para todas las
  barras internas (`body *::-webkit-scrollbar-track-piece…`): cola, barra lateral, etc. El autor
  explica que solo funciona con margen en los track-piece y con un selector sin elemento
  (`html::` no llega a la barra de la ventana).

**Réplica:** scroll principal con barra fina (pulgar de 6 px visible, blanco 20 %, 38 % en hover,
extremos redondeados), sin flechas, confinada verticalmente entre y=64 y alto−92. Barras internas:
mismo estilo, carril completo.

### 2.2 Tipografía de cabeceras de estantería (theme.css:52-61)

Comentario: se mantienen las fuentes de YTM, no se fuerza ninguna; solo se ajustan tamaños.

- `ytmusic-carousel-shelf-basic-header-renderer` = cabecera de cada "estantería" (fila horizontal
  de tarjetas en Inicio/Explorar/artista: p. ej. "Escuchar de nuevo", "Mixes para ti").
  - Título (`h2 .title`, `.title yt-formatted-string`): **24 px** (familia y peso los de YTM:
    YouTube Sans; el peso no lo fija el tema).
  - `.strapline` (la línea pequeña encima del título, p. ej. el nombre del usuario o "MÁS DE…"):
    color `--md-text-2` = `rgba(255,255,255,.7)`, `letter-spacing: 0.04em`.

(l.63-64) Comentario: fondos de página sin tocar: barra superior, barra lateral, Inicio y los
degradados según la carátula en álbumes/listas/artistas quedan exactamente como en YTM.

### 2.3 Barra lateral (guía): entradas (theme.css:66-79)

`ytmusic-guide-entry-renderer` = cada fila del menú lateral izquierdo (Inicio, Explorar,
Biblioteca, y cada playlist). `tp-yt-paper-item` es su caja clicable; `[active]` marca la página
actual; `.guide-icon` el icono; `.title` el texto.

- Fila: radio **6 px**; transición `background-color .12s linear`.
- Activa: fondo `--md-state-on` = `rgba(255,255,255,.14)`; icono `#fff`.
- No activa: icono y título en `--md-text-2` = `rgba(255,255,255,.7)`.
- No activa + hover: fondo `--md-state-hover` = `rgba(255,255,255,.08)`.
- (Título activo: no lo fija el tema; hereda `--ytmusic-text-primary` = `#fff` vía YTM, presumiblemente.)

### 2.4 Barra lateral de ancho ajustable y plegado (theme.css:81-178)

Comentario (l.81-87): ancho ajustable como Spotify. YTM dimensiona la barra y todo lo que depende
de ella (margen del contenido, barra superior, Now Playing) con `--ytmusic-guide-width` (240 px
abierta). Aquí sale de `--md-left-eff` (240–420 px, ver 1.2). Plegada (72 px), oculta o en
ventanas estrechas se deja como YTM.

Todo lo siguiente va dentro de **`@media (min-width: 936px)`** (l.91-171) salvo l.178.

Estructura DOM: `ytmusic-app` es la raíz de la app; atributo `[guide-collapsed]` = barra plegada;
`[is-hidden-guide-page]` = página sin barra. `tp-yt-app-drawer#guide` = cajón/panel lateral;
dentro, `ytmusic-guide-renderer` (contenido), `#sections`, `ytmusic-guide-section-renderer`
(secciones: la primaria `[is-primary]` = Inicio/Explorar/Biblioteca; la otra = playlists), con
`#items` (filas), `#buttons` (botón "Nueva lista") y `#divider`. `#mini-guide` = la columna
estrecha de 72 px que YTM usa cuando está plegada.

- l.92-94: con barra abierta y visible, `--ytmusic-guide-width = --md-left-eff`.
- l.95-100: `#sections` y cada sección al **100 %** del panel (YTM trae anchos fijos 240, 224,
  200, 192 px… y al ensanchar quedaba una franja negra).
- l.101-103: `ytmusic-guide-renderer` con `width: var(--md-guide-w, 100%)` — durante plegar/
  desplegar preload.js anima esta variable para que las filas se estrechen de verdad (no solo
  se recorten).
- l.104-109: `#items`, `#buttons`, `#divider` y `tp-yt-paper-item` → `width: auto`.
- l.110-115: cada entrada al 100 % y `box-sizing: border-box` (si no, `#items`, que es flex, las
  encogía a su contenido y las ponía de dos en dos).
- l.116-120: botón "Nueva lista" (`yt-button-renderer`, `yt-button-shape`, `button`) al 100 %.
- **Plegada (l.122-125):** se oculta `#mini-guide` y `#mini-guide-background` (`display:none`).
  En vez de la columna aparte de YTM (que ponía el texto bajo los iconos y hacía saltar todo al
  desplegar), se usa el mismo panel estrechado: cada fila mantiene su altura y solo se ocultan los
  textos.
- l.126-128: `.guide-nav` oculto: por debajo de ~1300 px YTM convierte la barra en panel flotante
  con cabecera propia (☰ y logo) que aparecía dentro de la tarjeta.
- l.129: el cajón siempre `visibility: visible`.
- l.130-133: `#contentContainer` sin `transform` y con `overflow: hidden`.
- l.134-137: `.title-column` (columna del texto) `min-width: 0; overflow: hidden`.
- l.138-139: `.guide-icon` con `flex: none` (no se encoge; se quedaba a 0 px).
- l.140-143: plegada (y no cerrándose): `.title-column` y `#play-button` (botón ▶ que aparece en
  las filas de playlist) con `opacity: 0` y `pointer-events: none`.
- **Geometría de iconos (l.144-148):** las filas van de x=16 px al borde de la tarjeta − 8
  (plegada: de 16 a 64). Iconos, portadas y el "+" en el mismo eje plegada o no, centrados en los
  72 px (centro x=40), así no se mueven. Icono 24 px a 12 px del borde de la fila
  (`padding-left: 12px` en la sección primaria); portada 40 px a 4 px
  (`padding-left: 4px` en filas `[data-md-id]`, es decir, playlists con portada).
  Comprobación: 16 + 12 + 12 = 40; 16 + 4 + 20 = 40.
- l.149-150: plegada o en movimiento (`#guide-wrapper[data-md-guide-moving]`, lo pone preload.js),
  `#items` sin barra de scroll (`scrollbar-width: none`).
- **"Nueva lista" (l.151-170):** la píldora se encoge hasta un botón redondo con su "+" que no se
  mueve (queda a la izquierda). `#buttons` con padding horizontal 10 px. Botón: `overflow: hidden`,
  `justify-content: flex-start`, `padding-left: 6px`; icono (`.ytSpecButtonShapeNextIcon`) con
  `margin-left: 0`; texto (`.ytSpecButtonShapeNextButtonTextContent`) `white-space: nowrap`.
  Plegada: contenedores `width: auto`, botón **36 px de ancho**, `padding: 0 6px`, icono sin
  margen, texto `display: none`. Con 10 px a cada lado, a 64 px de ancho la píldora mide 36 y ya
  es círculo (de x=22 a 58; centro 40). Mismo alto que la píldora (un círculo más alto empujaría lo
  de debajo). El alto exacto lo da YTM (no lo fija el tema; habitualmente 36 px).
- **Animación de plegado (l.172-178), comentario:** técnica FLIP en preload.js: la página pasa
  a su sitio nuevo de golpe y lo que se movió vuelve con `transform` a su posición anterior y llega
  con un muelle (spring); el panel se recorta con la forma de la tarjeta y las filas
  (`--md-guide-w`) se estrechan con él. Al plegar (`html[data-md-guide-closing]`) el panel sigue
  ancho (su ancho y las reglas de "plegada" esperan) mientras el recorte y las filas llegan a
  72 px; entonces se pliega de verdad y se ve igual, sin salto. l.178: durante el cierre,
  `--ytmusic-guide-width` del cajón se mantiene en `--md-left-eff`. (Duración/curva del muelle: en
  preload.js, no en este archivo.)

### 2.5 Barra lateral como tarjeta flotante (theme.css:180-231)

Comentario: la barra lateral es una tarjeta redondeada que flota sobre el fondo de la página (que
pasa por detrás), separada de la barra superior y de la del reproductor. La tarjeta es un
pseudoelemento detrás del contenido: sigue el ancho de la barra sin tocar elementos de YTM.
`z-index: -1` dentro del contexto de `#guide-wrapper` (que es `position: fixed`): sobre su fondo,
bajo sus filas.

- `#guide-wrapper` (contenedor fijo de la barra lateral; l.185-188): fondo transparente, sin
  borde derecho.
- `@media (min-width: 936px)` (l.189-191): `#mini-guide-background` transparente y sin borde
  derecho (su fondo negro tapaba la página).
- **Tarjeta `#guide-wrapper::before` (l.194-205):**
  - `position: absolute; z-index: -1`
  - `top: calc(64px + 8px)` = **72 px**
  - `height: calc(100vh - 64px - var(--ytmusic-player-bar-height, 92px) - 16px)` = alto ventana −
    172 px. Alto desde la ventana y no desde `#guide-wrapper` porque en pantalla completa YTM lo
    hace más alto que la pantalla.
  - `left: 8px; right: 0`
  - radio **10 px**, fondo `--md-glass` = `rgba(12,12,12,.6)`, borde 1 px `--md-line` =
    `rgba(255,255,255,.08)`.
  - Es decir: tarjeta a 8 px del borde izquierdo de la ventana, 8 px bajo la barra superior,
    8 px sobre la del reproductor, y su borde derecho coincide con el borde derecho de la barra
    lateral (no hay hueco a la derecha dentro del wrapper; el hueco entre tarjeta y contenido lo da
    el contenido de YTM).
- `#guide-content` (l.210-213): `padding-left: 8px`, `box-sizing: border-box`.
- `ytmusic-guide-renderer` (l.214): `padding-top: calc(8px + 8px)` = **16 px** (los 8 de YTM +
  8). Como su alto es fijo, la lista de playlists sigue terminando en el borde inferior de la
  tarjeta.
- **Desvanecido inferior (l.215-222):** la lista de playlists (`ytmusic-guide-section-renderer:
  not([is-primary]) #items`) y el panel derecho de Now Playing (`ytmusic-player-page #side-panel
  #tab-renderer`, que contiene la cola, letra y relacionados) llevan
  `mask-image: linear-gradient(#000 calc(100% - 24px), transparent)`: los últimos **24 px** se
  desvanecen a transparente en vez de cortarse. La máscara va en el contenedor que se desplaza (el
  contenido pasa por debajo sin repintarla).
- **Pantalla completa** (`html[data-md-fullscreen]`, preload.js; l.223-226): sin barra de
  reproducción; tarjeta `height: calc(100vh - 64px - 16px)` (llega a 8 px del borde inferior) y
  `ytmusic-guide-renderer` con `padding-bottom: 8px` (YTM dejaba 100 px de hueco).
- `#nav-bar-divider` (l.227): línea divisoria bajo la barra superior → borde superior transparente.
- `#nav-bar-background` (l.228): fondo de la barra superior → borde inferior transparente.
  (El fondo en sí se deja como YTM.)
- l.229-231: en Now Playing (`ytmusic-app-layout[player-page-open]`), `#nav-bar-background` hijo
  directo del layout → `background: transparent`: el fondo de la carátula pasa por detrás de la
  barra superior (como Apple Music). Se usa fondo y no opacidad porque YTM anima la opacidad.

### 2.6 Tiradores de redimensionado (theme.css:233-267)

Elementos creados por preload.js (`.md-resize`, `.md-resize-left`, `.md-resize-right`,
`.md-resize-cover`), colgados de `<body>`.

- `.md-resize` (l.237-249): oculto por defecto; `position: fixed`; `top: 64px; bottom: 92px`
  (fijos porque desde `<body>` no ven las variables de `ytmusic-app`; con los 72 px de YTM se
  metía 20 px en la barra del reproductor); **8 px de ancho**; `z-index: 5`;
  `cursor: col-resize`; `touch-action: none`; `outline: none`. **Sin línea visible** ni al pasar
  ni al arrastrar (rompía la inmersión): solo cambia el cursor.
- Izquierdo (l.250-253), solo con `html[data-md-guide-open]`: visible, `left: calc(--md-left-eff − 4px)`
  (centrado sobre el borde derecho de la barra lateral).
- Derecho (l.254-257), solo con `html[data-md-queue-open]`: visible,
  `right: calc(var(--md-right-w, clamp(340px, 28vw, 440px)) - 4px)` (centrado sobre el borde
  izquierdo del panel de cola).
- `.md-resize-cover` (l.258-267): durante el arrastre, capa fija a toda la ventana (`inset: 0`),
  `z-index: 7`, `cursor: col-resize`, `user-select: none` (evita seleccionar texto sin recalcular
  el estilo de la página).

### 2.7 Portadas de playlists en la barra lateral (theme.css:269-286)

Las añade preload.js: atributo `data-md-id` en la fila e `img.md-cover` dentro (como Spotify).

- Fila: `display: flex; align-items: center; gap: 12px`.
- Portada: `flex: none`, **40×40 px**, radio **4 px**, `object-fit: cover`, fondo de relleno
  mientras carga `--md-state-on` = `rgba(255,255,255,.14)`.
- `.title-column`: `min-width: 0; flex: 1 1 auto` (texto se trunca).

### 2.8 `#mini-guide` (columna plegada de YTM) (theme.css:287-339)

Comentario: en la columna plegada YTM solo deja Inicio, Explorar y Biblioteca; aquí vuelven las
playlists solo con portada (como Spotify), en su columna con scroll, y "Nueva lista" como botón
redondo. El título sigue para lectores de pantalla. **Nota:** a ≥ 936 px `#mini-guide` está
oculto (l.125), así que estas reglas solo se ven por debajo de 936 px (si YTM muestra la mini
guía a ese ancho).

- `#mini-guide #items` (l.290): `overflow-x: hidden` (1 px de más sacaba una barra horizontal).
- Sección de playlists (l.291-297): flex en columna, `align-items: center`, `flex: 1 1 auto`,
  `min-height: 0`.
- Su `#buttons` (l.298-302): flex, centrado, `padding: 8px 0`.
- Botón "Nueva lista" (l.303-312): **40×40 px**, `min-width: 0`, `padding: 0`, **círculo**
  (radio 50 %), contenido centrado; icono sin margen; texto oculto.
- `#items` de playlists (l.313-320): `flex: 1 1 auto; min-height: 0`; **72 px** de ancho;
  `height: auto`; `overflow-y: auto`; sin barra de scroll visible.
- Fila de playlist (l.321-325): `display: block`, **56 px** de ancho, centrada (`margin: 0 auto`).
- Su caja (l.326-330): centrada, `padding: 6px 0`, `min-height: 0` (→ fila de 52 px: 40 + 12).
- Portada (l.331): `display: block`.
- Título (l.332-338): visualmente oculto (absoluto, 1×1 px, `overflow: hidden`,
  `clip-path: inset(50%)`), accesible.
- `#play-button` (l.339): `display: none`.

### 2.9 Botón "Nueva lista" (theme.css:341-356)

Selector: `ytmusic-guide-section-renderer yt-button-renderer button` y
`ytmusic-guide-renderer #buttons button` (el botón píldora "+ Nueva lista" de la barra lateral).

- Reposo: fondo `--md-state-on` = `rgba(255,255,255,.14)` (en vez del gris fijo de YTM);
  transición `background-color .15s linear`.
- Hover: `--md-state-press` = `rgba(255,255,255,.2)` (el fondo forzado anulaba el hover de YTM).
- Active: `--md-state-press` = `rgba(255,255,255,.2)` (igual que hover).

### 2.10 Buscador (theme.css:358-446)

`ytmusic-search-box` = el campo de búsqueda de la barra superior. `#input-box` / `.search-box` =
la píldora con la lupa y el input; `.search-container` = contenedor interno; `#suggestion-list` =
desplegable de sugerencias; `[opened]` = con el foco/abierto.

**Cerrado (l.359-367):**
- Píldora: fondo `--md-field` = `rgba(255,255,255,.08)`, borde 1 px transparente, radio
  **999 px** (píldora completa).
- `ytmusic-search-box` con `border-color: transparent` (con texto escrito y cerrado, YTM dibujaba
  un borde blanco de esquinas rectas alrededor).

**Centrado, `@media (min-width: 820px)` (l.368-404).** Comentario: buscador centrado en la ventana
como Spotify. YTM tiene dos maquetaciones de la barra según el ancho; no se toca el contenedor: se
hace estático y el buscador se posiciona respecto a la barra superior (fija, empieza en x=0). Solo
con el buscador completo; en estrecho YTM lo convierte en un icono con panel propio y no se toca.
Hueco libre: a la izquierda barra lateral + botón atrás (`--md-nav-l`); a la derecha lo que mide
preload.js (`--md-nav-r`). Va centrado mientras quepa; si no, se desliza dentro del hueco y nunca
pisa a nadie.
- `ytmusic-nav-bar .center-content`: `position: static; transform: none` (YTM lo desplaza con
  translateX y eso lo convertiría en referencia de posición).
- Variables: ver 1.4.
- `ytmusic-search-box`: `position: absolute; top: 50%; translate: 0 -50%` (centrado vertical en
  la barra de 64 px), `right: auto`, `width: --md-search-w`, `max-width: none`, `margin: 0`, y
  **`left: clamp(--md-nav-l, 50vw − --md-search-w/2, 100vw − --md-nav-r − --md-search-w)`**.
  Fórmula para la réplica:
  `w = clamp(160, ventana − (max(barra,240)+70) − derecha, 480)`;
  `x = clamp((max(barra,240)+70), ventana/2 − w/2, ventana − derecha − w)`.
- `input` con `text-overflow: ellipsis` (el texto de ayuda acaba en "…").

**Abierto (l.406-446).** Comentario: una sola superficie redondeada sin el marco rectangular de
1 px de YTM. Como Spotify: la barra sigue siendo píldora y los resultados van en una tarjeta aparte
con todas las esquinas redondeadas y 8 px de separación.
- `ytmusic-search-box[opened]`: sin borde, radio 24 px, fondo transparente.
- `.search-container`: radio 24 px, fondo transparente, sin sombra.
- Píldora (`.search-box`, `#input-box`): sin borde, radio 999 px, fondo **`--md-raised` = `#1e1e1e`**.
- `#suggestion-list`: separada **8 px** de la píldora (`margin-top`), `padding: 8px 0` (top y
  bottom 8 px), radio **16 px**, fondo `#1e1e1e`, borde 1 px `--md-raised-line` =
  `rgba(255,255,255,.1)` (nota: hay un `border: none` y luego `border: 1px solid …`; gana el
  segundo), sombra `0 16px 40px rgba(0,0,0,.5)`, `overflow: hidden`.
- Secciones y sugerencias (`ytmusic-search-suggestions-section`, `ytmusic-search-suggestion`):
  fondo transparente.
- Sugerencia hover o `[selected]` (seleccionada con teclado): fondo `rgba(255,255,255,.08)`.
- (Alto de la píldora, tamaño de texto e iconos: los de YTM; el tema no los fija.)

### 2.11 Chips (theme.css:448-481)

`ytmusic-chip-cloud-chip-renderer` = cada "chip" de filtro (p. ej. "Relax", "Energía" en Inicio, o
"Canciones / Álbumes / Artistas" en búsqueda / biblioteca). Comentario: como las píldoras de la
app móvil: totalmente redondeadas, fondo blanco translúcido, sin contorno; la seleccionada es
blanca con texto oscuro.

- `.gradient-box` (envoltorio): transparente, sin borde, radio 999 px, `padding: 0`.
- Enlace `a` (la píldora): `display: inline-flex; align-items: center; box-sizing: border-box`;
  **alto 36 px**; `padding: 0 16px`; radio 999 px; sin borde; fondo `rgba(255,255,255,.14)`;
  texto `#fff`; transición `background-color .15s linear`.
- Texto (`a` y todos sus hijos): familia `--m3-font` (Roboto…), **14 px, peso 500, interlineado
  20 px, letter-spacing .1px**.
- Hover y active: fondo `rgba(255,255,255,.2)`.
- Seleccionada (`[is-selected]`): fondo `#fff`, texto `#000`. (El hover sobre la seleccionada:
  la regla de selección va después con mayor especificidad → sigue blanca.)

### 2.12 Selector Canción/Vídeo en Now Playing (theme.css:482-493)

`ytmusic-player-page` = la página "Now Playing" (reproductor a pantalla). `ytmusic-av-toggle
.av-toggle` = el conmutador "Canción | Vídeo" arriba de la carátula; `.song-button` y
`.video-button` sus dos botones.

- Cada botón lleva delante (`::before`) un icono **22×22 px**, `flex: none`, pintado en
  `currentColor` (el color del texto del botón) mediante máscara SVG centrada, `contain`,
  sin repetición. Canción → auriculares (`--m3-headphones`), Vídeo → pantalla con play
  (`--m3-video`). Comentario: "cada opción lleva su icono en vez de la marca" (sustituye la marca
  de verificación de YTM; el CSS mostrado solo añade el icono — si la marca se oculta, no es en
  este tramo).

### 2.13 Resultado principal de búsqueda (theme.css:495-501)

`ytmusic-card-shelf-renderer` = la tarjeta grande "Resultado principal" arriba de los resultados.

- `#undercards` (columna "Más de YouTube" con vídeos que casi nunca tienen que ver):
  `display: none`.
- `.card-container`: `grid-template-columns: 1fr`, `grid-template-areas: "card-content-container"`
  → la tarjeta de la canción ocupa todo el ancho.

### 2.14 Tarjetas de estantería (theme.css:503-562)

`ytmusic-two-row-item-renderer` = cada tarjeta de un carrusel/cuadrícula: carátula arriba y dos
líneas de texto debajo (título, subtítulo). `.image-wrapper` / `ytmusic-thumbnail-renderer` / `img`
= la carátula. `ytmusic-carousel ul#items` = la lista horizontal desplazable.

- Tarjeta (l.507-512): `padding: 8px`, `margin: 0 -8px` (el relleno solo existe para el fondo del
  hover; los márgenes negativos lo compensan: las carátulas siguen a **24 px** entre sí, el espacio
  de YTM, y la primera alineada con el título de la sección); radio **10 px**; transición
  `background-color .15s linear`.
- Lista del carrusel (l.513-520): `box-sizing: content-box; padding: 0 8px`. Por el margen
  negativo, la primera tarjeta empezaba 8 px antes que la lista y su fondo se cortaba; YTM la
  centra con `margin: 0 auto`, así que con content-box crece 8 px por lado y sigue centrada: las
  carátulas no se mueven y el fondo de la primera y la última tiene sitio.
- Hover (l.521-523): fondo `rgba(255,255,255,.08)`.
- Carátula (l.524-528): radio **8 px**.
- Carátula cuadrada (`[aspect-ratio="MUSIC_TWO_ROW_ITEM_THUMBNAIL_ASPECT_RATIO_SQUARE"]`, álbumes,
  playlists) (l.529-531): sombra `0 8px 20px rgba(0,0,0,.45)` (sin `!important`).
- Artistas (`[is-circular]`) (l.532-536): carátula circular (radio 50 %).
- **Botón reproducir sobre la carátula (l.538-562), como Spotify:**
  - `ytmusic-item-thumbnail-overlay-renderer` (capa oscura sobre la carátula al pasar): fondo
    transparente (se elimina el oscurecido).
  - `ytmusic-play-button-renderer`: variables de YTM → fondo `#fff`, icono `#000`, icono cargando
    `#000`, **tamaño de icono 24 px** (YTM dibujaba el icono del tamaño del círculo; con el
    círculo blanco el triángulo quedaba enorme: icono a la mitad, centrado). Fondo `#fff`, color
    `#000`, círculo (50 %), sombra `0 6px 14px rgba(0,0,0,.5)`. El diámetro del círculo lo da YTM
    (no se fija aquí; deducido del comentario: ~48 px).
  - Reposo: `opacity: 0; transform: translateY(8px)`.
  - Visible (hover en la tarjeta, o `[state="playing"]` / `[state="paused"]` = esa tarjeta es lo que
    suena): `opacity: 1; transform: none`.
  - Transición: `opacity .18s ease, transform .18s ease` → sube 8 px y aparece.
  - Posición en la carátula: la de YTM (habitualmente abajo a la derecha); no se cambia aquí.

### 2.15 Filas de canciones (theme.css:564-574)

`ytmusic-responsive-list-item-renderer` = cada fila de una lista de canciones (álbum, playlist,
resultados, "canciones" de artista).

- Fila: radio **6 px**.
- Hover: fondo `rgba(255,255,255,.08)`. (Sin transición declarada.)
- Miniatura (`ytmusic-thumbnail-renderer`, `img`): radio **4 px**.

### 2.16 Barra del reproductor (theme.css:576-946)

`ytmusic-app-layout` = layout raíz; `[player-visible]` lo pone YTM al reproducir por primera vez;
`[slot="player-bar"]` = `ytmusic-player-bar`, la barra inferior; `#player-bar-background` = capa
de fondo debajo de ella. Dentro de la barra, YTM tiene: `.middle-controls` (carátula + título +
artista; aquí se coloca a la izquierda), `#left-controls` (anterior/play/siguiente; aquí al
centro), `#progress-bar` (barra de progreso) y `#right-controls` (volumen, repetir, aleatorio,
abrir Now Playing). Comentario del autor (l.576-578): distribución de Spotify — izquierda carátula
+ título; centro controles con progreso debajo; derecha volumen, repetir, aleatorio. Solo se
reordena el grid de YTM.

**Visible siempre (l.579-600):**
- Sin `[player-visible]`, barra y fondo con `transform: translateZ(0); visibility: visible`: la
  barra se ve aunque no suene nada (vacía).
- `html[data-md-np-open]` (preload.js: Now Playing abierto): `#player-bar-background` con fondo
  `--md-glass-bar` = `rgba(12,12,12,.72)` (vidrio que deja pasar el color de la carátula). En el
  resto de páginas el fondo es opaco `--md-bar` = **`#0e0e0e`** (vía
  `--ytmusic-player-bar-background`, l.21), porque las filas que pasan por debajo se leían a
  través de los controles. Se pone en el elemento y no con variable en `<html>` para no recalcular
  toda la página.
- Sin `[player-visible]`: `#content` con `margin-bottom: var(--ytmusic-player-bar-height)` (92 px).
- Sin canción: `#left-controls`, `#right-controls` y `#progress-bar` a **opacidad .35** y sin
  clics ("sin canción no hay nada que controlar").

**Altura (l.602-605):** `--ytmusic-player-bar-height: 92px` en `ytmusic-app` y
`ytmusic-app-layout` (YTM: 72 px), para que botones y progreso no se pisen.

**Grid (l.606-628).** Comentario con el esquema:
```
[carátula + título] | [botones + ⋮ ]   | [me gusta · volumen · repetir · aleatorio · ▲]
                    | [0:00 ━━━━ 3:30] |
```
Flujo real, sin posiciones calculadas, para que nada se pise al redimensionar. Columna central de
ancho fijo proporcional; laterales `minmax(0, 1fr)` (pueden encoger sin empujar al centro).
preload.js mueve la píldora "me gusta" al bloque derecho y el menú "⋮" a la columna central.
- `--md-center-w: clamp(260px, 38vw, 600px)`.
- `height: 92px`; fondo transparente (lo pinta `#player-bar-background`; con los dos el vidrio se
  doblaba); `border-top: 1px solid rgba(255,255,255,.08)`; `box-sizing: border-box`.
- `grid-template-columns: minmax(0,1fr) var(--md-center-w) minmax(0,1fr)`.
- `grid-template-rows: auto auto`.
- `grid-template-areas: "middle start end" "middle progress end"` (área `middle` =
  `.middle-controls` a la izquierda; `start` = `#left-controls`, controles, centro arriba;
  `progress` centro abajo; `end` = `#right-controls`).
- `align-content: center`; `column-gap: 24px`; `row-gap: 2px`; `padding: 0 16px`.
- `.middle-controls` y `#right-controls`: `grid-row: 1 / span 2`, `min-width: 0`.

**Columna central (l.634-642, 679-683):** `#left-controls` es grid `1fr auto 1fr`,
`align-items: center`, `width: auto`, `padding: 0`. `.left-controls-buttons` (anterior, play,
siguiente) en la columna 2 (centrada); el menú "⋮" (`ytmusic-menu-renderer`, movido por preload.js)
en la columna 3, `justify-self: end`, `margin: 0` → al final de la fila.

**Detalles de disposición (l.684-699):**
- Carátula de la barra (`.thumbnail-image-wrapper`): `margin: 0 12px 0 0` (12 px al texto).
- `.content-info-wrapper` (título + artista): `margin-left: 0`.
- `.right-controls-buttons` y `.left-controls-buttons`: `gap: 4px`.
- `.middle-controls`: `justify-content: flex-start`.
- `#right-controls`: `justify-content: flex-end; gap: 8px`.
- (Tamaño de la carátula: el de YTM; no lo fija el tema.)

**Volumen flotante (l.700-729).** preload.js crea `.md-volume` (envoltorio del botón de volumen y
su deslizador `#volume-slider`).
- `.md-volume`: `position: relative; display: inline-flex`.
- `#volume-slider`: panel flotante encima del botón: `position: absolute`,
  `bottom: calc(100% + 12px)` (12 px por encima del botón), centrado horizontal
  (`left: 50%; transform: translateX(-50%)`), **140×40 px**, `margin: 0`, `padding: 0 14px`,
  border-box, fondo `#1e1e1e`, radio **12 px**, borde 1 px `rgba(255,255,255,.1)`, sombra
  `0 16px 40px rgba(0,0,0,.5)`, `z-index: 3`.
- `::after`: puente invisible de 12 px de alto bajo el panel (de su borde inferior al botón), para
  no perder el hover al subir del botón al panel.
- Cuándo se muestra/oculta: no está en este tramo (lo gestiona YTM por hover o preload.js/otra
  parte del CSS).

**Alineación vertical (l.730-752):**
- `.left-controls-buttons`, `.middle-controls`, `.middle-controls-buttons`, `#right-controls`,
  `.right-controls-buttons`: `align-items: center`.
- `.middle-controls`, `#right-controls`: `align-self: center`.
- Hijos directos de los grupos de botones: `margin-top/bottom: 0`.
- Botón abrir/cerrar Now Playing (`.toggle-player-page-button`, la flecha ▲): `margin: 0; top: 0`
  (traía un margen de 28 px de su tamaño original).

**Tiempos (l.753-755):** `.time-info` ("0:00 / 3:30" original) `display: none`; los tiempos van a
los lados de la barra de progreso y así los botones quedan centrados exactos sobre ella.

**Material 3 (l.756-767).** Comentario: misma posición de todo; solo cambia el estilo: colores de
la paleta, escala tipográfica en Roboto, botones de icono de 40 px con capas de estado, botón de
reproducir relleno, toggles tonales y slider con asa vertical. (El comentario dice que el botón de
reproducir "cambia de forma", pero en este tramo es siempre circular; ver más abajo.)
- `ytmusic-player-bar` y todos sus descendientes: `font-family: --m3-font`
  (Roboto, "Segoe UI Variable Text", system-ui, sans-serif).

**Tipografía (l.769-787):**
- Título de la canción (`.middle-controls .title`): **16 px, peso 500, interlineado 24 px,
  letter-spacing .15px, `#fff`** (title-medium).
- Artista/álbum (`.byline` y todo dentro): **14 px, interlineado 20 px, letter-spacing .25px,
  `rgba(255,255,255,.7)`** (body-medium; peso: el de YTM, no se fija).
- Carátula (`.thumbnail-image-wrapper` y `img`): radio **8 px** (forma "small" de M3).

**Botones de icono (l.789-807).** Selectores: todos los `yt-icon-button` salvo `#play-pause-button`
(y su `#button` interno), todos los `yt-button-shape button`, y los del menú "⋮".
- **40×40 px**, círculo, color del icono `rgba(255,255,255,.7)`, fondo transparente, transición
  `background-color .15s linear`. Icono 24 px (40 − 2×8 de padding).
- El padding de 8 px va solo en el botón interior (`#button` / `button`); el contenedor
  `yt-icon-button` lleva `padding: 0` (si no, el botón bajaba 8 px).
- Anterior (`.previous-button`) y siguiente (`.next-button`): icono **`#fff`** (más prominentes
  que el resto).

**Estados de los botones de icono (l.808-832):** ver sección 3.

**Barra de progreso (l.834-929).** `#progress-bar` es un `tp-yt-paper-slider` de YTM. Comentario:
slider M3, pista de 4 px, asa vertical; deja 50 px a cada lado para los tiempos.
- Variables del slider: activo `#fff`; asa `#fff`; pista de fondo `--md-line-strong` =
  `rgba(255,255,255,.24)`; secundario (buffer) `--md-text-off` = `rgba(255,255,255,.38)`; alto
  `--paper-slider-height: 4px`.
- Caja: `grid-area: progress`; `position: relative` y `top/bottom/left: auto`, `width: auto`,
  **alto 24 px**, `transform: none`, `overflow: visible`, **`margin: 0 50px`**.
- `#sliderContainer`: `margin: 0; width: 100%` (sus márgenes internos desplazaban la pista a la
  derecha).
- `.bar-container`: `left: 0; right: 0; padding: 0`.
- `#sliderContainer` y `.bar-container`: alto 24 px.
- `#sliderBar`: `padding: 10px 0` (24 − 4 = 20 → pista de 4 px centrada), `width: 100%`.
- **Tiempos (l.869-896)** (preload.js pone `data-md-elapsed` y `data-md-total` en
  `#progress-bar`): pseudoelementos absolutos, centrados en vertical (`top: 50%;
  translateY(-50%)`), **44 px de ancho**, familia `--m3-font`, **12 px, peso 500,
  letter-spacing .5px, interlineado 1**, color `rgba(255,255,255,.7)`, cifras tabulares,
  sin clics. Transcurrido (`::before`): a la izquierda de la barra (`right: 100%`), 6 px de
  separación, alineado a la derecha. Total (`::after`): a la derecha (`left: 100%`), 6 px,
  alineado a la izquierda. (44 + 6 = 50 = el margen lateral.)
- Pista (`#progressContainer`): radio 2 px, `overflow: hidden` (el progreso usa `scaleX`, así que
  se recorta el contenedor para tener extremos redondeados).
- Progreso (`#primaryProgress`): `#fff`.
- **Asa (l.903-929):** comentario: asa M3, barra vertical 4×16 px con un "hueco" del color de la
  barra alrededor mediante box-shadow, siempre visible. **Ojo: en el código el box-shadow es
  `none`**, así que el "hueco" no se dibuja en este tramo (puede que otra regla posterior lo
  añada; no consta aquí). El contenedor del asa traía 34 px (fila original) y quedaba 5 px por
  debajo de la pista.
  - `.slider-knob`: flex centrado, alto 24 px, `top: 0`, `margin-top: 0`.
  - `.slider-knob-inner`: **4×16 px**, sin margen, sin borde, radio 2 px, fondo `#fff`, sin sombra,
    `transform: none`, transición `height .15s cubic-bezier(.2,0,0,1)`.
  - Hover en la barra o `[dragging]` (arrastrando): alto **20 px**.

**Reproducir/pausa (l.931-946)** (`#play-pause-button.ytmusic-player-bar`): botón relleno, siempre
circular.
- Contenedor: **44×44 px**, `padding: 0`, **`margin: 0 16px`** (16 px a anterior y siguiente).
- `#button`: 44×44, `padding: 10px` (icono 24 px), fondo **`#fff`**, círculo, icono **`#000`**,
  transición `box-shadow .15s linear` (no hay box-shadow definido en este tramo para animar; puede
  aparecer en estados de la parte 2).

**Me gusta / no me gusta (comentario l.948-952; las reglas empiezan en la 953, parte 2):** como en
la app móvil: una sola píldora tonal con una línea divisoria fina en medio. Al marcar no cambia el
fondo: YTM cambia el icono a relleno. La píldora sigue en su sitio del DOM (si se mueve, YTM
confunde sus clics con clics en la barra) y se dibuja junto al volumen mediante anclaje CSS: el
bloque derecho es el ancla y reserva el hueco de la píldora con padding. Ancho reservado
`--md-like-w` = 105 px (52 + 1 + 52).

---

## 3. Estados (resumen)

| Elemento | Reposo | Hover | Active/pressed | Seleccionado/activo | Otros |
|---|---|---|---|---|---|
| Pulgar de scroll (l.34-40) | `rgba(255,255,255,.2)` | `rgba(255,255,255,.38)` | — | — | — |
| Entrada de la barra lateral (l.67-79) | transparente; icono y texto `rgba(255,255,255,.7)` | `rgba(255,255,255,.08)` (solo no activas) | — | fondo `rgba(255,255,255,.14)`, icono `#fff` | — |
| Botón "Nueva lista" (l.343-356) | `rgba(255,255,255,.14)` | `rgba(255,255,255,.2)` | `rgba(255,255,255,.2)` | — | — |
| Sugerencia de búsqueda (l.443-446) | transparente | `rgba(255,255,255,.08)` | — | `[selected]` (teclado): `rgba(255,255,255,.08)` | — |
| Píldora del buscador (l.359-364, 422-427) | `rgba(255,255,255,.08)` | — | — | abierto: `#1e1e1e` | — |
| Chip (l.456-481) | `rgba(255,255,255,.14)`, texto `#fff` | `rgba(255,255,255,.2)` | `rgba(255,255,255,.2)` | `#fff`, texto `#000` | — |
| Tarjeta de estantería (l.507-523) | transparente | `rgba(255,255,255,.08)` | — | — | — |
| Botón ▶ de tarjeta (l.553-562) | opacidad 0, +8 px abajo | opacidad 1, en su sitio | — | sonando/pausado: visible siempre | — |
| Fila de canción (l.565-570) | transparente | `rgba(255,255,255,.08)` | — | — | — |
| Botón de icono de la barra (l.790-822) | transparente, icono `rgba(255,255,255,.7)` (anterior/siguiente `#fff`) | `rgba(255,255,255,.08)` | `rgba(255,255,255,.14)` | Toggle activo (repetir con modo ≠ NONE, o cualquier `button[aria-pressed="true"]` como "me gusta" marcado): fondo `rgba(255,255,255,.14)`, icono `#fff` | — |
| Botones de la barra con teclado (l.824-827) | — | — | — | — | `:focus-visible`: contorno 2 px `#fff`, separado 2 px |
| Ripple de YTM en la barra (l.829-832) | color `#fff`, opacidad .5 | | | | |
| Asa del progreso (l.915-929) | 4×16 px | 4×20 px (hover en la barra) | 4×20 px (`[dragging]`) | — | — |
| Controles sin canción (l.595-600) | — | — | — | — | Deshabilitado: opacidad .35, sin clics |

Notas:
- Repetir activo: `ytmusic-player-bar:not([repeat-mode="NONE"]) .repeat #button` (YTM pone
  `repeat-mode` = NONE / ALL / ONE en la barra).
- Comentario l.808: "capas de estado 8 % al pasar, 12 % al pulsar"; el valor real al pulsar es
  `--md-state-on` = **14 %**.
- No hay estados `focus` fuera de la barra del reproductor en este tramo.

## 4. Animaciones y transiciones

No hay `@keyframes` en las líneas 1–950.

| Elemento | Propiedad | Duración | Easing | Retardo | Línea |
|---|---|---|---|---|---|
| Entrada de la barra lateral | background-color | 120 ms | linear | 0 | 69 |
| Botón "Nueva lista" | background-color | 150 ms | linear | 0 | 346 |
| Chip | background-color | 150 ms | linear | 0 | 466 |
| Tarjeta de estantería | background-color | 150 ms | linear | 0 | 511 |
| Botón ▶ de tarjeta | opacity y transform (translateY 8px → 0) | 180 ms cada una | ease | 0 | 555 |
| Botones de icono de la barra | background-color | 150 ms | linear | 0 | 800 |
| Asa del progreso | height (16 → 20 px) | 150 ms | `cubic-bezier(.2, 0, 0, 1)` (`--m3-ease`) | 0 | 924 |
| Botón reproducir/pausa | box-shadow | 150 ms | linear | 0 | 945 |

Sin transición (cambio instantáneo): hover de filas de canciones (l.568), sugerencias del buscador
(l.443), pulgar del scroll.

Animación de plegado/desplegado de la barra lateral: FLIP + muelle hecho en preload.js
(comentario l.172-177); en este CSS solo están los estados de reposo y `data-md-guide-closing`.
Parámetros del muelle: no constan aquí.

## 5. Responsive

| Condición | Línea | Qué cambia |
|---|---|---|
| Siempre | 88-90 | `--md-left-eff` limita la barra lateral a `max(240, 50vw − 164)`. |
| `@media (min-width: 936px)` | 91-171 | Barra lateral ajustable (ancho = `--md-left-eff`), contenido al 100 %, plegado en el mismo panel (mini-guide oculta, textos a opacidad 0, "Nueva lista" a círculo de 36 px), `.guide-nav` oculto. Por debajo de 936 px la barra queda como YTM. |
| `@media (min-width: 936px)` | 189-191 | Fondo de `#mini-guide-background` transparente. |
| `@media (min-width: 820px)` | 378-404 | Buscador centrado en la ventana con ancho `clamp(160, hueco, 480)`. Por debajo: como YTM (icono con panel). |
| `@media (max-width: 900px)` | 646-652 | Barra del reproductor: `--md-center-w: clamp(260px, 36vw, 600px)`, `column-gap: 12px`. |
| `@media (max-width: 680px)` | 653-658 | `column-gap: 8px`, `padding: 0 12px`. |
| `@media (max-width: 615px)` | 662-669 | Minirreproductor de YTM: grid `minmax(0,1fr) auto`, una fila, áreas `"middle end"`; `#left-controls` y `#progress-bar` ocultos (el progreso sigue en Now Playing). Solo título y sus botones (reproducir, siguiente, `#right-controls-mweb`), como YTM. |
| `@media (max-width: 760px)` | 670-678 | `--md-like-w: 40px` (con selector `ytmusic-app ytmusic-player-bar`, más específico que la regla general de l.966); "no me gusta" (`yt-shape.dislike`) oculto; "me gusta" 40 px de ancho, `padding: 8px`. |
| `html[data-md-fullscreen]` | 225-226 | Tarjeta de la barra lateral hasta 8 px del borde inferior; `padding-bottom: 8px`. |

Notas: el comentario l.643-645 dice que por debajo de ~640 px YTM cambia a su minirreproductor
móvil (la media usa 615 px). Ventanas por debajo de ~1300 px: YTM convierte la barra lateral en
panel flotante (l.126); el tema lo neutraliza a ≥ 936 px. No hay `@container`.

Orden de las medias en el archivo: 900, 680, 615, 760 (la de 760 va después de la de 615).

## 6. Comentarios del autor (intenciones, resumidos)

1. (l.1-4) Mezcla Spotify + Apple Music + estructura YTM; rendimiento: sin backdrop-filter ni
   blur, transiciones en transform/opacity.
2. (l.12-13) Fuentes originales de YTM, porque sus cajas de alto fijo están medidas para ellas.
3. (l.17-20) Sin acento propio: el blanco es el acento; el color lo pone la carátula.
4. (l.27-29) Scrollbar: devolver `scrollbar-color` a auto para que funcione el estilo webkit.
5. (l.41-46) Carril de la barra de la página entre la barra superior y la del reproductor, para no
   meterse bajo los botones de la ventana.
6. (l.53, 63-64) No se tocan fuentes ni fondos de página.
7. (l.81-87) Barra lateral de ancho ajustable tipo Spotify; ancho máximo efectivo para no chocar
   con el buscador centrado.
8. (l.95-96, 110-111) Romper los anchos fijos de YTM para que las filas llenen el panel.
9. (l.122-124) Plegar en el mismo panel, sin columna aparte, para que nada salte.
10. (l.126-127) Neutralizar el modo "panel flotante" de YTM por debajo de 1300 px.
11. (l.144-146) Iconos y portadas centrados en x=40 tanto plegada como desplegada.
12. (l.151-153) "Nueva lista" se encoge a círculo sin moverse ni cambiar de alto.
13. (l.172-177) Animación FLIP con muelle al plegar/desplegar, sin saltos.
14. (l.180-184) La barra lateral es una tarjeta flotante con la página detrás.
15. (l.215-217) Desvanecido de 24 px al final de las listas en vez de corte seco.
16. (l.229-230) En Now Playing el fondo de la carátula sube detrás de la barra superior (Apple
    Music).
17. (l.233-236) Tiradores sin línea visible, solo cursor, para no romper la inmersión.
18. (l.287-289) Mini guía con portadas de playlists, como Spotify; títulos accesibles.
19. (l.368-377) Buscador centrado en la ventana que nunca pisa a nadie.
20. (l.406-407, 420-421) Buscador abierto: píldora + tarjeta de resultados separada 8 px (Spotify).
21. (l.448-449) Chips como la app móvil.
22. (l.495-496) Quitar "Más de YouTube" del resultado principal.
23. (l.504-506, 513-516) Fondo de hover en tarjetas sin alterar el espaciado de 24 px de YTM.
24. (l.546-547) Icono de play a la mitad del círculo.
25. (l.579-580, 594) La barra del reproductor existe siempre, apagada sin canción.
26. (l.586-589) Fondo opaco en páginas, vidrio en Now Playing.
27. (l.602, 606-612) Barra más alta (92) y grid con flujo real para que nada se pise.
28. (l.643-645) Ajustes de ancho estrecho.
29. (l.721) Puente invisible para no perder el hover del volumen.
30. (l.753-754) Tiempos a los lados del progreso.
31. (l.756-759) Estilo Material 3 en los controles sin mover nada.
32. (l.903-906) Asa vertical M3 centrada en la fila de 24 px.
33. (l.948-952) Píldora me gusta/no me gusta dibujada con anclaje CSS sin moverla del DOM.

---

## 7. Checklist de réplica

1. Fuentes: Roboto para texto, YouTube Sans para títulos; en la barra del reproductor y chips,
   Roboto → "Segoe UI Variable Text" → system-ui.
2. Sin color de acento: blanco `#fff` como acento; fondos de página como YTM.
3. Texto primario `#fff`, secundario `rgba(255,255,255,.7)`.
4. Barra de scroll vertical de 10 px (horizontal 8 px), pulgar `rgba(255,255,255,.2)` con 2 px de
   margen transparente (6 px visibles), radio 8 px; hover `rgba(255,255,255,.38)`; sin flechas;
   carril transparente.
5. El carril de la barra de scroll de la página va de y=64 a alto−92; las internas, completas.
6. Título de estantería 24 px; strapline `rgba(255,255,255,.7)`, letter-spacing 0.04em.
7. Fila de la barra lateral: radio 6 px; activa `rgba(255,255,255,.14)` con icono `#fff`; no
   activa icono y texto `rgba(255,255,255,.7)`, hover `rgba(255,255,255,.08)`; transición 120 ms
   lineal.
8. Ancho de la barra lateral ajustable 240–420 px, limitado a `max(240, ancho/2 − 164)`; plegada
   72 px.
9. Barra lateral = tarjeta: x=8, y=72, alto = ventana − 172 (pantalla completa: ventana − 80),
   borde derecho = borde de la barra, radio 10 px, fondo `rgba(12,12,12,.6)`, borde 1 px
   `rgba(255,255,255,.08)`.
10. Contenido de la barra con padding izquierdo 8 px y superior 16 px.
11. Iconos de la sección primaria a 12 px del borde de la fila (24 px), portadas a 4 px (40×40,
    radio 4, gap 12 px al texto, relleno `rgba(255,255,255,.14)` mientras cargan); centro de
    iconos/portadas en x=40 plegada o no.
12. Plegada: textos y botón ▶ de las filas a opacidad 0 y sin clics; filas a la misma altura.
13. "Nueva lista": píldora con padding lateral de sección 10 px, padding-left interno 6 px, fondo
    `rgba(255,255,255,.14)`, hover/active `rgba(255,255,255,.2)`, 150 ms lineal; plegada se
    convierte en círculo de 36 px con solo el "+" (de x=22 a 58).
14. Lista de playlists y panel derecho de Now Playing con desvanecido en los últimos 24 px.
15. Sin línea divisoria bajo la barra superior.
16. En Now Playing, fondo de la barra superior transparente (la carátula sube por detrás).
17. Tiradores de redimensionado invisibles de 8 px de ancho, de y=64 a alto−92, centrados en el
    borde del panel (±4 px), solo cambian el cursor a col-resize; durante el arrastre, capa global
    con col-resize y sin selección de texto.
18. Mini guía (si se usa < 936 px): columna de 72 px, filas de 56 px centradas con padding 6 px
    vertical, portadas 40 px, sin texto visible, sin scrollbar; "Nueva lista" círculo 40×40.
19. Buscador cerrado: píldora (radio 999) fondo `rgba(255,255,255,.08)`, sin borde visible.
20. Buscador (≥ 820 px): ancho `clamp(160, ventana − (max(barra,240)+70) − derecha, 480)`,
    x = `clamp(max(barra,240)+70, ventana/2 − w/2, ventana − derecha − w)`, centrado vertical en
    la barra de 64 px; placeholder con "…".
21. Buscador abierto: píldora `#1e1e1e`; sugerencias en tarjeta aparte 8 px por debajo, radio
    16 px, fondo `#1e1e1e`, borde 1 px `rgba(255,255,255,.1)`, sombra `0 16px 40px rgba(0,0,0,.5)`,
    padding vertical 8 px; sugerencia hover/seleccionada `rgba(255,255,255,.08)`.
22. Chip: alto 36 px, padding 0 16 px, radio 999, fondo `rgba(255,255,255,.14)`, texto `#fff`
    Roboto 14/20 peso 500 letter-spacing .1px; hover/active `rgba(255,255,255,.2)`; seleccionado
    fondo `#fff` texto `#000`; 150 ms lineal.
23. Selector Canción/Vídeo con icono 22×22 (auriculares / pantalla con play) del color del texto.
24. Resultado principal de búsqueda a todo el ancho, sin columna "Más de YouTube".
25. Tarjeta de estantería: padding 8 px compensado (carátulas a 24 px entre sí), radio 10 px,
    hover `rgba(255,255,255,.08)`, 150 ms lineal; la lista del carrusel con 8 px de padding
    lateral para que el hover de la primera/última no se corte.
26. Carátulas de tarjeta radio 8 px; cuadradas con sombra `0 8px 20px rgba(0,0,0,.45)`; artistas
    circulares.
27. Botón ▶ de tarjeta: círculo blanco, icono negro de 24 px, sombra `0 6px 14px rgba(0,0,0,.5)`,
    oculto y 8 px más abajo; al hover (o si es lo que suena/pausado) aparece y sube en 180 ms ease;
    sin oscurecer la carátula.
28. Fila de canción: radio 6 px, hover `rgba(255,255,255,.08)` (instantáneo); miniatura radio 4 px.
29. Barra del reproductor: **92 px de alto**, fondo **`#0e0e0e`** (en Now Playing
    `rgba(12,12,12,.72)`), borde superior 1 px `rgba(255,255,255,.08)`, padding horizontal 16 px.
30. Siempre visible; sin canción, controles centrales, derechos y progreso a opacidad .35 y
    deshabilitados.
31. Grid de 3 columnas: `minmax(0,1fr) | clamp(260, 38vw, 600) | minmax(0,1fr)`, column-gap 24,
    row-gap 2, contenido centrado verticalmente; izquierda (2 filas) carátula+título; centro arriba
    botones; centro abajo progreso; derecha (2 filas) botones alineados al final con gap 8 px.
32. Fila central de controles: grid `1fr auto 1fr`: anterior/play/siguiente en el centro (gap
    4 px), "⋮" pegado a la derecha.
33. Carátula de la barra con 12 px de separación al texto y radio 8 px.
34. Título de la barra Roboto 16/24 peso 500, letter-spacing .15px, `#fff`; artista 14/20,
    letter-spacing .25px, `rgba(255,255,255,.7)`.
35. Botones de icono de la barra: 40×40, círculo, padding 8 (icono 24), icono
    `rgba(255,255,255,.7)` (anterior/siguiente `#fff`), hover `rgba(255,255,255,.08)`, pulsado
    `rgba(255,255,255,.14)`, 150 ms lineal; grupos con gap 4 px.
36. Toggles activos (repetir ≠ ninguno, me gusta marcado): fondo `rgba(255,255,255,.14)`, icono
    `#fff`.
37. Foco de teclado en botones de la barra: contorno 2 px `#fff` con separación 2 px.
38. Reproducir/pausa: círculo blanco 44×44, icono negro 24 px (padding 10), 16 px de margen a cada
    lado.
39. Sin texto "0:00 / 3:30" junto a los controles.
40. Progreso: fila de 24 px, margen lateral 50 px; pista de 4 px, radio 2 px, fondo
    `rgba(255,255,255,.24)`, buffer `rgba(255,255,255,.38)`, progreso `#fff`.
41. Tiempos a los lados del progreso: cajas de 44 px a 6 px de la pista, Roboto 12 px peso 500,
    letter-spacing .5px, `rgba(255,255,255,.7)`, cifras tabulares; transcurrido alineado a la
    derecha (izquierda de la pista), total alineado a la izquierda (derecha de la pista).
42. Asa del progreso: barra vertical blanca 4×16 px radio 2, siempre visible, centrada en la
    pista; crece a 20 px al pasar o arrastrar en 150 ms `cubic-bezier(.2,0,0,1)`.
43. Volumen: panel flotante 140×40 px 12 px por encima del botón, centrado, padding lateral 14 px,
    fondo `#1e1e1e`, radio 12, borde 1 px `rgba(255,255,255,.1)`, sombra
    `0 16px 40px rgba(0,0,0,.5)`, con zona puente de 12 px para no perder el hover.
44. Ancho ≤ 900 px: columna central `clamp(260, 36vw, 600)`, column-gap 12.
45. Ancho ≤ 760 px: solo "me gusta" (40 px, padding 8), sin "no me gusta"; hueco reservado 40 px.
46. Ancho ≤ 680 px: column-gap 8, padding horizontal 12.
47. Ancho ≤ 615 px: barra de 2 columnas (título | botones), sin controles centrales ni progreso.
48. Hueco reservado para la píldora me gusta/no me gusta a la izquierda de los botones derechos:
    105 px + 8 px (detalles en la parte 2).

## Dudas / cosas no definidas en este tramo

- Tamaño de la carátula de la barra del reproductor, diámetro del botón ▶ de las tarjetas, alto de
  la píldora del buscador y del botón "Nueva lista": los da YTM; el tema no los fija.
- "Hueco" del asa por box-shadow: anunciado en el comentario pero `box-shadow: none` en el código.
- Transición de `box-shadow` en el botón de reproducir sin sombra definida aquí.
- Cuándo se muestra el panel de volumen y la mecánica del muelle de plegado: fuera de este archivo
  o en la parte 2.
