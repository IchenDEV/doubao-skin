//! Desktop transactions for the two automatic-theme switches.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui::Context;

use skin_core::auto_theme::{self, AutoThemeSettings, LastApplied};

use crate::app::platform::{self, AutoThemeServiceStatus};
use crate::app::types::Msg;
use crate::app::SkinApp;
use crate::i18n::t;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AutoThemeControlState {
    pub keep_enabled: bool,
    pub login_enabled: bool,
    pub keep_requested: bool,
    pub login_requested: bool,
}

pub(crate) fn control_state(
    settings: &AutoThemeSettings,
    status: AutoThemeServiceStatus,
    busy: bool,
) -> AutoThemeControlState {
    let supported = status != AutoThemeServiceStatus::Unsupported;
    let has_theme = settings.last_applied().is_some();
    AutoThemeControlState {
        keep_enabled: supported && has_theme && !busy,
        login_enabled: supported
            && has_theme
            && settings.keep_requested()
            && status == AutoThemeServiceStatus::Enabled
            && !busy,
        keep_requested: settings.keep_requested(),
        login_requested: settings.open_at_login(),
    }
}

fn keep_toggle_enables(settings: &AutoThemeSettings, status: AutoThemeServiceStatus) -> bool {
    !settings.keep_requested() && status != AutoThemeServiceStatus::Enabled
}

fn applied_settings(
    current: &AutoThemeSettings,
    generation: u64,
    current_generation: u64,
    last_applied: Option<LastApplied>,
) -> Option<AutoThemeSettings> {
    if generation != current_generation {
        return None;
    }
    let mut next = current.clone();
    next.set_last_applied(last_applied?);
    Some(next)
}

impl SkinApp {
    fn save_auto_theme_settings(&mut self, next: AutoThemeSettings) -> bool {
        match auto_theme::save(&next) {
            Ok(()) => {
                self.auto_theme_settings = next;
                true
            }
            Err(_) => {
                self.message = t().auto_theme_save_failed.into();
                false
            }
        }
    }

    pub(crate) fn record_successful_apply(&mut self, generation: u64) {
        let last_applied = self.active_target.and_then(|target| {
            LastApplied::new(
                target,
                self.active_theme.clone()?,
                self.active_surface_opacity,
            )
            .ok()
        });
        if let Some(next) = applied_settings(
            &self.auto_theme_settings,
            generation,
            self.generation,
            last_applied,
        ) {
            self.save_auto_theme_settings(next);
        }
    }

    pub(crate) fn toggle_auto_theme_keep(&mut self, cx: &mut Context<Self>) {
        let state = control_state(
            &self.auto_theme_settings,
            self.auto_theme_service_status,
            self.auto_theme_busy,
        );
        if !state.keep_enabled {
            self.message = if self.auto_theme_settings.last_applied().is_none() {
                t().auto_theme_apply_first.into()
            } else {
                t().auto_theme_unsupported.into()
            };
            cx.notify();
            return;
        }

        let enabling =
            keep_toggle_enables(&self.auto_theme_settings, self.auto_theme_service_status);
        let previous = self.auto_theme_settings.clone();
        let mut next = self.auto_theme_settings.clone();
        next.set_keep_requested(enabling);
        if !self.save_auto_theme_settings(next) {
            cx.notify();
            return;
        }
        if !enabling {
            if let Some(stop) = self.live_stop.take() {
                stop.store(true, Ordering::Relaxed);
            }
            self.auto_theme_attempted_for_current_run = false;
        }
        self.auto_theme_busy = true;
        self.message = if enabling {
            t().auto_theme_enabling.into()
        } else {
            t().auto_theme_disabling.into()
        };
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let result = if enabling {
                platform::register_auto_theme_service()
            } else {
                platform::unregister_auto_theme_service()
            };
            let (status, error) = match result {
                Ok(status) => (status, None),
                Err(error) => (platform::auto_theme_service_status(), Some(error)),
            };
            let rollback_settings = (enabling && error.is_some()).then_some(previous);
            let _ = tx.send(Msg::AutoThemeServiceChanged {
                status,
                error,
                rollback_settings,
            });
        });
        cx.notify();
    }

    pub(crate) fn toggle_open_at_login(&mut self, cx: &mut Context<Self>) {
        let state = control_state(
            &self.auto_theme_settings,
            self.auto_theme_service_status,
            self.auto_theme_busy,
        );
        if !state.login_enabled {
            return;
        }
        let mut next = self.auto_theme_settings.clone();
        next.set_open_at_login(!next.open_at_login());
        if self.save_auto_theme_settings(next) {
            self.message = if self.auto_theme_settings.open_at_login() {
                t().auto_theme_login_enabled.into()
            } else {
                t().auto_theme_login_disabled.into()
            };
        }
        cx.notify();
    }

    pub(crate) fn open_auto_theme_settings(&mut self, cx: &mut Context<Self>) {
        if let Err(error) = platform::open_login_items_settings() {
            self.message = error;
        }
        cx.notify();
    }

    pub(crate) fn finish_successful_restore(&mut self) {
        let mut next = self.auto_theme_settings.clone();
        next.clear_and_disable();
        let saved = self.save_auto_theme_settings(next);
        if !saved {
            self.message = t().auto_theme_restore_cleanup_failed.into();
        }
        self.auto_theme_busy = true;
        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let result = platform::unregister_auto_theme_service();
            let (status, error) = match result {
                Ok(status) => (status, None),
                Err(error) => (platform::auto_theme_service_status(), Some(error)),
            };
            let _ = tx.send(Msg::AutoThemeServiceChanged {
                status,
                error,
                rollback_settings: None,
            });
        });
    }

    pub(crate) fn maintain_auto_theme(&mut self, cx: &mut Context<Self>) {
        if self.auto_theme_last_check.elapsed() < Duration::from_secs(1) {
            return;
        }
        self.auto_theme_last_check = Instant::now();
        if !self.auto_theme_settings.keep_requested() || self.applying {
            return;
        }
        let Some(saved) = self.auto_theme_settings.last_applied().cloned() else {
            return;
        };
        if self
            .live_thread
            .as_ref()
            .is_some_and(|thread| !thread.is_finished())
        {
            return;
        }
        if let Some(thread) = self.live_thread.take() {
            let _ = thread.join();
            self.live_stop = None;
        }

        if !saved.target().is_running() {
            self.auto_theme_attempted_for_current_run = false;
            return;
        }
        if self.auto_theme_attempted_for_current_run {
            return;
        }

        let Some(mut selected) = skin_core::theme::list_installed()
            .into_iter()
            .find(|theme| theme.id == saved.theme_id())
        else {
            self.message = t().theme_unavailable.into();
            self.auto_theme_attempted_for_current_run = true;
            cx.notify();
            return;
        };
        selected.surface_opacity = saved.surface_opacity();
        let target = saved.target();
        self.generation += 1;
        let generation = self.generation;
        self.active_target = Some(target);
        self.active_theme = Some(selected.id.clone());
        self.active_surface_opacity = selected.surface_opacity;
        self.applying = true;
        self.message = t().action_applying.into();
        self.auto_theme_attempted_for_current_run = true;
        let stop = Arc::new(AtomicBool::new(false));
        self.live_stop = Some(stop.clone());
        let tx = self.tx.clone();
        self.live_thread = Some(std::thread::spawn(move || {
            let tx_log = tx.clone();
            let mut reported = false;
            let result = skin_core::live::run_with_policy(
                &selected,
                target,
                false,
                skin_core::live::PortLossPolicy::Stop,
                stop,
                move |line| {
                    if !reported && line.trim_start().starts_with("injected:") {
                        reported = true;
                        let _ = tx_log.send(Msg::Applied(generation));
                    }
                    let _ = tx_log.send(Msg::Log(line));
                },
            );
            let _ = tx.send(Msg::Done {
                generation: Some(generation),
                ok: result.is_ok(),
                restoring: false,
            });
        }));
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use skin_core::auto_theme::{AutoThemeSettings, LastApplied};
    use skin_core::live::TargetApp;

    use super::{applied_settings, control_state, keep_toggle_enables, AutoThemeServiceStatus};

    fn saved_settings() -> AutoThemeSettings {
        let mut settings = AutoThemeSettings::default();
        settings.set_last_applied(
            LastApplied::new(TargetApp::DoubaoWork, "pure-dark", Some(0.72)).unwrap(),
        );
        settings
    }

    #[test]
    fn child_switch_requires_the_parent_and_an_enabled_service() {
        let mut settings = saved_settings();
        let off = control_state(&settings, AutoThemeServiceStatus::Enabled, false);
        assert!(off.keep_enabled);
        assert!(!off.login_enabled);

        settings.set_keep_requested(true);
        assert!(
            !control_state(&settings, AutoThemeServiceStatus::RequiresApproval, false)
                .login_enabled
        );
        assert!(control_state(&settings, AutoThemeServiceStatus::Enabled, false).login_enabled);
    }

    #[test]
    fn unsupported_busy_and_missing_theme_states_are_not_interactive() {
        let settings = saved_settings();
        assert!(!control_state(&settings, AutoThemeServiceStatus::Unsupported, false).keep_enabled);
        assert!(!control_state(&settings, AutoThemeServiceStatus::Enabled, true).keep_enabled);
        assert!(
            !control_state(
                &AutoThemeSettings::default(),
                AutoThemeServiceStatus::Enabled,
                false
            )
            .keep_enabled
        );
    }

    #[test]
    fn an_orphaned_enabled_service_is_cleaned_before_enabling_again() {
        let settings = saved_settings();
        assert!(!keep_toggle_enables(
            &settings,
            AutoThemeServiceStatus::Enabled
        ));
        assert!(keep_toggle_enables(
            &settings,
            AutoThemeServiceStatus::NotRegistered
        ));
    }

    #[test]
    fn only_the_matching_apply_generation_updates_the_saved_theme() {
        let settings = saved_settings();
        let replacement = LastApplied::new(TargetApp::Doubao, "new-theme", None).unwrap();
        assert!(applied_settings(&settings, 7, 8, Some(replacement.clone())).is_none());
        let next = applied_settings(&settings, 8, 8, Some(replacement)).unwrap();
        assert_eq!(next.last_applied().unwrap().theme_id(), "new-theme");
        assert_eq!(next.last_applied().unwrap().target(), TargetApp::Doubao);
    }
}
