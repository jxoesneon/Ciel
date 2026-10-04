//! Port of blender_mcp_server.py — authenticated length-prefixed JSON-RPC
//! bridge to the CIEL Live Bridge Addon inside a running Blender instance.

use serde_json::{json, Value};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::common::cli::{self, ArgSpec};
use crate::common::jsonfmt;
use crate::common::py;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 9876;

fn get_auth_token() -> String {
    if let Ok(t) = std::env::var("CIEL_BRIDGE_AUTH_TOKEN") {
        let t = t.trim().to_string();
        if !t.is_empty() {
            return t;
        }
    }
    let token_file = py::expanduser("~/.ciel/.bridge_token");
    if token_file.exists() {
        if let Ok(s) = fs::read_to_string(&token_file) {
            return s.trim().to_string();
        }
    }
    String::new()
}

fn send_framed_msg(stream: &mut TcpStream, data: &Value) -> std::io::Result<()> {
    let payload = jsonfmt::dumps(data).into_bytes();
    let header = (payload.len() as u32).to_be_bytes();
    stream.write_all(&header)?;
    stream.write_all(&payload)
}

fn recv_framed_msg(stream: &mut TcpStream) -> std::io::Result<Option<Value>> {
    let mut header = [0u8; 4];
    // sock.recv(4) may return short — Python treats <4 bytes as closed.
    let mut got = 0usize;
    while got < 4 {
        match stream.read(&mut header[got..]) {
            Ok(0) => return Ok(None),
            Ok(n) => got += n,
            Err(e) => return Err(e),
        }
    }
    let msg_len = u32::from_be_bytes(header) as usize;
    if msg_len > 64 * 1024 * 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Message length exceeds 64MB limit",
        ));
    }
    let mut buf = vec![0u8; msg_len];
    let mut recvd = 0usize;
    while recvd < msg_len {
        match stream.read(&mut buf[recvd..]) {
            Ok(0) => break,
            Ok(n) => recvd += n,
            Err(e) => return Err(e),
        }
    }
    buf.truncate(recvd);
    let text = String::from_utf8_lossy(&buf).into_owned();
    match serde_json::from_str::<Value>(&text) {
        Ok(v) => Ok(Some(v)),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
    }
}

pub fn send_rpc_request(
    action: &str,
    payload: Value,
    host: &str,
    port: u16,
    timeout: f64,
) -> Value {
    let addr = format!("{}:{}", host, port);
    let dur = Duration::from_secs_f64(timeout.max(0.001));
    let stream = match TcpStream::connect_timeout(
        &match addr.parse::<std::net::SocketAddr>() {
            Ok(a) => a,
            Err(_) => {
                // resolve hostname
                match std::net::ToSocketAddrs::to_socket_addrs(&addr.as_str()) {
                    Ok(mut it) => match it.next() {
                        Some(a) => a,
                        None => {
                            return json!({
                                "status": "ERROR",
                                "error": format!("Could not resolve {}", addr)
                            })
                        }
                    },
                    Err(e) => return json!({"status": "ERROR", "error": e.to_string()}),
                }
            }
        },
        dur,
    ) {
        Ok(s) => s,
        Err(e) => {
            return if e.kind() == std::io::ErrorKind::ConnectionRefused {
                json!({
                    "status": "UNAVAILABLE",
                    "error": format!(
                        "Could not connect to Blender on {}:{}. Ensure Blender is open and CIEL Live Bridge is running.",
                        host, port
                    )
                })
            } else {
                json!({"status": "ERROR", "error": e.to_string()})
            };
        }
    };
    let mut s = stream;
    let _ = s.set_read_timeout(Some(dur));
    let _ = s.set_write_timeout(Some(dur));

    let token = get_auth_token();
    let mut req = serde_json::Map::new();
    req.insert("action".into(), json!(action));
    req.insert("auth_token".into(), json!(token));
    if let Value::Object(m) = payload {
        for (k, v) in m {
            req.insert(k, v);
        }
    }
    let req = Value::Object(req);

    let result = (|| -> std::io::Result<Value> {
        send_framed_msg(&mut s, &req)?;
        match recv_framed_msg(&mut s)? {
            Some(v) => Ok(v),
            None => Ok(json!({
                "status": "ERROR",
                "error": "Empty or closed response from Blender socket."
            })),
        }
    })();

    match result {
        Ok(v) => v,
        Err(e)
            if e.kind() == std::io::ErrorKind::WouldBlock
                || e.kind() == std::io::ErrorKind::TimedOut =>
        {
            json!({
                "status": "TIMEOUT",
                "error": format!("Blender RPC timed out after {} seconds.", py::py_num(timeout))
            })
        }
        Err(e) => json!({"status": "ERROR", "error": e.to_string()}),
    }
}

fn execute_bpy_live(code: &str, host: &str, port: u16) -> Value {
    send_rpc_request("execute_code", json!({"code": code}), host, port, 30.0)
}

fn get_live_scene_summary(host: &str, port: u16) -> Value {
    send_rpc_request("get_scene_summary", json!({}), host, port, 30.0)
}

fn capture_viewport_frame(out_png_path: &str, host: &str, port: u16) -> Value {
    let escaped_out = jsonfmt::dumps(&json!(py::abspath(out_png_path)));
    let code = format!(
        "\nimport bpy\nbpy.context.scene.render.image_settings.file_format = 'PNG'\nbpy.context.scene.render.filepath = {e}\nbpy.ops.render.opengl(write_still=True)\nprint(f\"[Live Bridge] Viewport frame saved to: {{{e}}}\")\n",
        e = escaped_out
    );
    execute_bpy_live(&code, host, port)
}

pub fn run(argv: &[String]) -> i32 {
    let args = cli::parse(
        "ciel-studio blender-mcp",
        argv,
        &[
            ArgSpec::value("exec", Some('e'), "exec"),
            ArgSpec::value("file", Some('f'), "file"),
            ArgSpec::flag("summary", Some('s'), "summary"),
            ArgSpec::value("screenshot", None, "screenshot"),
            ArgSpec::value("host", None, "host").def(DEFAULT_HOST),
            ArgSpec::int("port", None, "port").def("9876"),
        ],
    );

    let host = args.get_or("host", DEFAULT_HOST);
    let port = args.int("port", DEFAULT_PORT as i64) as u16;

    if args.flag("summary") {
        let res = get_live_scene_summary(&host, port);
        println!("{}", jsonfmt::dumps_indent(&res, 2));
        return 0;
    }
    if let Some(shot) = args.get("screenshot") {
        let res = capture_viewport_frame(shot, &host, port);
        println!("{}", jsonfmt::dumps_indent(&res, 2));
        return 0;
    }
    if let Some(file) = args.get("file") {
        if !std::path::Path::new(file).exists() {
            eprintln!("Error: File '{}' not found.", file);
            return 1;
        }
        let code = fs::read_to_string(file).unwrap_or_default();
        let res = execute_bpy_live(&code, &host, port);
        println!("{}", jsonfmt::dumps_indent(&res, 2));
        return 0;
    }
    if let Some(code) = args.get("exec") {
        let res = execute_bpy_live(code, &host, port);
        println!("{}", jsonfmt::dumps_indent(&res, 2));
        return 0;
    }
    eprintln!("usage: ciel-studio blender-mcp [--exec CODE|--file FILE|--summary|--screenshot PNG] [--host H] [--port P]");
    0
}
