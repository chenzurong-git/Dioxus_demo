use crate::models::{FirmwareInfo, FlashDevice, FlashStatus};
use crate::services::util::{iso_from_secs, sanitize};
use dioxus::prelude::SyncSignal;
use dioxus::signals::{ReadableExt, WritableExt};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub struct FirmwareService {
    pub dir: PathBuf,
}

impl FirmwareService {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn device_dir(&self, serial: &str) -> PathBuf {
        self.dir.join(sanitize(serial))
    }

    pub fn upload(&self, serial: &str, name: &str, data: &[u8]) -> Result<(), String> {
        let d = self.device_dir(serial);
        fs::create_dir_all(&d).map_err(|e| e.to_string())?;
        fs::write(d.join(name), data).map_err(|e| e.to_string())
    }

    pub fn list(&self, serial: &str) -> Vec<FirmwareInfo> {
        let mut out = Vec::new();
        if let Ok(rd) = fs::read_dir(self.device_dir(serial)) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_file() {
                    if let Ok(md) = fs::metadata(&p) {
                        out.push(FirmwareInfo {
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

    pub fn delete(&self, serial: &str, name: &str) -> Result<(), String> {
        fs::remove_file(self.device_dir(serial).join(name)).map_err(|e| e.to_string())
    }

    pub fn path(&self, serial: &str, name: &str) -> PathBuf {
        self.device_dir(serial).join(name)
    }
}

#[allow(dead_code)]
pub trait FlashBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn scan(&self) -> Vec<FlashDevice>;
}

pub struct FlashService {
    pub firmware: FirmwareService,
    pub backend: Option<Box<dyn FlashBackend>>,
}

impl FlashService {
    pub fn new(firmware_dir: PathBuf) -> Self {
        Self {
            firmware: FirmwareService::new(firmware_dir),
            backend: None,
        }
    }

    pub fn scan(&self) -> Vec<FlashDevice> {
        match &self.backend {
            Some(b) => b.scan(),
            None => Vec::new(),
        }
    }

    pub fn flash(&self, firmware_path: &Path, status: SyncSignal<FlashStatus>) -> Result<(), String> {
        let path = firmware_path.to_path_buf();
        std::thread::spawn(move || {
            simulate_flash(path, status);
        });
        Ok(())
    }
}

fn simulate_flash(firmware_path: PathBuf, mut status: SyncSignal<FlashStatus>) {
    let name = firmware_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let phases = [
        ("初始化烧录引擎", 5.0),
        ("连接 FES 设备", 20.0),
        ("发送固件头", 35.0),
        ("写入固件数据", 75.0),
        ("校验固件", 95.0),
    ];
    status.set(FlashStatus {
        running: true,
        phase: "初始化".into(),
        progress: 0.0,
        message: format!("开始烧录 {name}"),
        log: vec![format!("[{}] 开始烧录 {name}", crate::services::util::now_iso())],
        finished_at: None,
        success: None,
    });
    for (phase, target) in phases {
        std::thread::sleep(std::time::Duration::from_millis(900));
        let mut s = status.cloned();
        s.phase = phase.to_string();
        s.progress = target;
        s.message = format!("{phase}（{target:.0}%）");
        s.log.push(format!("[{}] {phase}", crate::services::util::now_iso()));
        status.set(s);
    }
    std::thread::sleep(std::time::Duration::from_millis(500));
    status.set(FlashStatus {
        running: false,
        phase: "完成".into(),
        progress: 100.0,
        message: "烧录完成（占位后端，未写入真实设备）".into(),
        log: vec!["[完成] 烧录流程结束".into()],
        finished_at: Some(crate::services::util::now_iso()),
        success: Some(true),
    });
}
