use crate::models::McpServerStatus;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub const MCP_TOOLS: &[(&str, &str)] = &[
    ("echo", "原样返回输入文本"),
    ("now", "返回当前时间"),
    ("list_serial_ports", "列出本机串口"),
];

pub struct McpService {
    stop: Arc<AtomicBool>,
    handle: Mutex<Option<std::thread::JoinHandle<()>>>,
    running: AtomicBool,
    port: Mutex<u16>,
    token: Mutex<String>,
}

impl McpService {
    pub fn new() -> Self {
        Self {
            stop: Arc::new(AtomicBool::new(false)),
            handle: Mutex::new(None),
            running: AtomicBool::new(false),
            port: Mutex::new(9847),
            token: Mutex::new(String::new()),
        }
    }

    pub fn start(&self, port: u16, token: String) -> Result<(), String> {
        self.stop();
        let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| format!("绑定端口 {port} 失败：{e}"))?;
        let stop = self.stop.clone();
        let token_for_store = token.clone();
        let token = Arc::new(Mutex::new(token));
        let handle = std::thread::spawn(move || {
            let _ = listener.set_nonblocking(true);
            loop {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                match listener.accept() {
                    Ok((stream, _)) => {
                        let t = Arc::clone(&token);
                        std::thread::spawn(move || handle_client(stream, t));
                    }
                    Err(_) => std::thread::sleep(std::time::Duration::from_millis(50)),
                }
            }
        });
        *self.handle.lock().unwrap() = Some(handle);
        self.running.store(true, Ordering::Relaxed);
        *self.port.lock().unwrap() = port;
        *self.token.lock().unwrap() = token_for_store;
        Ok(())
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.lock().unwrap().take() {
            let _ = h.join();
        }
        self.stop.store(false, Ordering::Relaxed);
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn status(&self) -> McpServerStatus {
        McpServerStatus {
            running: self.running.load(Ordering::Relaxed),
            port: *self.port.lock().unwrap(),
            token_required: !self.token.lock().unwrap().is_empty(),
            tools: MCP_TOOLS.iter().map(|(n, _)| n.to_string()).collect(),
            error: None,
        }
    }
}

fn handle_client(stream: TcpStream, token: Arc<Mutex<String>>) {
    let mut writer = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let Ok(line) = line else { break };
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        let resp = dispatch(&line, &token);
        let _ = writeln!(writer, "{resp}");
    }
}

fn dispatch(line: &str, token: &Arc<Mutex<String>>) -> String {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
        return json_err(None, -32700, "Invalid JSON");
    };
    let id = v.get("id").cloned();
    let method = v.get("method").and_then(|m| m.as_str()).unwrap_or("").to_string();
    let params = v.get("params").cloned().unwrap_or(serde_json::Value::Null);

    let stored = token.lock().unwrap();
    if !stored.is_empty() && params.get("token").and_then(|x| x.as_str()) != Some(stored.as_str()) {
        return json_err(id, -32000, "无效的访问令牌");
    }

    match method.as_str() {
        "initialize" => json_ok(
            id,
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "openworkbench-mcp", "version": "0.1.0" }
            }),
        ),
        "tools/list" => json_ok(
            id,
            serde_json::json!({
                "tools": MCP_TOOLS.iter().map(|(n, d)| serde_json::json!({"name": n, "description": d})).collect::<Vec<_>>()
            }),
        ),
        "tools/call" => {
            let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
            let text = match name.as_str() {
                "echo" => params
                    .get("arguments")
                    .and_then(|a| a.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string(),
                "now" => crate::services::util::now_iso(),
                "list_serial_ports" => crate::services::serial::SerialService::list_ports().join(", "),
                other => return json_err(id, -32602, format!("未知工具：{other}")),
            };
            json_ok(
                id,
                serde_json::json!({ "content": [{ "type": "text", "text": text }] }),
            )
        }
        "ping" => json_ok(id, serde_json::json!({})),
        other => json_err(id, -32601, format!("未知方法：{other}")),
    }
}

fn json_ok(id: Option<serde_json::Value>, result: serde_json::Value) -> String {
    serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": result }).to_string()
}

fn json_err(id: Option<serde_json::Value>, code: i64, message: impl ToString) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message.to_string() }
    })
    .to_string()
}
