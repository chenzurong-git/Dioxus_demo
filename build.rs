use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let Ok(out_dir) = env::var("OUT_DIR") else {
        return;
    };
    let out_dir = Path::new(&out_dir);

    // out_dir: target/<triple>/<profile>/build/<crate>-<hash>/out
    let Some(profile_dir) = out_dir.ancestors().nth(3) else {
        return;
    };
    let Some(build_dir) = out_dir.ancestors().nth(2) else {
        return;
    };

    let arch = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86") => "x86",
        _ => "x64",
    };

    // webview2-com-sys 构建时会把 DLL 复制到它自己的 OUT_DIR，优先在那里找
    let src = find_loader_in_build_dir(build_dir, arch)
        .or_else(|| find_loader_in_registry(arch));

    match src {
        Some(src) => {
            let dest = profile_dir.join("WebView2Loader.dll");
            if fs::copy(&src, &dest).is_err() {
                println!(
                    "cargo:warning=复制 WebView2Loader.dll 到 {} 失败",
                    dest.display()
                );
            } else {
                println!("cargo:rerun-if-changed={}", src.display());
            }
        }
        None => {
            println!("cargo:warning=未找到 WebView2Loader.dll，运行 dioxus-demo.exe 可能报缺少该 DLL");
        }
    }
}

fn find_loader_in_build_dir(build_dir: &Path, arch: &str) -> Option<PathBuf> {
    for entry in fs::read_dir(build_dir).ok()?.flatten() {
        let name = entry.file_name();
        let name = name.to_str()?;
        if !name.starts_with("webview2-com-sys-") {
            continue;
        }
        let loader = entry.path().join("out").join(arch).join("WebView2Loader.dll");
        if loader.is_file() {
            return Some(loader);
        }
    }
    None
}

fn find_loader_in_registry(arch: &str) -> Option<PathBuf> {
    let cargo_home = env::var("CARGO_HOME").ok()?;
    let src_root = Path::new(&cargo_home).join("registry").join("src");
    for entry in fs::read_dir(src_root).ok()?.flatten() {
        let path = entry.path();
        let is_sys = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("webview2-com-sys-"))
            .unwrap_or(false);
        if !is_sys {
            continue;
        }
        let loader = path.join(arch).join("WebView2Loader.dll");
        if loader.is_file() {
            return Some(loader);
        }
    }
    None
}
