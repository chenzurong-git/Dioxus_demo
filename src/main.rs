use dioxus::prelude::*;

fn main() {
    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::new().with_window(
                dioxus::desktop::WindowBuilder::new()
                    .with_title("Dioxus 桌面演示")
                    .with_inner_size(dioxus::desktop::LogicalSize::new(920.0, 640.0)),
            ),
        )
        .launch(app);
}

/// 应用根组件：一个计数器 + 一个待办事项列表，演示
/// use_signal 状态管理、事件处理、组件渲染等 Dioxus 核心概念。
fn app() -> Element {
    let mut count = use_signal(|| 0);
    let mut todos = use_signal(Vec::<String>::new);
    let mut draft = use_signal(String::new);

    rsx! {
        style { {include_str!("./style.css")} }

        div { class: "app",
            header { class: "hero",
                h1 { "Dioxus 🦀 Windows" }
                p { "用 Rust + Dioxus 编写的跨平台桌面应用示例" }
            }

            main {
                section { class: "card",
                    h2 { "计数器" }
                    div { class: "counter",
                        button { onclick: move |_| count -= 1, "−" }
                        span { class: "count", "{count}" }
                        button { onclick: move |_| count += 1, "+" }
                    }
                    button { class: "ghost", onclick: move |_| count.set(0), "清零" }
                }

                section { class: "card",
                    h2 { "待办事项（{todos.len()}）" }
                    div { class: "row",
                        input {
                            placeholder: "输入新事项，回车添加",
                            value: "{draft}",
                            oninput: move |e| draft.set(e.value()),
                            onkeydown: move |e| {
                                if e.key() == Key::Enter {
                                    add_todo(todos, draft);
                                }
                            },
                        }
                        button { onclick: move |_| add_todo(todos, draft), "添加" }
                    }
                    ul { class: "todos",
                        for (i, item) in todos.iter().enumerate() {
                            li { key: "{i}",
                                span { "{item}" }
                                button { class: "del", onclick: move |_| { todos.remove(i); }, "删除" }
                            }
                        }
                    }
                    if todos.is_empty() {
                        p { class: "empty", "暂无待办事项，先加一个吧～" }
                    }
                }
            }
        }
    }
}

fn add_todo(mut todos: Signal<Vec<String>>, mut draft: Signal<String>) {
    let text = draft().trim().to_string();
    if !text.is_empty() {
        todos.push(text);
        draft.set(String::new());
    }
}
