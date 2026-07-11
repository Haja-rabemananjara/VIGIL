use std::thread;
use tiny_http::{Header, Response, Server};

fn is_uuid_like(segment: &str) -> bool {
    let parts: Vec<&str> = segment.split('-').collect();
    parts.len() == 5
        && [8, 4, 4, 4, 12]
            .iter()
            .zip(&parts)
            .all(|(&len, p)| p.len() == len && p.chars().all(|c| c.is_ascii_hexdigit()))
}

fn resolve_asset_path(raw: &str) -> String {
    let path = raw.split('?').next().unwrap_or(raw);
    let trimmed = path.trim_start_matches('/').trim_end_matches('/');
    if trimmed.is_empty() {
        return "index.html".into();
    }
    let mapped: Vec<String> = trimmed
        .split('/')
        .map(|s| {
            if is_uuid_like(s) {
                "placeholder".to_string()
            } else {
                s.to_string()
            }
        })
        .collect();
    let is_file = mapped.last().map(|s| s.contains('.')).unwrap_or(false);
    let joined = mapped.join("/");
    if is_file {
        joined
    } else {
        format!("{}/index.html", joined)
    }
}

const PORT: u16 = 9527;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let handle = app.handle().clone();
            thread::spawn(move || {
                let server = Server::http(("127.0.0.1", PORT))
                    .expect("failed to start local server");
                let resolver = handle.asset_resolver();

                for request in server.incoming_requests() {
                    let asset_path = resolve_asset_path(request.url());
                    let response = match resolver.get(asset_path) {
                        Some(asset) => {
                            let header = Header::from_bytes(
                                &b"Content-Type"[..],
                                asset.mime_type.as_bytes(),
                            )
                            .unwrap();
                            Response::from_data(asset.bytes).with_header(header)
                        }
                        None => {

                            match resolver.get("index.html".to_string()) {
                                Some(asset) => Response::from_data(asset.bytes).with_header(
                                    Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..])
                                        .unwrap(),
                                ),
                                None => Response::from_data(Vec::new()),
                            }
                        }
                    };
                    let _ = request.respond(response);
                }
            });

            let url = format!("http://localhost:{PORT}").parse().unwrap();
            tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::External(url))
                .title("VIGIL")
                .inner_size(1400.0, 900.0)
                .min_inner_size(1024.0, 700.0)
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}