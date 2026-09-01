//! Platform-specific application setup.

#[cfg(any(test, target_os = "windows"))]
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoThemeServiceStatus {
    Unsupported,
    NotRegistered,
    Enabled,
    RequiresApproval,
    NotFound,
}

#[cfg(any(test, target_os = "windows"))]
const WINDOWS_MAIN_EXECUTABLE_NAME: &str = "doubao-skin.exe";
#[cfg(any(test, target_os = "windows"))]
const WINDOWS_AGENT_EXECUTABLE_NAME: &str = "doubao-skin-agent.exe";
#[cfg(any(test, target_os = "windows"))]
const WINDOWS_RUN_COMMAND_MAX_UTF16: usize = 260;

#[cfg(any(test, target_os = "windows"))]
fn windows_helper_path_from_main(main_executable: &Path) -> Option<PathBuf> {
    let file_name = main_executable.file_name()?.to_str()?;
    if !file_name.eq_ignore_ascii_case(WINDOWS_MAIN_EXECUTABLE_NAME) {
        return None;
    }
    Some(
        main_executable
            .parent()?
            .join("helpers")
            .join(WINDOWS_AGENT_EXECUTABLE_NAME),
    )
}

#[cfg(any(test, target_os = "windows"))]
fn windows_run_command(helper: &Path) -> Result<String, String> {
    if !helper.is_absolute() {
        return Err("豆皮后台服务路径必须是绝对路径".into());
    }
    let path = helper
        .to_str()
        .ok_or_else(|| "豆皮后台服务路径无法用于 Windows 启动项".to_string())?;
    if path.contains('\0') || path.contains('"') {
        return Err("豆皮后台服务路径包含不支持的字符".into());
    }
    let command = format!("\"{path}\"");
    if command.encode_utf16().count() + 1 > WINDOWS_RUN_COMMAND_MAX_UTF16 {
        return Err("豆皮安装路径过长，请移动到更短的目录后重试".into());
    }
    Ok(command)
}

#[cfg(any(test, target_os = "windows"))]
fn windows_service_status(helper: &Path, registered_value: Option<&str>) -> AutoThemeServiceStatus {
    if !helper.is_file() {
        return AutoThemeServiceStatus::NotFound;
    }
    let Some(value) = registered_value else {
        return AutoThemeServiceStatus::NotRegistered;
    };
    match windows_run_command(helper) {
        Ok(expected) if value == expected => AutoThemeServiceStatus::Enabled,
        _ => AutoThemeServiceStatus::NotFound,
    }
}

#[cfg(any(test, target_os = "windows"))]
trait WindowsStartupBackend {
    fn read_value(&self) -> Result<Option<String>, String>;
    fn write_value(&self, value: &str) -> Result<(), String>;
    fn remove_value(&self) -> Result<(), String>;
    fn spawn_helper(&self, helper: &Path) -> Result<(), String>;
}

#[cfg(any(test, target_os = "windows"))]
fn register_windows_startup(
    backend: &impl WindowsStartupBackend,
    helper: &Path,
) -> Result<AutoThemeServiceStatus, String> {
    if !helper.is_file() {
        return Err("当前豆皮安装包不包含后台服务，请重新解压最新版".into());
    }
    let command = windows_run_command(helper)?;
    backend.write_value(&command)?;
    if backend.spawn_helper(helper).is_err() {
        return Err(registration_failure(
            backend,
            "无法启动豆皮后台服务，启动项已回滚",
        ));
    }
    let registered_value = backend
        .read_value()
        .map_err(|_| registration_failure(backend, "无法确认豆皮后台启动项，已回滚"))?;
    let status = windows_service_status(helper, registered_value.as_deref());
    if status != AutoThemeServiceStatus::Enabled {
        return Err(registration_failure(
            backend,
            "无法确认豆皮后台启动项，已回滚",
        ));
    }
    Ok(status)
}

#[cfg(any(test, target_os = "windows"))]
fn registration_failure(backend: &impl WindowsStartupBackend, rolled_back: &str) -> String {
    if backend.remove_value().is_ok() {
        rolled_back.to_string()
    } else {
        "后台服务启用失败，Windows 启动项也未能回滚".into()
    }
}

#[cfg(any(test, target_os = "windows"))]
fn unregister_windows_startup(
    backend: &impl WindowsStartupBackend,
) -> Result<AutoThemeServiceStatus, String> {
    backend.remove_value()?;
    Ok(AutoThemeServiceStatus::NotRegistered)
}

fn auto_theme_service_status_from_raw(raw: Option<isize>) -> AutoThemeServiceStatus {
    match raw {
        Some(0) => AutoThemeServiceStatus::NotRegistered,
        Some(1) => AutoThemeServiceStatus::Enabled,
        Some(2) => AutoThemeServiceStatus::RequiresApproval,
        Some(3) => AutoThemeServiceStatus::NotFound,
        _ => AutoThemeServiceStatus::Unsupported,
    }
}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
mod service_management {
    use std::path::{Path, PathBuf};

    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;
    use objc::runtime::Class;
    use objc::{msg_send, sel, sel_impl};

    use super::{auto_theme_service_status_from_raw, AutoThemeServiceStatus};

    const HELPER_BUNDLE_ID: &str = "dev.ichen.doubao-skin.agent";
    const HELPER_BUNDLE_NAME: &str = "豆皮后台服务.app";

    #[link(name = "ServiceManagement", kind = "framework")]
    unsafe extern "C" {}

    fn helper_bundle_path(main_bundle: &Path) -> PathBuf {
        main_bundle
            .join("Contents/Library/LoginItems")
            .join(HELPER_BUNDLE_NAME)
    }

    unsafe fn main_bundle_path() -> Option<PathBuf> {
        let bundle: id = unsafe { msg_send![objc::class!(NSBundle), mainBundle] };
        let value: id = unsafe { msg_send![bundle, bundlePath] };
        if value == nil {
            return None;
        }
        let utf8: *const std::ffi::c_char = unsafe { msg_send![value, UTF8String] };
        if utf8.is_null() {
            return None;
        }
        let value = unsafe { std::ffi::CStr::from_ptr(utf8) }
            .to_string_lossy()
            .into_owned();
        Some(PathBuf::from(value))
    }

    unsafe fn service() -> Option<id> {
        let class = Class::get("SMAppService")?;
        let identifier = unsafe { NSString::alloc(nil).init_str(HELPER_BUNDLE_ID) };
        let service: id = unsafe { msg_send![class, loginItemServiceWithIdentifier: identifier] };
        unsafe {
            let _: () = msg_send![identifier, release];
        }
        (service != nil).then_some(service)
    }

    pub fn status() -> AutoThemeServiceStatus {
        // SAFETY: every selector is guarded by the macOS 13 SMAppService class
        // lookup. On macOS 12 the class is absent and the feature is disabled.
        unsafe {
            let Some(service) = service() else {
                return AutoThemeServiceStatus::Unsupported;
            };
            let raw: isize = msg_send![service, status];
            auto_theme_service_status_from_raw(Some(raw))
        }
    }

    pub fn register() -> Result<AutoThemeServiceStatus, String> {
        // SAFETY: paths are only inspected inside the current main bundle and
        // selectors are guarded by a runtime class lookup.
        unsafe {
            let main_bundle = main_bundle_path()
                .filter(|path| path.extension().is_some_and(|extension| extension == "app"))
                .ok_or_else(|| "请从已安装的豆皮应用中开启自动保持主题".to_string())?;
            if !helper_bundle_path(&main_bundle).is_dir() {
                return Err("当前豆皮安装包不包含后台服务，请重新安装最新版".into());
            }
            let service =
                service().ok_or_else(|| "自动保持主题需要 macOS 13 或更高版本".to_string())?;
            let current: isize = msg_send![service, status];
            if current == 1 || current == 2 {
                return Ok(auto_theme_service_status_from_raw(Some(current)));
            }
            let mut error: id = nil;
            let registered: bool = msg_send![service, registerAndReturnError: &mut error];
            if !registered {
                return Err("无法启用豆皮后台服务，请检查系统登录项设置".into());
            }
            let raw: isize = msg_send![service, status];
            Ok(auto_theme_service_status_from_raw(Some(raw)))
        }
    }

    pub fn unregister() -> Result<AutoThemeServiceStatus, String> {
        // SAFETY: selector use is guarded by a runtime class lookup.
        unsafe {
            let service =
                service().ok_or_else(|| "自动保持主题需要 macOS 13 或更高版本".to_string())?;
            let raw: isize = msg_send![service, status];
            if raw == 3 {
                return Ok(AutoThemeServiceStatus::NotFound);
            }
            if raw != 0 {
                let mut error: id = nil;
                let unregistered: bool = msg_send![service, unregisterAndReturnError: &mut error];
                if !unregistered {
                    return Err("无法关闭豆皮后台服务，请检查系统登录项设置".into());
                }
            }
            let raw: isize = msg_send![service, status];
            Ok(auto_theme_service_status_from_raw(Some(raw)))
        }
    }

    pub fn open_settings() -> Result<(), String> {
        // SAFETY: selector use is guarded by a runtime class lookup.
        unsafe {
            let class = Class::get("SMAppService")
                .ok_or_else(|| "自动保持主题需要 macOS 13 或更高版本".to_string())?;
            let _: () = msg_send![class, openSystemSettingsLoginItems];
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use std::path::Path;

        use super::helper_bundle_path;

        #[test]
        fn helper_path_is_inside_the_main_bundle_login_items_directory() {
            assert_eq!(
                helper_bundle_path(Path::new("/Test/豆皮.app")),
                Path::new("/Test/豆皮.app/Contents/Library/LoginItems/豆皮后台服务.app")
            );
        }
    }
}

#[cfg(target_os = "windows")]
mod windows_startup {
    use std::os::windows::process::CommandExt;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use windows_registry::CURRENT_USER;
    use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

    use super::{
        register_windows_startup, unregister_windows_startup, windows_helper_path_from_main,
        windows_service_status, AutoThemeServiceStatus, WindowsStartupBackend,
    };

    const RUN_KEY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    const RUN_VALUE_NAME: &str = "DoubaoSkinAutoTheme";
    const FILE_NOT_FOUND_HRESULT: i32 = 0x8007_0002_u32 as i32;

    struct RegistryStartupBackend;

    impl WindowsStartupBackend for RegistryStartupBackend {
        fn read_value(&self) -> Result<Option<String>, String> {
            let key = match CURRENT_USER.open(RUN_KEY_PATH) {
                Ok(key) => key,
                Err(error) if error.code().0 == FILE_NOT_FOUND_HRESULT => return Ok(None),
                Err(_) => return Err("无法读取 Windows 后台启动项".into()),
            };
            match key.get_string(RUN_VALUE_NAME) {
                Ok(value) => Ok(Some(value)),
                Err(error) if error.code().0 == FILE_NOT_FOUND_HRESULT => Ok(None),
                Err(_) => Err("无法读取 Windows 后台启动项".into()),
            }
        }

        fn write_value(&self, value: &str) -> Result<(), String> {
            let key = CURRENT_USER
                .create(RUN_KEY_PATH)
                .map_err(|_| "无法创建 Windows 后台启动项".to_string())?;
            key.set_string(RUN_VALUE_NAME, value)
                .map_err(|_| "无法注册 Windows 后台启动项".to_string())
        }

        fn remove_value(&self) -> Result<(), String> {
            let key = match CURRENT_USER.open(RUN_KEY_PATH) {
                Ok(key) => key,
                Err(error) if error.code().0 == FILE_NOT_FOUND_HRESULT => return Ok(()),
                Err(_) => return Err("无法打开 Windows 后台启动项".into()),
            };
            match key.remove_value(RUN_VALUE_NAME) {
                Ok(()) => Ok(()),
                Err(error) if error.code().0 == FILE_NOT_FOUND_HRESULT => Ok(()),
                Err(_) => Err("无法移除 Windows 后台启动项".into()),
            }
        }

        fn spawn_helper(&self, helper: &Path) -> Result<(), String> {
            Command::new(helper)
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .map(|_| ())
                .map_err(|_| "无法启动豆皮后台服务".to_string())
        }
    }

    fn helper_path() -> Result<PathBuf, String> {
        let main_executable =
            std::env::current_exe().map_err(|_| "无法识别当前豆皮安装位置".to_string())?;
        windows_helper_path_from_main(&main_executable)
            .ok_or_else(|| "请从解压后的 doubao-skin.exe 开启自动保持主题".to_string())
    }

    pub fn status() -> AutoThemeServiceStatus {
        let Ok(helper) = helper_path() else {
            return AutoThemeServiceStatus::NotFound;
        };
        let backend = RegistryStartupBackend;
        match backend.read_value() {
            Ok(value) => windows_service_status(&helper, value.as_deref()),
            Err(_) => AutoThemeServiceStatus::NotFound,
        }
    }

    pub fn register() -> Result<AutoThemeServiceStatus, String> {
        register_windows_startup(&RegistryStartupBackend, &helper_path()?)
    }

    pub fn unregister() -> Result<AutoThemeServiceStatus, String> {
        unregister_windows_startup(&RegistryStartupBackend)
    }

    pub fn open_settings() -> Result<(), String> {
        Command::new("explorer.exe")
            .arg("ms-settings:startupapps")
            .spawn()
            .map(|_| ())
            .map_err(|_| "无法打开 Windows 启动应用设置".to_string())
    }
}

pub fn auto_theme_service_status() -> AutoThemeServiceStatus {
    #[cfg(target_os = "macos")]
    {
        service_management::status()
    }
    #[cfg(target_os = "windows")]
    {
        windows_startup::status()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        AutoThemeServiceStatus::Unsupported
    }
}

pub fn register_auto_theme_service() -> Result<AutoThemeServiceStatus, String> {
    #[cfg(target_os = "macos")]
    {
        service_management::register()
    }
    #[cfg(target_os = "windows")]
    {
        windows_startup::register()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("自动保持主题仅支持 macOS 13 或更高版本".into())
    }
}

pub fn unregister_auto_theme_service() -> Result<AutoThemeServiceStatus, String> {
    #[cfg(target_os = "macos")]
    {
        service_management::unregister()
    }
    #[cfg(target_os = "windows")]
    {
        windows_startup::unregister()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("自动保持主题仅支持 macOS 13 或更高版本".into())
    }
}

pub fn open_login_items_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        service_management::open_settings()
    }
    #[cfg(target_os = "windows")]
    {
        windows_startup::open_settings()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("自动保持主题仅支持 macOS 13 或更高版本".into())
    }
}

pub fn init_logger() {
    struct StderrLogger;
    impl log::Log for StderrLogger {
        fn enabled(&self, _: &log::Metadata) -> bool {
            true
        }
        fn log(&self, record: &log::Record) {
            eprintln!("[{} {}] {}", record.level(), record.target(), record.args());
        }
        fn flush(&self) {}
    }
    static LOGGER: StderrLogger = StderrLogger;
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Info);
}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
pub fn set_development_icon() {
    use cocoa::appkit::{NSApp, NSApplication, NSImage};
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSData, NSString};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let bundle: id = msg_send![objc::class!(NSBundle), mainBundle];
        let path: id = msg_send![bundle, bundlePath];
        let ext = NSString::alloc(nil).init_str(".app");
        let is_bundle: bool = msg_send![path, hasSuffix: ext];
        if is_bundle {
            return;
        }
    }
    let bytes = include_bytes!("../../../../assets/app-icon/AppIcon.icns");
    unsafe {
        let data = NSData::dataWithBytes_length_(nil, bytes.as_ptr().cast(), bytes.len() as _);
        let image = NSImage::initWithData_(NSImage::alloc(nil), data);
        assert!(image != nil, "embedded AppIcon.icns must be valid");
        NSApp().setApplicationIconImage_(image);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::{
        auto_theme_service_status_from_raw, register_windows_startup, unregister_windows_startup,
        windows_helper_path_from_main, windows_run_command, windows_service_status,
        AutoThemeServiceStatus, WindowsStartupBackend,
    };

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(1);

    fn test_helper() -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "doubao-skin-windows-startup-{}-{}",
            std::process::id(),
            NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        let helper = root.join("豆皮 便携包/helpers/doubao-skin-agent.exe");
        fs::create_dir_all(helper.parent().unwrap()).unwrap();
        fs::write(&helper, b"test helper").unwrap();
        (root, helper)
    }

    fn absolute_test_path(suffix: &str) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            PathBuf::from(format!("C:\\{suffix}"))
        }
        #[cfg(not(target_os = "windows"))]
        {
            PathBuf::from(format!("/{suffix}"))
        }
    }

    fn absolute_test_path_with_utf16_len(len: usize) -> PathBuf {
        #[cfg(target_os = "windows")]
        let prefix = "C:\\";
        #[cfg(not(target_os = "windows"))]
        let prefix = "/";

        let prefix_len = prefix.encode_utf16().count();
        assert!(len >= prefix_len);
        PathBuf::from(format!("{prefix}{}", "a".repeat(len - prefix_len)))
    }

    #[derive(Default)]
    struct FakeStartupBackend {
        value: RefCell<Option<String>>,
        read_fails: Cell<bool>,
        write_fails: Cell<bool>,
        spawn_fails: Cell<bool>,
        remove_fails: Cell<bool>,
        spawn_count: Cell<usize>,
    }

    impl WindowsStartupBackend for FakeStartupBackend {
        fn read_value(&self) -> Result<Option<String>, String> {
            if self.read_fails.get() {
                return Err("read failed".into());
            }
            Ok(self.value.borrow().clone())
        }

        fn write_value(&self, value: &str) -> Result<(), String> {
            if self.write_fails.get() {
                return Err("write failed".into());
            }
            self.value.replace(Some(value.to_string()));
            Ok(())
        }

        fn remove_value(&self) -> Result<(), String> {
            if self.remove_fails.get() {
                return Err("remove failed".into());
            }
            self.value.replace(None);
            Ok(())
        }

        fn spawn_helper(&self, _helper: &Path) -> Result<(), String> {
            self.spawn_count.set(self.spawn_count.get() + 1);
            if self.spawn_fails.get() {
                Err("spawn failed".into())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn maps_every_service_management_status_without_guessing_unknown_values() {
        assert_eq!(
            auto_theme_service_status_from_raw(Some(0)),
            AutoThemeServiceStatus::NotRegistered
        );
        assert_eq!(
            auto_theme_service_status_from_raw(Some(1)),
            AutoThemeServiceStatus::Enabled
        );
        assert_eq!(
            auto_theme_service_status_from_raw(Some(2)),
            AutoThemeServiceStatus::RequiresApproval
        );
        assert_eq!(
            auto_theme_service_status_from_raw(Some(3)),
            AutoThemeServiceStatus::NotFound
        );
        assert_eq!(
            auto_theme_service_status_from_raw(Some(9)),
            AutoThemeServiceStatus::Unsupported
        );
        assert_eq!(
            auto_theme_service_status_from_raw(None),
            AutoThemeServiceStatus::Unsupported
        );
    }

    #[test]
    fn derives_only_the_packaged_windows_helper_path() {
        assert_eq!(
            windows_helper_path_from_main(Path::new("/Test/豆皮/doubao-skin.exe")),
            Some(PathBuf::from("/Test/豆皮/helpers/doubao-skin-agent.exe"))
        );
        assert!(windows_helper_path_from_main(Path::new(
            "/workspace/target/debug/doubao-skin-app.exe"
        ))
        .is_none());
    }

    #[test]
    fn quotes_unicode_paths_and_enforces_the_documented_run_limit() {
        let unicode = absolute_test_path("Test/豆皮 便携包/helpers/agent.exe");
        assert_eq!(
            windows_run_command(&unicode).unwrap(),
            format!("\"{}\"", unicode.to_string_lossy())
        );
        assert!(windows_run_command(&absolute_test_path("Test/bad\"name/agent.exe")).is_err());

        let largest = absolute_test_path_with_utf16_len(257);
        assert_eq!(
            windows_run_command(&largest)
                .unwrap()
                .encode_utf16()
                .count()
                + 1,
            260
        );
        let too_long = absolute_test_path_with_utf16_len(258);
        assert!(windows_run_command(&too_long).is_err());
    }

    #[test]
    fn windows_status_requires_both_the_helper_and_the_exact_value() {
        let (root, helper) = test_helper();
        let command = windows_run_command(&helper).unwrap();
        assert_eq!(
            windows_service_status(&helper, None),
            AutoThemeServiceStatus::NotRegistered
        );
        assert_eq!(
            windows_service_status(&helper, Some("\"C:\\stale\\agent.exe\"")),
            AutoThemeServiceStatus::NotFound
        );
        assert_eq!(
            windows_service_status(&helper, Some(&command)),
            AutoThemeServiceStatus::Enabled
        );
        fs::remove_file(&helper).unwrap();
        assert_eq!(
            windows_service_status(&helper, Some(&command)),
            AutoThemeServiceStatus::NotFound
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn registration_is_idempotent_and_rolls_back_a_spawn_failure() {
        let (root, helper) = test_helper();
        let backend = FakeStartupBackend::default();
        assert_eq!(
            register_windows_startup(&backend, &helper).unwrap(),
            AutoThemeServiceStatus::Enabled
        );
        assert_eq!(
            register_windows_startup(&backend, &helper).unwrap(),
            AutoThemeServiceStatus::Enabled
        );
        assert_eq!(backend.spawn_count.get(), 2);

        backend.spawn_fails.set(true);
        assert!(register_windows_startup(&backend, &helper).is_err());
        assert_eq!(*backend.value.borrow(), None);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn registration_rolls_back_a_failed_confirmation() {
        let (root, helper) = test_helper();
        let backend = FakeStartupBackend::default();
        backend.read_fails.set(true);
        assert!(register_windows_startup(&backend, &helper).is_err());
        assert_eq!(*backend.value.borrow(), None);

        backend.value.replace(Some("written value".into()));
        backend.remove_fails.set(true);
        let error = register_windows_startup(&backend, &helper).unwrap_err();
        assert!(error.contains("未能回滚"));
        assert!(backend.value.borrow().is_some());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn write_failure_does_not_create_a_value_and_unregister_is_precise() {
        let (root, helper) = test_helper();
        let backend = FakeStartupBackend::default();
        backend.write_fails.set(true);
        assert!(register_windows_startup(&backend, &helper).is_err());
        assert_eq!(*backend.value.borrow(), None);

        backend.write_fails.set(false);
        backend.value.replace(Some("neighbor-safe".into()));
        assert_eq!(
            unregister_windows_startup(&backend).unwrap(),
            AutoThemeServiceStatus::NotRegistered
        );
        assert_eq!(*backend.value.borrow(), None);
        fs::remove_dir_all(root).unwrap();
    }
}
