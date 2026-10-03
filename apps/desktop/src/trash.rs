//! Move a directory to the macOS Trash so removed themes stay recoverable.
//! There is deliberately no permanent-delete fallback: if the Trash refuses,
//! the caller reports the failure and the files stay where they are.

use std::path::{Path, PathBuf};

/// Moves `path` to the Trash and returns where it ended up.
#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
pub fn move_to_trash(path: &Path) -> Result<PathBuf, String> {
    use cocoa::base::{id, nil, BOOL, YES};
    use cocoa::foundation::NSString;
    use objc::{msg_send, sel, sel_impl};

    let path_text = path
        .to_str()
        .ok_or_else(|| "主题路径包含无法识别的字符".to_string())?;
    if !path.exists() {
        return Err("主题目录已不存在".into());
    }
    unsafe {
        let ns_path = NSString::alloc(nil).init_str(path_text);
        let url: id = msg_send![objc::class!(NSURL), fileURLWithPath: ns_path];
        let manager: id = msg_send![objc::class!(NSFileManager), defaultManager];
        let mut resulting: id = nil;
        let mut error: id = nil;
        let moved: BOOL = msg_send![
            manager,
            trashItemAtURL: url
            resultingItemURL: &mut resulting
            error: &mut error
        ];
        if moved != YES {
            let reason = if error == nil {
                "系统拒绝了这次操作".to_string()
            } else {
                let description: id = msg_send![error, localizedDescription];
                ns_string_to_rust(description).unwrap_or_else(|| "系统拒绝了这次操作".into())
            };
            return Err(format!("无法移入废纸篓：{reason}"));
        }
        if resulting == nil {
            return Ok(path.to_path_buf());
        }
        let resulting_path: id = msg_send![resulting, path];
        Ok(ns_string_to_rust(resulting_path)
            .map(PathBuf::from)
            .unwrap_or_else(|| path.to_path_buf()))
    }
}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
unsafe fn ns_string_to_rust(value: cocoa::base::id) -> Option<String> {
    use cocoa::base::nil;
    use objc::{msg_send, sel, sel_impl};

    if value == nil {
        return None;
    }
    let bytes: *const std::os::raw::c_char = msg_send![value, UTF8String];
    if bytes.is_null() {
        return None;
    }
    Some(
        std::ffi::CStr::from_ptr(bytes)
            .to_string_lossy()
            .into_owned(),
    )
}

#[cfg(not(target_os = "macos"))]
pub fn move_to_trash(_path: &Path) -> Result<PathBuf, String> {
    Err("当前系统不支持移入废纸篓".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn unique_dir(label: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "doubao-skin-trash-{label}-{}-{nanos}",
            std::process::id()
        ))
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn moving_a_directory_to_the_trash_removes_it_but_keeps_it_recoverable() {
        let dir = unique_dir("ok");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("theme.json"), "{}").unwrap();

        let trashed = move_to_trash(&dir).expect("trashing a temp directory must succeed");

        assert!(!dir.exists(), "original path must be gone");
        assert!(
            trashed.join("theme.json").exists(),
            "contents must be recoverable from {}",
            trashed.display()
        );
        let _ = fs::remove_dir_all(&trashed);
    }

    #[test]
    fn trashing_a_missing_path_fails_without_touching_anything_else() {
        let sibling = unique_dir("sibling");
        fs::create_dir_all(&sibling).unwrap();
        let missing = unique_dir("missing");

        let error = move_to_trash(&missing).unwrap_err();

        assert!(!error.is_empty());
        assert!(sibling.exists());
        let _ = fs::remove_dir_all(&sibling);
    }
}
