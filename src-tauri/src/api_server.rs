//! @file api_server.rs
//! @brief Lightweight embedded HTTP REST API server for Stream Deck and OBS integration.
//! @details Binds to `127.0.0.1:49152` and listens for local HTTP requests to query mixer status,
//! toggle mute states, adjust volume, and load profile presets without third-party dependencies.

use crate::commands::state::AppState;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

/// @brief Default port number for the embedded local control API server.
pub const DEFAULT_API_PORT: u16 = 49152;

/// @brief Payload format for mute configuration requests.
#[derive(Debug, Deserialize, Serialize)]
pub struct MuteRequest {
    /// @brief Target mixer node ID to mute or unmute.
    pub node_id: String,
    /// @brief Target mute state.
    pub muted: bool,
}

/// @brief Payload format for volume adjustment requests.
#[derive(Debug, Deserialize, Serialize)]
pub struct VolumeRequest {
    /// @brief Target mixer node ID.
    pub node_id: String,
    /// @brief Linear volume level (0.0 to 1.5).
    pub volume: f32,
}

/// @brief Payload format for profile preset switching requests.
#[derive(Debug, Deserialize, Serialize)]
pub struct ProfileRequest {
    /// @brief Name of the profile preset to load.
    pub profile_name: String,
}

/// @brief Handles an incoming HTTP request on an accepted TCP socket stream.
/// @param mut stream Connected client TCP stream.
/// @param state Shared application state container.
fn handle_client(mut stream: TcpStream, state: &AppState) {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));

    let mut buffer = [0u8; 4096];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    let mut lines = request_str.lines();
    let request_line = match lines.next() {
        Some(line) => line,
        None => return,
    };

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");

    // Extract optional body after double CRLF
    let body = if let Some(pos) = request_str.find("\r\n\r\n") {
        &request_str[pos + 4..]
    } else {
        ""
    };

    // CORS preflight response
    if method == "OPTIONS" {
        let response = "HTTP/1.1 204 No Content\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
            Access-Control-Allow-Headers: Content-Type\r\n\
            Content-Length: 0\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    let (status_code, json_response) = match (method, path) {
        ("GET", "/api/status") => {
            let meter_snapshot = state.generate_meter_snapshot();
            let is_running = state.is_running.load(Ordering::Relaxed);
            let response_body = serde_json::json!({
                "status": "ok",
                "running": is_running,
                "meters": meter_snapshot.meters,
            });
            ("200 OK", response_body.to_string())
        }
        ("POST", "/api/mute") => {
            if let Ok(req) = serde_json::from_str::<MuteRequest>(body.trim()) {
                if let Ok(mut matrix) = state.write_matrix() {
                    if let Some(node) = matrix.get_node_mut(&req.node_id) {
                        node.muted = req.muted;
                        ("200 OK", serde_json::json!({"success": true, "node_id": req.node_id, "muted": req.muted}).to_string())
                    } else {
                        ("404 Not Found", serde_json::json!({"error": "Node not found"}).to_string())
                    }
                } else {
                    ("500 Internal Error", serde_json::json!({"error": "Lock error"}).to_string())
                }
            } else {
                ("400 Bad Request", serde_json::json!({"error": "Invalid JSON body"}).to_string())
            }
        }
        ("POST", "/api/volume") => {
            if let Ok(req) = serde_json::from_str::<VolumeRequest>(body.trim()) {
                if let Ok(mut matrix) = state.write_matrix() {
                    if let Some(node) = matrix.get_node_mut(&req.node_id) {
                        node.volume = req.volume.clamp(0.0, 1.5);
                        ("200 OK", serde_json::json!({"success": true, "node_id": req.node_id, "volume": node.volume}).to_string())
                    } else {
                        ("404 Not Found", serde_json::json!({"error": "Node not found"}).to_string())
                    }
                } else {
                    ("500 Internal Error", serde_json::json!({"error": "Lock error"}).to_string())
                }
            } else {
                ("400 Bad Request", serde_json::json!({"error": "Invalid JSON body"}).to_string())
            }
        }
        ("POST", "/api/profile") => {
            if let Ok(req) = serde_json::from_str::<ProfileRequest>(body.trim()) {
                if let Ok(mgr) = state.read_profiles() {
                    if let Ok(preset) = mgr.load_profile(&req.profile_name) {
                        if let Ok(mut matrix) = state.write_matrix() {
                            *matrix = preset.graph.to_routing_matrix();
                            ("200 OK", serde_json::json!({"success": true, "profile": req.profile_name}).to_string())
                        } else {
                            ("500 Internal Error", serde_json::json!({"error": "Lock error"}).to_string())
                        }
                    } else {
                        ("404 Not Found", serde_json::json!({"error": "Profile not found"}).to_string())
                    }
                } else {
                    ("500 Internal Error", serde_json::json!({"error": "Lock error"}).to_string())
                }
            } else {
                ("400 Bad Request", serde_json::json!({"error": "Invalid JSON body"}).to_string())
            }
        }
        _ => ("404 Not Found", serde_json::json!({"error": "Not found"}).to_string()),
    };

    let response = format!(
        "HTTP/1.1 {}\r\n\
        Content-Type: application/json\r\n\
        Access-Control-Allow-Origin: *\r\n\
        Content-Length: {}\r\n\r\n{}",
        status_code,
        json_response.len(),
        json_response
    );
    let _ = stream.write_all(response.as_bytes());
}

/// @brief Starts the background HTTP API listener on `127.0.0.1:port`.
/// @details Binds the TCP listener. If the specified port is unavailable or restricted by OS firewall/Hyper-V,
/// falls back to an available ephemeral port. Spawns a dedicated background thread for client processing.
/// @param state Shared application state container.
/// @param port Local TCP port number to bind (pass 0 for automatic ephemeral port assignment).
/// @return `Some(u16)` containing the actual bound port number, or `None` if binding failed.
pub fn start_api_server(state: AppState, port: u16) -> Option<u16> {
    let listener = if port > 0 {
        match TcpListener::bind(format!("127.0.0.1:{}", port)) {
            Ok(l) => Some(l),
            Err(e) => {
                eprintln!("[API Server] Warning: Could not bind to 127.0.0.1:{}: {}. Falling back to ephemeral port.", port, e);
                TcpListener::bind("127.0.0.1:0").ok()
            }
        }
    } else {
        TcpListener::bind("127.0.0.1:0").ok()
    }?;

    let bound_port = listener.local_addr().ok()?.port();
    println!("[API Server] Bound and listening on 127.0.0.1:{}", bound_port);

    thread::spawn(move || {
        for stream in listener.incoming() {
            if !state.is_running.load(Ordering::Relaxed) {
                break;
            }
            if let Ok(s) = stream {
                handle_client(s, &state);
            }
        }
    });

    Some(bound_port)
}
