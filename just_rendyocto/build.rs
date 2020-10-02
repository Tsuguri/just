
fn main() {
    let path = "../dev_app/renderer.octo";
    octo::process_file(&path).unwrap();
    println!("cargo:rerun-if-changed={}", path);
}
