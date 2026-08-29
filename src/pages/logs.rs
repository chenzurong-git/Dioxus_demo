use crate::store::AppStore;
use dioxus::prelude::*;
use dioxus::signals::ReadableExt;

#[allow(non_snake_case)]
pub fn SerialLogsPage() -> Element {
    let store = use_context::<Signal<AppStore>>();
    let content = use_signal(String::new);
    let current = use_signal(|| None::<String>);
    let error = use_signal(String::new);

    use_effect(move || {
        store.read().refresh_logs();
    });

    let logs = store.read().serial_logs;

    rsx! {
        h1 { class: "page-title", "串口日志" }
        if !error.cloned().is_empty() {
            div { class: "err", "{error}" }
        }
        div { class: "grid",
            div { class: "card",
                div { class: "card-title", "已保存的日志（{logs.read().len()}）" }
                if logs.read().is_empty() {
                    div { class: "hint", "暂无日志文件，串口打开时勾选录制即可生成" }
                }
                for (l, l_name, l_name_del) in logs.read().iter().cloned().map(|l| {
                    let name = l.name.clone();
                    (l, name.clone(), name)
                }) {
                    div { class: "row", style: "justify-content: space-between; margin-bottom: 6px",
                        button { onclick: move |_| read_log(store, l_name.clone(), content, current, error), "{l_name}" }
                        span { class: "hint", "{l.size} B · {l.modified_at}" }
                        button { class: "danger", onclick: move |_| delete_log(store, l_name_del.clone(), content, current, error), "删除" }
                    }
                }
            }
            div { class: "card", style: "min-width: 480px",
                div { class: "card-title", if let Some(c) = current.cloned() { "{c}" } else { "未选择日志" } }
                pre { class: "log-box", style: "max-height: 520px",
                    if content.cloned().is_empty() { "点击左侧日志文件查看内容" } else { "{content}" }
                }
            }
        }
    }
}

fn read_log(
    store: Signal<AppStore>,
    name: String,
    mut content: Signal<String>,
    mut current: Signal<Option<String>>,
    mut error: Signal<String>,
) {
    let s = store.read();
    match s.logs.read(&name, 1024 * 1024) {
        Ok(text) => {
            content.set(text);
            current.set(Some(name));
            error.set(String::new());
        }
        Err(e) => error.set(e),
    }
}

fn delete_log(
    store: Signal<AppStore>,
    name: String,
    mut content: Signal<String>,
    mut current: Signal<Option<String>>,
    mut error: Signal<String>,
) {
    let s = store.read();
    match s.logs.delete(&name) {
        Ok(()) => {
            if current.cloned().as_deref() == Some(name.as_str()) {
                content.set(String::new());
                current.set(None);
            }
            s.refresh_logs();
            error.set(String::new());
        }
        Err(e) => error.set(e),
    }
}
