//! @file test_api_server.rs
//! @brief Integration tests for embedded HTTP REST API server on local loopback.
//! @details Validates `GET /api/status`, `POST /api/mute`, and `POST /api/volume` endpoints
//! used by external automation tools such as Stream Deck, OBS, and home automation scripts.

use dyno_virtual_audio_mixer_lib::api_server::start_api_server;
use dyno_virtual_audio_mixer_lib::audio::matrix::MixerNode;
use dyno_virtual_audio_mixer_lib::commands::AppState;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

/// @brief Helper to send a raw HTTP request and receive the full response string.
/// @param port Local port to connect to.
/// @param request Raw HTTP request string.
/// @return Server HTTP response string.
fn send_request(port: u16, request: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port))
        .expect("Failed to connect to local API server");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("Set read timeout");
    stream.write_all(request.as_bytes()).expect("Write request");

    let mut response = String::new();
    let mut buffer = [0u8; 1024];
    while let Ok(n) = stream.read(&mut buffer) {
        if n == 0 {
            break;
        }
        response.push_str(&String::from_utf8_lossy(&buffer[..n]));
        if response.contains("\r\n\r\n") && (response.contains("}") || response.contains("204 No Content")) {
            break;
        }
    }
    response
}

/// @brief Tests the embedded API server handling GET /api/status, POST /api/mute, and POST /api/volume.
/// @return Returns `()` on assertion success.
#[test]
fn test_api_server_endpoints() {
    let state = AppState::new();

    // Populate a test node
    {
        let mut matrix = state.write_matrix().unwrap();
        matrix.add_node(MixerNode::new("mic_stream", "Streaming Mic", 1, 0).with_volume(1.0));
    }

    let port = start_api_server(state.clone(), 0).expect("API server should bind to an available port");
    thread::sleep(Duration::from_millis(150));

    // Test 1: GET /api/status
    let status_req = "GET /api/status HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    let status_resp = send_request(port, status_req);
    assert!(status_resp.contains("200 OK"));
    assert!(status_resp.contains("\"status\":\"ok\""));

    // Test 2: POST /api/mute
    let mute_body = "{\"node_id\":\"mic_stream\",\"muted\":true}";
    let mute_req = format!(
        "POST /api/mute HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}",
        mute_body.len(),
        mute_body
    );
    let mute_resp = send_request(port, &mute_req);
    assert!(mute_resp.contains("200 OK"));
    assert!(mute_resp.contains("\"muted\":true"));

    // Verify matrix state was updated
    {
        let matrix = state.read_matrix().unwrap();
        let mic = matrix.get_node("mic_stream").unwrap();
        assert!(mic.muted, "Node should be muted after API call");
    }

    // Test 3: POST /api/volume
    let vol_body = "{\"node_id\":\"mic_stream\",\"volume\":0.65}";
    let vol_req = format!(
        "POST /api/volume HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}",
        vol_body.len(),
        vol_body
    );
    let vol_resp = send_request(port, &vol_req);
    assert!(vol_resp.contains("200 OK"));
    assert!(vol_resp.contains("\"success\":true"));

    {
        let matrix = state.read_matrix().unwrap();
        let mic = matrix.get_node("mic_stream").unwrap();
        assert!((mic.volume - 0.65).abs() < 1e-4);
    }
}
