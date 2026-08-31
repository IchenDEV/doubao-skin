//! Theme apply, restore, and target switching operations.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use gpui::Context;

use skin_core::live;

use crate::app::theme_sessions::TargetSession;
use crate::app::types::Msg;
use crate::app::{save_target_preference, SkinApp};
use crate::i18n::t;
use crate::ui::constants::MIN_SURFACE_OPACITY;

impl SkinApp {
    pub(crate) fn switch_target(&mut self, target: live::TargetApp, cx: &mut Context<Self>) {
        if target == self.selected_target {
            return;
        }
        self.restart_confirmation_target = None;
        if !target.is_installed() {
            self.message = t().format_not_installed(target.display_name());
            cx.notify();
            return;
        }
        self.selected_target = target;
        save_target_preference(target);
        for row in &mut self.themes {
            row.preview = row.theme.preview_style_for(target);
        }
        self.ensure_selected_match();
        self.ensure_store_selected_match();
        self.message.clear();
        cx.notify();
    }

    pub(crate) fn apply_selected(&mut self, cx: &mut Context<Self>) {
        let Some(row) = self.themes.get(self.selected) else {
            return;
        };
        let target = self.selected_target;
        if self.theme_sessions.is_busy(target) {
            return;
        }
        if !row.theme.supports_target(target) {
            self.restart_confirmation_target = None;
            self.message = format!("这个主题不支持{}", target.display_name());
            cx.notify();
            return;
        }
        if !target.is_installed() {
            self.restart_confirmation_target = None;
            self.message = t().format_please_install(target.display_name());
            cx.notify();
            return;
        }
        let active = self.theme_sessions.is_active(
            target,
            row.theme.id.as_str(),
            row.preview.has_background.then_some(self.surface_opacity),
        );
        if active {
            self.restart_confirmation_target = None;
            return;
        }
        let allow_restart = self.restart_confirmation_target == Some(target);
        match live::prepare_state(target) {
            Ok(live::PrepareState::RestartConfirmationRequired) if !allow_restart => {
                self.restart_confirmation_target = Some(target);
                self.message =
                    "WorkBuddy 正在运行。请先保存正在进行的任务，再明确重启并应用。".into();
                cx.notify();
                return;
            }
            Ok(live::PrepareState::WrongPortOwner) => {
                self.restart_confirmation_target = None;
                self.message = format!("端口 {} 已被其他程序占用", target.port());
                cx.notify();
                return;
            }
            Ok(live::PrepareState::NotInstalled) => {
                self.restart_confirmation_target = None;
                self.message = format!("请先安装{}", target.display_name());
                cx.notify();
                return;
            }
            Ok(
                live::PrepareState::Ready
                | live::PrepareState::LaunchRequired
                | live::PrepareState::RestartConfirmationRequired,
            ) => {}
            Err(error) => {
                self.restart_confirmation_target = None;
                self.message = format!("无法准备{}：{error}", target.display_name());
                cx.notify();
                return;
            }
        }
        let mut theme = row.theme.clone();
        if row.preview.has_background {
            theme.surface_opacity = Some(self.surface_opacity);
        }
        self.generation += 1;
        let generation = self.generation;
        let stop = Arc::new(AtomicBool::new(false));
        let previous = self.theme_sessions.begin_applying(
            target,
            TargetSession::pending(
                theme.id.clone(),
                theme.surface_opacity,
                generation,
                stop.clone(),
            ),
        );
        let previous_thread = previous.and_then(TargetSession::into_thread);
        self.message = t().action_applying.into();
        self.restart_confirmation_target = None;
        let tx = self.tx.clone();
        let thread = std::thread::spawn(move || {
            if let Some(thread) = previous_thread {
                let _ = thread.join();
            }
            let tx_log = tx.clone();
            let mut reported = false;
            let result = live::run_with_restart_permission(
                &theme,
                target,
                false,
                stop,
                allow_restart,
                move |line| {
                    if !reported && line.trim_start().starts_with("injected:") {
                        reported = true;
                        let _ = tx_log.send(Msg::Applied { target, generation });
                    }
                    let _ = tx_log.send(Msg::Log(line));
                },
            );
            let _ = tx.send(Msg::Done {
                target,
                generation,
                ok: result.is_ok(),
                restoring: false,
            });
        });
        self.theme_sessions
            .attach_thread(target, generation, thread);
        cx.notify();
    }

    pub(crate) fn restore_default(&mut self, cx: &mut Context<Self>) {
        self.restart_confirmation_target = None;
        let target = self.selected_target;
        if self.theme_sessions.is_busy(target) {
            return;
        }
        self.generation += 1;
        let generation = self.generation;
        let previous = self.theme_sessions.begin_restoring(target, generation);
        let previous_thread = previous.and_then(TargetSession::into_thread);
        self.message = t().action_restoring.into();
        let tx = self.tx.clone();
        let thread = std::thread::spawn(move || {
            if let Some(thread) = previous_thread {
                let _ = thread.join();
            }
            let tx_log = tx.clone();
            let result = live::restore(target, move |line| {
                let _ = tx_log.send(Msg::Log(line));
            });
            let _ = tx.send(Msg::Done {
                target,
                generation,
                ok: result.is_ok(),
                restoring: true,
            });
        });
        self.theme_sessions
            .attach_thread(target, generation, thread);
        cx.notify();
    }

    pub(crate) fn set_surface_opacity(&mut self, value: f32, cx: &mut Context<Self>) {
        let next = (value.clamp(MIN_SURFACE_OPACITY, 1.0) * 100.0).round() / 100.0;
        if (next - self.surface_opacity).abs() < 0.001 {
            return;
        }
        self.surface_opacity = next;
        self.message.clear();
        cx.notify();
    }

    pub(crate) fn selected_settings_are_active(&self, row: &crate::app::types::ThemeRow) -> bool {
        self.theme_sessions.is_active(
            self.selected_target,
            row.theme.id.as_str(),
            row.preview.has_background.then_some(self.surface_opacity),
        )
    }
}

pub fn parse_store_accent(value: Option<&str>) -> u32 {
    let Some(value) = value.map(str::trim) else {
        return 0xa64e24;
    };
    if let Some(hex) = value.strip_prefix('#') {
        if hex.len() == 6 {
            if let Ok(color) = u32::from_str_radix(hex, 16) {
                return color;
            }
        }
    }
    if let Some(inner) = value
        .strip_prefix("rgba(")
        .or_else(|| value.strip_prefix("rgb("))
        .and_then(|inner| inner.strip_suffix(')'))
    {
        let parts = inner
            .split(',')
            .take(3)
            .filter_map(|part| part.trim().parse::<u32>().ok())
            .collect::<Vec<_>>();
        if parts.len() == 3 {
            return (parts[0].min(255) << 16) | (parts[1].min(255) << 8) | parts[2].min(255);
        }
    }
    0xa64e24
}
