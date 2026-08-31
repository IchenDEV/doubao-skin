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
}

#[derive(Default)]
pub(crate) struct ThemeSessions {
    by_target: [Option<TargetSession>; 3],
}

impl ThemeSessions {
    const fn index(target: TargetApp) -> usize {
        match target {
            TargetApp::Doubao => 0,
            TargetApp::DoubaoWork => 1,
            TargetApp::WorkBuddy => 2,
        }
    }

    pub(crate) fn replace(
        &mut self,
        target: TargetApp,
        session: TargetSession,
    ) -> Option<TargetSession> {
        let previous = self.by_target[Self::index(target)].replace(session);
        if let Some(previous) = &previous {
            previous.request_stop();
        }
        previous
    }

    pub(crate) fn take(&mut self, target: TargetApp) -> Option<TargetSession> {
        self.by_target[Self::index(target)].take()
    }

    pub(crate) fn attach_thread(
        &mut self,
        target: TargetApp,
        generation: u64,
        thread: JoinHandle<()>,
    ) {
        let session = self.by_target[Self::index(target)]
            .as_mut()
            .filter(|session| session.generation == generation)
            .expect("target session must exist before its watcher starts");
        session.attach_thread(thread);
    }

    pub(crate) fn get(&self, target: TargetApp) -> Option<&TargetSession> {
        self.by_target[Self::index(target)].as_ref()
    }

    pub(crate) fn is_active(
        &self,
        target: TargetApp,
        theme_id: &str,
        surface_opacity: Option<f32>,
    ) -> bool {
        self.get(target)
            .is_some_and(|session| session.settings_are_active(theme_id, surface_opacity))
    }

    pub(crate) fn generation_matches(&self, target: TargetApp, generation: u64) -> bool {
        self.get(target)
            .is_some_and(|session| session.generation == generation)
    }

    pub(crate) fn take_if_generation(
        &mut self,
        target: TargetApp,
        generation: u64,
    ) -> Option<TargetSession> {
        self.generation_matches(target, generation)
            .then(|| self.take(target))
            .flatten()
    }
}
