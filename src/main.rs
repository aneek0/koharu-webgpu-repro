//! Minimal repro for tauri-apps/tauri#16210 / #16203:
//! `Cef::command_line_args` reach only the browser process — the GPU child
//! process never sees `use-angle=vulkan` / `enable-unsafe-webgpu`, so on Linux
//! `navigator.gpu.requestAdapter()` returns null (no Vulkan-capable ANGLE build
//! in the GPU process, SwiftShader excluded) or falls back to software.
//!
//! Run: `cargo run`, then inspect the CEF children of this process:
//!   for pid in $(pgrep -f koharu-webgpu-repro); do
//!     echo "PID $pid: $(tr '\0' ' ' < /proc/$pid/cmdline)"; echo
//!   done
//! On stock v3 the GPU child (`--type=gpu-process`) has no `--use-angle` /
//! `--enable-unsafe-webgpu`. With PR #16210's `child_process_command_line_args`
//! (see the `pr-16210` branch of this repo) they are there.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let cef = tauri_runtime_cef::Cef::default()
        // exactly what koharu sets in crates/koharu-app/src/app.rs
        .enable_features(["Vulkan", "VulkanFromANGLE"])
        .command_line_args([
            ("--enable-unsafe-webgpu", None),
            ("use-angle", Some("vulkan")),
            ("--ozone-platform", Some("x11")),
        ]);
    tauri::Builder::<tauri_runtime_cef::CefRuntime>::new()
        .runtime(cef)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
