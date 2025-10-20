use gtk::glib::{self, clone};
use serde_json::{json, Value};
use webkit6::{prelude::*, WebView};

#[cfg(any(feature = "sway", feature = "hypr"))]
use crate::ipc;

pub fn wire_bridge(web: &WebView) {
    let ucm = web.user_content_manager().expect("ucm");
    ucm.connect_script_message_received(
        Some("native"),
        clone!(@weak web => move |_ucm, msg| {
            // Parse message from JS
            let payload = msg
                .js_value()
                .and_then(|v| v.to_string(&web.context()).ok())
                .unwrap_or_default();

            // Try to parse as JSON to extract command
            if let Ok(data) = serde_json::from_str::<Value>(&payload) {
                if let Some(cmd) = data.get("cmd").and_then(|v| v.as_str()) {
                    match cmd {
                        #[cfg(feature = "hypr")]
                        "get_workspaces" => {
                            glib::spawn_future_local(clone!(@weak web => async move {
                                match ipc::hypr::workspaces().await {
                                    Ok(workspaces) => {
                                        let resp = json!({
                                            "ok": true,
                                            "cmd": "workspaces",
                                            "data": workspaces,
                                        });
                                        let js = format!("window.__nativeReceive && window.__nativeReceive({});", resp);
                                        web.evaluate_javascript(&js, None::<&gio::Cancellable>, |_| {});
                                    }
                                    Err(e) => {
                                        let resp = json!({
                                            "ok": false,
                                            "cmd": "workspaces",
                                            "error": e.to_string(),
                                        });
                                        let js = format!("window.__nativeReceive && window.__nativeReceive({});", resp);
                                        web.evaluate_javascript(&js, None::<&gio::Cancellable>, |_| {});
                                    }
                                }
                            }));
                        }
                        #[cfg(feature = "sway")]
                        "get_workspaces" => {
                            glib::spawn_future_local(clone!(@weak web => async move {
                                match ipc::sway::workspaces().await {
                                    Ok(workspaces) => {
                                        let resp = json!({
                                            "ok": true,
                                            "cmd": "workspaces",
                                            "data": workspaces,
                                        });
                                        let js = format!("window.__nativeReceive && window.__nativeReceive({});", resp);
                                        web.evaluate_javascript(&js, None::<&gio::Cancellable>, |_| {});
                                    }
                                    Err(e) => {
                                        let resp = json!({
                                            "ok": false,
                                            "cmd": "workspaces",
                                            "error": e.to_string(),
                                        });
                                        let js = format!("window.__nativeReceive && window.__nativeReceive({});", resp);
                                        web.evaluate_javascript(&js, None::<&gio::Cancellable>, |_| {});
                                    }
                                }
                            }));
                        }
                        _ => {
                            // Unknown command - echo back
                            let resp = json!({
                                "ok": false,
                                "error": "unknown command",
                                "echo": payload,
                            });
                            let js = format!("window.__nativeReceive && window.__nativeReceive({});", resp);
                            web.evaluate_javascript(&js, None::<&gio::Cancellable>, |_| {});
                        }
                    }
                } else {
                    // No cmd field - echo back for backwards compatibility
                    let resp = json!({
                        "ok": true,
                        "echo": payload,
                    });
                    let js = format!("window.__nativeReceive && window.__nativeReceive({});", resp);
                    web.evaluate_javascript(&js, None::<&gio::Cancellable>, |_| {});
                }
            } else {
                // Not valid JSON - echo back
                let resp = json!({
                    "ok": true,
                    "echo": payload,
                });
                let js = format!("window.__nativeReceive && window.__nativeReceive({});", resp);
                web.evaluate_javascript(&js, None::<&gio::Cancellable>, |_| {});
            }
        }),
    );
}
