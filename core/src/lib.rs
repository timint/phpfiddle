use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::Duration;
use std::{fs, thread};

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri::Manager;
use tokio::io::AsyncWriteExt;

static PHP_CHILD: Lazy<Mutex<Option<Child>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Deserialize)]
struct Config {
  php: PhpConfig,
  window: WindowConfig,
}

#[derive(Debug, Deserialize)]
struct PhpConfig {
  directory: String,
  docroot: String,
  port: u16,
  #[serde(default)]
  fastcgi_port: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct WindowConfig {
  width: f64,
  height: f64,
}

/// Resolve the bundled `ui/` directory regardless of the current working directory.
///
/// During `tauri dev` the app is launched from `core/`, so a relative
/// `ui/` path would resolve to `core/ui` (which does not exist).
/// The resources live next to the project root, one level above `core`.
fn resolve_ui_dir() -> std::path::PathBuf {
  let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let manifest = strip_extended_prefix(&manifest);

  let mut candidates = Vec::new();
  // Compiled-in location: <project root>/ui (dev + bundled builds).
  candidates.push(manifest.join("..").join("ui"));
  // Relative to the current working directory.
  if let Ok(cwd) = std::env::current_dir() {
    candidates.push(cwd.join("ui"));
    candidates.push(cwd.join("..").join("ui"));
  }
  // Relative to the running executable (bundled install layout).
  if let Ok(exe) = std::env::current_exe() {
    if let Some(dir) = exe.parent() {
      candidates.push(dir.join("ui"));
      candidates.push(dir.join("..").join("ui"));
      candidates.push(dir.join("..").join("..").join("ui"));
      candidates.push(dir.join("..").join("..").join("..").join("ui"));
    }
  }

  for c in &candidates {
    if c.is_dir() {
      let resolved = c.canonicalize().unwrap_or_else(|_| c.clone());
      // `canonicalize` returns the extended-length `\\?\` prefix on Windows.
      // The bundled PHP binaries crash (STATUS_ACCESS_VIOLATION) when handed
      // such a path, so strip it.
      return strip_extended_prefix(&resolved);
    }
  }

  // Nothing found: return the compiled-in default so error messages stay meaningful.
  strip_extended_prefix(&manifest.join("..").join("ui"))
}

/// Remove the Windows extended-length path prefix (`\\?\`) from a path.
fn strip_extended_prefix(path: &std::path::Path) -> std::path::PathBuf {
  let s = path.to_string_lossy();
  if let Some(stripped) = s.strip_prefix(r"\\?\") {
    // Keep the UNC form intact (`\\?\UNC\server\share` -> `\\server\share`).
    if let Some(unc) = stripped.strip_prefix(r"UNC\") {
      return std::path::PathBuf::from(format!(r"\\{}", unc));
    }
    return std::path::PathBuf::from(stripped);
  }
  path.to_path_buf()
}

/// Spawn a PHP FastCGI listener (php-cgi -b host:fcgi_port)
fn spawn_php_fastcgi(php_dir: &str, fcgi_port: u16) -> std::io::Result<Child> {
  use std::ffi::OsString;
  let mut tried = Vec::new();

  // Candidate executables (absolute within app dir)
  let mut candidates = Vec::new();
  let base = resolve_ui_dir().join(php_dir);
  let mut p = base.join("php-cgi");
  if cfg!(target_os = "windows") { p.set_extension("exe"); }
  candidates.push(p);
  let mut p2 = base.join("php");
  if cfg!(target_os = "windows") { p2.set_extension("exe"); }
  candidates.push(p2);

  // Also try system binaries relying on PATH
  candidates.push(OsString::from("php-cgi").into());
  candidates.push(OsString::from("php").into());

  // Use the bundled php.ini (if present) so the bundled extensions in
  // `ui/php/ext` are found instead of a system-wide php.ini.
  let php_ini = resolve_ui_dir().join(php_dir).join("php.ini");
  let php_ini = if php_ini.is_file() { Some(php_ini) } else { None };

  let addr = format!("127.0.0.1:{}", fcgi_port);
  for candidate in candidates {
    tried.push(candidate.to_string_lossy().to_string());
    let mut cmd = Command::new(&candidate);
    cmd.arg("-b").arg(&addr);
    if let Some(ini) = &php_ini {
      cmd.arg("-c").arg(ini);
    }
    // Run from the PHP directory so relative paths (e.g. `extension_dir`)
    // resolve to the bundled copies.
    cmd.current_dir(resolve_ui_dir().join(php_dir));
    match cmd.spawn() {
      Ok(child) => return Ok(child),
      Err(e) => {
        // continue to next candidate
        eprintln!("spawn {:?} failed: {}", candidate, e);
        continue;
      }
    }
  }

  Err(std::io::Error::new(
    std::io::ErrorKind::NotFound,
    format!("No php executable found. Tried: {}", tried.join(", ")),
  ))
}

fn check_tcp_port(port: u16) -> bool {
  use std::net::TcpStream;
  TcpStream::connect(("127.0.0.1", port)).is_ok()
}

fn wait_for_tcp_port(port: u16) {
  for _ in 0..30 {
    if check_tcp_port(port) {
      return;
    }
    thread::sleep(Duration::from_millis(500));
  }
}

fn kill_php_child() {
  if let Ok(mut guard) = PHP_CHILD.lock() {
    if let Some(mut c) = guard.take() {
      let _ = c.kill();
      let _ = c.wait();
    }
  }
}

/// Start a tiny HTTP server that proxies .php requests to a FastCGI listener
fn start_http_to_fastcgi_proxy(listen_port: u16, fcgi_addr: &str, docroot: &str) -> std::io::Result<Child> {
  // We will spawn a detached thread running tiny-http server instead of a child process.
  // Use std::thread::spawn and return a dummy Child-like process handle by spawning `cmd /c start` on Windows is not necessary.
  let listen = format!("127.0.0.1:{}", listen_port);
  let docroot = std::path::PathBuf::from(docroot);
  let fcgi_addr = fcgi_addr.to_string();

  // Create a Tokio runtime for FastCGI async operations
  let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

  thread::Builder::new()
    .name("http-fcgi-proxy".into())
    .spawn(move || {
      // tiny-http is synchronous and easy to use for small projects
      let server = tiny_http::Server::http(&listen).expect("failed to bind proxy");

      for mut request in server.incoming_requests() {
        let url = request.url().to_string();
        let method = request.method().as_str().to_string();
        let path = url.split('?').next().unwrap_or("/");
        let local_path = {
          let mut p = docroot.clone();
          // Safe-join: remove leading '/'
          let rel = path.trim_start_matches('/');
          if rel.is_empty() {
            p.push("index.php");
          } else {
            p.push(rel);
          }
          p
        };

        // If the file exists and is not a PHP file, serve as static file
        if local_path.exists() && local_path.extension().map(|e| e != "php").unwrap_or(true) {
          // serve file directly
          match std::fs::read(&local_path) {
            Ok(body) => {
              let content_type = mime_guess::from_path(&local_path).first_or_text_plain();
              let response = tiny_http::Response::from_data(body).with_header(
                tiny_http::Header::from_bytes("Content-Type", content_type.as_ref()).unwrap(),
              );
              let _ = request.respond(response);
            }
            Err(_) => {
              let _ = request.respond(tiny_http::Response::from_string("Not found").with_status_code(404));
            }
          }
          continue;
        }

        // Otherwise, treat it as a PHP request and proxy to FastCGI using async client
        // Build FastCGI params
        let script_filename = local_path.to_string_lossy().to_string();
        let query = url.split('?').nth(1).unwrap_or("");
        let content_type = request
          .headers()
          .iter()
          .find(|h| h.field.equiv("Content-Type"))
          .map(|h| h.value.as_str().to_string())
          .unwrap_or_default();
        let mut params = std::collections::HashMap::new();
        params.insert("SCRIPT_FILENAME".to_string(), script_filename.clone());
        params.insert("SCRIPT_NAME".to_string(), path.to_string());
        params.insert("REQUEST_METHOD".to_string(), method.clone());
        params.insert("QUERY_STRING".to_string(), query.to_string());
        params.insert("DOCUMENT_ROOT".to_string(), docroot.to_string_lossy().to_string());
        params.insert("SERVER_SOFTWARE".to_string(), "php-fiddle/tauri".to_string());
        params.insert("SERVER_PROTOCOL".to_string(), "HTTP/1.1".to_string());
        params.insert("REQUEST_URI".to_string(), url.to_string());
        params.insert("REMOTE_ADDR".to_string(), "127.0.0.1".to_string());
        params.insert("SERVER_ADDR".to_string(), "127.0.0.1".to_string());
        params.insert("SERVER_PORT".to_string(), listen_port.to_string());
        params.insert("SERVER_NAME".to_string(), "127.0.0.1".to_string());
        params.insert("CONTENT_TYPE".to_string(), content_type);

        // Read request body
        let mut body = Vec::new();
        let _ = request.as_reader().read_to_end(&mut body);
        // PHP needs the exact body length to know when the STDIN stream ends.
        params.insert("CONTENT_LENGTH".to_string(), body.len().to_string());

        // Use Tokio runtime to perform async FastCGI operations
        let fcgi_addr_clone = fcgi_addr.clone();
        let params_clone = params.clone();
        let body_clone = body.clone();
        let response_result = rt.block_on(async move {
          match tokio::net::TcpStream::connect(&fcgi_addr_clone).await {
            Ok(stream) => {
              let mut client = fastcgi_client::Client::new_keep_alive_tokio(stream);

              let mut fcgi_params = fastcgi_client::Params::default();
              for (k, v) in params_clone.iter() {
                fcgi_params.insert(
                  std::borrow::Cow::Borrowed(k.as_str()),
                  std::borrow::Cow::Borrowed(v.as_str()),
                );
              }

              // Feed the request body to PHP as FastCGI stdin. The duplex
              // stream lets us hand the body to the client as an async reader.
              let (mut writer, reader) = tokio::io::duplex(body_clone.len().max(1024));
              let write_body = body_clone.clone();
              tokio::spawn(async move {
                let _ = writer.write_all(&write_body).await;
                let _ = writer.shutdown().await;
              });

              let req = fastcgi_client::Request::new_tokio(fcgi_params, reader);

              match client.execute(req).await {
                Ok(resp) => Ok(resp.stdout.unwrap_or_default()),
                Err(e) => Err(format!("fastcgi execute error: {}", e)),
              }
            }
            Err(e) => Err(format!("connect err: {}", e)),
          }
        });

        match response_result {
          Ok(resp_bytes) => {
            if let Some(pos) = resp_bytes.windows(4).position(|w| w==b"\r\n\r\n") {
              let (head, body) = resp_bytes.split_at(pos+4);
              let head_str = String::from_utf8_lossy(head);
              let mut status_code = 200;
              let mut response = tiny_http::Response::from_data(body.to_vec());
              for line in head_str.lines() {
                if line.to_lowercase().starts_with("status:") {
                  if let Some(s) = line.split(':').nth(1) {
                    if let Ok(code) = s.trim().split_whitespace().next().unwrap_or("200").parse::<u16>() {
                      status_code = code;
                    }
                  }
                } else if let Some((k,v)) = line.split_once(":") {
                  if let Ok(h) = tiny_http::Header::from_bytes(k.trim(), v.trim()) {
                    let _ = response.add_header(h);
                  }
                }
              }
              let _ = request.respond(response.with_status_code(status_code));
            } else {
              let _ = request.respond(tiny_http::Response::from_data(resp_bytes));
            }
          }
          Err(e) => {
            let _ = request.respond(tiny_http::Response::from_string(format!("FastCGI error: {}", e)).with_status_code(502));
          }
        }
      }
    })?;

  Ok(std::process::Command::new(if cfg!(target_os = "windows") { "cmd" } else { "true" })
    .args(if cfg!(target_os = "windows") {
      &["/C", "exit", "0"][..]
    } else {
      &[""][..]
    })
    .spawn()
    .unwrap_or_else(|_| panic!("failed to create dummy child")))
}

fn read_config() -> Config {
  // Try several candidate locations to support running from `core` during dev
  let mut tried = Vec::new();
  let ui_dir = resolve_ui_dir();
  let candidates = [
    ui_dir.join("config.json"),
    std::path::PathBuf::from("ui/config.json"),
    std::path::PathBuf::from("../ui/config.json"),
    std::path::PathBuf::from("../../ui/config.json"),
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../ui/config.json")),
  ];

  for p in &candidates {
    tried.push(p.to_string_lossy().to_string());
    if p.exists() {
      let data = fs::read_to_string(p).expect(&format!("Failed to read {}", p.display()));
      return serde_json::from_str(&data).expect("Invalid config.json");
    }
  }

  panic!("Failed to read ui/config.json: tried paths: {}", tried.join(", "))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      let config = read_config();

      // Resolve the docroot relative to the bundled `ui/` directory so it does not
      // depend on the current working directory.
      let docroot = {
        let p = std::path::PathBuf::from(&config.php.docroot);
        if p.is_absolute() {
          p
        } else {
          resolve_ui_dir().join(p)
        }
      };

      // Decide FastCGI and HTTP ports
      let fcgi_port = config.php.fastcgi_port.unwrap_or(9000);

      // Spawn PHP FastCGI listener
      match spawn_php_fastcgi(&config.php.directory, fcgi_port) {
        Ok(child) => {
          if let Ok(mut guard) = PHP_CHILD.lock() {
            *guard = Some(child);
          }
          wait_for_tcp_port(fcgi_port);
        }
        Err(e) => {
          println!("Failed to spawn PHP FastCGI: {}", e);
        }
      }

      // Start the HTTP -> FastCGI proxy that listens on config.php.port and forwards .php to fcgi_port
      if let Err(e) = start_http_to_fastcgi_proxy(config.php.port, &format!("127.0.0.1:{}", fcgi_port), &docroot.to_string_lossy()) {
        println!("Failed to start proxy server: {}", e);
      }

      // Create main window loading the proxy HTTP server
      let url = format!("http://127.0.0.1:{}/", config.php.port);
      if let Some(window) = app.get_webview_window("main") {
        // Set the window size from configuration
        let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize { width: config.window.width, height: config.window.height }));
        let js = format!("window.location.replace('{}');", url);
        let _ = window.eval(&js);

        // Kill PHP child when the window is requested to close
        window.on_window_event(move |event| match event {
          tauri::WindowEvent::CloseRequested { api, .. } => {
            let _ = api;
            kill_php_child();
          }
          _ => {}
        });
      }

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
