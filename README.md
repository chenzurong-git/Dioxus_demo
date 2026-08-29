# Open Workbench（Dioxus 版）

用 [Dioxus](https://dioxuslabs.com)（Rust 生态的 React 风格 UI 框架）实现的
桌面工具台，功能参照 `up_tool/chenji/openworkbench/`（Tauri 2 复刻版）移植，
当前代码已验证可编译到 Windows 目标（`cargo check` 0 错误 0 警告）。

功能页面（左侧导航，8 个）：

- 我的设备：设备增删改查、打开对应串口
- 串口终端：串口开关、波特率/数据位设置、收发与录制
- 串口日志：日志文件列表、读取与删除
- 固件烧录：工作区固件上传/删除/烧录（`FlashBackend` 占位，可替换为 libefex 实现）
- ADB 设备：设备扫描/连接、文件浏览、命令执行、文件推送
- 主机操作：目录浏览、文件读写（受 `host_root` 约束）
- 快捷命令：多命令窗口、一键发送到串口
- 设置：串口/ADB/主机/烧录/隧道/MCP 参数持久化

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

如果运行 exe 报“找不到 WebView2Loader.dll”：GNU 工具链不会自动把
`WebView2Loader.dll`（WebView2 的加载器）放到 exe 旁边，MSVC 工具链会自动带。
本项目已内置 `build.rs`，每次 `cargo build`/`cargo run` 都会从
cargo 源码缓存（`webview2-com-sys` 包内）自动复制一份到 `target/<profile>/`，
一般无需手动处理。若仍提示缺少，手动复制一次即可：

```cmd
copy /y C:\Users\<用户名>\.cargo\registry\src\*\webview2-com-sys-*\x64\WebView2Loader.dll target\debug\
```

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

仓库内已附 `sync_cc_bench.bat`，一键把源码增量同步到 `D:\Desktop_MY\cc_bench`
（跳过 `target` / `.toolchain` / `.git`，本地已编译的 `target` 原样保留）：

```cmd
Z:\wspace\tina-v821\brandy\up_tool\Dioxus>sync_cc_bench.bat
```

## 正式发布

### 1. 构建 release 版

在本地盘目录（如 `D:\Desktop_MY\cc_bench`，避开 Z: 网络盘的 dlltool 问题）：

```cmd
cd /d D:\Desktop_MY\cc_bench
set PATH=D:\w64devkit\bin;%PATH%
cargo build --release
```

产物为 `target\release\dioxus-demo.exe`。想改 exe 名称，在 `Cargo.toml` 加：

```toml
[[bin]]
name = "open-workbench"
path = "src/main.rs"
```

### 2. 收集运行依赖

- `WebView2Loader.dll`：`build.rs` 已自动复制到 `target\release\`，发布时连同 exe 一起带上。
- WebView2 运行时：Win11 自带；Win10 需安装
  [Evergreen Runtime](https://developer.microsoft.com/microsoft-edge/webview2)（Win10 一般自带 Edge，也可能已满足）。
- 检查是否依赖 w64devkit 的运行库 DLL（一般不需要，Rust std 是静态链接的）：

  ```cmd
  objdump -p target\release\dioxus-demo.exe | findstr "DLL Name"
  ```

  若出现 `libgcc_s_seh-1.dll` / `libstdc++-6.dll` / `libwinpthread-1.dll`，
  把 `D:\w64devkit\bin` 里对应 DLL 一起放进发布目录。

### 3. 打包分发

- 绿色版：把 `dioxus-demo.exe` + `WebView2Loader.dll`（+ 需要的运行库 DLL）打成 zip，
  附一个说明（Win10 需装 WebView2 运行时）。
- 安装包：用 Inno Setup 或 NSIS 装到 Program Files、建开始菜单快捷方式，
  可在安装时静默安装 WebView2 bootstrapper（`EvergreenBootstrapper.exe /silent /install`）。
- 分发渠道：内网共享 / GitHub Releases。

### 4. 版本与数据

- 发布前在 `Cargo.toml` 更新版本号（如 `1.0.0`），完成后打 git tag。
- 用户数据存放在 `%APPDATA%\openworkbench\`（`settings.json` / `devices.json` /
  `quick_commands.json` / `logs\`），无需管理员权限；发布前把设置页默认值调好。
- 应用窗口图标目前是 Dioxus 默认图标，正式发布建议自定义图标
  （用 `.rc` 资源文件 + `winresource`/`embed-resource`，或 Dioxus 窗口图标 API）。
- 安全提醒：主机操作页的 `host_exec` 需在设置页显式开启，发布时保持默认关闭。

## 在 Linux 上交叉编译（可选）

需要 mingw-w64 工具链和 Rust 的 `x86_64-pc-windows-gnu` 目标：

```bash
sudo apt install gcc-mingw-w64-x86-64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

## 代码结构

- `src/main.rs` — 入口（窗口配置）与左侧导航布局
- `src/store.rs` — 全局状态（`Signal<AppStore>`，设置持久化到 JSON）
- `src/models.rs` — 数据模型
- `src/pages/` — 8 个功能页面（`devices` / `serial` / `logs` / `firmware` / `adb` / `host` / `quick` / `settings`）
- `src/services/` — 业务服务（串口、ADB、主机、日志、隧道、MCP、烧录占位）
- `src/style.css` — 全局样式（通过 `include_str!` 注入）
- `build.rs` — 构建时自动把 `WebView2Loader.dll` 复制到输出目录
- `Cargo.toml` — 依赖：`dioxus 0.7 (desktop)`、`serde`、`serialport`

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
