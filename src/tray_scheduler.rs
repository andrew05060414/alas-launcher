//! 托盘菜单「暂停任务 / 继续任务」。
//!
//! 经本机回环 + 启动器信任密钥调用后端 `/api/launcher/scheduler`：暂停会停止所有
//! 运行中的实例并由后端记住，继续时重新启动它们。后台线程每隔几秒刷新菜单文字，
//! 后端未就绪或不支持该端点时菜单项置灰。

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use reqwest::blocking::Client;
use rust_i18n::t;
use tauri::{menu::MenuItem, Runtime};
use tracing::{info, warn};

use crate::launcher_trust_secret;

const SECRET_HEADER: &str = "X-Webui-Launcher-Secret";
const POLL_INTERVAL: Duration = Duration::from_secs(3);
const QUERY_TIMEOUT: Duration = Duration::from_secs(5);
/// 停止实例要等子进程退出，给足时间。
const ACTION_TIMEOUT: Duration = Duration::from_secs(90);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SchedulerState {
    Running,
    Stopped,
    Unavailable,
}

fn endpoint(port: u16) -> String {
    format!("http://127.0.0.1:{port}/api/launcher/scheduler")
}

/// 从端点返回的 JSON 推断状态：`running` 非空即视为运行中。
pub(crate) fn parse_state(body: &str) -> SchedulerState {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
        return SchedulerState::Unavailable;
    };
    match value.get("running").and_then(|running| running.as_array()) {
        Some(running) if !running.is_empty() => SchedulerState::Running,
        Some(_) => SchedulerState::Stopped,
        None => SchedulerState::Unavailable,
    }
}

fn query(port: u16) -> SchedulerState {
    let Ok(client) = Client::builder().timeout(QUERY_TIMEOUT).build() else {
        return SchedulerState::Unavailable;
    };
    match client
        .get(endpoint(port))
        .header(SECRET_HEADER, launcher_trust_secret())
        .send()
    {
        Ok(response) if response.status().is_success() => response
            .text()
            .map(|body| parse_state(&body))
            .unwrap_or(SchedulerState::Unavailable),
        _ => SchedulerState::Unavailable,
    }
}

fn label(state: SchedulerState) -> String {
    match state {
        SchedulerState::Running => t!("tray.pause_scheduler").to_string(),
        SchedulerState::Stopped => t!("tray.resume_scheduler").to_string(),
        SchedulerState::Unavailable => t!("tray.scheduler_unavailable").to_string(),
    }
}

fn apply<R: Runtime>(item: &MenuItem<R>, state: SchedulerState) {
    let _ = item.set_text(label(state));
    let _ = item.set_enabled(state != SchedulerState::Unavailable);
}

/// 菜单项控制器：轮询状态并处理点击；执行期间暂停轮询，避免文字来回跳。
pub(crate) struct TraySchedulerToggle<R: Runtime> {
    item: MenuItem<R>,
    port: u16,
    busy: Arc<AtomicBool>,
}

// 手写 Clone：derive 会额外要求 `R: Clone`，而 Runtime 并不实现 Clone。
impl<R: Runtime> Clone for TraySchedulerToggle<R> {
    fn clone(&self) -> Self {
        Self {
            item: self.item.clone(),
            port: self.port,
            busy: self.busy.clone(),
        }
    }
}

impl<R: Runtime> TraySchedulerToggle<R> {
    pub(crate) fn new(item: MenuItem<R>, port: u16) -> Self {
        Self {
            item,
            port,
            busy: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) fn spawn_poller(&self) {
        let this = self.clone();
        thread::spawn(move || {
            let mut last = None;
            loop {
                if this.busy.load(Ordering::SeqCst) {
                    last = None;
                } else {
                    let state = query(this.port);
                    if last != Some(state) {
                        apply(&this.item, state);
                        last = Some(state);
                    }
                }
                thread::sleep(POLL_INTERVAL);
            }
        });
    }

    pub(crate) fn toggle(&self) {
        if self.busy.swap(true, Ordering::SeqCst) {
            return;
        }
        let this = self.clone();
        thread::spawn(move || {
            let action = match query(this.port) {
                SchedulerState::Running => "pause",
                SchedulerState::Stopped => "resume",
                SchedulerState::Unavailable => {
                    apply(&this.item, SchedulerState::Unavailable);
                    this.busy.store(false, Ordering::SeqCst);
                    return;
                }
            };
            let pending = if action == "pause" {
                t!("tray.pausing_scheduler")
            } else {
                t!("tray.resuming_scheduler")
            };
            let _ = this.item.set_text(pending.to_string());
            let _ = this.item.set_enabled(false);

            let result = Client::builder()
                .timeout(ACTION_TIMEOUT)
                .build()
                .map_err(|err| err.to_string())
                .and_then(|client| {
                    client
                        .post(endpoint(this.port))
                        .header(SECRET_HEADER, launcher_trust_secret())
                        .header(reqwest::header::CONTENT_TYPE, "application/json")
                        .body(format!("{{\"action\":\"{action}\"}}"))
                        .send()
                        .map_err(|err| err.to_string())
                });
            match result {
                Ok(response) if response.status().is_success() => {
                    info!(action, "Tray scheduler action succeeded");
                }
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().unwrap_or_default();
                    warn!(action, %status, body, "Tray scheduler action failed");
                }
                Err(err) => warn!(action, err, "Tray scheduler action request failed"),
            }
            apply(&this.item, query(this.port));
            this.busy.store(false, Ordering::SeqCst);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_state_reads_running_list() {
        assert_eq!(
            parse_state(r#"{"success":true,"running":["alas"],"paused":[]}"#),
            SchedulerState::Running
        );
        assert_eq!(
            parse_state(r#"{"success":true,"running":[],"paused":["alas"]}"#),
            SchedulerState::Stopped
        );
        assert_eq!(parse_state("not json"), SchedulerState::Unavailable);
        assert_eq!(parse_state(r#"{"success":false}"#), SchedulerState::Unavailable);
    }
}
