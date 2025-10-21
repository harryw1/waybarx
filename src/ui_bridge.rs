use gtk::glib::{self, clone};
use serde_json::{json, Value};
use webkit6::{prelude::*, WebView};
use std::rc::Rc;

#[cfg(any(feature = "sway", feature = "hypr"))]
use crate::ipc;
use crate::providers::SystemProvider;

pub fn wire_bridge(web: &WebView, system_provider: Rc<SystemProvider>) {
    let ucm = web.user_content_manager().expect("ucm");
    ucm.connect_script_message_received(
        Some("native"),
        clone!(#[weak] web, #[strong] system_provider, move |_ucm, value| {
            // Parse message from JS (value is already a javascriptcore::Value)
            let payload = value.to_json(0).map(|s| s.to_string()).unwrap_or_default();

            // Try to parse as JSON to extract command
            if let Ok(data) = serde_json::from_str::<Value>(&payload) {
                if let Some(cmd) = data.get("cmd").and_then(|v| v.as_str()) {
                    match cmd {
                        #[cfg(feature = "hypr")]
                        "get_workspaces" => {
                            glib::spawn_future_local(clone!(#[weak] web, async move {
                                match ipc::hypr::workspaces().await {
                                    Ok(workspaces) => {
                                        let resp = json!({
                                            "ok": true,
                                            "cmd": "workspaces",
                                            "data": workspaces,
                                        });
                                        let js = format!("window.__nativeReceive && window.__nativeReceive({});", resp);
                                        web.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
                                    }
                                    Err(e) => {
                                        let resp = json!({
                                            "ok": false,
                                            "cmd": "workspaces",
                                            "error": e.to_string(),
                                        });
                                        let js = format!("window.__nativeReceive && window.__nativeReceive({});", resp);
                                        web.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
                                    }
                                }
                            }));
                        }
                        #[cfg(feature = "sway")]
                        "get_workspaces" => {
                            glib::spawn_future_local(clone!(#[weak] web, async move {
                                match ipc::sway::workspaces().await {
                                    Ok(workspaces) => {
                                        let resp = json!({
                                            "ok": true,
                                            "cmd": "workspaces",
                                            "data": workspaces,
                                        });
                                        let js = format!("window.__nativeReceive && window.__nativeReceive({});", resp);
                                        web.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
                                    }
                                    Err(e) => {
                                        let resp = json!({
                                            "ok": false,
                                            "cmd": "workspaces",
                                            "error": e.to_string(),
                                        });
                                        let js = format!("window.__nativeReceive && window.__nativeReceive({});", resp);
                                        web.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
                                    }
                                }
                            }));
                        }
                        "get_system_info" => {
                            // Get cached system info (refreshed in background)
                            match system_provider.get_info() {
                                Some(info) => {
                                    let resp = json!({
                                        "ok": true,
                                        "cmd": "system_info",
                                        "data": info,
                                    });
                                    let js = format!("window.__nativeReceive && window.__nativeReceive({});", resp);
                                    web.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
                                }
                                None => {
                                    let resp = json!({
                                        "ok": false,
                                        "cmd": "system_info",
                                        "error": "Failed to get system info",
                                    });
                                    let js = format!("window.__nativeReceive && window.__nativeReceive({});", resp);
                                    web.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
                                }
                            }
                        }
                        _ => {
                            // Unknown command - echo back
                            let resp = json!({
                                "ok": false,
                                "error": "unknown command",
                                "echo": payload,
                            });
                            let js = format!("window.__nativeReceive && window.__nativeReceive({});", resp);
                            web.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
                        }
                    }
                } else {
                    // No cmd field - echo back for backwards compatibility
                    let resp = json!({
                        "ok": true,
                        "echo": payload,
                    });
                    let js = format!("window.__nativeReceive && window.__nativeReceive({});", resp);
                    web.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
                }
            } else {
                // Not valid JSON - echo back
                let resp = json!({
                    "ok": true,
                    "echo": payload,
                });
                let js = format!("window.__nativeReceive && window.__nativeReceive({});", resp);
                web.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
            }
        }),
    );
}
