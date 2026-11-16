//! 个人外观：在启动器 WebView 中把 AzurPilot 新前端的品牌头像换成黎塞留头像，名称与字体沿用上游。
//!
//! 只在页面 DOM 上覆盖，不写入 AzurPilot 仓库，所以主仓库的 git 更新不会冲掉它，
//! 也不会在工作区留下改动。React 重新渲染会把元素恢复成上游内容，
//! 因此用 MutationObserver 在每次变化后重新覆盖。

use std::sync::OnceLock;

use base64::{prelude::BASE64_STANDARD, Engine};

const AVATAR_PNG: &[u8] = include_bytes!("../branding/alas-app/avatar-192.png");

const SCRIPT_TEMPLATE: &str = r#"
        (() => {
            const avatar = '__ALAS_AVATAR__';
            const state = window.__ALAS_PERSONAL_BRANDING__
                || (window.__ALAS_PERSONAL_BRANDING__ = {});
            const apply = () => {
                if (document.head && !document.getElementById('alas-personal-titlebar-style')) {
                    const style = document.createElement('style');
                    style.id = 'alas-personal-titlebar-style';
                    style.textContent = '__ALAS_TITLEBAR_CSS__';
                    document.head.appendChild(style);
                }
                // 侧栏标题和登录页的头像都使用 img.brand-logo
                for (const img of document.querySelectorAll('img.brand-logo')) {
                    if (img.getAttribute('src') !== avatar) {
                        img.setAttribute('src', avatar);
                    }
                }
                if (document.head) {
                    let icon = document.querySelector('link[rel~="icon"]');
                    if (!icon) {
                        icon = document.createElement('link');
                        icon.rel = 'icon';
                        document.head.appendChild(icon);
                    }
                    if (icon.getAttribute('href') !== avatar) {
                        icon.setAttribute('type', 'image/png');
                        icon.setAttribute('href', avatar);
                    }
                }
            };
            const schedule = () => {
                if (state.scheduled) {
                    return;
                }
                state.scheduled = true;
                requestAnimationFrame(() => {
                    state.scheduled = false;
                    apply();
                });
            };
            apply();
            if (!state.observer && document.documentElement) {
                state.observer = new MutationObserver(schedule);
                state.observer.observe(document.documentElement, {
                    childList: true,
                    subtree: true,
                    attributes: true,
                    attributeFilter: ['src', 'href'],
                });
            }
        })();
"#;

/// 返回注入到主窗口页面的品牌覆盖脚本，头像以内嵌 data URL 提供，不依赖运行目录中的文件。
pub fn personal_branding_injection_script() -> &'static str {
    static SCRIPT: OnceLock<String> = OnceLock::new();
    SCRIPT.get_or_init(|| {
        let avatar = format!(
            "data:image/png;base64,{}",
            BASE64_STANDARD.encode(AVATAR_PNG)
        );
        SCRIPT_TEMPLATE
            .replace("__ALAS_AVATAR__", &avatar)
            .replace("__ALAS_TITLEBAR_CSS__", "")
    })
}

/// 显式设置窗口的大图标（任务栏用）和小图标（标题栏、缩略图用），都来自内嵌的 ALAS 图标。
///
/// tao 只设置了小图标，大图标为空时 Windows 任务栏会回退到系统缓存里的旧图标。
/// 同一个窗口只设置一次，避免页面每次加载都创建新的图标句柄。
#[cfg(windows)]
pub fn apply_window_icons(window: &tauri::WebviewWindow) {
    use std::sync::atomic::{AtomicIsize, Ordering};
    use winapi::um::winuser::{
        CreateIconFromResourceEx, SendMessageW, ICON_BIG, ICON_SMALL, LR_DEFAULTCOLOR, WM_SETICON,
    };

    static LAST_HWND: AtomicIsize = AtomicIsize::new(0);
    const ICON_PNG: &[u8] = include_bytes!("../icons/icon.png");

    let Ok(hwnd) = window.hwnd() else { return };
    let raw = hwnd.0 as isize;
    if LAST_HWND.swap(raw, Ordering::SeqCst) == raw {
        return;
    }
    for (kind, size) in [(ICON_BIG, 256), (ICON_SMALL, 32)] {
        // SAFETY: 传入的是内嵌 PNG 字节和有效窗口句柄；创建出的图标交给窗口持有。
        unsafe {
            let icon = CreateIconFromResourceEx(
                ICON_PNG.as_ptr() as *mut u8,
                ICON_PNG.len() as u32,
                1,
                0x0003_0000,
                size,
                size,
                LR_DEFAULTCOLOR,
            );
            if !icon.is_null() {
                SendMessageW(raw as _, WM_SETICON, kind as usize, icon as isize);
            }
        }
    }
}

#[cfg(not(windows))]
pub fn apply_window_icons(_window: &tauri::WebviewWindow) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_targets_the_react_frontend_brand_elements() {
        let script = personal_branding_injection_script();

        assert!(script.contains("img.brand-logo"));
        assert!(!script.contains(".brand-title"), "名称与字体沿用上游");
        assert!(!script.contains("document.title"));
        assert!(script.contains("link[rel~=\"icon\"]"));
        assert!(script.contains("new MutationObserver(schedule)"));
        assert!(!script.contains("pywebio"));
    }

    #[test]
    fn avatar_is_embedded_instead_of_read_from_the_runtime_directory() {
        let script = personal_branding_injection_script();

        assert!(script.contains("data:image/png;base64,iVBORw0KGgo"));
        for placeholder in ["__ALAS_AVATAR__", "__ALAS_TITLEBAR_CSS__"] {
            assert!(!script.contains(placeholder), "{placeholder}");
        }
        assert!(!script.contains("static/assets"));
    }
}
