# ByTune-rs

Versión en Rust de [ByTune](https://github.com/Jscruzgg/ByTune), para comparar su rendimiento
con la versión en Electron.

- **Interfaz nativa** con [egui](https://github.com/emilk/egui): búsqueda, cola, Now Playing
  con portada y letras sincronizadas, y barra del reproductor.
- **Audio del reproductor oficial** de music.youtube.com, en un WebView2 de 1×1 px dentro de
  la ventana. La app le da órdenes (cargar, play/pausa, seek, volumen) y lee su estado
  ([`src/bridge.js`](src/bridge.js)). No descarga ni descifra streams.
- **Búsqueda y radio** con la API interna de YouTube Music (InnerTube), firmadas con tu sesión
  si la has iniciado.
- **Letras** de [LRCLIB](https://lrclib.net), con respaldo a la letra de YouTube Music: es un
  port directo de `src/lib` de ByTune.
- **Teclas multimedia** y panel de medios de Windows.
- **Inicio de sesión con Google** en el mismo WebView (botón abajo a la izquierda).

Solo Windows 10/11 (necesita el runtime de WebView2, que ya viene instalado).

## Compilar

Necesitas [Rust](https://rustup.rs/) y las Build Tools de Visual Studio (C++).

```sh
cargo build --release
```

El ejecutable queda en `target/release/bytune.exe`.

## Pruebas

```sh
cargo test                                   # tests unitarios (letras, InnerTube)
bytune --probe "Bohemian Rhapsody Queen"     # búsqueda, radio y letra sin ventana, con tiempos
```

Para un escenario repetible (comparativa de rendimiento), la variable `BYTUNE_AUTOPLAY` busca
y reproduce el primer resultado al arrancar:

```powershell
$env:BYTUNE_AUTOPLAY = "Bohemian Rhapsody Queen"; $env:RUST_LOG = "bytune=info"; .\bytune.exe
```

El log incluye el tiempo hasta el primer sonido.

## Estructura

| Archivo | Qué hace |
|---|---|
| `src/app.rs` | estado y vistas (egui) |
| `src/webplayer.rs` + `src/bridge.js` | WebView2 con el reproductor de YouTube Music, y login |
| `src/innertube.rs` | búsqueda, radio y letra de YouTube Music |
| `src/lyrics/` | LRCLIB: LRC, consultas, tiempos (port de ByTune) |
| `src/login.rs` | sesión de Google leída de las cookies del WebView |
| `src/media.rs` | teclas multimedia (SMTC) |
| `src/theme.rs` | colores de ByTune e icono |

Más detalle en [docs/design.md](docs/design.md).

## Licencia

MIT, como ByTune.
