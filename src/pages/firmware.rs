use crate::models::{FirmwareInfo, FlashDevice};
use crate::services::util::fmt_size;
use crate::store::AppStore;
use dioxus::prelude::*;
use dioxus::signals::ReadableExt;
use std::path::Path;

#[allow(non_snake_case)]
pub fn FirmwarePage() -> Element {
    let store = use_context::<Signal<AppStore>>();
    let mut serial = use_signal(String::new);
    let mut upload_path = use_signal(String::new);
    let mut firmwares = use_signal(Vec::<FirmwareInfo>::new);
    let mut flash_devices = use_signal(Vec::<FlashDevice>::new);
    let error = use_signal(String::new);

    use_effect(move || {
        if !serial.cloned().is_empty() {
            let s = store.read();
            firmwares.set(s.flash.firmware.list(&serial.cloned()));
        }
    });

    let devices = store.read().devices;
    let flash_status = store.read().flash_status;
    let st = flash_status.cloned();

    rsx! {
        h1 { class: "page-title", "固件烧录" }
        if !error.cloned().is_empty() {
            div { class: "err", "{error}" }
        }
        div { class: "card",
            div { class: "row",
                select {
                    value: "{serial}",
                    onchange: move |e| serial.set(e.value()),
                    option { value: "", "选择目标设备（设备工作区）" }
                    for d in devices.read().iter().cloned() {
                        option { value: "{d.serial}", "{d.name}（{d.serial}）" }
                    }
                }
                input {
                    style: "width: 260px",
                    placeholder: "本地固件文件路径",
                    value: "{upload_path}",
                    oninput: move |e| upload_path.set(e.value()),
                }
                button { class: "primary", onclick: move |_| upload(store, serial, upload_path, firmwares, error), "上传固件" }
                button { onclick: move |_| { let s = store.read(); firmwares.set(s.flash.firmware.list(&serial.cloned())); }, "刷新" }
            }
            div { class: "hint", "固件须先上传到设备工作区，确认进入烧录模式后启动烧录" }
        }

        div { class: "card",
            div { class: "card-title", "烧录模式设备" }
            div { class: "row",
                button { onclick: move |_| { flash_devices.set(store.read().flash.scan()); }, "扫描全志" }
                span { class: "hint", "扫描连接的全志设备，判断是否已进入烧录模式" }
            }
            if flash_devices.read().is_empty() {
                div { class: "hint", style: "margin-top: 8px", "未检测到烧录模式设备（当前为占位后端，接入 libefex 替换实现后可扫描）" }
            }
            for d in flash_devices.read().iter().cloned() {
                div { class: "row", style: "margin-top: 8px",
                    span { class: if d.mode == "fes" { "badge on" } else { "badge off" }, "{d.mode}" }
                    span { "{d.name}" }
                    span { class: "hint", "{d.detail}" }
                }
            }
        }

        div { class: "card",
            div { class: "card-title", "设备工作区固件" }
            if firmwares.read().is_empty() {
                div { class: "hint", "暂无固件" }
            }
            for (f, f_name, f_name_del) in firmwares.read().iter().cloned().map(|f| {
                let name = f.name.clone();
                (f, name.clone(), name)
            }) {
                div { class: "row", style: "justify-content: space-between; margin-bottom: 6px",
                    span { "{f.name} " span { class: "hint", "（{fmt_size(f.size)} · {f.modified_at}）" } }
                    div { class: "row",
                        button { class: "primary", disabled: serial.cloned().is_empty(), onclick: move |_| start_flash(store, serial, f_name.clone(), flash_devices, error), "烧录" }
                        button { class: "danger", onclick: move |_| {
                            let s = store.read();
                            let _ = s.flash.firmware.delete(&serial.cloned(), &f_name_del);
                            firmwares.set(s.flash.firmware.list(&serial.cloned()));
                        }, "删除" }
                    }
                }
            }
        }

        div { class: "card",
            div { class: "card-title", "烧录状态" }
            div { class: "row",
                span { class: if st.running { "badge running" } else if st.success == Some(true) { "badge on" } else { "badge off" },
                    if st.running { "烧录中" } else if st.success.is_none() { "未开始" } else if st.success == Some(true) { "烧录完成" } else { "烧录失败" }
                }
                span { "阶段: {st.phase}" }
                span { "进度: {st.progress:.0}%" }
            }
            div { class: "progress", div { style: "width: {st.progress}%", "" } }
            div { class: "hint", "{st.message}" }
            pre { class: "log-box", "{st.log.join(\"\\n\")}" }
            div { class: "row", style: "margin-top: 8px",
                button { onclick: move |_| demo_flash(store), "模拟烧录（演示）" }
            }
        }
    }
}

fn upload(
    store: Signal<AppStore>,
    serial: Signal<String>,
    upload_path: Signal<String>,
    mut firmwares: Signal<Vec<FirmwareInfo>>,
    mut error: Signal<String>,
) {
    if serial.cloned().trim().is_empty() {
        error.set("请先选择目标设备".into());
        return;
    }
    let path = upload_path.cloned().trim().to_string();
    if path.is_empty() {
        error.set("请填写本地固件文件路径".into());
        return;
    }
    let s = store.read();
    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(e) => {
            error.set(format!("读取文件失败：{e}"));
            return;
        }
    };
    let name = Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "firmware.img".into());
    match s.flash.firmware.upload(&serial.cloned(), &name, &data) {
        Ok(()) => {
            firmwares.set(s.flash.firmware.list(&serial.cloned()));
            error.set(String::new());
        }
        Err(e) => error.set(e),
    }
}

fn start_flash(
    store: Signal<AppStore>,
    serial: Signal<String>,
    firmware: String,
    flash_devices: Signal<Vec<FlashDevice>>,
    mut error: Signal<String>,
) {
    if flash_devices.read().is_empty() {
        error.set("未检测到烧录模式设备，请先扫描".into());
        return;
    }
    let s = store.read();
    let path = s.flash.firmware.path(&serial.cloned(), &firmware);
    if let Err(e) = s.flash.flash(&path, s.flash_status) {
        error.set(e);
    }
}

fn demo_flash(store: Signal<AppStore>) {
    let s = store.read();
    let _ = s.flash.flash(std::path::Path::new("demo"), s.flash_status);
}
