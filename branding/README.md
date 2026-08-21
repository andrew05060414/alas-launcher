# Personal ALAS branding

The files in `branding/alas/` are the canonical launcher icons for the
personal ALAS-style build.

`build.rs` copies these files to `icons/` before every Cargo build. This keeps
the launcher executable, window, taskbar, and tray branding stable even when
upstream changes `icons/`.

The presence of these canonical assets also disables launcher binary
self-update for this personal build. AzurPilot repository and dependency
updates still run normally. Rebuild this branch manually when adopting a new
launcher release so a cloud binary cannot silently replace the personal icon.

The files in `branding/alas-app/` are embedded into the launcher and copied
to `assets/gui/icon/` and `assets/spa/` after the AzurPilot repository update
completes and before `gui.py` starts. This keeps the WebUI avatar, PWA icon,
and status icons independent from the main repository source.

Only allowlisted static branding paths are copied. Missing targets produce a
warning and do not prevent the scheduler from starting.

Build the Windows release from this branch with:

```powershell
cargo build --release --locked
```

Deploy `target/release/alas-launcher.exe` beside the AzurPilot repository.
Do not copy certificates, deploy configuration, or other machine-local files
into this repository.
