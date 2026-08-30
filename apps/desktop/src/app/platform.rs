//! Platform-specific application setup.

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
