use crate::models::{Device, DeviceInput};
use crate::services::util::now_iso;
use crate::store::{AppStore, Page};
use dioxus::prelude::*;
use dioxus::signals::ReadableExt;

#[allow(non_snake_case)]
pub fn DevicesPage() -> Element {
    let store = use_context::<Signal<AppStore>>();
    let mut editing = use_signal(|| None::<(Option<i64>, DeviceInput)>);
    let error = use_signal(String::new);

    let mut devices = store.read().devices;

    rsx! {
        h1 { class: "page-title", "我的设备" }
        if !error.cloned().is_empty() {
            div { class: "err", "{error}" }
        }
        div { class: "row", style: "margin-bottom: 12px",
            button { class: "primary", onclick: move |_| editing.set(Some((None, DeviceInput::default()))), "+ 新设备" }
        }

        if let Some((edit_id, draft)) = editing.cloned() {
            div { class: "card",
                div { class: "card-title", if edit_id.is_none() { "添加设备" } else { "编辑设备" } }
                div { class: "grid",
                    {field("设备名称", draft.name.clone(), move |v| set_field(editing, "name", v))}
                    {field("设备地址 (IP/主机名)", draft.address.clone(), move |v| set_field(editing, "address", v))}
                    {field("设备序列号", draft.serial.clone(), move |v| set_field(editing, "serial", v))}
                    div { class: "col",
                        label { "系统类型" }
                        select {
                            value: "{draft.os_type}",
                            onchange: move |e| set_field(editing, "os_type", e.value()),
                            option { value: "auto", "自动探测" }
                            option { value: "android", "Android" }
                            option { value: "linux", "Linux" }
                        }
                    }
                    {field("登录用户名", draft.username.clone(), move |v| set_field(editing, "username", v))}
                    div { class: "col",
                        label { "认证方式" }
                        select {
                            value: "{draft.auth_method}",
                            onchange: move |e| set_field(editing, "auth_method", e.value()),
                            option { value: "password", "密码" }
                            option { value: "key", "私钥" }
                        }
                    }
                    if draft.auth_method == "password" {
                        {field("密码", draft.password.clone(), move |v| set_field(editing, "password", v))}
                    } else {
                        {field("私钥路径", draft.private_key_path.clone(), move |v| set_field(editing, "private_key_path", v))}
                    }
                    {field("默认串口", draft.serial_port.clone(), move |v| set_field(editing, "serial_port", v))}
                    div { class: "col",
                        label { "波特率" }
                        input {
                            r#type: "number",
                            value: "{draft.baud_rate}",
                            oninput: move |e| set_field_baud(editing, e.value()),
                        }
                    }
                    {field("备注", draft.remark.clone(), move |v| set_field(editing, "remark", v))}
                }
                div { class: "row", style: "margin-top: 10px",
                    button { class: "primary", onclick: move |_| save(store, editing), "保存" }
                    button { onclick: move |_| editing.set(None), "取消" }
                }
            }
        }

        div { class: "grid",
            for (d, d_id, d_edit) in devices.read().iter().cloned().map(|d| {
                let edit = DeviceInput::from_device(&d);
                let d_id = d.id;
                (d, d_id, edit)
            }) {
                div { class: "card", key: "{d_id}",
                    div { class: "card-title",
                        "{d.name} "
                        span { class: if d.connected { "badge on" } else { "badge off" }, if d.connected { "已连接" } else { "未连接" } }
                    }
                    div { class: "hint", "地址: {d.address}" }
                    div { class: "hint", "序列号: {d.serial}" }
                    div { class: "hint", "备注: {d.remark}" }
                    div { class: "row", style: "margin-top: 8px",
                        button { onclick: move |_| open_serial(store, d_id), "打开串口" }
                        button { onclick: move |_| editing.set(Some((Some(d_id), d_edit.clone()))), "编辑" }
                        button { class: "danger", onclick: move |_| { devices.write().retain(|x| x.id != d_id); store.read().save_devices(); }, "删除" }
                    }
                }
            }
        }
        if devices.read().is_empty() {
            p { class: "hint", "暂无设备，点击右上角「+ 新设备」添加" }
        }
    }
}

fn field(name: &str, value: String, oninput: impl Fn(String) + 'static) -> Element {
    rsx! {
        div { class: "col",
            label { "{name}" }
            input { value: "{value}", oninput: move |e| oninput(e.value()) }
        }
    }
}

fn set_field(mut editing: Signal<Option<(Option<i64>, DeviceInput)>>, key: &'static str, v: String) {
    let Some((id, mut d)) = editing.cloned() else { return };
    match key {
        "name" => d.name = v,
        "address" => d.address = v,
        "serial" => d.serial = v,
        "username" => d.username = v,
        "auth_method" => d.auth_method = v,
        "password" => d.password = v,
        "private_key_path" => d.private_key_path = v,
        "os_type" => d.os_type = v,
        "serial_port" => d.serial_port = v,
        "remark" => d.remark = v,
        _ => {}
    }
    editing.set(Some((id, d)));
}

fn set_field_baud(mut editing: Signal<Option<(Option<i64>, DeviceInput)>>, v: String) {
    let Some((id, mut d)) = editing.cloned() else { return };
    d.baud_rate = v.parse().unwrap_or(d.baud_rate);
    editing.set(Some((id, d)));
}

fn save(store: Signal<AppStore>, mut editing: Signal<Option<(Option<i64>, DeviceInput)>>) {
    let Some((edit_id, draft)) = editing.cloned() else { return };
    let now = now_iso();
    let mut devices = store.read().devices;
    match edit_id {
        None => {
            let next = devices.read().iter().map(|d| d.id).max().unwrap_or(0) + 1;
            devices.write().push(Device {
                id: next,
                name: draft.name,
                address: draft.address,
                serial: draft.serial,
                username: draft.username,
                auth_method: draft.auth_method,
                password: draft.password,
                private_key_path: draft.private_key_path,
                os_type: draft.os_type,
                serial_port: draft.serial_port,
                baud_rate: draft.baud_rate,
                remark: draft.remark,
                connected: false,
                created_at: now.clone(),
                updated_at: now,
            });
        }
        Some(id) => {
            if let Some(d) = devices.write().iter_mut().find(|d| d.id == id) {
                d.name = draft.name;
                d.address = draft.address;
                d.serial = draft.serial;
                d.username = draft.username;
                d.auth_method = draft.auth_method;
                d.password = draft.password;
                d.private_key_path = draft.private_key_path;
                d.os_type = draft.os_type;
                d.serial_port = draft.serial_port;
                d.baud_rate = draft.baud_rate;
                d.remark = draft.remark;
                d.updated_at = now_iso();
            }
        }
    }
    editing.set(None);
    store.read().save_devices();
}

fn open_serial(store: Signal<AppStore>, device_id: i64) {
    let s = store.read();
    let mut serial_device = s.serial_device;
    let mut page = s.page;
    serial_device.set(Some(device_id));
    page.set(Page::Serial);
}
