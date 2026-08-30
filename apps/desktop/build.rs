fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    println!("cargo:rerun-if-changed=../../assets/app-icon/AppIcon.ico");
    let manifest_dir = std::path::PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory"),
    );
    let icon = manifest_dir
        .join("../../assets/app-icon/AppIcon.ico")
        .canonicalize()
        .expect("Windows application icon");
    let resource = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo OUT_DIR"))
        .join("app-icon.rc");
    let icon = icon.to_string_lossy().replace('\\', "/");
    std::fs::write(&resource, format!("1 ICON \"{icon}\"\n"))
        .expect("write Windows icon resource file");
    embed_resource::compile(resource, embed_resource::NONE)
        .manifest_optional()
        .expect("compile Windows application icon");
}
