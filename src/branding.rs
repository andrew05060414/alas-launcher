use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
};

use sha2::{Digest, Sha256};
use tracing::{info, warn};

struct AppBrandingAsset {
    target: &'static str,
    data: &'static [u8],
    sha256: &'static str,
}

const APP_BRANDING_ASSETS: &[AppBrandingAsset] = &[
    AppBrandingAsset {
        target: "assets/gui/icon/alas.svg",
        data: include_bytes!("../branding/alas-app/alas.svg"),
        sha256: "60a6b095cc872bb149c9b213f0b2ea8e9344369aea808dd21d2c57a9f9818832",
    },
    AppBrandingAsset {
        target: "assets/spa/spa-icon-192x192.png",
        data: include_bytes!("../branding/alas-app/spa-icon-192x192.png"),
        sha256: "52690fec792f6d60c64ba40d856474b19d5697ebdeb01aabfa37a931015017dc",
    },
    AppBrandingAsset {
        target: "assets/spa/spa-icon-512x512.png",
        data: include_bytes!("../branding/alas-app/spa-icon-512x512.png"),
        sha256: "69436a6754a93bdaaa91e982319cf6e9eba5670fcaf67c1921d46da7a77bc61a",
    },
    AppBrandingAsset {
        target: "assets/gui/icon/add.svg",
        data: include_bytes!("../branding/alas-app/add.svg"),
        sha256: "b1b4094551a123d98222991c61a6d76c1b92de7b6060d1d038e2402f45fc2a6b",
    },
    AppBrandingAsset {
        target: "assets/gui/icon/develop.svg",
        data: include_bytes!("../branding/alas-app/develop.svg"),
        sha256: "6a70ac25bfd505a6df3996eeade6208a40cb581add9bb060c0d55f165141b840",
    },
    AppBrandingAsset {
        target: "assets/gui/icon/run.svg",
        data: include_bytes!("../branding/alas-app/run.svg"),
        sha256: "d51d6789a1837e8aab350ce7880854dd55ef0b965dace5025fdf7a1ccde51426",
    },
    AppBrandingAsset {
        target: "assets/gui/icon/setting.svg",
        data: include_bytes!("../branding/alas-app/setting.svg"),
        sha256: "06a17b1b9746dd8be8f444e1c23ebab4d701aab357d02c22f70c72b0392642037",
    },
    AppBrandingAsset {
        target: "assets/gui/icon/status_error.svg",
        data: include_bytes!("../branding/alas-app/status_error.svg"),
        sha256: "eb68374b1e1e1dd0fdade890d13435515a888bd7216d8d09055b7c56967bc7a4",
    },
    AppBrandingAsset {
        target: "assets/gui/icon/status_running.svg",
        data: include_bytes!("../branding/alas-app/status_running.svg"),
        sha256: "2e6079af85a1344fff21407a8f7b40a25c3d5950ed2b0e62cbf982e40222992c",
    },
    AppBrandingAsset {
        target: "assets/gui/icon/status_update.svg",
        data: include_bytes!("../branding/alas-app/status_update.svg"),
        sha256: "ff860932fce9a82614ba6bb5fce53cf4dd9d4b410280650e42a19248adaf1e36",
    },
];

const CSS_PATCH_START: &str = "/* alas-personal-webui-start */";
const CSS_PATCH_END: &str = "/* alas-personal-webui-end */";
const HEADER_UPSTREAM: &str = "put_text(\"港区OA\")";
const HEADER_PERSONAL: &str = "put_text(\"ALAS\")";

const ALAS_CSS_PATCH: &str = r#"/* alas-personal-webui-start */
#pywebio-scope-header {
    position: relative;
}
#pywebio-scope-header > #pywebio-scope-header_title {
    position: absolute !important;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    width: max-content;
    max-width: calc(100% - 14rem);
    margin: 0;
}
#pywebio-scope-header_title>p {
    margin: 0;
}
/* alas-personal-webui-end */
"#;

const ALAS_PC_CSS_PATCH: &str = r#"/* alas-personal-webui-start */
#pywebio-scope-overview {
    grid-template-rows: 1fr;
}
#pywebio-scope-daemon-overview {
    grid-template-rows: 1fr;
}
/* alas-personal-webui-end */
"#;

pub fn apply_app_branding() {
    let root = PathBuf::from(".");
    for asset in APP_BRANDING_ASSETS {
        if let Err(error) = apply_asset(&root, asset) {
            warn!(
                "Unable to apply personal ALAS app branding {}: {error:#}",
                asset.target
            );
        }
    }
    apply_webui_overrides(&root);
}

fn apply_webui_overrides(root: &Path) {
    for (target, patch) in [
        ("assets/gui/css/alas.css", ALAS_CSS_PATCH),
        ("assets/gui/css/alas-pc.css", ALAS_PC_CSS_PATCH),
    ] {
        if let Err(error) = ensure_css_patch(root, Path::new(target), patch) {
            warn!("Unable to apply personal ALAS CSS overlay {target}: {error:#}");
        }
    }
    if let Err(error) = ensure_header_branding(root, Path::new("module/webui/base.py")) {
        warn!("Unable to apply personal ALAS header overlay: {error:#}");
    }
}

fn apply_asset(root: &Path, asset: &AppBrandingAsset) -> anyhow::Result<()> {
    let relative = Path::new(asset.target);
    if !is_safe_relative_path(relative) {
        anyhow::bail!("branding target is not a safe relative path");
    }

    let target = root.join(relative);
    let Some(parent) = target.parent() else {
        anyhow::bail!("branding target has no parent directory");
    };
    if !parent.exists() {
        anyhow::bail!("target directory does not exist");
    }

    let mut hasher = Sha256::new();
    hasher.update(asset.data);
    let digest = format!("{:x}", hasher.finalize());
    if digest != asset.sha256 {
        anyhow::bail!("embedded branding asset hash mismatch");
    }

    if fs::read(&target).ok().as_deref() == Some(asset.data) {
        return Ok(());
    }

    write_atomic(&target, asset.data)?;
    info!("Applied personal ALAS app branding: {}", asset.target);
    Ok(())
}

fn ensure_css_patch(root: &Path, relative: &Path, patch: &str) -> anyhow::Result<()> {
    if !is_safe_patch_path(relative) {
        anyhow::bail!("webui overlay target is not a safe relative path");
    }
    let target = require_existing_file(root, relative)?;
    let current = fs::read_to_string(&target)?;
    let updated = upsert_marked_block(&current, patch);
    if updated == current {
        return Ok(());
    }
    write_atomic(&target, updated.as_bytes())?;
    info!("Applied personal ALAS CSS overlay: {}", relative.display());
    Ok(())
}

fn ensure_header_branding(root: &Path, relative: &Path) -> anyhow::Result<()> {
    if !is_safe_patch_path(relative) {
        anyhow::bail!("webui overlay target is not a safe relative path");
    }
    let target = require_existing_file(root, relative)?;
    let current = fs::read_to_string(&target)?;
    if !current.contains(HEADER_UPSTREAM) {
        return Ok(());
    }
    let updated = current.replace(HEADER_UPSTREAM, HEADER_PERSONAL);
    write_atomic(&target, updated.as_bytes())?;
    info!(
        "Applied personal ALAS header overlay: {}",
        relative.display()
    );
    Ok(())
}

fn upsert_marked_block(source: &str, patch: &str) -> String {
    let patch = patch.trim_end_matches(['\r', '\n']);
    if let Some(start) = source.find(CSS_PATCH_START) {
        let before = source[..start].trim_end_matches(['\r', '\n']);
        let rest = &source[start..];
        let after = rest
            .find(CSS_PATCH_END)
            .map(|end| rest[end + CSS_PATCH_END.len()..].trim_start_matches(['\r', '\n']))
            .unwrap_or("");
        return join_nonempty(&[before, patch, after]);
    }
    join_nonempty(&[source.trim_end_matches(['\r', '\n']), patch])
}

fn join_nonempty(parts: &[&str]) -> String {
    let mut out = String::new();
    for part in parts {
        if part.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(part);
    }
    out.push('\n');
    out
}

fn require_existing_file(root: &Path, relative: &Path) -> anyhow::Result<PathBuf> {
    let target = root.join(relative);
    if !target.is_file() {
        anyhow::bail!("target file does not exist");
    }
    Ok(target)
}

fn write_atomic(target: &Path, data: &[u8]) -> anyhow::Result<()> {
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("target has no file name"))?;
    let temporary = target.with_file_name(format!("{file_name}.alas-branding.tmp"));
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary)?;
    file.write_all(data)?;
    file.sync_all()?;
    if target.exists() {
        fs::remove_file(target)?;
    }
    fs::rename(&temporary, target)?;
    Ok(())
}

fn is_safe_relative_path(path: &Path) -> bool {
    path.is_relative()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        && (path.starts_with("assets/gui/icon")
            || path == Path::new("assets/spa/spa-icon-192x192.png")
            || path == Path::new("assets/spa/spa-icon-512x512.png"))
}

fn is_safe_patch_path(path: &Path) -> bool {
    path.is_relative()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        && (path == Path::new("assets/gui/css/alas.css")
            || path == Path::new("assets/gui/css/alas-pc.css")
            || path == Path::new("module/webui/base.py"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn branding_targets_are_confined_to_public_branding_assets() {
        assert!(is_safe_relative_path(Path::new("assets/gui/icon/alas.svg")));
        assert!(is_safe_relative_path(Path::new(
            "assets/spa/spa-icon-192x192.png"
        )));
        assert!(!is_safe_relative_path(Path::new("../config/deploy.yaml")));
        assert!(!is_safe_relative_path(Path::new(
            "assets/gui/css/custom.css"
        )));
        assert!(!is_safe_relative_path(Path::new(
            "assets/spa/manifest.json"
        )));
    }

    #[test]
    fn webui_patch_targets_are_confined_to_personal_overlay_files() {
        assert!(is_safe_patch_path(Path::new("assets/gui/css/alas.css")));
        assert!(is_safe_patch_path(Path::new("assets/gui/css/alas-pc.css")));
        assert!(is_safe_patch_path(Path::new("module/webui/base.py")));
        assert!(!is_safe_patch_path(Path::new("assets/gui/css/custom.css")));
        assert!(!is_safe_patch_path(Path::new("../module/webui/base.py")));
    }

    #[test]
    fn css_overlay_appends_once_and_replaces_existing_block() {
        let first = upsert_marked_block("body { color: red; }\n", ALAS_CSS_PATCH);
        assert!(first.contains("body { color: red; }"));
        assert_eq!(first.matches(CSS_PATCH_START).count(), 1);
        let second = upsert_marked_block(&first, ALAS_CSS_PATCH);
        assert_eq!(second.matches(CSS_PATCH_START).count(), 1);
        assert_eq!(second, first);
    }

    #[test]
    fn webui_overrides_patch_header_and_css_without_copying_whole_files() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("assets/gui/css")).unwrap();
        fs::create_dir_all(root.join("module/webui")).unwrap();
        fs::write(
            root.join("assets/gui/css/alas.css"),
            "#pywebio-scope-header_title { margin: auto; }\n",
        )
        .unwrap();
        fs::write(
            root.join("assets/gui/css/alas-pc.css"),
            "#pywebio-scope-overview { display: grid; }\n",
        )
        .unwrap();
        fs::write(
            root.join("module/webui/base.py"),
            "put_html(Icon.ALAS).style(\"--header-icon--\"),\nput_text(\"港区OA\").style(\"--header-text--\"),\n",
        )
        .unwrap();

        apply_webui_overrides(&root);
        apply_webui_overrides(&root);

        let css = fs::read_to_string(root.join("assets/gui/css/alas.css")).unwrap();
        let pc = fs::read_to_string(root.join("assets/gui/css/alas-pc.css")).unwrap();
        let header = fs::read_to_string(root.join("module/webui/base.py")).unwrap();
        assert!(css.contains("#pywebio-scope-header_title { margin: auto; }"));
        assert_eq!(css.matches(CSS_PATCH_START).count(), 1);
        assert!(pc.contains("grid-template-rows: 1fr;"));
        assert!(header.contains(HEADER_PERSONAL));
        assert!(!header.contains(HEADER_UPSTREAM));
        let _ = fs::remove_dir_all(&root);
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("alas-branding-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
