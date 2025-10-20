use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use gtk_layer_shell as gls;
use webkit6::{prelude::*, Settings, UserContentManager, WebView};

mod ui_bridge;
#[cfg(any(feature = "sway", feature = "hypr"))]
mod ipc;

#[cfg(not(debug_assertions))]
mod assets {
    use rust_embed::RustEmbed;
    #[derive(RustEmbed)]
    #[folder = "web-dist/"]
    pub struct Assets;
}

fn build_webview() -> WebView {
    let ucm = UserContentManager::new();
    // Channel for JS -> Rust messages
    ucm.register_script_message_handler("native").expect("register handler");
    let web = WebView::with_user_content_manager(&ucm);

    // Enable useful features (depends on system WebKitGTK version)
    let settings = Settings::new();
    settings.set_enable_javascript(true);
    settings.set_enable_back_forward_navigation_gestures(true);
    #[cfg(debug_assertions)]
    settings.set_enable_developer_extras(true); // Web Inspector (right-click -> Inspect) - debug only
    web.set_settings(&settings);

    // Wire JS <-> Rust bridge
    ui_bridge::wire_bridge(&web);

    #[cfg(debug_assertions)]
    {
        // Point to your dev server (Vite/Parcel). Adjust port as needed.
        web.load_uri("http://127.0.0.1:5173/");
    }

    #[cfg(not(debug_assertions))]
    {
        use assets::Assets;
        let index = Assets::get("index.html").expect("embed index.html");
        let html = std::str::from_utf8(index.data.as_ref()).expect("utf8");
        web.load_html(html, None);
    }

    web
}

fn spawn_bar_on_monitor(app: &Application, monitor: &gdk::Monitor) {
    let win = ApplicationWindow::builder().application(app).title("waybarx").build();

    // Give this window the layer-shell role (a real panel)
    gls::init_for_window(&win);
    gls::set_layer(&win, gls::Layer::Top);
    gls::set_anchor(&win, gls::Edge::Top, true);
    gls::set_anchor(&win, gls::Edge::Left, true);
    gls::set_anchor(&win, gls::Edge::Right, true);
    gls::set_exclusive_zone(&win, 36); // reserve 36px on this output
    gls::set_keyboard_interactivity(&win, gls::KeyboardInteractivity::Exclusive);
    gls::set_monitor(&win, monitor);

    let web = build_webview();
    win.set_child(Some(&web));
    // Size request: height fixed, width follows output
    win.set_default_size(800, 36);
    win.present();
}

fn main() {
    gtk::init().expect("gtk init");

    let app = Application::builder()
        .application_id("dev.example.waybarx")
        .build();

    app.connect_activate(|app| {
        // Spawn one bar per monitor
        if let Some(display) = gdk::Display::default() {
            for i in 0..display.n_monitors() {
                if let Some(monitor) = display.monitor(i) {
                    spawn_bar_on_monitor(app, &monitor);
                }
            }

            // Listen for monitor hotplug (add/remove)
            display.connect_monitor_added(glib::clone!(@weak app => move |_dpy, monitor| {
                spawn_bar_on_monitor(&app, &monitor);
            }));
            // Note: Removing bars on monitor removal is left as an exercise;
            // store windows per monitor in a registry if you need that.
        } else {
            // Fallback: single window if monitor enumeration fails
            let win = ApplicationWindow::builder().application(app).title("waybarx").build();
            gls::init_for_window(&win);
            gls::set_layer(&win, gls::Layer::Top);
            gls::set_anchor(&win, gls::Edge::Top, true);
            gls::set_anchor(&win, gls::Edge::Left, true);
            gls::set_anchor(&win, gls::Edge::Right, true);
            gls::set_exclusive_zone(&win, 36);
            let web = build_webview();
            win.set_child(Some(&web));
            win.set_default_size(800, 36);
            win.present();
        }
    });

    app.run();
}
