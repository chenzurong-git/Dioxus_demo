use crate::models::Settings;
use crate::store::AppStore;
use dioxus::prelude::*;
use dioxus::signals::ReadableExt;

#[allow(non_snake_case)]
pub fn SettingsPage() -> Element {
    let store = use_context::<Signal<AppStore>>();
    let mut s = use_signal(|| store.read().settings.cloned());
    let mut msg = use_signal(String::new);
    let err = use_signal(String::new);

    let tunnel_status = store.read().tunnel_status;
    let mcp_status = store.read().mcp_status;
    let st = s.cloned();

    rsx! {
        h1 { class: "page-title", "设置" }
        if !err.cloned().is_empty() {
            div { class: "err", "{err}" }
        }
        if !msg.cloned().is_empty() {
            div { class: "ok", "{msg}" }
        }

        div { class: "card",
            div { class: "card-title", "串口日志" }
            div { class: "grid",
                label {
                    input { r#type: "checkbox", checked: st.log_enabled, oninput: move |e| { let mut v = s.cloned(); v.log_enabled = e.value() == "true"; s.set(v); } }
                    " 自动保存串口日志"
                }
                div { class: "col",
                    label { "单文件大小上限 (字节)" }
                    input { r#type: "number", value: "{st.log_max_file_size}", oninput: move |e| { let mut v = s.cloned(); v.log_max_file_size = e.value().parse().unwrap_or(v.log_max_file_size); s.set(v); } }
                }
                div { class: "col",
                    label { "保留天数" }
                    input { r#type: "number", value: "{st.log_max_days}", oninput: move |e| { let mut v = s.cloned(); v.log_max_days = e.value().parse().unwrap_or(v.log_max_days); s.set(v); } }
                }
            }
        }

        div { class: "card",
            div { class: "card-title", "主机操作" }
            div { class: "grid",
                div { class: "col",
                    label { "允许访问根目录（留空放开全盘）" }
                    input { value: "{st.host_root}", oninput: move |e| { let mut v = s.cloned(); v.host_root = e.value(); s.set(v); } }
                }
                label {
                    input { r#type: "checkbox", checked: st.host_shell_enabled, oninput: move |e| { let mut v = s.cloned(); v.host_shell_enabled = e.value() == "true"; s.set(v); } }
                    " 启用主机 shell 命令"
                }
            }
            div { class: "hint", "建议填写一个工作目录以缩小暴露面；shell 命令不受根目录约束" }
        }

        div { class: "card",
            div { class: "card-title", "ADB" }
            div { class: "col", style: "max-width: 420px",
                label { "adb 可执行文件路径" }
                input { value: "{st.adb_path}", oninput: move |e| { let mut v = s.cloned(); v.adb_path = e.value(); s.set(v); } }
            }
        }

        div { class: "card",
            div { class: "card-title", "远程编译服务器隧道（SSH 反向端口转发）" }
            div { class: "grid",
                label {
                    input { r#type: "checkbox", checked: st.tunnel_enabled, oninput: move |e| { let mut v = s.cloned(); v.tunnel_enabled = e.value() == "true"; s.set(v); } }
                    " 启用隧道"
                }
                div { class: "col",
                    label { "主机地址" }
                    input { value: "{st.tunnel_host}", oninput: move |e| { let mut v = s.cloned(); v.tunnel_host = e.value(); s.set(v); } }
                }
                div { class: "col",
                    label { "SSH 端口" }
                    input { r#type: "number", value: "{st.tunnel_ssh_port}", oninput: move |e| { let mut v = s.cloned(); v.tunnel_ssh_port = e.value().parse().unwrap_or(v.tunnel_ssh_port); s.set(v); } }
                }
                div { class: "col",
                    label { "用户名" }
                    input { value: "{st.tunnel_user}", oninput: move |e| { let mut v = s.cloned(); v.tunnel_user = e.value(); s.set(v); } }
                }
                div { class: "col",
                    label { "认证方式" }
                    select {
                        value: "{st.tunnel_auth_method}",
                        onchange: move |e| { let mut v = s.cloned(); v.tunnel_auth_method = e.value(); s.set(v); },
                        option { value: "key", "私钥" }
                        option { value: "password", "密码" }
                    }
                }
                if st.tunnel_auth_method == "key" {
                    div { class: "col",
                        label { "私钥路径" }
                        input { value: "{st.tunnel_private_key}", oninput: move |e| { let mut v = s.cloned(); v.tunnel_private_key = e.value(); s.set(v); } }
                    }
                } else {
                    div { class: "col",
                        label { "密码（ssh 子进程需系统交互，建议用私钥）" }
                        input { r#type: "password", value: "{st.tunnel_password}", oninput: move |e| { let mut v = s.cloned(); v.tunnel_password = e.value(); s.set(v); } }
                    }
                }
                div { class: "col",
                    label { "远端绑定地址" }
                    input { value: "{st.tunnel_remote_bind}", oninput: move |e| { let mut v = s.cloned(); v.tunnel_remote_bind = e.value(); s.set(v); } }
                }
                div { class: "col",
                    label { "远端端口" }
                    input { r#type: "number", value: "{st.tunnel_remote_port}", oninput: move |e| { let mut v = s.cloned(); v.tunnel_remote_port = e.value().parse().unwrap_or(v.tunnel_remote_port); s.set(v); } }
                }
            }
            div { class: "hint", "反向隧道：远端编译服务器通过 远端绑定地址:远端端口 访问本机同一端口" }
            div { class: "row", style: "margin-top: 8px",
                span { class: if tunnel_status.cloned().running { "badge running" } else { "badge off" },
                    if tunnel_status.cloned().running { "隧道运行中" } else { "隧道未运行" }
                }
                if tunnel_status.cloned().running {
                    span { class: "hint", "{tunnel_status.cloned().command}" }
                }
            }
        }

        div { class: "card",
            div { class: "card-title", "MCP 服务器（AI 客户端接入）" }
            div { class: "grid",
                label {
                    input { r#type: "checkbox", checked: st.mcp_enabled, oninput: move |e| { let mut v = s.cloned(); v.mcp_enabled = e.value() == "true"; s.set(v); } }
                    " 启用 MCP 服务"
                }
                div { class: "col",
                    label { "监听端口" }
                    input { r#type: "number", value: "{st.mcp_port}", oninput: move |e| { let mut v = s.cloned(); v.mcp_port = e.value().parse().unwrap_or(v.mcp_port); s.set(v); } }
                }
                div { class: "col",
                    label { "访问令牌（留空则不校验）" }
                    input { value: "{st.mcp_token}", oninput: move |e| { let mut v = s.cloned(); v.mcp_token = e.value(); s.set(v); } }
                }
            }
            div { class: "row", style: "margin-top: 8px",
                button { onclick: move |_| {
                    let st = store.read().mcp.status();
                    let running = st.running;
                    let port = st.port;
                    let tools = st.tools.len();
                    msg.set(if running { format!("MCP 服务运行中（127.0.0.1:{port}，{tools} 个工具）") } else { "MCP 服务未运行".into() });
                }, "查询 MCP 状态" }
                span { class: if mcp_status.cloned().running { "badge running" } else { "badge off" },
                    if mcp_status.cloned().running { "MCP 运行中" } else { "MCP 未运行" }
                }
            }
        }

        div { class: "row", style: "margin-top: 16px",
            button { class: "primary", onclick: move |_| save(store, s, msg, err), "保存并应用" }
        }
    }
}

fn save(mut store: Signal<AppStore>, s: Signal<Settings>, mut msg: Signal<String>, mut err: Signal<String>) {
    match store.write().apply_settings(s.cloned()) {
        Ok(()) => {
            msg.set("设置已保存并应用".into());
            err.set(String::new());
        }
        Err(e) => {
            msg.set(String::new());
            err.set(e);
        }
    }
}
