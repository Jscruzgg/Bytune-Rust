# ByTune-rs

Versión en Rust de [ByTune](https://github.com/Jscruzgg/ByTune), para comparar su rendimiento
con la versión en Electron.

La interfaz es **nativa** ([egui](https://github.com/emilk/egui)); el **audio** sale del
reproductor oficial de music.youtube.com en un WebView2 de 1×1 px, al que la app da órdenes
([`src/bridge.js`](src/bridge.js)). No descarga ni descifra streams.

## Secciones

Todas las de ByTune, cada una como página propia (con historial atrás / adelante):

| Sección | Qué tiene |
|---|---|
| **Inicio** | Tus estanterías de YouTube Music (selección rápida, mixes, vuelve a escucharlo...) y chips de ánimo |
| **Explorar** | Novedades, listas de éxitos, ánimo y géneros, tendencias, vídeos nuevos |
| **Biblioteca** | Playlists, canciones que te gustan, álbumes, artistas, suscripciones, historial |
| **Búsqueda** | Resultado principal y resultados, con filtros (canciones, vídeos, álbumes, artistas, listas...) |
| **Álbum / artista / playlist** | Cabecera con Reproducir, Aleatorio y Radio; pistas, discografía, "puede que te guste"; "Cargar más" en listas largas |
| **Reproduciendo** | Portada o vídeo (conmutador canción / vídeo), letra sincronizada (clic en una línea para saltar), me gusta, pantalla completa |
| **Cola** | Arrastrar para reordenar, quitar, vaciar; al acabar sigue con canciones parecidas |
| **Ajustes** | Notificaciones, bandeja, inicio con Windows, autoplay, idioma (es / en), menú rápido, atajos, cuenta |

## Funciones

- **Menú rápido** (`Ctrl+Shift+Space`, configurable; solo con la app enfocada o desde cualquier
  app): buscar canciones, tus playlists y recientes; Enter reproduce, Shift+Enter a la cola,
  Alt+←↓→ controles.
- **Bandeja**: canción actual, reproducir / pausa, siguiente, anterior, mostrar, salir. Al cerrar
  la ventana sigue en la bandeja (opcional).
- **Atajos globales**: `Ctrl+Alt+→ / ←` siguiente / anterior, `Ctrl+Alt+M` mostrar / ocultar.
- **Teclado en la ventana**: Espacio, Alt+← / → (también los botones laterales del ratón),
  Ctrl+F, F5, F11 / Esc, Ctrl + / - / 0.
- **Notificación** al cambiar de canción, **teclas multimedia** y panel de medios de Windows.
- **Menú contextual** (clic derecho) en canciones: reproducir a continuación, añadir a la cola,
  radio, ir al artista / álbum.
- **Me gusta** desde la barra y Now Playing; **inicio de sesión con Google** en el mismo WebView.
- **Letras** de [LRCLIB](https://lrclib.net) con respaldo a la de YouTube Music (port de
  `src/lib` de ByTune).

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
| `src/app.rs` | estado, navegación entre secciones, cola y reproducción |
| `src/ui/` | vistas: páginas, Now Playing, menú rápido, ajustes |
| `src/pages.rs` | modelo común de páginas de YouTube Music (secciones, tarjetas, chips) |
| `src/desktop.rs` | bandeja, atajos globales, notificaciones, inicio con Windows |
| `src/settings.rs`, `src/i18n.rs` | ajustes persistentes y textos es / en |
| `src/webplayer.rs` + `src/bridge.js` | WebView2 con el reproductor de YouTube Music, vídeo y login |
| `src/innertube.rs` | páginas, búsqueda, cola, radio, me gusta y letra de YouTube Music |
| `src/lyrics/` | LRCLIB: LRC, consultas, tiempos (port de ByTune) |
| `src/login.rs` | sesión de Google leída de las cookies del WebView |
| `src/media.rs` | teclas multimedia (SMTC) |
| `src/theme.rs` | colores de ByTune e icono |

Más detalle en [docs/design.md](docs/design.md).

## Licencia

MIT, como ByTune.
