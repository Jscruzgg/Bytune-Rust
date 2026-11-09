// Puente entre ByTune-rs y el reproductor oficial de music.youtube.com.
// Se inyecta en cada carga de página (with_initialization_script). Rust llama a
// window.__bt.<orden>(...) con evaluate_script y recibe el estado por window.ipc.postMessage.
(() => {
  if (window.__bt || location.hostname !== 'music.youtube.com') return;

  const app = () => document.querySelector('ytmusic-app');
  const api = () => {
    const p = document.getElementById('movie_player');
    return p && typeof p.getVideoData === 'function' ? p : null;
  };
  const video = () => document.querySelector('video');
  const send = (msg) => { try { window.ipc.postMessage(JSON.stringify(msg)); } catch {} };
  const currentId = () => api()?.getVideoData?.()?.video_id || null;

  // La cola es de Rust. YouTube Music tiene la suya (y al abrir restaura la última sesión):
  // si cambia de canción por su cuenta, se vuelve a pedir la nuestra; solo cuando la nuestra
  // termina se avisa a Rust ("ended") para que ponga la siguiente.
  let wanted = null;
  let askedAt = 0;
  let tries = 0;
  let nearEnd = false;
  let endedSent = null;

  function navigate(id) {
    const a = app();
    askedAt = Date.now();
    tries++;
    if (a && tries <= 2) {
      // Lo mismo que hace la web al pulsar una canción.
      a.dispatchEvent(new CustomEvent('yt-navigate', {
        bubbles: true,
        composed: true,
        detail: { endpoint: { watchEndpoint: { videoId: id } } },
      }));
    } else {
      location.href = 'https://music.youtube.com/watch?v=' + encodeURIComponent(id);
    }
  }

  window.__bt = {
    load(id) {
      wanted = id;
      tries = 0;
      nearEnd = false;
      endedSent = null;
      navigate(id);
    },
    toggle() { const v = video(); if (v) (v.paused ? v.play() : v.pause()); },
    play() { video()?.play(); },
    pause() { video()?.pause(); },
    seek(s) { const v = video(); if (v) v.currentTime = s; },
    volume(x) {
      const p = api();
      if (p?.setVolume) { p.setVolume(Math.round(x * 100)); p.unMute?.(); }
      const v = video(); if (v) v.volume = x;
    },
    // Canción / vídeo: el mismo conmutador que la web (ytmusic-av-toggle).
    av(mode) {
      document.querySelector(`ytmusic-player-page #av-id .${mode === 'video' ? 'video' : 'song'}-button`)?.click();
    },
    // Modo vídeo: el reproductor ocupa toda la página (el WebView se coloca sobre la portada).
    videoMode(on) {
      document.documentElement.classList.toggle('bt-video', !!on);
    },
  };

  // Estilo del modo vídeo: solo el reproductor, a pantalla completa dentro del WebView.
  const style = document.createElement('style');
  style.textContent = `
    html.bt-video, html.bt-video body { overflow: hidden !important; background: #000 !important; }
    html.bt-video ytmusic-player {
      position: fixed !important; inset: 0 !important; width: 100vw !important; height: 100vh !important;
      max-width: none !important; max-height: none !important; margin: 0 !important;
      z-index: 2147483647 !important; background: #000 !important; visibility: visible !important; display: block !important;
    }
    html.bt-video ytmusic-player #song-media-window, html.bt-video ytmusic-player #song-video,
    html.bt-video ytmusic-player .html5-video-player { width: 100% !important; height: 100% !important; }
    html.bt-video ytmusic-player video { object-fit: contain !important; width: 100% !important; height: 100% !important; left: 0 !important; top: 0 !important; }
    html.bt-video ytmusic-player .ytp-chrome-top, html.bt-video ytmusic-player .ytp-chrome-bottom { display: none !important; }`;
  // El script corre al crear el documento, antes de que exista <html>: se espera a que lo haya.
  const addStyle = () => (document.head || document.documentElement).appendChild(style);
  if (document.documentElement) addStyle();
  else document.addEventListener('DOMContentLoaded', addStyle, { once: true });

  // Si la página se recargó para poner una canción (?v=...), esa es la que se quiere.
  const fromUrl = new URLSearchParams(location.search).get('v');
  if (fromUrl) wanted = fromUrl;

  let last = '';
  function tick() {
    const v = video();
    const p = api();
    const id = currentId();
    const state = {
      t: 'state',
      ready: !!(p && app()),
      id,
      pos: v ? v.currentTime : 0,
      dur: v && isFinite(v.duration) ? v.duration : 0,
      playing: !!v && !v.paused && !v.ended && v.readyState > 2,
      buffering: !!v && !v.paused && v.readyState <= 2,
      ps: p?.getPlayerState?.() ?? null,
      err: v?.error ? v.error.code + ' ' + (v.error.message || '') : null,
      // ¿Tiene vídeo la canción? (el conmutador está activo) y cuál está puesto.
      hasVideo: !!document.querySelector('ytmusic-player-page ytmusic-av-toggle:not([toggle-disabled])'),
      av: document.querySelector('ytmusic-player-page #av-id .video-button[aria-pressed="true"], ytmusic-player-page[video-mode]') ? 'video' : 'song',
    };

    if (wanted && endedSent !== wanted) {
      if (id === wanted) {
        if (state.dur > 0 && state.pos >= state.dur - 2) nearEnd = true;
        if (v && v.ended) { endedSent = wanted; send({ t: 'ended', id: wanted }); }
        // A veces la web no arranca sola tras navegar.
        else if (v && v.paused && state.pos === 0 && Date.now() - askedAt > 1500 && Date.now() - askedAt < 8000) v.play().catch(() => {});
      } else if (nearEnd) {
        // La nuestra acabó y YouTube Music pasó a la suya: se para y se avisa a Rust.
        v?.pause();
        endedSent = wanted;
        send({ t: 'ended', id: wanted });
      } else if (Date.now() - askedAt > 2500) {
        // YouTube Music puso otra canción (su cola, o no hizo caso): se vuelve a pedir.
        v?.pause();
        navigate(wanted);
      }
    }

    const key = JSON.stringify([state.ready, id, Math.floor(state.pos * 4), state.dur, state.playing, state.buffering, state.ps, state.err, state.hasVideo, state.av]);
    if (key !== last) { last = key; send(state); }
  }
  setInterval(tick, 250);
  document.addEventListener('play', tick, true);
  document.addEventListener('pause', tick, true);
  document.addEventListener('ended', tick, true);
})();
