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

`branding/alas-app/avatar-192.png` is embedded into the launcher. After each
page load, `src/branding.rs` injects a DOM overlay into the main WebView of the
React frontend:

- `img.brand-logo` (sidebar and login card) shows the Richelieu avatar (brand name and font stay upstream)
- the favicon uses the same avatar
- the titlebar buttons keep the rectangular Windows style (see `LAUNCHER_BUILD.md`)

A `MutationObserver` reapplies the overlay after React re-renders. Nothing is
written into the AzurPilot checkout, so `git reset` during repository updates
cannot undo it and the checkout stays clean. The old PyWebIO overlay (files
under `assets/gui/`, CSS patches, `module/webui/base.py`) was removed because
AzurPilot no longer ships that WebUI.

Build the Windows release from this branch with:

```powershell
cargo build --release --locked
```

Deploy `target/release/alas-launcher.exe` beside the AzurPilot repository.
Do not copy certificates, deploy configuration, or other machine-local files
into this repository.
