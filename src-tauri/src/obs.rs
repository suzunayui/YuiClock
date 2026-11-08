//! A tiny HTTP server on 127.0.0.1 that serves the clock as a page for an OBS browser source.
//!
//! The page polls `/settings`, so it follows changes made in the settings window.

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use tauri::{AppHandle, Manager};

use crate::SettingsStore;

pub const PORT: u16 = 51730;

/// Whether the server is listening, kept so the settings window can explain a failure.
pub struct ObsServer {
    pub error: Option<String>,
}

const FILES: &[(&str, &str, &str)] = &[
    ("/", "text/html; charset=utf-8", include_str!("../../ui/obs.html")),
    ("/style.css", "text/css; charset=utf-8", include_str!("../../ui/style.css")),
    ("/format.js", "text/javascript; charset=utf-8", include_str!("../../ui/format.js")),
    ("/clock.js", "text/javascript; charset=utf-8", include_str!("../../ui/clock.js")),
    ("/obs.js", "text/javascript; charset=utf-8", include_str!("../../ui/obs.js")),
];

pub fn url() -> String {
    format!("http://127.0.0.1:{PORT}/")
}

pub fn start(app: AppHandle) -> ObsServer {
    match TcpListener::bind(("127.0.0.1", PORT)) {
        Ok(listener) => {
            thread::spawn(move || {
                for stream in listener.incoming().flatten() {
                    let app = app.clone();
                    thread::spawn(move || {
                        let _ = handle(&app, stream);
                    });
                }
            });
            ObsServer { error: None }
        }
        Err(error) => ObsServer {
            error: Some(format!(
                "ポート {PORT} を使えないため、OBS用の表示を開始できませんでした（{error}）。"
            )),
        },
    }
}

fn handle(app: &AppHandle, mut stream: TcpStream) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut buf = [0u8; 8192];
    let len = stream.read(&mut buf)?;
    let request = String::from_utf8_lossy(&buf[..len]);
    let (method, target, host) = parse_request(&request);

    // Only answer requests addressed to us by name, so a web page that rebinds its own
    // domain to 127.0.0.1 can't read anything from here.
    let host_ok = host.is_some_and(|host| {
        host == format!("127.0.0.1:{PORT}") || host == format!("localhost:{PORT}")
    });
    if method != "GET" || !host_ok {
        return respond(&mut stream, "403 Forbidden", "text/plain; charset=utf-8", b"Forbidden");
    }

    let path = target.split('?').next().unwrap_or("/");
    if path == "/settings" {
        let json = app
            .try_state::<SettingsStore>()
            .and_then(|store| serde_json::to_vec(&*store.0.lock().unwrap()).ok())
            .unwrap_or_else(|| b"{}".to_vec());
        return respond(&mut stream, "200 OK", "application/json", &json);
    }
    match FILES.iter().find(|(file, _, _)| *file == path) {
        Some((_, mime, body)) => respond(&mut stream, "200 OK", mime, body.as_bytes()),
        None => respond(&mut stream, "404 Not Found", "text/plain; charset=utf-8", b"Not Found"),
    }
}

/// The method, request target and Host header of an HTTP request.
fn parse_request(request: &str) -> (&str, &str, Option<&str>) {
    let mut lines = request.lines();
    let mut first = lines.next().unwrap_or("").split_whitespace();
    let method = first.next().unwrap_or("");
    let target = first.next().unwrap_or("");
    let host = lines
        .take_while(|line| !line.is_empty())
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.trim().eq_ignore_ascii_case("host"))
        .map(|(_, value)| value.trim());
    (method, target, host)
}

fn respond(stream: &mut TcpStream, status: &str, mime: &str, body: &[u8]) -> std::io::Result<()> {
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

#[cfg(test)]
mod tests {
    use super::parse_request;

    #[test]
    fn reads_method_target_and_host() {
        let request = "GET /settings?x=1 HTTP/1.1\r\nHOST: 127.0.0.1:51730\r\nAccept: */*\r\n\r\n";
        assert_eq!(parse_request(request), ("GET", "/settings?x=1", Some("127.0.0.1:51730")));
    }

    #[test]
    fn missing_host_is_none() {
        assert_eq!(parse_request("GET / HTTP/1.0\r\n\r\n"), ("GET", "/", None));
    }
}
