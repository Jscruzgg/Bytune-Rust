// Sin consola en las builds de release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod desktop;
mod i18n;
mod innertube;
mod login;
mod lyrics;
mod media;
mod pages;
mod probe;
mod settings;
mod theme;
mod ui;
mod webplayer;

fn main() -> eframe::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("bytune=info")).init();

    // Un único runtime de tokio, pequeño: solo red (páginas, búsqueda, cola, letras).
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("net")
        .enable_all()
        .build()
        .expect("runtime de tokio");

    // Sin ventana: prueba de la parte nativa (páginas, búsqueda, radio, letra) con tiempos.
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--probe") {
        let q = args.get(i + 1).cloned().unwrap_or_else(|| "Bohemian Rhapsody Queen".into());
        let code = rt.block_on(probe::run(&q));
        std::process::exit(code);
    }
    // Arranque con Windows: directo a la bandeja, sin ventana.
    let background = args.iter().any(|a| a == "--background");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ByTune")
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([860.0, 560.0])
            .with_icon(theme::app_icon())
            .with_visible(!background),
        persist_window: true,
        ..Default::default()
    };
    let handle = rt.handle().clone();
    eframe::run_native("ByTune-rs", options, Box::new(move |cc| Ok(Box::new(app::App::new(cc, handle, background)))))
}
