use crate::models::HostEntry;
use crate::services::util::{iso_from_secs, run_cmd};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub struct HostService;

impl HostService {
    fn check(root: &str, path: &str) -> Result<PathBuf, String> {
        let p = PathBuf::from(path);
        if root.trim().is_empty() {
            return Ok(p);
        }
        let canonical_root = Path::new(root).canonicalize().unwrap_or_else(|_| PathBuf::from(root));
        let canonical_p = p.canonicalize().unwrap_or_else(|_| p.clone());
        if canonical_p.starts_with(&canonical_root) {
            Ok(p)
        } else {
            Err(format!("路径超出允许访问根目录：{root}"))
        }
    }

    pub fn list_dir(root: &str, path: &str) -> Result<Vec<HostEntry>, String> {
        let p = Self::check(root, path)?;
        let mut out = Vec::new();
        for e in fs::read_dir(&p).map_err(|e| e.to_string())?.flatten() {
            let ep = e.path();
            let md = fs::metadata(&ep).ok();
            out.push(HostEntry {
                name: e.file_name().to_string_lossy().into_owned(),
                path: ep.to_string_lossy().into_owned(),
                is_dir: md.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                size: md.as_ref().map(|m| m.len()).unwrap_or(0),
                modified_at: md
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| iso_from_secs(d.as_secs() as i64))
                    .unwrap_or_default(),
            });
        }
        out.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
        Ok(out)
    }

    pub fn read_file(root: &str, path: &str, max_bytes: u64) -> Result<String, String> {
        let p = Self::check(root, path)?;
        let data = fs::read(&p).map_err(|e| e.to_string())?;
        let start = data.len().saturating_sub(max_bytes as usize);
        Ok(String::from_utf8_lossy(&data[start..]).into_owned())
    }

    pub fn write_file(root: &str, path: &str, content: &str, append: bool) -> Result<(), String> {
        let p = Self::check(root, path)?;
        if append {
            use std::io::Write;
            let mut f = fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&p)
                .map_err(|e| e.to_string())?;
            f.write_all(content.as_bytes()).map_err(|e| e.to_string())
        } else {
            fs::write(&p, content).map_err(|e| e.to_string())
        }
    }

    pub fn exec(shell_enabled: bool, command: &str) -> Result<String, String> {
        if !shell_enabled {
            return Err("主机 shell 未启用（请在设置页开启）".into());
        }
        #[cfg(windows)]
        {
            run_cmd("cmd", &["/c", command], 30)
        }
        #[cfg(not(windows))]
        {
            run_cmd("sh", &["-c", command], 30)
        }
    }
}
