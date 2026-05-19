fn main() {
    // Ensure the Flutter web bundle directory exists at compile time so that
    // `rust-embed` doesn't fail on a fresh checkout. The directory may be
    // empty — in that case the static HTTP server logs a warning at runtime
    // and refuses to start. To actually embed assets, run
    //   (from repo root)  cd mobile && flutter build web --release
    // before `npm run tauri dev` / `npm run tauri build`.
    let web_dir = std::path::PathBuf::from("../../mobile/build/web");
    if !web_dir.exists() {
        let _ = std::fs::create_dir_all(&web_dir);
    }
    println!("cargo:rerun-if-changed=../../mobile/build/web");

    tauri_build::build()
}
