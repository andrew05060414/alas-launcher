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

    let temporary = target.with_extension("svg.alas-branding.tmp");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary)?;
    file.write_all(asset.data)?;
    file.sync_all()?;
    if target.exists() {
        fs::remove_file(&target)?;
    }
    fs::rename(&temporary, &target)?;
    info!("Applied personal ALAS app branding: {}", asset.target);
    Ok(())
}

fn is_safe_relative_path(path: &Path) -> bool {
    path.is_relative()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        && path.starts_with("assets/gui/icon")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branding_targets_are_confined_to_gui_icons() {
        assert!(is_safe_relative_path(Path::new("assets/gui/icon/alas.svg")));
        assert!(!is_safe_relative_path(Path::new("../config/deploy.yaml")));
        assert!(!is_safe_relative_path(Path::new(
            "assets/gui/css/custom.css"
        )));
    }
}
