fn main() {
  println!("cargo:rerun-if-changed=../workdir/messenger-demo.happ");
  tauri_build::build()
}
