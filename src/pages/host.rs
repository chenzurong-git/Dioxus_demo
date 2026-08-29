use crate::models::HostEntry;
use crate::services::host::HostService;
use crate::store::AppStore;
use dioxus::prelude::*;
use dioxus::signals::ReadableExt;

#[allow(non_snake_case)]
pub fn HostOpsPage() -> Element {
    let store = use_context::<Signal<AppStore>>();
    let mut path = use_signal(|| "C:\\".to_string());
    let entries = use_signal(Vec::<HostEntry>::new);
    let mut file_path = use_signal(String::new);
    let content = use_signal(String::new);
    let mut write_path = use_signal(String::new);
    let mut write_content = use_signal(String::new);
    let mut cmd = use_signal(String::new);
    let cmd_out = use_signal(String::new);
    let error = use_signal(String::new);

    rsx! {
        h1 { class: "page-title", "主机操作" }
        if !error.cloned().is_empty() {
            div { class: "err", "{error}" }
        }
        div { class: "hint", "文件路径受设置页「允许访问根目录」约束；命令执行需在设置页启用主机 shell" }

        div { class: "card",
            div { class: "card-title", "浏览目录" }
            div { class: "row",
                input { style: "flex: 1", value: "{path}", oninput: move |e| path.set(e.value()) }
                button { class: "primary", onclick: move |_| { let root = store.read().settings.cloned().host_root; browse(&root, path, entries, error); }, "列出" }
            }
            if !entries.read().is_empty() {
                table { class: "table", style: "margin-top: 8px",
                    thead { tr { th { "名称" } th { "类型" } th { "大小" } th { "修改时间" } } }
                    tbody {
                        for (e, e_full) in entries.read().iter().cloned().map(|e| {
                            let full = e.clone();
                            (e, full)
                        }) {
                            tr { key: "{e.path}",
                                td {
                                    button { style: "border: none; background: none; padding: 0",
                                        onclick: move |_| {
                                            let root = store.read().settings.cloned().host_root;
                                            if e_full.is_dir {
                                                let mut path = path;
                                                path.set(e_full.path.clone());
                                                browse(&root, path, entries, error);
                                            } else {
                                                let mut file_path = file_path;
                                                file_path.set(e_full.path.clone());
                                                read_file(&root, file_path, content, error);
                                            }
                                        },
                                        if e_full.is_dir { "📁 {e_full.name}" } else { "📄 {e_full.name}" }
                                    }
                                }
                                td { if e_full.is_dir { "目录" } else { "文件" } }
                                td { "{e_full.size}" }
                                td { class: "muted", "{e_full.modified_at}" }
                            }
                        }
                    }
                }
            }
        }

        div { class: "grid",
            div { class: "card",
                div { class: "card-title", "读取文件" }
                div { class: "row",
                    input { style: "flex: 1", value: "{file_path}", oninput: move |e| file_path.set(e.value()) }
                    button { class: "primary", onclick: move |_| { let root = store.read().settings.cloned().host_root; read_file(&root, file_path, content, error); }, "读取" }
                }
                textarea { rows: 8, style: "width: 100%; margin-top: 8px", value: "{content}", readonly: true, oninput: move |_| {} }
            }
            div { class: "card",
                div { class: "card-title", "写入文件" }
                input { style: "width: 100%", placeholder: "目标文件绝对路径", value: "{write_path}", oninput: move |e| write_path.set(e.value()) }
                textarea { rows: 6, style: "width: 100%; margin-top: 8px", placeholder: "要写入的内容", value: "{write_content}", oninput: move |e| write_content.set(e.value()) }
                button { class: "primary", style: "margin-top: 8px", onclick: move |_| { let root = store.read().settings.cloned().host_root; write_file(&root, write_path, write_content, error); }, "写入（覆盖）" }
            }
        }

        div { class: "card",
            div { class: "card-title", "执行命令" }
            div { class: "row",
                input {
                    style: "flex: 1",
                    placeholder: "例如 dir /b 或 ipconfig",
                    value: "{cmd}",
                    oninput: move |e| cmd.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            let c = cmd.cloned();
                            let shell = store.read().settings.cloned().host_shell_enabled;
                            exec(shell, &c, cmd_out, error);
                        }
                    },
                }
                button { class: "primary", onclick: move |_| { let c = cmd.cloned(); let shell = store.read().settings.cloned().host_shell_enabled; exec(shell, &c, cmd_out, error); }, "执行" }
            }
            pre { class: "log-box", style: "margin-top: 8px", if cmd_out.cloned().is_empty() { "（输出将显示在这里）" } else { "{cmd_out}" } }
        }
    }
}

fn browse(root: &str, path: Signal<String>, mut entries: Signal<Vec<HostEntry>>, mut error: Signal<String>) {
    match HostService::list_dir(root, &path.cloned()) {
        Ok(e) => {
            entries.set(e);
            error.set(String::new());
        }
        Err(e) => error.set(e),
    }
}

fn read_file(root: &str, file_path: Signal<String>, mut content: Signal<String>, mut error: Signal<String>) {
    match HostService::read_file(root, &file_path.cloned(), 1024 * 1024) {
        Ok(text) => {
            content.set(text);
            error.set(String::new());
        }
        Err(e) => error.set(e),
    }
}

fn write_file(root: &str, write_path: Signal<String>, write_content: Signal<String>, mut error: Signal<String>) {
    match HostService::write_file(root, &write_path.cloned(), &write_content.cloned(), false) {
        Ok(()) => error.set(String::new()),
        Err(e) => error.set(e),
    }
}

fn exec(shell_enabled: bool, command: &str, mut cmd_out: Signal<String>, mut error: Signal<String>) {
    match HostService::exec(shell_enabled, command) {
        Ok(out) => {
            cmd_out.set(out);
            error.set(String::new());
        }
        Err(e) => error.set(e),
    }
}
