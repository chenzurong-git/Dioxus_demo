use crate::models::{QuickCommandInput, QuickCommandWindow};
use crate::services::util::now_iso;
use crate::store::AppStore;
use dioxus::prelude::*;
use dioxus::signals::ReadableExt;

#[allow(non_snake_case)]
pub fn QuickCommandsPage() -> Element {
    let store = use_context::<Signal<AppStore>>();
    let mut port = use_signal(String::new);
    let mut name = use_signal(String::new);
    let mut commands = use_signal(String::new);
    let mut interval_ms = use_signal(|| "0".to_string());
    let mut append_newline = use_signal(|| true);
    let mut editing_id = use_signal(|| None::<i64>);
    let error = use_signal(String::new);

    let mut store_windows = store.read().quick_windows;
    let open_ports = store.read().open_ports;

    rsx! {
        h1 { class: "page-title", "快捷命令" }
        if !error.cloned().is_empty() {
            div { class: "err", "{error}" }
        }
        div { class: "card",
            div { class: "card-title", if editing_id.cloned().is_none() { "新建快捷命令窗口" } else { "编辑快捷命令窗口" } }
            div { class: "row",
                input { style: "width: 200px", placeholder: "窗口名称", value: "{name}", oninput: move |e| name.set(e.value()) }
                select {
                    value: "{port}",
                    onchange: move |e| port.set(e.value()),
                    option { value: "", "选择已打开串口" }
                    for p in open_ports.read().iter().cloned() {
                        option { value: "{p.port_name}", "{p.port_name}" }
                    }
                }
                input { style: "width: 120px", r#type: "number", placeholder: "行间隔 ms", value: "{interval_ms}", oninput: move |e| interval_ms.set(e.value()) }
                label {
                    input { r#type: "checkbox", checked: append_newline.cloned(), oninput: move |e| append_newline.set(e.value() == "true") }
                    " 追加换行"
                }
            }
            textarea { rows: 6, style: "width: 100%; margin-top: 8px", placeholder: "每行一条命令", value: "{commands}", oninput: move |e| commands.set(e.value()) }
            div { class: "row", style: "margin-top: 8px",
                button { class: "primary", onclick: move |_| save(store, store_windows, editing_id, name, commands, interval_ms, append_newline, error), "保存" }
                if editing_id.cloned().is_some() {
                    button { onclick: move |_| { editing_id.set(None); name.set(String::new()); commands.set(String::new()); }, "取消" }
                }
            }
        }

        for (_w, w_id, w_send, w_name, w_commands_joined, w_interval, w_append) in
            store_windows.read().iter().cloned().map(|w| {
                let w_id = w.id;
                let w_send = w.clone();
                let w_name = w.name.clone();
                let w_joined = w.commands.join("\n");
                let w_interval = w.interval_ms;
                let w_append = w.append_newline;
                (w, w_id, w_send, w_name, w_joined, w_interval, w_append)
            })
        {
            div { class: "card", key: "{w_id}",
                div { class: "card-title", "{w_name}" }
                pre { class: "mono", style: "margin: 0", "{w_commands_joined}" }
                div { class: "row", style: "margin-top: 8px",
                    button { class: "primary", onclick: move |_| send_all(store, w_send.clone(), port, error), "发送到串口" }
                    button { onclick: move |_| {
                        editing_id.set(Some(w_id));
                        name.set(w_name.clone());
                        commands.set(w_commands_joined.clone());
                        interval_ms.set(w_interval.to_string());
                        append_newline.set(w_append);
                    }, "编辑" }
                    button { class: "danger", onclick: move |_| {
                        store_windows.write().retain(|x| x.id != w_id);
                        store.read().save_quick();
                    }, "删除" }
                }
            }
        }
        if store_windows.read().is_empty() {
            div { class: "hint", "暂无快捷命令窗口" }
        }
    }
}

fn save(
    store: Signal<AppStore>,
    mut windows: Signal<Vec<QuickCommandWindow>>,
    mut editing_id: Signal<Option<i64>>,
    mut name: Signal<String>,
    mut commands: Signal<String>,
    mut interval_ms: Signal<String>,
    append_newline: Signal<bool>,
    mut error: Signal<String>,
) {
    if name.cloned().trim().is_empty() {
        error.set("请填写窗口名称".into());
        return;
    }
    let input = QuickCommandInput {
        name: name.cloned().trim().to_string(),
        commands: commands
            .cloned()
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect(),
        interval_ms: interval_ms.cloned().parse().unwrap_or(0),
        append_newline: append_newline.cloned(),
    };
    let now = now_iso();
    match editing_id.cloned() {
        None => {
            let next = windows.read().iter().map(|w| w.id).max().unwrap_or(0) + 1;
            windows.write().push(QuickCommandWindow {
                id: next,
                name: input.name,
                commands: input.commands,
                interval_ms: input.interval_ms,
                append_newline: input.append_newline,
                created_at: now.clone(),
                updated_at: now,
            });
        }
        Some(id) => {
            if let Some(w) = windows.write().iter_mut().find(|w| w.id == id) {
                w.name = input.name;
                w.commands = input.commands;
                w.interval_ms = input.interval_ms;
                w.append_newline = input.append_newline;
                w.updated_at = now;
            }
        }
    }
    editing_id.set(None);
    name.set(String::new());
    commands.set(String::new());
    interval_ms.set("0".into());
    store.read().save_quick();
    error.set(String::new());
}

fn send_all(store: Signal<AppStore>, w: QuickCommandWindow, port: Signal<String>, mut error: Signal<String>) {
    let s = store.read();
    if port.cloned().trim().is_empty() {
        error.set("请先打开一个串口并选择目标端口".into());
        return;
    }
    let mut first_err: Option<String> = None;
    for c in &w.commands {
        if let Err(e) = s.serial.write(&port.cloned(), c, w.append_newline) {
            first_err = Some(e);
            break;
        }
        if w.interval_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(w.interval_ms));
        }
    }
    if let Some(e) = first_err {
        error.set(e);
    }
}
