use gtk::prelude::*;
use gtk::{gdk, glib, Application, ApplicationWindow};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use webkit6::{prelude::*, Settings, UserContentManager, WebView};
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashSet;

mod ui_bridge;
mod providers;
#[cfg(any(feature = "sway", feature = "hypr"))]
mod ipc;

#[cfg(not(debug_assertions))]
mod assets {
    use rust_embed::RustEmbed;
    #[derive(RustEmbed)]
    #[folder = "web-dist/"]
    pub struct Assets;
}

fn build_webview(system_provider: Rc<providers::SystemProvider>) -> WebView {
    let ucm = UserContentManager::new();
    // Channel for JS -> Rust messages (None for world_name means default world)
    ucm.register_script_message_handler("native", None);
    let web = WebView::builder().user_content_manager(&ucm).build();

    // Enable useful features (depends on system WebKitGTK version)
    let settings = Settings::new();
    settings.set_enable_javascript(true);
    settings.set_enable_back_forward_navigation_gestures(true);
    #[cfg(debug_assertions)]
    settings.set_enable_developer_extras(true); // Web Inspector (right-click -> Inspect) - debug only
    web.set_settings(&settings);

    // Wire JS <-> Rust bridge with shared system provider
    ui_bridge::wire_bridge(&web, system_provider);

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

fn spawn_bar_on_monitor(app: &Application, monitor: &gdk::Monitor, system_provider: Rc<providers::SystemProvider>) {
    let win = ApplicationWindow::builder().application(app).title("waybarx").build();

    // Give this window the layer-shell role (a real panel)
    win.init_layer_shell();
    win.set_layer(Layer::Top);
    win.set_anchor(Edge::Top, true);
    win.set_anchor(Edge::Left, true);
    win.set_anchor(Edge::Right, true);
    win.set_exclusive_zone(36); // reserve 36px on this output
    win.set_keyboard_mode(KeyboardMode::Exclusive);
    win.set_monitor(monitor);

    let web = build_webview(system_provider);
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
            let monitors = display.monitors();
            // Registry to track which monitors have bars (by connector name)
            let monitor_registry: Rc<RefCell<HashSet<String>>> = Rc::new(RefCell::new(HashSet::new()));

            // Create ONE shared SystemProvider for ALL bars
            let system_provider = Rc::new(providers::SystemProvider::new());

            // Start background refresh thread for SystemProvider
            // This prevents blocking the GTK main thread
            glib::spawn_future_local(glib::clone!(
                #[strong] system_provider,
                async move {
                    loop {
                        // Refresh system info on background thread
                        let provider = system_provider.clone();
                        glib::spawn_future_local(async move {
                            provider.refresh();
                        });

                        // Wait 2 seconds before next refresh
                        glib::timeout_future(std::time::Duration::from_secs(2)).await;
                    }
                }
            ));

            // Spawn initial bars for all monitors
            for i in 0..monitors.n_items() {
                if let Some(monitor) = monitors.item(i).and_then(|obj| obj.downcast::<gdk::Monitor>().ok()) {
                    if let Some(connector) = monitor.connector() {
                        monitor_registry.borrow_mut().insert(connector.to_string());
                        spawn_bar_on_monitor(app, &monitor, Rc::clone(&system_provider));
                    }
                }
            }

            // Listen for monitor hotplug (add/remove)
            monitors.connect_items_changed(glib::clone!(
                #[weak] app,
                #[strong] monitor_registry,
                #[strong] system_provider,
                move |monitors, _pos, _removed, _added| {
                    // Check all current monitors and spawn bars for any new ones
                    for i in 0..monitors.n_items() {
                        if let Some(monitor) = monitors.item(i).and_then(|obj| obj.downcast::<gdk::Monitor>().ok()) {
                            if let Some(connector) = monitor.connector() {
                                let connector_str = connector.to_string();
                                // Only spawn if this monitor doesn't have a bar yet
                                if !monitor_registry.borrow().contains(&connector_str) {
                                    monitor_registry.borrow_mut().insert(connector_str);
                                    spawn_bar_on_monitor(&app, &monitor, Rc::clone(&system_provider));
                                }
                            }
                        }
                    }
                }
            ));
        } else {
            // Fallback: single window if monitor enumeration fails
            let win = ApplicationWindow::builder().application(app).title("waybarx").build();
            win.init_layer_shell();
            win.set_layer(Layer::Top);
            win.set_anchor(Edge::Top, true);
            win.set_anchor(Edge::Left, true);
            win.set_anchor(Edge::Right, true);
            win.set_exclusive_zone(36);

            let system_provider = Rc::new(providers::SystemProvider::new());

            // Start background refresh thread for SystemProvider
            glib::spawn_future_local(glib::clone!(
                #[strong] system_provider,
                async move {
                    loop {
                        let provider = system_provider.clone();
                        glib::spawn_future_local(async move {
                            provider.refresh();
                        });
                        glib::timeout_future(std::time::Duration::from_secs(2)).await;
                    }
                }
            ));

            let web = build_webview(system_provider);
            win.set_child(Some(&web));
            win.set_default_size(800, 36);
            win.present();
        }
    });

    app.run();
}
