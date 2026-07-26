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

Build the Windows release from this branch with:

```powershell
cargo build --release --locked
```

Deploy `target/release/alas-launcher.exe` beside the AzurPilot repository.
Do not copy certificates, deploy configuration, or other machine-local files
into this repository.
