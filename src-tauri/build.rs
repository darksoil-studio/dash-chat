fn main() {
    println!("cargo:rerun-if-changed=../workdir/dash-chat.happ");
    tauri_build::build()
}
