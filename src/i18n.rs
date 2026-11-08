//! Textos de la interfaz en español e inglés (como ByTune).

use crate::settings::Lang;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum L {
    Es,
    En,
}

impl L {
    pub fn resolve(lang: Lang, system: &str) -> L {
        match lang {
            Lang::Es => L::Es,
            Lang::En => L::En,
            Lang::Auto => {
                if system.starts_with("es") {
                    L::Es
                } else {
                    L::En
                }
            }
        }
    }

    pub fn hl(self) -> &'static str {
        match self {
            L::Es => "es",
            L::En => "en",
        }
    }
}

macro_rules! strings {
    ($($key:ident => $es:literal, $en:literal;)*) => {
        #[allow(non_camel_case_types, dead_code)]
        #[derive(Clone, Copy)]
        pub enum S { $($key,)* }

        pub fn t(l: L, s: S) -> &'static str {
            match (l, s) {
                $( (L::Es, S::$key) => $es, (L::En, S::$key) => $en, )*
            }
        }
    };
}

strings! {
    home => "Inicio", "Home";
    explore => "Explorar", "Explore";
    library => "Biblioteca", "Library";
    search => "Buscar", "Search";
    now_playing => "Reproduciendo", "Now playing";
    settings => "Ajustes", "Settings";
    queue => "Cola", "Queue";
    search_hint => "Buscar canciones, álbumes, artistas", "Search songs, albums, artists";
    search_help => "Escribe y pulsa Enter.", "Type and press Enter.";
    nothing_playing => "Nada sonando", "Nothing playing";
    no_lyrics => "Sin letra para esta canción", "No lyrics for this song";
    lyrics => "Letra", "Lyrics";
    sign_in => "Iniciar sesión con Google", "Sign in with Google";
    sign_in_hint => "Para tu biblioteca y recomendaciones", "For your library and recommendations";
    sign_out => "Cerrar sesión", "Sign out";
    signed_in => "Sesión de Google iniciada", "Signed in with Google";
    waiting_google => "Esperando a Google…", "Waiting for Google…";
    cancel => "Cancelar", "Cancel";
    play => "Reproducir", "Play";
    shuffle => "Aleatorio", "Shuffle";
    radio => "Radio", "Radio";
    more => "Más", "More";
    load_more => "Cargar más", "Load more";
    play_next => "Reproducir a continuación", "Play next";
    add_to_queue => "Añadir a la cola", "Add to queue";
    go_to_artist => "Ir al artista", "Go to artist";
    go_to_album => "Ir al álbum", "Go to album";
    start_radio => "Iniciar radio", "Start radio";
    like => "Me gusta", "Like";
    unlike => "Quitar me gusta", "Remove like";
    remove => "Quitar de la cola", "Remove from queue";
    clear_queue => "Vaciar", "Clear";
    error => "No se pudo cargar", "Couldn't load";
    retry => "Reintentar", "Retry";
    need_login => "Inicia sesión para ver tu biblioteca", "Sign in to see your library";
    lib_playlists => "Playlists", "Playlists";
    lib_songs => "Canciones que me gustan", "Liked songs";
    lib_albums => "Álbumes", "Albums";
    lib_artists => "Artistas", "Artists";
    lib_subscriptions => "Suscripciones", "Subscriptions";
    lib_history => "Historial", "History";
    lib_uploads => "Subidas", "Uploads";
    ex_new => "Novedades", "New releases";
    ex_charts => "Listas de éxitos", "Charts";
    ex_moods => "Ánimo y géneros", "Moods & genres";
    song => "Canción", "Song";
    video => "Vídeo", "Video";
    fullscreen => "Pantalla completa", "Fullscreen";
    exit_fullscreen => "Salir de pantalla completa (Esc)", "Exit fullscreen (Esc)";
    // Ajustes
    set_general => "General", "General";
    set_notify => "Notificar el cambio de canción", "Notify on song change";
    set_close_to_tray => "Al cerrar, seguir en la bandeja", "Keep running in the tray when closed";
    set_start_with_windows => "Iniciar con Windows", "Start with Windows";
    set_autoplay => "Al acabar la cola, seguir con canciones parecidas", "When the queue ends, keep playing similar songs";
    set_lang => "Idioma", "Language";
    set_lang_auto => "Automático", "Automatic";
    set_quick_menu => "Menú rápido", "Quick menu";
    set_qm_shortcut => "Atajo", "Shortcut";
    set_qm_record => "Pulsa la combinación…", "Press the shortcut…";
    set_qm_change => "Cambiar", "Change";
    set_qm_invalid => "Ese atajo no se puede usar", "That shortcut can't be used";
    set_qm_reach => "Se abre", "Opens";
    set_qm_focused => "Solo con ByTune enfocado", "Only when ByTune is focused";
    set_qm_anywhere => "Desde cualquier app", "From any app";
    set_qm_shift_enter => "Shift+Enter", "Shift+Enter";
    set_qm_queue => "Añade al final de la cola", "Adds to the end of the queue";
    set_qm_next => "Reproduce a continuación", "Plays next";
    set_shortcuts => "Atajos de teclado", "Keyboard shortcuts";
    set_account => "Cuenta", "Account";
    set_about => "Acerca de", "About";
    // Bandeja
    tray_show => "Mostrar ByTune", "Show ByTune";
    tray_quit => "Salir", "Quit";
    tray_play_pause => "Reproducir / pausa", "Play / pause";
    tray_next => "Siguiente", "Next";
    tray_prev => "Anterior", "Previous";
    // Menú rápido
    qm_hint => "Busca una canción o una playlist", "Search a song or a playlist";
    qm_songs => "Canciones", "Songs";
    qm_playlists => "Tus playlists", "Your playlists";
    qm_recent => "Recientes", "Recent";
    qm_help => "Enter reproduce · Shift+Enter cola · Alt+←↓→ controles · Esc cierra", "Enter plays · Shift+Enter queue · Alt+←↓→ controls · Esc closes";
}
