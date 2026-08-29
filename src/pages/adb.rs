use crate::models::{AdbDevice, HostEntry};
use crate::services::adb::AdbService;
use crate::store::AppStore;
use dioxus::prelude::*;
use dioxus::signals::ReadableExt;

#[allow(non_snake_case)]
pub fn AdbPage() -> Element {
    let store = use_context::<Signal<AppStore>>();
    let mut devices = use_signal(Vec::<AdbDevice>::new);
    let mut selected = use_signal(String::new);
    let mut fs_path = use_signal(|| "/".to_string());
    let entries = use_signal(Vec::<HostEntry>::new);
    let mut cmd = use_signal(String::new);
    let cmd_out = use_signal(String::new);
    let mut address = use_signal(String::new);
    let mut push_local = use_signal(String::new);
    let mut push_remote = use_signal(String::new);
    let mut error = use_signal(String::new);

    use_effect(move || {
        let (path, svc) = adb(store);
        match svc.scan(&path) {
            Ok(d) => devices.set(d),
            Err(e) => error.set(e),
        }
    });

    rsx! {
        h1 { class: "page-title", "ADB 设备" }
        if !error.cloned().is_empty() {
            div { class: "err", "{error}" }
        }
        div { class: "card",
            div { class: "row",
                button { class: "primary", onclick: move |_| scan(store, devices, error), "刷新设备" }
                input { style: "width: 180px", placeholder: "IP:端口", value: "{address}", oninput: move |e| address.set(e.value()) }
                button { onclick: move |_| connect(store, address, devices, cmd_out, error), "连接" }
                button { onclick: move |_| disconnect(store, address, devices, cmd_out, error), "断开" }
            }
            if devices.read().is_empty() {
                div { class: "hint", style: "margin-top: 8px", "未检测到 ADB 设备" }
            }
            for (d, d_serial) in devices.read().iter().cloned().map(|d| {
                let serial = d.serial.clone();
                (d, serial)
            }) {
                div { class: "row", style: "margin-top: 8px",
                    input { r#type: "radio", name: "adb", checked: selected.cloned() == d_serial, oninput: move |_| selected.set(d_serial.clone()) }
                    span { "{d.serial}" }
                    span { class: if d.state == "device" { "badge on" } else { "badge off" }, "{d.state}" }
                    if !d.model.is_empty() {
                        span { class: "hint", "{d.model}" }
                    }
                }
            }
        }

        if !selected.cloned().is_empty() {
            div { class: "card",
                div { class: "card-title", "设备文件系统" }
                div { class: "row",
                    input { style: "flex: 1", value: "{fs_path}", oninput: move |e| fs_path.set(e.value()) }
                    button { class: "primary", onclick: move |_| fs_list(store, selected, fs_path, entries, error), "列出" }
                    button { onclick: move |_| run(store, selected, "uname -a && cat /proc/version", cmd_out, error), "探测系统" }
                    button { onclick: move |_| run(store, selected, "reboot", cmd_out, error), "重启" }
                }
                if !entries.read().is_empty() {
                    table { class: "table", style: "margin-top: 8px",
                        thead { tr { th { "名称" } th { "类型" } th { "大小" } } }
                        tbody {
                            for (e, e_full) in entries.read().iter().cloned().map(|e| {
                                let full = e.clone();
                                (e, full)
                            }) {
                                tr { key: "{e.path}",
                                    td {
                                        button { style: "border: none; background: none; padding: 0",
                                            onclick: move |_| on_entry(store, selected, e_full.clone(), fs_path, entries, cmd_out, error),
                                            if e.is_dir { "📁 {e.name}" } else { "📄 {e.name}" }
                                        }
                                    }
                                    td { if e.is_dir { "目录" } else { "文件" } }
                                    td { "{e.size}" }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "card",
                div { class: "card-title", "执行命令 / 文件推送" }
                div { class: "row",
                    input {
                        style: "flex: 1",
                        placeholder: "adb shell 命令，例如 ls -l /",
                        value: "{cmd}",
                        oninput: move |e| cmd.set(e.value()),
                        onkeydown: move |e| {
                            if e.key() == Key::Enter {
                                let c = cmd.cloned();
                                run(store, selected, &c, cmd_out, error);
                            }
                        },
                    }
                    button { class: "primary", onclick: move |_| { let c = cmd.cloned(); run(store, selected, &c, cmd_out, error); }, "执行" }
                }
                div { class: "row", style: "margin-top: 8px",
                    input { style: "flex: 1", placeholder: "本地文件路径", value: "{push_local}", oninput: move |e| push_local.set(e.value()) }
                    input { style: "flex: 1", placeholder: "设备目标路径", value: "{push_remote}", oninput: move |e| push_remote.set(e.value()) }
                    button { onclick: move |_| push(store, selected, push_local, push_remote, cmd_out, error), "推送" }
                }
                pre { class: "log-box", style: "margin-top: 8px", if cmd_out.cloned().is_empty() { "（输出显示在这里）" } else { "{cmd_out}" } }
            }
        }
    }
}

fn adb(store: Signal<AppStore>) -> (String, AdbService) {
    let settings = store.read().settings;
    let path = settings.cloned().adb_path;
    (path, AdbService)
}

fn scan(store: Signal<AppStore>, mut devices: Signal<Vec<AdbDevice>>, mut error: Signal<String>) {
    let (path, svc) = adb(store);
    match svc.scan(&path) {
        Ok(d) => {
            devices.set(d);
            error.set(String::new());
        }
        Err(e) => error.set(e),
    }
}

fn connect(
    store: Signal<AppStore>,
    address: Signal<String>,
    mut devices: Signal<Vec<AdbDevice>>,
    mut cmd_out: Signal<String>,
    mut error: Signal<String>,
) {
    let addr = address.cloned().trim().to_string();
    if addr.is_empty() {
        error.set("请输入 IP:端口".into());
        return;
    }
    let (path, svc) = adb(store);
    match svc.connect(&path, &addr) {
        Ok(out) => {
            cmd_out.set(out);
            match svc.scan(&path) {
                Ok(d) => devices.set(d),
                Err(e) => error.set(e),
            }
        }
        Err(e) => error.set(e),
    }
}

fn disconnect(
    store: Signal<AppStore>,
    address: Signal<String>,
    mut devices: Signal<Vec<AdbDevice>>,
    mut cmd_out: Signal<String>,
    mut error: Signal<String>,
) {
    let addr = address.cloned().trim().to_string();
    if addr.is_empty() {
        error.set("请输入 IP:端口".into());
        return;
    }
    let (path, svc) = adb(store);
    match svc.disconnect(&path, &addr) {
        Ok(out) => {
            cmd_out.set(out);
            match svc.scan(&path) {
                Ok(d) => devices.set(d),
                Err(e) => error.set(e),
            }
        }
        Err(e) => error.set(e),
    }
}

fn run(
    store: Signal<AppStore>,
    selected: Signal<String>,
    command: &str,
    mut cmd_out: Signal<String>,
    mut error: Signal<String>,
) {
    let (path, svc) = adb(store);
    match svc.shell(&path, &selected.cloned(), command) {
        Ok(out) => {
            cmd_out.set(out);
            error.set(String::new());
        }
        Err(e) => error.set(e),
    }
}

fn fs_list(
    store: Signal<AppStore>,
    selected: Signal<String>,
    fs_path: Signal<String>,
    mut entries: Signal<Vec<HostEntry>>,
    mut error: Signal<String>,
) {
    let (path, svc) = adb(store);
    match svc.fs_list(&path, &selected.cloned(), &fs_path.cloned()) {
        Ok(e) => {
            entries.set(e);
            error.set(String::new());
        }
        Err(e) => error.set(e),
    }
}

fn on_entry(
    store: Signal<AppStore>,
    selected: Signal<String>,
    e: HostEntry,
    fs_path: Signal<String>,
    entries: Signal<Vec<HostEntry>>,
    mut cmd_out: Signal<String>,
    mut error: Signal<String>,
) {
    if e.is_dir {
        let mut fs_path = fs_path;
        fs_path.set(e.path);
        fs_list(store, selected, fs_path, entries, error);
    } else {
        let (path, svc) = adb(store);
        match svc.fs_read(&path, &selected.cloned(), &e.path) {
            Ok(out) => cmd_out.set(out),
            Err(err) => error.set(err),
        }
    }
}

fn push(
    store: Signal<AppStore>,
    selected: Signal<String>,
    push_local: Signal<String>,
    push_remote: Signal<String>,
    mut cmd_out: Signal<String>,
    mut error: Signal<String>,
) {
    if push_local.cloned().trim().is_empty() || push_remote.cloned().trim().is_empty() {
        error.set("请填写本地路径与设备目标路径".into());
        return;
    }
    let (path, svc) = adb(store);
    match svc.push(&path, &selected.cloned(), &push_local.cloned(), &push_remote.cloned()) {
        Ok(out) => {
            cmd_out.set(out);
            error.set(String::new());
        }
        Err(e) => error.set(e),
    }
}
