# Blast radius of the bug fixed by tauri#16210

Scope of "`Cef::command_line_args` switches never reach CEF child processes",
established 2026-10-16 from `evidence/chrome_content_browser_client_cef.cc` and
live runs of this app. Not posted anywhere — reference only.

## Affected

| Who | Verdict | Evidence |
|---|---|---|
| koharu (Linux) | affected, confirmed e2e | `evidence/stock-gpu-cmdline.txt` (switches absent, `Failed to create and initialize Vulkan implementation`) vs `evidence/pr-gpu-cmdline.txt` + `pr-adapter-result*.png` (`{"vendor":"amd","architecture":"rdna-3"}`) |
| any current v3 app wanting its own ANGLE/WebGPU backend | affected class | no app configures such switches today: `tauri-runtime-cef` has 0 dependents on crates.io, tauri v3 is alpha |
| any future app passing a GPU/utility-process switch through `command_line_arg` | affected class | the CEF allowlist below |

## Not affected

| Who | Why |
|---|---|
| Tauri on wry (v2) | bug is in the CEF-specific `command_line_arg` API; wry has no such path |
| JetBrains JCEF, CefSharp, cefpython, root (`gui/cefdisplay`), NanUI, linux-wallpaperengine, MTA:SA, PresentMon | all already implement `OnBeforeChildProcessLaunch`, i.e. they work around it |
| `internal_command_line_args` (`--ozone-platform`, `--no-first-run`) | the runtime delivers those to every process itself via `on_before_command_line_processing` |

## Mechanism

`command_line_arg` is documented "applied to the **browser process only**" — a
CEF restriction, not a koharu one. In
`ChromeContentBrowserClientCef::AppendExtraCommandLineSwitches`
(`evidence/chrome_content_browser_client_cef.cc`):

- L227 calls the Chromium base implementation;
- L242-252: every child gets exactly `kLocalesDirPath`, `kLogItems`,
  `kLogSeverity`, `kResourcesDirPath`, `kUserAgentProductAndVersion`;
- L288-291: renderer additionally gets `kUncaughtExceptionStackSize`;
- L298-307: then the embedder's `OnBeforeChildProcessLaunch` is called.

Anything else the app put on the browser command line is dropped. That hook is
the only supported delivery path — #16241 already used it for
`--tauri-cef-api-version`, #16210 extends it to application switches.

## Wider class

Any switch Chromium does not forward on its own but a child process needs:
`--use-angle`, `--enable-unsafe-webgpu`, `--use-gl`,
`--enable-gpu-rasterization`, `--in-process-gpu`. Mostly a Linux problem — the
GPU and renderer are separate processes there, while on Windows/macOS they are
part of the browser process. `[INFERENCE]` — not verified directly on those
platforms.

## Related

- koharu-rs/koharu#1236 (hug0-l) fixes the symptom on the koharu config side
  by dropping `VulkanFromANGLE`; complementary, not exclusive with #16210.