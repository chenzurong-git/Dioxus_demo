# Dioxus 桌面应用示例

用 [Dioxus](https://dioxuslabs.com)（Rust 生态的 React 风格 UI 框架）写的一个跨平台桌面应用，
当前代码已验证可编译到 Windows 目标。

应用包含两个小组件，覆盖 Dioxus 最常用的概念：

- 计数器：`use_signal` 状态读写、事件处理
- 待办事项：输入框双向绑定、列表渲染、增删操作

## 在 Windows 上运行

1. 安装 Rust：https://rustup.rs （安装时会自动带上 MSVC 工具链）
2. 确认已安装 WebView2 运行时（Win10/11 + Edge 一般自带；
   没有的话到 https://developer.microsoft.com/microsoft-edge/webview2 下载 Evergreen 运行时）
3. 在本目录执行：

   ```powershell
   cargo run
   ```

发布构建（体积更小、更流畅）：

   ```powershell
   cargo build --release
   ```

产物在 `target/release/dioxus-demo.exe`。

### 无 Visual Studio：用 w64devkit（GNU/MinGW）

装不了 Visual Studio 时，用 [w64devkit](https://github.com/skeeto/w64devkit/releases)
（绿色免安装，自带 gcc / dlltool / mingw32-make）。注意要下 **x64** 版本，
包名形如 `w64devkit-x64-2.9.0.exe`（自解压 exe，解压后是 `w64devkit/bin`）。

在 cmd 中（新开一个干净的窗口）：

```cmd
cd /d Z:\wspace\tina-v821\brandy\up_tool\Dioxus
set PATH=D:\w64devkit\bin;%PATH%

:: 确认是 64 位，应输出 x86_64-w64-mingw32
D:\w64devkit\bin\gcc.exe -dumpmachine

:: 安装并切换 64 位 GNU 工具链（只需一次）
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu

cargo run
```

注意：不要把 32 位工具链（`stable-i686-pc-windows-gnu`）设成默认，
也不要让 PATH 里混着 i686 的 `self-contained` 目录，否则会重现 dlltool 报错。
`.cargo/config.toml` 已把 `x86_64-pc-windows-gnu` 的链接器设为 `gcc`。

如果链接报 `ld.exe: cannot find -lgcc_eh`，这是 w64devkit 的已知问题
（它没有构建 `libgcc_eh.a`）。把 `libgcc.a` 复制一份命名为 `libgcc_eh.a` 即可，
在 cmd 中执行一次：

```cmd
for /r D:\w64devkit %i in (libgcc.a) do @copy /y "%i" "%~dpi\libgcc_eh.a" >nul
gcc -print-file-name=libgcc_eh.a
```

`gcc -print-file-name=libgcc_eh.a` 输出真实路径而不是 `libgcc_eh.a` 字样，就说明生效了。

#### 如果在网络盘/映射盘上编译报 `dlltool: os error 267`

`Z:` 这类网络映射盘上，rustc 调 dlltool 生成导入库时会失败
（报错 `error calling dlltool '...': 目录名称无效。 (os error 267)`）。
这与 dlltool 版本无关，把项目拷到本地盘（如 `C:`）编译即可：

```cmd
robocopy Z:\wspace\tina-v821\brandy\up_tool\Dioxus C:\dioxus-demo /E /XD target .git
cd /d C:\dioxus-demo
set PATH=D:\w64devkit\bin;%PATH%
cargo run
```

每次改完代码同步一下即可（robocopy 只复制差异）。

## 在 Linux 上交叉编译（可选）

需要 mingw-w64 工具链和 Rust 的 `x86_64-pc-windows-gnu` 目标：

```bash
sudo apt install gcc-mingw-w64-x86-64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

## 代码结构

- `src/main.rs` — 入口（窗口配置）与 UI 组件
- `src/style.css` — 全局样式（通过 `include_str!` 注入）
- `Cargo.toml` — 依赖：`dioxus = { version = "0.7", features = ["desktop"] }`

## 本仓库内的工具链

`/.toolchain/` 下有一份通过 USTC 镜像安装的 Rust 1.98 工具链（已 gitignore），
使用方式：

```bash
export PATH=$PWD/.toolchain/cargo/bin:$PATH
export CARGO_HOME=$PWD/.toolchain/cargo
export RUSTUP_HOME=$PWD/.toolchain/rustup
```

cargo 已配置 USTC 稀疏索引镜像（`/.toolchain/cargo/config.toml`）。
注意：本机没有安装 GTK/WebKit 等 Linux GUI 依赖，桌面应用请直接在 Windows 上运行。

## 学习资源

- 官方文档：https://dioxuslabs.com/learn/0.7/
- 官方示例仓库：https://github.com/DioxusLabs/dioxus/tree/main/packages/desktop/examples
