use crate::models::*;
use crate::services::firmware::FlashService;
use crate::services::logs::LogService;
use crate::services::mcp::McpService;
use crate::services::serial::SerialService;
use crate::services::tunnel::TunnelService;
use dioxus::prelude::*;
use dioxus::signals::WritableExt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Devices,
    Serial,
    Logs,
    Firmware,
    Adb,
    Host,
    Quick,
    Settings,
}

pub struct AppStore {
    pub page: Signal<Page>,
    pub serial_device: Signal<Option<i64>>,
    pub devices: Signal<Vec<Device>>,
    pub settings: Signal<Settings>,
    pub quick_windows: Signal<Vec<QuickCommandWindow>>,
    pub serial_buffer: SyncSignal<String>,
    pub open_ports: SyncSignal<Vec<OpenSerialPort>>,
    pub serial_logs: SyncSignal<Vec<LogFileInfo>>,
    pub flash_status: SyncSignal<FlashStatus>,
    pub tunnel_status: SyncSignal<TunnelStatus>,
    pub mcp_status: SyncSignal<McpServerStatus>,
    pub serial: SerialService,
    pub logs: LogService,
    pub flash: FlashService,
    pub tunnel: TunnelService,
    pub mcp: McpService,
    pub data_dir: PathBuf,
}

impl AppStore {
    pub fn new() -> Self {
        let data_dir = data_dir();
        for sub in ["logs", "firmware"] {
            let _ = std::fs::create_dir_all(data_dir.join(sub));
        }
        let devices = load_json::<Vec<Device>>(&data_dir.join("devices.json")).unwrap_or_default();
        let settings = load_json::<Settings>(&data_dir.join("settings.json")).unwrap_or_default();
        let quick_windows =
            load_json::<Vec<QuickCommandWindow>>(&data_dir.join("quick_commands.json")).unwrap_or_default();
        let logs = LogService::new(data_dir.join("logs"), settings.log_max_file_size);
        let serial_logs = logs.list();
        Self {
            page: Signal::new(Page::Devices),
            serial_device: Signal::new(None),
            devices: Signal::new(devices),
            settings: Signal::new(settings),
            quick_windows: Signal::new(quick_windows),
            serial_buffer: SyncSignal::new_maybe_sync(String::new()),
            open_ports: SyncSignal::new_maybe_sync(Vec::new()),
            serial_logs: SyncSignal::new_maybe_sync(serial_logs),
            flash_status: SyncSignal::new_maybe_sync(FlashStatus::default()),
            tunnel_status: SyncSignal::new_maybe_sync(TunnelStatus {
                running: false,
                pid: None,
                command: String::new(),
                started_at: None,
                last_error: None,
            }),
            mcp_status: SyncSignal::new_maybe_sync(McpServerStatus {
                running: false,
                port: 9847,
                token_required: false,
                tools: Vec::new(),
                error: None,
            }),
            serial: SerialService::new(),
            logs,
            flash: FlashService::new(data_dir.join("firmware")),
            tunnel: TunnelService::new(),
            mcp: McpService::new(),
            data_dir,
        }
    }

    pub fn save_devices(&self) {
        let devices = self.devices.read().clone();
        save_json(&self.data_dir.join("devices.json"), &devices);
    }

    pub fn save_quick(&self) {
        let windows = self.quick_windows.read().clone();
        save_json(&self.data_dir.join("quick_commands.json"), &windows);
    }

    pub fn refresh_logs(&self) {
        let mut serial_logs = self.serial_logs;
        serial_logs.set(self.logs.list());
    }

    pub fn apply_settings(&mut self, s: Settings) -> Result<(), String> {
        let mut errs = Vec::new();
        self.tunnel.stop(self.tunnel_status);
        self.mcp.stop();
        self.settings.set(s.clone());
        save_json(&self.data_dir.join("settings.json"), &s);
        self.logs.max_file_size = s.log_max_file_size;
        if s.tunnel_enabled {
            if let Err(e) = self.tunnel.start(&s, self.tunnel_status) {
                errs.push(format!("隧道：{e}"));
            }
        }
        if s.mcp_enabled {
            if let Err(e) = self.mcp.start(s.mcp_port, s.mcp_token.clone()) {
                errs.push(format!("MCP：{e}"));
            }
        }
        self.mcp_status.set(self.mcp.status());
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs.join("；"))
        }
    }
}

fn data_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("openworkbench");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".openworkbench");
    }
    PathBuf::from(".")
}

fn load_json<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Option<T> {
    std::fs::read(path)
        .ok()
        .and_then(|d| serde_json::from_slice(&d).ok())
}

fn save_json<T: serde::Serialize>(path: &std::path::Path, value: &T) {
    if let Ok(s) = serde_json::to_string_pretty(value) {
        let _ = std::fs::write(path, s);
    }
}
