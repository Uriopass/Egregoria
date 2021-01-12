use std::path::Path;

// Example custom build script.
fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("no env var");
    let assets = Path::new(&manifest_dir).join("..").join("assets");

    let mk_houses_py = assets.join("mk_houses.py");

    let canon_house =
        std::fs::canonicalize(mk_houses_py).expect("could not canonicalize path to assets");
    let os_path = canon_house.to_str().unwrap();

    let child = std::process::Command::new("python3")
        .arg(os_path)
        .current_dir(std::fs::canonicalize(assets).expect("could not canonicalize path to assets"))
        .spawn()
        .expect("couldn't start python command to make houses");

    let out = child.wait_with_output().expect("error running command");
    if !out.status.success() {
        panic!(
            "error generating houses {}",
            String::from_utf8_lossy(&out.stderr)
        )
    }

    println!("cargo:rerun-if-changed={}", os_path);
}
