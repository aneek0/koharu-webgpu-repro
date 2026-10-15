# koharu-webgpu-repro

Minimal repro for [tauri-apps/tauri#16210](https://github.com/tauri-apps/tauri/pull/16210) / [#16203](https://github.com/tauri-apps/tauri/issues/16203):
switches an application sets through `Cef::command_line_args` never reach CEF's child processes, so on Linux the GPU process has no ANGLE backend / feature flags and `navigator.gpu.requestAdapter()` finds no hardware adapter.

One window, no plugins, no frontend build step (`ui/index.html` is the whole frontend). Verified on Linux (X11) with CEF 154.0.34 / tauri v3 @ `1991873f5`.

## Run

```bash
cargo run            # or: CEF_PATH=<cef cache> cargo run
```

Then inspect the CEF children of the process — the GPU process is the `--type=zygote` child whose threads include `GpuWatchdog`/`VizCompositorTh` (it forks from the zygote, so `/proc/<pid>/cmdline` still says zygote):

```bash
for pid in $(pgrep -f koharu-webgpu-repro); do
  gpu=0
  for t in $(ls /proc/$pid/task); do
    case $(cat /proc/$pid/task/$t/comm 2>/dev/null) in GpuWatchdog*|VizCompositor*) gpu=1;; esac
  done
  [ "$gpu" = 1 ] && { echo "GPU PID $pid:"; tr '\0' '\n' < /proc/$pid/cmdline; }
done
```

## What you should see

| | GPU child command line | `requestAdapter()` in the window |
|---|---|---|
| `main` (stock v3 `1991873f5`) | no `--use-angle`, no `--enable-unsafe-webgpu` | software/null; GPU log: `Failed to create and initialize Vulkan implementation.` |
| `pr-16210` (this PR) | `--enable-unsafe-webgpu --use-angle=vulkan` appended | `OK — {"vendor":"amd","architecture":"rdna-3"}` |

Captured evidence in [`evidence/`](evidence/): both GPU command lines, the stock GPU log, and a screenshot of the adapter result on the PR branch.

The app sets exactly what koharu sets (`crates/koharu-app/src/app.rs`):
`.enable_features(["Vulkan", "VulkanFromANGLE"]).command_line_args([("--enable-unsafe-webgpu", None), ("use-angle", Some("vulkan")), ("--ozone-platform", Some("x11"))])`.

## Why the switches vanish

`Cef::command_line_args` is browser-process only. libcef builds each child's command line in
`ChromeContentBrowserClientCef::AppendExtraCommandLineSwitches`
(`libcef/browser/chrome/chrome_content_browser_client_cef.cc`, copy in `evidence/`)
from a fixed allowlist — renderer gets only `kUncaughtExceptionStackSize`, everything else gets resource/log paths — and then calls the embedder's `OnBeforeChildProcessLaunch`. `use-angle` / `enable-unsafe-webgpu` are not on any allowlist, so the only supported way to get them to the GPU process is the hook this PR adds.

## Branches

- `main` — stock `v3` git deps (`1991873f5`), the bug.
- `pr-16210` — same app plus `Cef::child_process_command_line_args([...])`, built against the PR branch through a `[patch."https://github.com/tauri-apps/tauri"]` section pointing at a local checkout of the PR branch (swap the paths for your own checkout).
