//! Theme apply, restore, and target switching operations.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use gpui::Context;

use skin_core::live;

use crate::app::types::Msg;
use crate::app::{save_target_preference, theme_is_active, SkinApp};
use crate::i18n::t;
use crate::ui::constants::MIN_SURFACE_OPACITY;

impl SkinApp {
    pub(crate) fn switch_target(&mut self, target: live::TargetApp, cx: &mut Context<Self>) {
        if target == self.selected_target {
            return;
        }
        if !target.is_installed() {
            self.message = t().format_not_installed(target.display_name());
            cx.notify();
            return;
        }
        if let Some(stop) = self.live_stop.take() {
            stop.store(true, Ordering::Relaxed);
        }
        self.generation += 1;
        self.applying = false;
        let old_target = self.active_target.take();
        self.active_theme = None;
        self.active_surface_opacity = None;
        self.selected_target = target;
        save_target_preference(target);
        self.message.clear();
        let previous_thread = self.live_thread.take();
        if previous_thread.is_some() || old_target.is_some() {
            let tx = self.tx.clone();
            self.live_thread = Some(std::thread::spawn(move || {
                if let Some(thread) = previous_thread {
                    let _ = thread.join();
                }
                if let Some(old_target) = old_target {
                    let tx_log = tx.clone();
                    if let Err(error) = live::restore(old_target, move |line| {
                        let _ = tx_log.send(Msg::Log(line));
                    }) {
                        let _ = tx.send(Msg::Log(format!("restore failed: {error}")));
                    }
                }
            }));
        }
        cx.notify();
    }

    pub(crate) fn apply_selected(&mut self, cx: &mut Context<Self>) {
        if self.applying {
            return;
        }
        let Some(row) = self.themes.get(self.selected) else {
            return;
        };
        let target = self.selected_target;
        if !target.is_installed() {
            self.message = t().format_please_install(target.display_name());
            cx.notify();
            return;
        }
        let active = theme_is_active(
            self.active_target,
            self.active_theme.as_deref(),
            target,
            row.theme.id.as_str(),
        ) && (!row.preview.has_background
            || self
                .active_surface_opacity
                .is_some_and(|value| (value - self.surface_opacity).abs() < 0.001));
        if active {
            return;
        }
        if let Some(stop) = self.live_stop.take() {
            stop.store(true, Ordering::Relaxed);
        }
        let mut theme = row.theme.clone();
        if row.preview.has_background {
            theme.surface_opacity = Some(self.surface_opacity);
        }
        self.generation += 1;
        let generation = self.generation;
        let stop = Arc::new(AtomicBool::new(false));
        self.live_stop = Some(stop.clone());
        self.active_target = Some(target);
        self.active_theme = Some(theme.id.clone());
        self.active_surface_opacity = theme.surface_opacity;
        self.applying = true;
        self.message = t().action_applying.into();
        let tx = self.tx.clone();
        let previous_thread = self.live_thread.take();
        self.live_thread = Some(std::thread::spawn(move || {
            if let Some(thread) = previous_thread {
                let _ = thread.join();
            }
            let tx_log = tx.clone();
            let mut reported = false;
            let result = live::run(&theme, target, false, stop, move |line| {
                if !reported && line.trim_start().starts_with("injected:") {
                    reported = true;
                    let _ = tx_log.send(Msg::Applied(generation));
                }
                let _ = tx_log.send(Msg::Log(line));
            });
            let _ = tx.send(Msg::Done {
                generation: Some(generation),
                ok: result.is_ok(),
                restoring: false,
            });
        }));
        cx.notify();
    }

    pub(crate) fn restore_default(&mut self, cx: &mut Context<Self>) {
        if let Some(stop) = self.live_stop.take() {
            stop.store(true, Ordering::Relaxed);
        }
        self.generation += 1;
        let generation = self.generation;
        let target = self.selected_target;
        self.active_target = None;
        self.active_theme = None;
        self.active_surface_opacity = None;
        self.applying = true;
        self.message = t().action_restoring.into();
        let tx = self.tx.clone();
        let previous_thread = self.live_thread.take();
        self.live_thread = Some(std::thread::spawn(move || {
            if let Some(thread) = previous_thread {
                let _ = thread.join();
            }
            let tx_log = tx.clone();
            let result = live::restore(target, move |line| {
                let _ = tx_log.send(Msg::Log(line));
            });
            let _ = tx.send(Msg::Done {
                generation: Some(generation),
                ok: result.is_ok(),
                restoring: true,
            });
        }));
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
        theme_is_active(
            self.active_target,
            self.active_theme.as_deref(),
            self.selected_target,
            row.theme.id.as_str(),
        ) && (!row.preview.has_background
            || self
                .active_surface_opacity
                .is_some_and(|value| (value - self.surface_opacity).abs() < 0.001))
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
