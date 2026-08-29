#![windows_subsystem = "windows"]

mod models;
mod pages;
mod services;
mod store;

use dioxus::prelude::*;
use pages::{adb::AdbPage, devices::DevicesPage, firmware::FirmwarePage, host::HostOpsPage, logs::SerialLogsPage, quick::QuickCommandsPage, serial::SerialTerminalPage, settings::SettingsPage};
use store::{AppStore, Page};

fn main() {
    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::new().with_window(
                dioxus::desktop::WindowBuilder::new()
                    .with_title("Open Workbench")
                    .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 760.0)),
            ),
        )
        .launch(app);
}

const NAV: [(Page, &str, &str); 8] = [
    (Page::Devices, "▣", "我的设备"),
    (Page::Serial, "⌁", "串口终端"),
    (Page::Logs, "≡", "串口日志"),
    (Page::Firmware, "◈", "固件烧录"),
    (Page::Adb, "▤", "ADB 设备"),
    (Page::Host, "□", "主机操作"),
    (Page::Quick, "⚡", "快捷命令"),
    (Page::Settings, "⚙", "设置"),
];

fn app() -> Element {
    let store = use_context_provider(|| Signal::new(AppStore::new()));
    let mut page = store.read().page;

    rsx! {
        style { {include_str!("./style.css")} }

        div { class: "app-shell",
            aside { class: "sidebar",
                div { class: "brand", "Open Workbench" }
                nav {
                    for (p, icon, label) in NAV {
                        button {
                            class: if page() == p { "nav-item active" } else { "nav-item" },
                            onclick: move |_| page.set(p),
                            span { class: "nav-icon", "{icon}" }
                            "{label}"
                        }
                    }
                }
            }
            main { class: "content",
                match page() {
                    Page::Devices => rsx! { DevicesPage {} },
                    Page::Serial => rsx! { SerialTerminalPage {} },
                    Page::Logs => rsx! { SerialLogsPage {} },
                    Page::Firmware => rsx! { FirmwarePage {} },
                    Page::Adb => rsx! { AdbPage {} },
                    Page::Host => rsx! { HostOpsPage {} },
                    Page::Quick => rsx! { QuickCommandsPage {} },
                    Page::Settings => rsx! { SettingsPage {} },
                }
            }
        }
    }
}
