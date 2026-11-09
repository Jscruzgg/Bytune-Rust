# ByTune-rs: diseño

Versión en Rust de [ByTune](https://github.com/Jscruzgg/ByTune), para comparar rendimiento
(RAM, CPU, arranque, tamaño) con la versión en Electron.

## Arquitectura

ByTune (Electron) carga music.youtube.com entera y la retoca con CSS/JS. ByTune-rs dibuja una
interfaz **nativa** (egui) y solo usa la web para lo que no puede hacerse legítimamente fuera
de ella: **reproducir el audio**.

```
┌──────────────────────── ventana (eframe/egui) ───────────────────────┐
│  barra lateral │ búsqueda / Now Playing + letras │ cola             │
│  barra del reproductor                                               │
│  [WebView2 1×1 px con music.youtube.com]  ← bridge.js               │
└──────────────────────────────────────────────────────────────────────┘
         │ órdenes (evaluate_script)        ▲ estado (ipc.postMessage)
         ▼                                  │
   reproductor oficial de YouTube Music: carga, decodifica y suena
```

| Pieza | Cómo |
|---|---|
| Interfaz | `eframe` / `egui` (OpenGL) |
| Audio | reproductor de music.youtube.com en WebView2 (`wry`) |
| Búsqueda, radio, letra de YTM | InnerTube (`reqwest`), con la sesión del WebView |
| Letras sincronizadas | LRCLIB, port de `src/lib` de ByTune |
| Teclas multimedia | `souvlaki` (SMTC) |

### Por qué el audio va por la web

Se probó primero una versión 100 % nativa (descargar el stream y decodificarlo con
symphonia). No es viable de forma legítima:

- Sin sesión, `/player` responde "confirma que no eres un bot".
- Con sesión (cliente web), los 18 formatos de audio vienen **cifrados** y la descarga exige
  además un token de BotGuard. Reproducirlos fuera de un navegador obligaría a saltarse esas
  protecciones, cosa que este proyecto no hace.

Así que el sonido lo produce el reproductor oficial, y la comparativa queda así:
**interfaz nativa + WebView2 solo para audio** frente a **Electron con toda la web de YTM**.

### Detalles del WebView

- **1×1 px y visible**: Chromium aplaza la carga de audio en páginas ocultas.
  `CalculateNativeWinOcclusion` está desactivado para que tampoco se pare con la ventana
  minimizada.
- **La cola es de Rust.** YouTube Music tiene la suya (y al abrir restaura la última sesión). El
  puente navega con `yt-navigate` (lo mismo que un clic en la web) y, si la web cambia de
  canción por su cuenta, vuelve a pedir la nuestra. Solo cuando la nuestra termina avisa a Rust
  (`ended`).
- **Login**: el mismo WebView se coloca sobre la zona central con la página de Google. Al volver
  a music.youtube.com con la cookie `SAPISID` se oculta. Las cookies viven en el perfil del
  WebView (`%LOCALAPPDATA%\ByTune-rs\webview`) y no se guardan en ningún otro sitio.

## Siguientes fases

1. Bandeja, atajos globales, menú rápido, notificaciones, inicio con Windows.
2. Biblioteca, playlists, me gusta e historial (InnerTube con sesión).
3. Tema pulido al nivel de ByTune, i18n es/en, instalador y actualizaciones.

## Comparativa

El mismo escenario en las dos apps (`BYTUNE_AUTOPLAY` en ByTune-rs):

- RAM total (suma de procesos, WebView2 incluido) en reposo, reproduciendo, y con Now Playing
  y letras.
- CPU media reproduciendo 5 min con la ventana visible y minimizada.
- Tiempo hasta ventana visible y hasta el primer sonido.
- Tamaño del instalador y en disco.
