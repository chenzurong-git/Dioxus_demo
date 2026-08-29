use crate::services::serial::SerialService;
use crate::store::AppStore;
use dioxus::prelude::*;
use dioxus::signals::{ReadableExt, WritableExt};

#[allow(non_snake_case)]
pub fn SerialTerminalPage() -> Element {
    let store = use_context::<Signal<AppStore>>();
    let mut ports = use_signal(Vec::<String>::new);
    let mut device_id = use_signal(|| None::<i64>);
    let mut port = use_signal(String::new);
    let mut baud = use_signal(|| "115200".to_string());
    let mut recording = use_signal(|| true);
    let error = use_signal(String::new);
    let mut input = use_signal(String::new);

    use_effect(move || {
        let mut preset = store.read().serial_device;
        let id = preset.cloned();
        if id.is_some() {
            preset.set(None);
            device_id.set(id);
        }
        ports.set(SerialService::list_ports());
    });

    let devices = store.read().devices;
    let open_ports = store.read().open_ports;
    let buffer = store.read().serial_buffer;
    let opened = open_ports.read().iter().next().cloned();

    rsx! {
        h1 { class: "page-title", "串口终端" }
        if !error.cloned().is_empty() {
            div { class: "err", "{error}" }
        }
        div { class: "card",
            div { class: "row",
                select {
                    value: if let Some(id) = device_id.cloned() { id.to_string() } else { String::new() },
                    onchange: move |e| {
                        let v = e.value();
                        device_id.set(if v.is_empty() { None } else { v.parse().ok() });
                    },
                    option { value: "", "直接选择端口" }
                    for d in devices.read().iter().cloned() {
                        option { value: "{d.id}", "{d.name}（{d.serial_port}）" }
                    }
                }
                select {
                    value: "{port}",
                    disabled: device_id.cloned().is_some(),
                    onchange: move |e| port.set(e.value()),
                    for p in ports.read().iter().cloned() {
                        option { value: "{p}", "{p}" }
                    }
                }
                input {
                    r#type: "number",
                    style: "width: 110px",
                    value: "{baud}",
                    oninput: move |e| baud.set(e.value()),
                }
                label {
                    input { r#type: "checkbox", checked: recording.cloned(), oninput: move |e| {
                        let on = e.value() == "true";
                        recording.set(on);
                        store.read().serial.set_recording(on);
                    } }
                    " 自动录制日志"
                }
                if opened.is_none() {
                    button { class: "primary", onclick: move |_| open(store, device_id, port, baud, recording, error), "打开串口" }
                } else {
                    button { class: "danger", onclick: move |_| close(store), "关闭串口" }
                }
                span { class: "hint",
                    if let Some(op) = &opened { "已打开: {op.port_name} @ {op.baud_rate}" } else { "未打开" }
                }
                button { onclick: move |_| ports.set(SerialService::list_ports()), "刷新端口" }
            }
        }

        div { class: "card",
            div { class: "card-title", if opened.is_some() { "串口输出（最近 200KB）" } else { "实时缓冲日志（最近 200KB）" } }
            pre { class: "log-box", style: "max-height: 420px", "{buffer}" }
            if opened.is_some() {
                div { class: "row", style: "margin-top: 8px",
                    input {
                        style: "flex: 1",
                        placeholder: "输入命令，Enter 发送",
                        value: "{input}",
                        oninput: move |e| input.set(e.value()),
                        onkeydown: move |e| {
                            if e.key() == Key::Enter {
                                send(store, input);
                            }
                        },
                    }
                    button { class: "primary", onclick: move |_| send(store, input), "发送" }
                    button { onclick: move |_| { let mut buf = store.read().serial_buffer; buf.set(String::new()); }, "清空" }
                }
            } else {
                div { class: "row", style: "margin-top: 8px",
                    button { onclick: move |_| { let mut buf = store.read().serial_buffer; buf.set(String::new()); }, "清空" }
                }
            }
        }
        div { class: "hint", "提示：串口数据实时显示在此；勾选「自动录制日志」后数据会同时保存到日志目录（可在「串口日志」页查看）" }
    }
}

fn open(
    store: Signal<AppStore>,
    device_id: Signal<Option<i64>>,
    port: Signal<String>,
    baud: Signal<String>,
    recording: Signal<bool>,
    mut error: Signal<String>,
) {
    let s = store.read();
    let dev = device_id
        .cloned()
        .and_then(|id| s.devices.read().iter().find(|d| d.id == id).cloned());
    let (pname, baud_rate) = match dev {
        Some(d) => {
            let p = if d.serial_port.trim().is_empty() {
                port.cloned()
            } else {
                d.serial_port.clone()
            };
            let b = if d.baud_rate > 0 { d.baud_rate } else { baud.cloned().parse().unwrap_or(115200) };
            (p, b)
        }
        None => (port.cloned(), baud.cloned().parse().unwrap_or(115200)),
    };
    if pname.trim().is_empty() {
        error.set("未选择串口端口".into());
        return;
    }
    let settings = s.settings.cloned();
    match s.serial.open(
        &pname,
        baud_rate,
        recording.cloned(),
        settings.log_enabled,
        settings.log_max_file_size,
        s.logs.dir.clone(),
        s.serial_buffer,
        s.open_ports,
    ) {
        Ok(()) => error.set(String::new()),
        Err(e) => error.set(e),
    }
}

fn close(store: Signal<AppStore>) {
    let s = store.read();
    s.serial.close(s.open_ports);
}

fn send(store: Signal<AppStore>, mut input: Signal<String>) {
    let s = store.read();
    let port_name = s.open_ports.read().iter().next().map(|op| op.port_name.clone());
    let text = input.cloned();
    if !text.is_empty() {
        if let Some(pname) = port_name {
            let _ = s.serial.write(&pname, &text, true);
        }
    }
    input.set(String::new());
}
