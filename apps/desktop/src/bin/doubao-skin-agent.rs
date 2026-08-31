#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

//! Invisible login helper that restores the last successful live theme.

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod agent {
    #[cfg(any(test, target_os = "windows"))]
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread::JoinHandle;
    use std::time::Duration;

    use skin_core::auto_theme::{
        self, AutoThemeSettings, SupervisorAction, SupervisorSnapshot, SupervisorState,
    };
    use skin_core::{live, theme};

    struct Watcher {
        stop: Arc<AtomicBool>,
        thread: JoinHandle<()>,
    }

    impl Watcher {
        fn stop(self) {
            self.stop.store(true, Ordering::Relaxed);
            let _ = self.thread.join();
        }
    }

    fn load_theme(settings: &AutoThemeSettings) -> Result<(theme::Theme, live::TargetApp), String> {
        let saved = settings
            .last_applied()
            .ok_or_else(|| "没有可自动恢复的主题".to_string())?;
        let mut selected = theme::list_installed()
            .into_iter()
            .find(|candidate| candidate.id == saved.theme_id())
            .ok_or_else(|| "上次使用的主题已不可用".to_string())?;
        selected.surface_opacity = saved.surface_opacity();
        Ok((selected, saved.target()))
    }

    fn start_watcher(settings: &AutoThemeSettings) -> Result<Watcher, String> {
        let (selected, target) = load_theme(settings)?;
        let stop = Arc::new(AtomicBool::new(false));
        let watcher_stop = stop.clone();
        let thread = std::thread::spawn(move || {
            if let Err(error) = live::run_with_policy(
                &selected,
                target,
                false,
                live::PortLossPolicy::Stop,
                watcher_stop,
                |_| {},
            ) {
                eprintln!("豆皮后台服务已停止：{error}");
            }
        });
        Ok(Watcher { stop, thread })
    }

    pub fn run() -> Result<(), String> {
        let Some(_instance_guard) = platform::acquire_single_instance()? else {
            return Ok(());
        };
        let current_executable =
            std::env::current_exe().map_err(|_| "无法识别豆皮后台服务位置".to_string())?;
        let main_executable = platform::main_executable_from_agent(&current_executable)
            .filter(|path| path.is_file())
            .ok_or_else(|| "豆皮后台服务只能从已安装的豆皮应用运行".to_string())?;
        let settings = auto_theme::load()?;
        if !settings.keep_requested() || settings.last_applied().is_none() {
            return Ok(());
        }

        let main_running = platform::main_app_is_running(&main_executable);
        let login_session_id = platform::login_session_id()?;
        let login_start_pending = auto_theme::consume_login_open(
            login_session_id,
            main_running,
            settings.open_at_login(),
        )?;
        let mut supervisor = SupervisorState::new(login_start_pending);
        let mut watcher: Option<Watcher> = None;

        loop {
            let settings = auto_theme::load()?;
            let target_running = settings
                .last_applied()
                .is_some_and(|saved| saved.target().is_running());
            let snapshot = SupervisorSnapshot {
                keep_requested: settings.keep_requested() && settings.last_applied().is_some(),
                main_app_running: platform::main_app_is_running(&main_executable),
                target_running,
                watcher_finished: watcher
                    .as_ref()
                    .is_some_and(|active| active.thread.is_finished()),
            };
            match supervisor.next(snapshot) {
                SupervisorAction::Exit => {
                    if let Some(active) = watcher.take() {
                        active.stop();
                    }
                    return Ok(());
                }
                SupervisorAction::StopWatcher => {
                    if let Some(active) = watcher.take() {
                        active.stop();
                    }
                }
                SupervisorAction::StartWatcher => match start_watcher(&settings) {
                    Ok(active) => watcher = Some(active),
                    Err(error) => eprintln!("豆皮后台服务未能恢复主题：{error}"),
                },
                SupervisorAction::Wait => {
                    if watcher
                        .as_ref()
                        .is_some_and(|active| active.thread.is_finished())
                    {
                        if let Some(active) = watcher.take() {
                            let _ = active.thread.join();
                        }
                    }
                }
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    #[cfg(any(test, target_os = "windows"))]
    fn windows_main_executable_from_agent(agent: &Path) -> Option<PathBuf> {
        if !agent
            .file_name()?
            .to_str()?
            .eq_ignore_ascii_case("doubao-skin-agent.exe")
        {
            return None;
        }
        let helpers = agent.parent()?;
        if !helpers
            .file_name()?
            .to_str()?
            .eq_ignore_ascii_case("helpers")
        {
            return None;
        }
        Some(helpers.parent()?.join("doubao-skin.exe"))
    }

    #[cfg(any(test, target_os = "windows"))]
    fn login_session_id_from_luid(low_part: u32, high_part: i32) -> u64 {
        ((high_part as u32 as u64) << 32) | u64::from(low_part)
    }

    #[cfg(target_os = "macos")]
    #[allow(unexpected_cfgs)]
    mod platform {
        use std::path::{Path, PathBuf};

        use cocoa::base::{id, nil};
        use cocoa::foundation::NSString;
        use objc::runtime::Class;
        use objc::{msg_send, sel, sel_impl};
        use skin_core::live;

        const MAIN_BUNDLE_ID: &str = "dev.ichen.doubao-skin";
        const MAIN_EXECUTABLE_NAME: &str = "豆皮";

        #[link(name = "bsm")]
        unsafe extern "C" {
            fn audit_session_self() -> u32;
        }

        pub struct InstanceGuard;

        pub fn acquire_single_instance() -> Result<Option<InstanceGuard>, String> {
            Ok(Some(InstanceGuard))
        }

        pub fn main_executable_from_agent(agent: &Path) -> Option<PathBuf> {
            let login_items = agent
                .ancestors()
                .find(|path| path.file_name().is_some_and(|name| name == "LoginItems"))?;
            let library = login_items.parent()?;
            if library.file_name().is_none_or(|name| name != "Library") {
                return None;
            }
            let contents = library.parent()?;
            if contents.file_name().is_none_or(|name| name != "Contents") {
                return None;
            }
            Some(contents.join("MacOS").join(MAIN_EXECUTABLE_NAME))
        }

        fn main_running_from_observations(
            executable_running: bool,
            registered_application_running: bool,
        ) -> bool {
            executable_running || registered_application_running
        }

        pub fn main_app_is_running(main_executable: &Path) -> bool {
            let executable_running = live::executable_is_running(main_executable);
            // Login items may run in a service-management process context where
            // process-path enumeration does not observe a GUI app reliably.
            let registered_application_running = unsafe {
                let Some(class) = Class::get("NSRunningApplication") else {
                    return executable_running;
                };
                let identifier = NSString::alloc(nil).init_str(MAIN_BUNDLE_ID);
                let applications: id =
                    msg_send![class, runningApplicationsWithBundleIdentifier: identifier];
                let count: usize = msg_send![applications, count];
                let _: () = msg_send![identifier, release];
                count > 0
            };
            main_running_from_observations(executable_running, registered_application_running)
        }

        pub fn login_session_id() -> Result<u64, String> {
            // SAFETY: audit_session_self has no arguments and does not mutate state.
            let session = unsafe { audit_session_self() };
            if session == 0 {
                Err("无法识别当前登录会话".into())
            } else {
                Ok(u64::from(session))
            }
        }

        #[cfg(test)]
        mod tests {
            use std::path::Path;

            use super::{main_executable_from_agent, main_running_from_observations};

            #[test]
            fn derives_the_outer_main_executable_from_the_nested_login_item() {
                let helper = Path::new(
                    "/Test/豆皮.app/Contents/Library/LoginItems/豆皮后台服务.app/Contents/MacOS/豆皮后台服务",
                );
                assert_eq!(
                    main_executable_from_agent(helper).unwrap(),
                    Path::new("/Test/豆皮.app/Contents/MacOS/豆皮")
                );
            }

            #[test]
            fn rejects_a_standalone_development_binary() {
                assert!(main_executable_from_agent(Path::new(
                    "/workspace/target/debug/doubao-skin-agent"
                ))
                .is_none());
            }

            #[test]
            fn detects_main_through_registered_application_when_path_lookup_misses() {
                assert!(main_running_from_observations(false, true));
                assert!(!main_running_from_observations(false, false));
            }
        }
    }

    #[cfg(target_os = "windows")]
    mod platform {
        use std::ffi::c_void;
        use std::path::{Path, PathBuf};
        use std::ptr::null_mut;

        use skin_core::live;
        use windows_sys::Win32::Foundation::{
            CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE,
        };
        use windows_sys::Win32::Security::{
            GetTokenInformation, TokenStatistics, TOKEN_QUERY, TOKEN_STATISTICS,
        };
        use windows_sys::Win32::System::Threading::{
            CreateMutexW, GetCurrentProcess, OpenProcessToken,
        };

        use super::{login_session_id_from_luid, windows_main_executable_from_agent};

        const INSTANCE_MUTEX_NAME: &str = "Local\\dev.ichen.doubao-skin.agent";

        struct OwnedHandle(HANDLE);

        impl Drop for OwnedHandle {
            fn drop(&mut self) {
                // SAFETY: this guard owns one non-null Win32 handle.
                unsafe {
                    let _ = CloseHandle(self.0);
                }
            }
        }

        pub struct InstanceGuard(OwnedHandle);

        pub fn acquire_single_instance() -> Result<Option<InstanceGuard>, String> {
            let name: Vec<u16> = INSTANCE_MUTEX_NAME.encode_utf16().chain(Some(0)).collect();
            // SAFETY: the name is NUL-terminated and lives through the call. No custom
            // security descriptor or initial ownership is requested.
            let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
            if handle.is_null() {
                return Err("无法创建豆皮后台服务单实例标记".into());
            }
            // GetLastError must be read before another Win32 call.
            let already_exists = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
            let handle = OwnedHandle(handle);
            if already_exists {
                Ok(None)
            } else {
                Ok(Some(InstanceGuard(handle)))
            }
        }

        pub fn main_executable_from_agent(agent: &Path) -> Option<PathBuf> {
            windows_main_executable_from_agent(agent)
        }

        pub fn main_app_is_running(main_executable: &Path) -> bool {
            live::executable_is_running(main_executable)
        }

        pub fn login_session_id() -> Result<u64, String> {
            let mut token: HANDLE = null_mut();
            // SAFETY: the pseudo process handle is valid for OpenProcessToken and token
            // points to writable storage for the returned owned handle.
            if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
                return Err("无法读取当前 Windows 登录会话".into());
            }
            let token = OwnedHandle(token);
            let mut statistics = TOKEN_STATISTICS::default();
            let mut returned = 0_u32;
            // SAFETY: statistics is correctly sized and writable; token remains alive.
            if unsafe {
                GetTokenInformation(
                    token.0,
                    TokenStatistics,
                    (&mut statistics as *mut TOKEN_STATISTICS).cast::<c_void>(),
                    std::mem::size_of::<TOKEN_STATISTICS>() as u32,
                    &mut returned,
                )
            } == 0
            {
                return Err("无法识别当前 Windows 登录会话".into());
            }
            let session = login_session_id_from_luid(
                statistics.AuthenticationId.LowPart,
                statistics.AuthenticationId.HighPart,
            );
            if session == 0 {
                Err("无法识别当前 Windows 登录会话".into())
            } else {
                Ok(session)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::path::Path;

        use super::{login_session_id_from_luid, windows_main_executable_from_agent};

        #[test]
        fn derives_windows_main_only_from_the_packaged_helpers_directory() {
            assert_eq!(
                windows_main_executable_from_agent(Path::new(
                    "/Test/豆皮/helpers/doubao-skin-agent.exe"
                )),
                Some(Path::new("/Test/豆皮/doubao-skin.exe").to_path_buf())
            );
            assert!(windows_main_executable_from_agent(Path::new(
                "/Test/豆皮/doubao-skin-agent.exe"
            ))
            .is_none());
        }

        #[test]
        fn combines_the_full_windows_authentication_luid() {
            assert_eq!(
                login_session_id_from_luid(0x89ab_cdef, 0x0123_4567),
                0x0123_4567_89ab_cdef
            );
            assert_eq!(
                login_session_id_from_luid(0x0000_0001, -1),
                0xffff_ffff_0000_0001
            );
        }
    }
}

fn main() {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    if let Err(error) = agent::run() {
        eprintln!("豆皮后台服务已退出：{error}");
    }
}
