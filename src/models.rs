use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: i64,
    pub name: String,
    pub address: String,
    pub serial: String,
    pub username: String,
    pub auth_method: String,
    pub password: String,
    pub private_key_path: String,
    pub os_type: String,
    pub serial_port: String,
    pub baud_rate: u32,
    pub remark: String,
    pub connected: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceInput {
    pub name: String,
    pub address: String,
    pub serial: String,
    pub username: String,
    pub auth_method: String,
    pub password: String,
    pub private_key_path: String,
    pub os_type: String,
    pub serial_port: String,
    pub baud_rate: u32,
    pub remark: String,
}

impl DeviceInput {
    pub fn from_device(d: &Device) -> Self {
        Self {
            name: d.name.clone(),
            address: d.address.clone(),
            serial: d.serial.clone(),
            username: d.username.clone(),
            auth_method: d.auth_method.clone(),
            password: d.password.clone(),
            private_key_path: d.private_key_path.clone(),
            os_type: d.os_type.clone(),
            serial_port: d.serial_port.clone(),
            baud_rate: d.baud_rate,
            remark: d.remark.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenSerialPort {
    pub port_name: String,
    pub baud_rate: u32,
    pub opened_at: String,
    pub recording: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdbDevice {
    pub serial: String,
    pub state: String,
    pub model: String,
    pub device: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareInfo {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashDevice {
    pub id: String,
    pub name: String,
    pub mode: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlashStatus {
    pub running: bool,
    pub phase: String,
    pub progress: f32,
    pub message: String,
    pub log: Vec<String>,
    pub finished_at: Option<String>,
    pub success: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFileInfo {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickCommandWindow {
    pub id: i64,
    pub name: String,
    pub commands: Vec<String>,
    pub interval_ms: u64,
    pub append_newline: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuickCommandInput {
    pub name: String,
    pub commands: Vec<String>,
    pub interval_ms: u64,
    pub append_newline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub command: String,
    pub started_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerStatus {
    pub running: bool,
    pub port: u16,
    pub token_required: bool,
    pub tools: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub log_enabled: bool,
    pub log_max_file_size: u64,
    pub log_max_days: u32,
    pub adb_path: String,
    pub host_root: String,
    pub host_shell_enabled: bool,
    pub mcp_enabled: bool,
    pub mcp_port: u16,
    pub mcp_token: String,
    pub tunnel_enabled: bool,
    pub tunnel_host: String,
    pub tunnel_ssh_port: u16,
    pub tunnel_user: String,
    pub tunnel_auth_method: String,
    pub tunnel_private_key: String,
    pub tunnel_password: String,
    pub tunnel_remote_bind: String,
    pub tunnel_remote_port: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            log_enabled: true,
            log_max_file_size: 16 * 1024 * 1024,
            log_max_days: 30,
            adb_path: "adb".into(),
            host_root: String::new(),
            host_shell_enabled: false,
            mcp_enabled: false,
            mcp_port: 9847,
            mcp_token: String::new(),
            tunnel_enabled: false,
            tunnel_host: String::new(),
            tunnel_ssh_port: 22,
            tunnel_user: String::new(),
            tunnel_auth_method: "key".into(),
            tunnel_private_key: String::new(),
            tunnel_password: String::new(),
            tunnel_remote_bind: "127.0.0.1".into(),
            tunnel_remote_port: 0,
        }
    }
}
