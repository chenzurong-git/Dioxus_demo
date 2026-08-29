use crate::models::LogFileInfo;
use crate::services::util::{fmt_size, iso_from_secs};
use std::fs;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

pub struct LogService {
    pub dir: PathBuf,
    pub max_file_size: u64,
}

impl LogService {
    pub fn new(dir: PathBuf, max_file_size: u64) -> Self {
        Self { dir, max_file_size }
    }

    pub fn list(&self) -> Vec<LogFileInfo> {
        let mut out = Vec::new();
        if let Ok(rd) = fs::read_dir(&self.dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().map(|x| x == "log").unwrap_or(false) {
                    if let Ok(md) = fs::metadata(&p) {
                        out.push(LogFileInfo {
                            name: e.file_name().to_string_lossy().into_owned(),
                            size: md.len(),
                            modified_at: md
                                .modified()
                                .ok()
                                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                                .map(|d| iso_from_secs(d.as_secs() as i64))
                                .unwrap_or_default(),
                        });
                    }
                }
            }
        }
        out.sort_by(|a, b| b.name.cmp(&a.name));
        out
    }

    pub fn read(&self, name: &str, max_bytes: u64) -> Result<String, String> {
        let name = safe_log_name(name)?;
        let data = fs::read(self.dir.join(name)).map_err(|e| e.to_string())?;
        let start = data.len().saturating_sub(max_bytes as usize);
        Ok(String::from_utf8_lossy(&data[start..]).into_owned())
    }

    pub fn delete(&self, name: &str) -> Result<(), String> {
        let name = safe_log_name(name)?;
        fs::remove_file(self.dir.join(name)).map_err(|e| e.to_string())
    }

    #[allow(dead_code)]
    pub fn debug_size(&self) -> String {
        fmt_size(self.max_file_size)
    }
}

fn safe_log_name(name: &str) -> Result<&str, String> {
    if PathBuf::from(name).file_name().and_then(|n| n.to_str()) != Some(name) || !name.ends_with(".log") {
        return Err("日志文件名无效".into());
    }
    Ok(name)
}
