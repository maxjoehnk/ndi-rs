use std::env;
use std::path::{Path, PathBuf};

fn get_output_path() -> PathBuf {
    // TODO: find a better path to this stuff
    Path::new(&env::var("OUT_DIR").unwrap()).join("../../../deps")
}

fn win_link_and_load() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    if arch == "x86_64" {
        println!("cargo:rustc-link-lib=Processing.NDI.Lib.x64");
    } else {
        println!("cargo:rustc-link-lib=Processing.NDI.Lib.x86");
    }

    let mut lib_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    lib_path.push("thirdparty\\Windows\\Lib");
    println!("cargo:rustc-link-search={}", lib_path.to_str().unwrap());

    // copy dll to OUT_DIR
    let out_path = get_output_path();

    let dll_name = if arch == "x86_64" {
        "Processing.NDI.Lib.x64.dll"
    } else {
        "Processing.NDI.Lib.x86.dll"
    };

    let dll_path = format!("thirdparty\\Windows\\Bin\\{}", dll_name);

    let src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join(dll_path);
    let dst = Path::join(&out_path, dll_name);
    std::fs::copy(src, dst).unwrap();
}

fn linux_link_and_load() {
    println!("cargo:rustc-link-lib=ndi",);
    let mut lib_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    lib_path.push("thirdparty/Linux/Lib");
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    match arch.as_ref() {
        "i686" => lib_path.push("i686-linux-gnu"),
        "x86_64" => lib_path.push("x86_64-linux-gnu"),
        "aarch64" => lib_path.push("aarch64-linux-gnu"),
        "arm" => lib_path.push("arm-linux-gnu"),
        _ => panic!("Unsupported architecture for NDI"),
    }
    println!("cargo:rustc-link-search={}", lib_path.to_str().unwrap());

    // copy dll to OUT_DIR
    let out_path = get_output_path();
    let mut lib_path =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("thirdparty/Linux/Lib");
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    match arch.as_ref() {
        "i686" => lib_path.push("i686-linux-gnu"),
        "x86_64" => lib_path.push("x86_64-linux-gnu"),
        "aarch64" => lib_path.push("aarch64-linux-gnu"),
        "arm" => lib_path.push("arm-linux-gnu"),
        _ => panic!("Unsupported architecture for NDI"),
    }
    let src = lib_path.join("libndi.so.5");
    let dst = Path::join(&out_path, "libndi.so.5");
    std::fs::copy(src, dst).unwrap();
}

fn macos_link_and_load() {
    println!("cargo:rustc-link-lib=ndi",);
    let mut lib_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    lib_path.push("thirdparty/Macos/lib/macOS");
    println!("cargo:rustc-link-search={}", lib_path.to_str().unwrap());

    // copy dll to OUT_DIR
    let out_path = get_output_path();
    let mut lib_path =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("thirdparty/Macos/lib/macOS");
    let src = lib_path.join("libndi.dylib");
    let dst = Path::join(&out_path, "libndi.dylib");
    std::fs::copy(src, dst).unwrap();
}

fn main() {
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    match os.as_str() {
        "windows" => win_link_and_load(),
        "linux" => linux_link_and_load(),
        "macos" => macos_link_and_load(),
        _ => panic!("Unsupported OS for NDI"),
    };
}
