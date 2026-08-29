use crate::models::{AdbDevice, HostEntry};
use crate::services::util::{iso_from_secs, run_cmd};
use std::time::UNIX_EPOCH;

pub struct AdbService;

#[allow(dead_code)]
impl AdbService {
    pub fn cmd(&self, adb_path: &str, args: &[&str], timeout: u64) -> Result<String, String> {
        let path = if adb_path.trim().is_empty() {
            "adb".to_string()
        } else {
            adb_path.to_string()
        };
        run_cmd(&path, args, timeout)
    }

    pub fn scan(&self, adb_path: &str) -> Result<Vec<AdbDevice>, String> {
        let out = self.cmd(adb_path, &["devices", "-l"], 10)?;
        let mut devs = Vec::new();
        for line in out.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() || line.starts_with('*') {
                continue;
            }
            let mut parts = line.split_whitespace();
            let serial = parts.next().unwrap_or("").to_string();
            let state = parts.next().unwrap_or("").to_string();
            let mut model = String::new();
            let mut device = String::new();
            for tok in parts {
                if let Some(v) = tok.strip_prefix("model:") {
                    model = v.to_string();
                }
                if let Some(v) = tok.strip_prefix("device:") {
                    device = v.to_string();
                }
            }
            devs.push(AdbDevice { serial, state, model, device });
        }
        Ok(devs)
    }

    pub fn connect(&self, adb_path: &str, addr: &str) -> Result<String, String> {
        self.cmd(adb_path, &["connect", addr], 15)
    }

    pub fn disconnect(&self, adb_path: &str, addr: &str) -> Result<String, String> {
        self.cmd(adb_path, &["disconnect", addr], 10)
    }

    pub fn shell(&self, adb_path: &str, serial: &str, command: &str) -> Result<String, String> {
        self.cmd(adb_path, &["-s", serial, "shell", command], 30)
    }

    pub fn push(&self, adb_path: &str, serial: &str, local: &str, remote: &str) -> Result<String, String> {
        self.cmd(adb_path, &["-s", serial, "push", local, remote], 60)
    }

    pub fn pull(&self, adb_path: &str, serial: &str, remote: &str, local: &str) -> Result<String, String> {
        self.cmd(adb_path, &["-s", serial, "pull", remote, local], 60)
    }

    pub fn reboot(&self, adb_path: &str, serial: &str, mode: &str) -> Result<String, String> {
        if mode.is_empty() {
            self.cmd(adb_path, &["-s", serial, "reboot"], 15)
        } else {
            self.cmd(adb_path, &["-s", serial, "reboot", mode], 15)
        }
    }

    pub fn fs_detect(&self, adb_path: &str, serial: &str) -> Result<String, String> {
        self.shell(adb_path, serial, "uname -a && cat /proc/version")
    }

    pub fn fs_list(&self, adb_path: &str, serial: &str, path: &str) -> Result<Vec<HostEntry>, String> {
        let quoted = format!("'{}'", path.replace('\'', "'\\''"));
        let out = self.cmd(adb_path, &["-s", serial, "shell", &format!("ls -la {quoted}")], 15)?;
        let mut entries = Vec::new();
        for line in out.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("total") {
                continue;
            }
            let mut parts = line.split_whitespace();
            let perms = parts.next().unwrap_or("").to_string();
            if perms.is_empty() {
                continue;
            }
            let _links = parts.next().unwrap_or("").to_string();
            let _owner = parts.next().unwrap_or("").to_string();
            let _group = parts.next().unwrap_or("").to_string();
            let size = parts.next().unwrap_or("0").parse::<u64>().unwrap_or(0);
            let month = parts.next().unwrap_or("").to_string();
            let day = parts.next().unwrap_or("").to_string();
            let time = parts.next().unwrap_or("").to_string();
            let mut name = parts.collect::<Vec<_>>().join(" ");
            if let Some(idx) = name.find(" -> ") {
                name.truncate(idx);
            }
            if name.is_empty() {
                continue;
            }
            entries.push(HostEntry {
                name: name.clone(),
                path: format!("{}/{}", path.trim_end_matches('/'), name),
                is_dir: perms.starts_with('d'),
                size,
                modified_at: format!("{month} {day} {time}"),
            });
        }
        Ok(entries)
    }

    pub fn fs_read(&self, adb_path: &str, serial: &str, path: &str) -> Result<String, String> {
        let quoted = format!("'{}'", path.replace('\'', "'\\''"));
        self.cmd(adb_path, &["-s", serial, "exec-out", "cat", &quoted], 30)
    }

    pub fn start_server(&self, adb_path: &str) -> Result<String, String> {
        self.cmd(adb_path, &["start-server"], 15)
    }

    pub fn kill_server(&self, adb_path: &str) -> Result<String, String> {
        self.cmd(adb_path, &["kill-server"], 15)
    }

    pub fn mtime_from_str(_s: &str) -> String {
        let secs = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        iso_from_secs(secs)
    }
}
