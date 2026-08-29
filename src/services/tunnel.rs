use crate::models::{Settings, TunnelStatus};
use crate::services::util::now_iso;
use dioxus::prelude::SyncSignal;
use dioxus::signals::WritableExt;
use std::process::{Child, Command};
use std::sync::Mutex;

pub struct TunnelService {
    state: Mutex<Option<TunnelHandle>>,
}

#[allow(dead_code)]
struct TunnelHandle {
    child: Child,
    pid: u32,
    started_at: String,
    command: String,
}

#[allow(dead_code)]
impl TunnelService {
    pub fn new() -> Self {
        Self { state: Mutex::new(None) }
    }

    pub fn start(&self, s: &Settings, mut status: SyncSignal<TunnelStatus>) -> Result<(), String> {
        self.stop(status);
        if s.tunnel_host.trim().is_empty() || s.tunnel_remote_port == 0 {
            return Err("隧道参数不完整（主机地址或远端端口为空）".into());
        }
        if s.tunnel_auth_method == "password" {
            return Err("密码认证需要交互式 ssh，建议改用私钥认证".into());
        }
        let mut args = vec!["-N".to_string()];
        let fwd = format!(
            "{}:{}:127.0.0.1:{}",
            if s.tunnel_remote_bind.trim().is_empty() { "127.0.0.1" } else { &s.tunnel_remote_bind },
            s.tunnel_remote_port,
            s.tunnel_remote_port
        );
        args.push("-R".into());
        args.push(fwd);
        if s.tunnel_ssh_port != 22 {
            args.push("-p".into());
            args.push(s.tunnel_ssh_port.to_string());
        }
        if !s.tunnel_private_key.trim().is_empty() {
            args.push("-i".into());
            args.push(s.tunnel_private_key.trim().to_string());
        }
        let user_host = format!("{}@{}", s.tunnel_user.trim(), s.tunnel_host.trim());
        args.push(user_host);

        let child = Command::new("ssh").args(&args).spawn().map_err(|e| e.to_string())?;
        let pid = child.id();
        let command = format!("ssh {}", args.join(" "));
        *self.state.lock().unwrap() = Some(TunnelHandle {
            child,
            pid,
            started_at: now_iso(),
            command: command.clone(),
        });
        status.set(TunnelStatus {
            running: true,
            pid: Some(pid),
            command,
            started_at: Some(now_iso()),
            last_error: None,
        });
        Ok(())
    }

    pub fn stop(&self, mut status: SyncSignal<TunnelStatus>) {
        if let Some(mut h) = self.state.lock().unwrap().take() {
            let _ = h.child.kill();
            let _ = h.child.wait();
        }
        status.set(TunnelStatus {
            running: false,
            pid: None,
            command: String::new(),
            started_at: None,
            last_error: None,
        });
    }

    pub fn refresh_status(&self, mut status: SyncSignal<TunnelStatus>) {
        let mut guard = self.state.lock().unwrap();
        if let Some(h) = guard.as_mut() {
            match h.child.try_wait() {
                Ok(Some(_)) => {
                    guard.take();
                    status.set(TunnelStatus {
                        running: false,
                        pid: None,
                        command: String::new(),
                        started_at: None,
                        last_error: Some("ssh 进程已退出".into()),
                    });
                }
                _ => {}
            }
        }
    }
}
