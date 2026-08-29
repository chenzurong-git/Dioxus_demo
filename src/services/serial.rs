use crate::models::OpenSerialPort;
use crate::services::util::{now_compact, sanitize};
use dioxus::prelude::SyncSignal;
use dioxus::signals::WritableExt;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct SerialService {
    state: Mutex<Option<OpenPort>>,
}

struct OpenPort {
    info: OpenSerialPort,
    port: Arc<Mutex<Box<dyn serialport::SerialPort>>>,
    stop: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

#[allow(dead_code)]
impl SerialService {
    pub fn new() -> Self {
        Self { state: Mutex::new(None) }
    }

    pub fn list_ports() -> Vec<String> {
        serialport::available_ports()
            .map(|ps| ps.into_iter().map(|p| p.port_name).collect())
            .unwrap_or_default()
    }

    pub fn opened(&self) -> Option<OpenSerialPort> {
        self.state.lock().unwrap().as_ref().map(|o| o.info.clone())
    }

    pub fn is_open(&self, port: &str) -> bool {
        self.state
            .lock()
            .unwrap()
            .as_ref()
            .map(|o| o.info.port_name == port)
            .unwrap_or(false)
    }

    pub fn open(
        &self,
        port_name: &str,
        baud: u32,
        recording: bool,
        log_enabled: bool,
        max_file_size: u64,
        logs_dir: PathBuf,
        buffer: SyncSignal<String>,
        mut open_ports: SyncSignal<Vec<OpenSerialPort>>,
    ) -> Result<(), String> {
        self.close(open_ports);
        let port = serialport::new(port_name, baud)
            .timeout(Duration::from_millis(100))
            .open()
            .map_err(|e| format!("打开串口失败：{e}"))?;
        let info = OpenSerialPort {
            port_name: port_name.to_string(),
            baud_rate: baud,
            opened_at: crate::services::util::now_iso(),
            recording,
        };
        let port = Arc::new(Mutex::new(port));
        let stop = Arc::new(AtomicBool::new(false));

        let t_port = Arc::clone(&port);
        let t_stop = Arc::clone(&stop);
        let t_name = port_name.to_string();
        let t_recording = recording;
        let t_log_enabled = log_enabled;
        let t_max = max_file_size;
        let t_logs_dir = logs_dir;
        let mut t_buffer = buffer;
        let handle = std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            let mut log_path: Option<PathBuf> = None;
            if t_recording && t_log_enabled {
                let p = t_logs_dir.join(format!("{}_{}.log", sanitize(&t_name), now_compact()));
                let _ = std::fs::File::create(&p);
                log_path = Some(p);
            }
            loop {
                if t_stop.load(Ordering::Relaxed) {
                    break;
                }
                let n = {
                    let mut p = t_port.lock().unwrap();
                    p.read(&mut buf)
                };
                match n {
                    Ok(0) => continue,
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buf[..n]).into_owned();
                        t_buffer.with_mut(|s| {
                            s.push_str(&text);
                            if s.len() > 200_000 {
                                let cut = s.len() - 200_000;
                                s.drain(..cut);
                            }
                        });
                        if let Some(path) = &log_path {
                            if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(path) {
                                let _ = f.write_all(text.as_bytes());
                                if let Ok(md) = f.metadata() {
                                    if md.len() > t_max {
                                        drop(f);
                                        let p =
                                            t_logs_dir.join(format!("{}_{}.log", sanitize(&t_name), now_compact()));
                                        let _ = std::fs::File::create(&p);
                                        log_path = Some(p);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if matches!(e.kind(), std::io::ErrorKind::TimedOut) {
                            continue;
                        }
                        break;
                    }
                }
            }
        });

        *self.state.lock().unwrap() = Some(OpenPort {
            info: info.clone(),
            port,
            stop,
            handle: Some(handle),
        });
        open_ports.set(vec![info]);
        Ok(())
    }

    pub fn close(&self, mut open_ports: SyncSignal<Vec<OpenSerialPort>>) {
        let mut guard = self.state.lock().unwrap();
        if let Some(op) = guard.take() {
            op.stop.store(true, Ordering::Relaxed);
            if let Some(h) = op.handle {
                let _ = h.join();
            }
        }
        drop(guard);
        open_ports.set(Vec::new());
    }

    pub fn write(&self, port: &str, data: &str, append_newline: bool) -> Result<(), String> {
        let guard = self.state.lock().unwrap();
        let op = guard.as_ref().ok_or("串口未打开")?;
        if op.info.port_name != port {
            return Err(format!("串口 {port} 未打开"));
        }
        let mut p = op.port.lock().unwrap();
        let mut bytes = data.as_bytes().to_vec();
        if append_newline {
            bytes.push(b'\n');
        }
        p.write(&bytes).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn set_recording(&self, enabled: bool) {
        if let Some(op) = self.state.lock().unwrap().as_mut() {
            op.info.recording = enabled;
        }
    }
}
