//! Target-scoped live theme session state.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

use skin_core::live::TargetApp;

pub(crate) struct TargetSession {
    theme_id: String,
    surface_opacity: Option<f32>,
    generation: u64,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

struct TargetRestore {
    generation: u64,
    thread: Option<JoinHandle<()>>,
}

enum TargetState {
    Applying(TargetSession),
    Active(TargetSession),
    Restoring(TargetRestore),
}

impl TargetState {
    fn generation(&self) -> u64 {
        match self {
            Self::Applying(session) | Self::Active(session) => session.generation,
            Self::Restoring(restore) => restore.generation,
        }
    }
}

impl TargetSession {
    pub(crate) fn pending(
        theme_id: String,
        surface_opacity: Option<f32>,
        generation: u64,
        stop: Arc<AtomicBool>,
    ) -> Self {
        Self {
            theme_id,
            surface_opacity,
            generation,
            stop,
            thread: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        theme_id: &str,
        surface_opacity: Option<f32>,
        generation: u64,
        stop: Arc<AtomicBool>,
    ) -> Self {
        Self {
            theme_id: theme_id.into(),
            surface_opacity,
            generation,
            stop,
            thread: None,
        }
    }

    pub(crate) fn request_stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    pub(crate) fn attach_thread(&mut self, thread: JoinHandle<()>) {
        debug_assert!(self.thread.is_none());
        self.thread = Some(thread);
    }

    pub(crate) fn into_thread(mut self) -> Option<JoinHandle<()>> {
        self.thread.take()
    }

    fn settings_are_active(&self, theme_id: &str, surface_opacity: Option<f32>) -> bool {
        self.theme_id == theme_id
            && surface_opacity.is_none_or(|selected| {
                self.surface_opacity
                    .is_some_and(|active| (active - selected).abs() < 0.001)
            })
    }

    fn settings(&self) -> (String, Option<f32>) {
        (self.theme_id.clone(), self.surface_opacity)
    }
}

#[derive(Default)]
pub(crate) struct ThemeSessions {
    by_target: [Option<TargetState>; 3],
}

impl ThemeSessions {
    const fn index(target: TargetApp) -> usize {
        match target {
            TargetApp::Doubao => 0,
            TargetApp::DoubaoWork => 1,
            TargetApp::WorkBuddy => 2,
        }
    }

    pub(crate) fn is_busy(&self, target: TargetApp) -> bool {
        matches!(
            self.by_target[Self::index(target)],
            Some(TargetState::Applying(_) | TargetState::Restoring(_))
        )
    }

    pub(crate) fn has_session(&self, target: TargetApp) -> bool {
        self.by_target[Self::index(target)].is_some()
    }

    pub(crate) fn request_stop(&self, target: TargetApp) {
        if let Some(TargetState::Applying(session) | TargetState::Active(session)) =
            self.by_target[Self::index(target)].as_ref()
        {
            session.request_stop();
        }
    }

    pub(crate) fn begin_applying(
        &mut self,
        target: TargetApp,
        session: TargetSession,
    ) -> Option<TargetSession> {
        assert!(
            !self.is_busy(target),
            "a target cannot start applying while another operation is running"
        );
        let index = Self::index(target);
        let previous = self.by_target[index].take().and_then(|state| match state {
            TargetState::Active(session) => Some(session),
            TargetState::Applying(_) | TargetState::Restoring(_) => unreachable!(),
        });
        if let Some(previous) = &previous {
            previous.request_stop();
        }
        self.by_target[index] = Some(TargetState::Applying(session));
        previous
    }

    pub(crate) fn mark_applied(&mut self, target: TargetApp, generation: u64) -> bool {
        let index = Self::index(target);
        let Some(state) = self.by_target[index].take() else {
            return false;
        };
        match state {
            TargetState::Applying(session) if session.generation == generation => {
                self.by_target[index] = Some(TargetState::Active(session));
                true
            }
            state => {
                self.by_target[index] = Some(state);
                false
            }
        }
    }

    pub(crate) fn active_settings(
        &self,
        target: TargetApp,
        generation: u64,
    ) -> Option<(String, Option<f32>)> {
        match self.by_target[Self::index(target)].as_ref()? {
            TargetState::Active(session) if session.generation == generation => {
                Some(session.settings())
            }
            TargetState::Applying(_) | TargetState::Active(_) | TargetState::Restoring(_) => None,
        }
    }

    pub(crate) fn begin_restoring(
        &mut self,
        target: TargetApp,
        generation: u64,
    ) -> Option<TargetSession> {
        assert!(
            !self.is_busy(target),
            "a target cannot start restoring while another operation is running"
        );
        let index = Self::index(target);
        let previous = self.by_target[index].take().and_then(|state| match state {
            TargetState::Active(session) => Some(session),
            TargetState::Applying(_) | TargetState::Restoring(_) => unreachable!(),
        });
        if let Some(previous) = &previous {
            previous.request_stop();
        }
        self.by_target[index] = Some(TargetState::Restoring(TargetRestore {
            generation,
            thread: None,
        }));
        previous
    }

    pub(crate) fn attach_thread(
        &mut self,
        target: TargetApp,
        generation: u64,
        thread: JoinHandle<()>,
    ) {
        let state = self.by_target[Self::index(target)]
            .as_mut()
            .filter(|state| state.generation() == generation)
            .expect("target operation must exist before its thread starts");
        match state {
            TargetState::Applying(session) | TargetState::Active(session) => {
                session.attach_thread(thread);
            }
            TargetState::Restoring(restore) => {
                debug_assert!(restore.thread.is_none());
                restore.thread = Some(thread);
            }
        }
    }

    pub(crate) fn is_active(
        &self,
        target: TargetApp,
        theme_id: &str,
        surface_opacity: Option<f32>,
    ) -> bool {
        matches!(
            self.by_target[Self::index(target)].as_ref(),
            Some(TargetState::Active(session))
                if session.settings_are_active(theme_id, surface_opacity)
        )
    }

    pub(crate) fn generation_matches(&self, target: TargetApp, generation: u64) -> bool {
        self.by_target[Self::index(target)]
            .as_ref()
            .is_some_and(|state| state.generation() == generation)
    }

    pub(crate) fn complete_if_generation(&mut self, target: TargetApp, generation: u64) -> bool {
        if !self.generation_matches(target, generation) {
            return false;
        }
        self.by_target[Self::index(target)].take();
        true
    }
}
