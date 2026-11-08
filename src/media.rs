//! Teclas multimedia y panel de medios de Windows (SMTC), vía souvlaki.

use crossbeam_channel::Sender;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};

#[derive(Debug, Clone, Copy)]
pub enum MediaKey {
    Toggle,
    Play,
    Pause,
    Next,
    Previous,
}

pub struct Media {
    controls: MediaControls,
}

impl Media {
    /// `hwnd` es la ventana principal (en Windows SMTC la exige).
    pub fn new(hwnd: Option<*mut std::ffi::c_void>, tx: Sender<MediaKey>, wake: impl Fn() + Send + 'static) -> Option<Self> {
        let config = PlatformConfig { dbus_name: "bytune_rs", display_name: "ByTune", hwnd };
        let mut controls = MediaControls::new(config).map_err(|e| log::warn!("media controls: {e:?}")).ok()?;
        controls
            .attach(move |e: MediaControlEvent| {
                let key = match e {
                    MediaControlEvent::Toggle => MediaKey::Toggle,
                    MediaControlEvent::Play => MediaKey::Play,
                    MediaControlEvent::Pause => MediaKey::Pause,
                    MediaControlEvent::Next => MediaKey::Next,
                    MediaControlEvent::Previous => MediaKey::Previous,
                    _ => return,
                };
                let _ = tx.send(key);
                wake();
            })
            .map_err(|e| log::warn!("media attach: {e:?}"))
            .ok()?;
        Some(Self { controls })
    }

    pub fn set_track(&mut self, title: &str, artist: &str, album: &str, cover: &str, secs: u32) {
        let _ = self.controls.set_metadata(MediaMetadata {
            title: Some(title),
            artist: Some(artist),
            album: Some(album),
            cover_url: (!cover.is_empty()).then_some(cover),
            duration: (secs > 0).then(|| std::time::Duration::from_secs(secs as u64)),
        });
    }

    pub fn set_playing(&mut self, playing: bool) {
        let _ = self.controls.set_playback(if playing {
            MediaPlayback::Playing { progress: None }
        } else {
            MediaPlayback::Paused { progress: None }
        });
    }
}
